use std::collections::BTreeSet;

use crate::{
    Context, EdgeGeometry, Error, FaceGeometry, FaceSample, LoopRole, Orientation, PortedCurve,
    PortedSurface, Shape, ShapeKind, ShapeSummary, TopologySnapshot,
};
use crate::ported_geometry::{analytic_sampled_wire_signed_area, planar_wire_signed_area};

#[derive(Clone, Copy, Debug)]
pub struct BrepVertex {
    pub index: usize,
    pub position: [f64; 3],
}

#[derive(Clone, Debug)]
pub struct BrepWire {
    pub index: usize,
    pub edge_indices: Vec<usize>,
    pub edge_orientations: Vec<Orientation>,
    pub vertex_indices: Vec<usize>,
}

#[derive(Clone, Copy, Debug)]
pub struct BrepFaceLoop {
    pub wire_index: usize,
    pub orientation: Orientation,
    pub role: LoopRole,
}

#[derive(Clone, Debug)]
pub struct BrepEdge {
    pub index: usize,
    pub geometry: EdgeGeometry,
    pub ported_curve: Option<PortedCurve>,
    pub length: f64,
    pub start_vertex: Option<usize>,
    pub end_vertex: Option<usize>,
    pub start_point: Option<[f64; 3]>,
    pub end_point: Option<[f64; 3]>,
    pub adjacent_face_indices: Vec<usize>,
}

#[derive(Clone, Debug)]
pub struct BrepFace {
    pub index: usize,
    pub geometry: FaceGeometry,
    pub ported_surface: Option<PortedSurface>,
    pub orientation: Orientation,
    pub area: f64,
    pub sample: FaceSample,
    pub loops: Vec<BrepFaceLoop>,
    pub adjacent_face_indices: Vec<usize>,
}

#[derive(Debug)]
pub struct BrepShape {
    pub summary: ShapeSummary,
    pub topology: TopologySnapshot,
    pub vertices: Vec<BrepVertex>,
    pub wires: Vec<BrepWire>,
    pub edges: Vec<BrepEdge>,
    pub faces: Vec<BrepFace>,
}

impl Context {
    pub fn ported_brep(&self, shape: &Shape) -> Result<BrepShape, Error> {
        let summary = self.describe_shape(shape)?;
        let topology = self.topology(shape)?;
        let vertices = topology
            .vertex_positions
            .iter()
            .copied()
            .enumerate()
            .map(|(index, position)| BrepVertex { index, position })
            .collect::<Vec<_>>();

        let wires = topology
            .wires
            .iter()
            .enumerate()
            .map(|(index, range)| {
                let edge_indices =
                    topology.wire_edge_indices[range.offset..range.offset + range.count].to_vec();
                let edge_orientations = topology.wire_edge_orientations
                    [range.offset..range.offset + range.count]
                    .to_vec();
                let vertex_range = topology.wire_vertices[index];
                let vertex_indices = topology.wire_vertex_indices
                    [vertex_range.offset..vertex_range.offset + vertex_range.count]
                    .to_vec();
                BrepWire {
                    index,
                    edge_indices,
                    edge_orientations,
                    vertex_indices,
                }
            })
            .collect::<Vec<_>>();

        let edge_shapes = self.subshapes(shape, ShapeKind::Edge)?;
        let edges = edge_shapes
            .iter()
            .enumerate()
            .map(|(index, edge_shape)| {
                let topology_edge = topology_edge(&topology, index)?;
                let geometry = self.edge_geometry(edge_shape)?;
                let ported_curve =
                    PortedCurve::from_context_with_geometry(self, edge_shape, geometry)?;
                let adjacent_face_indices = adjacent_face_indices(&topology, index)?;
                let (start_point, end_point) = edge_points(&topology, index);
                let length = match ported_curve {
                    Some(curve) => curve.length_with_geometry(geometry),
                    None => topology_edge.length,
                };

                Ok(BrepEdge {
                    index,
                    geometry,
                    ported_curve,
                    length,
                    start_vertex: topology_edge.start_vertex,
                    end_vertex: topology_edge.end_vertex,
                    start_point,
                    end_point,
                    adjacent_face_indices,
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;

        let face_shapes = self.subshapes(shape, ShapeKind::Face)?;
        let faces = face_shapes
            .iter()
            .enumerate()
            .map(|(index, face_shape)| {
                let geometry = self.face_geometry(face_shape)?;
                let ported_surface =
                    PortedSurface::from_context_with_geometry(self, face_shape, geometry)?;
                let orientation = self.shape_orientation(face_shape)?;
                let sample = match ported_surface {
                    Some(surface) => {
                        surface.sample_normalized_with_orientation(geometry, [0.5, 0.5], orientation)
                    }
                    None => self.face_sample_normalized(face_shape, [0.5, 0.5])?,
                };
                let loops = face_loops(&topology, index)?;
                let area = match ported_surface {
                    Some(surface) => {
                        analytic_face_area(self, surface, geometry, &loops, &wires, &edges, &edge_shapes)
                            .unwrap_or(self.describe_shape(face_shape)?.surface_area)
                    }
                    None => self.describe_shape(face_shape)?.surface_area,
                };
                let adjacent_face_indices = face_adjacent_face_indices(&topology, &wires, index)?;

                Ok(BrepFace {
                    index,
                    geometry,
                    ported_surface,
                    orientation,
                    area,
                    sample,
                    loops,
                    adjacent_face_indices,
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;

        Ok(BrepShape {
            summary,
            topology,
            vertices,
            wires,
            edges,
            faces,
        })
    }
}

fn analytic_face_area(
    context: &Context,
    surface: PortedSurface,
    face_geometry: FaceGeometry,
    loops: &[BrepFaceLoop],
    wires: &[BrepWire],
    edges: &[BrepEdge],
    edge_shapes: &[Shape],
) -> Option<f64> {
    let plane = match surface {
        PortedSurface::Plane(plane) => Some(plane),
        _ => None,
    };

    let mut area = 0.0;
    for face_loop in loops {
        let wire = wires.get(face_loop.wire_index)?;
        let mut curve_segments = Vec::with_capacity(wire.edge_indices.len());
        let mut sampled_points = Vec::new();
        for (edge_index, edge_orientation) in wire
            .edge_indices
            .iter()
            .copied()
            .zip(wire.edge_orientations.iter().copied())
        {
            let edge = edges.get(edge_index)?;
            let geometry = oriented_edge_geometry(edge.geometry, edge_orientation);
            if let Some(curve) = edge.ported_curve {
                curve_segments.push((curve, geometry));
            }
            append_edge_sample_points(
                context,
                edge_shapes.get(edge_index)?,
                edge,
                geometry,
                &mut sampled_points,
            )
            .ok()?;
        }

        let wire_area = match plane {
            Some(plane) if curve_segments.len() == wire.edge_indices.len() => {
                planar_wire_signed_area(plane, &curve_segments).abs()
            }
            Some(_) => analytic_sampled_wire_signed_area(surface, face_geometry, &sampled_points)?.abs(),
            None => analytic_sampled_wire_signed_area(surface, face_geometry, &sampled_points)?.abs(),
        };
        match face_loop.role {
            LoopRole::Inner => area -= wire_area,
            LoopRole::Outer | LoopRole::Unknown => area += wire_area,
        }
    }
    Some(area.abs())
}

fn append_edge_sample_points(
    context: &Context,
    edge_shape: &Shape,
    edge: &BrepEdge,
    geometry: EdgeGeometry,
    out_points: &mut Vec<[f64; 3]>,
) -> Result<(), Error> {
    let segment_count = edge_sample_count(edge, geometry);
    for step in 0..=segment_count {
        if !out_points.is_empty() && step == 0 {
            continue;
        }
        let t = step as f64 / segment_count as f64;
        let parameter = interpolate_range(geometry.start_parameter, geometry.end_parameter, t);
        let position = match edge.ported_curve {
            Some(curve) => curve.sample_with_geometry(geometry, parameter).position,
            None => context.edge_sample_at_parameter(edge_shape, parameter)?.position,
        };
        out_points.push(position);
    }
    Ok(())
}

fn edge_sample_count(edge: &BrepEdge, geometry: EdgeGeometry) -> usize {
    let span = (geometry.end_parameter - geometry.start_parameter).abs();
    let base = match edge.geometry.kind {
        crate::CurveKind::Line => 8,
        crate::CurveKind::Circle | crate::CurveKind::Ellipse => {
            (span / (std::f64::consts::TAU / 32.0)).ceil() as usize
        }
        _ => 48,
    };
    base.clamp(8, 256)
}

fn interpolate_range(start: f64, end: f64, t: f64) -> f64 {
    start + (end - start) * t
}

fn oriented_edge_geometry(mut geometry: EdgeGeometry, orientation: Orientation) -> EdgeGeometry {
    if matches!(orientation, Orientation::Reversed) {
        std::mem::swap(&mut geometry.start_parameter, &mut geometry.end_parameter);
    }
    geometry
}

fn topology_edge(topology: &TopologySnapshot, index: usize) -> Result<crate::TopologyEdge, Error> {
    topology
        .edges
        .get(index)
        .copied()
        .ok_or_else(|| Error::new(format!("topology is missing edge index {index}")))
}

fn adjacent_face_indices(topology: &TopologySnapshot, edge_index: usize) -> Result<Vec<usize>, Error> {
    let range = topology
        .edge_faces
        .get(edge_index)
        .copied()
        .ok_or_else(|| Error::new(format!("topology is missing edge-face range {edge_index}")))?;
    Ok(topology.edge_face_indices[range.offset..range.offset + range.count].to_vec())
}

fn edge_points(
    topology: &TopologySnapshot,
    edge_index: usize,
) -> (Option<[f64; 3]>, Option<[f64; 3]>) {
    let Some(edge) = topology.edges.get(edge_index) else {
        return (None, None);
    };
    (
        optional_vertex_position(topology, edge.start_vertex),
        optional_vertex_position(topology, edge.end_vertex),
    )
}

fn optional_vertex_position(
    topology: &TopologySnapshot,
    vertex_index: Option<usize>,
) -> Option<[f64; 3]> {
    vertex_index.and_then(|index| topology.vertex_positions.get(index).copied())
}

fn face_loops(topology: &TopologySnapshot, face_index: usize) -> Result<Vec<BrepFaceLoop>, Error> {
    let range = topology
        .faces
        .get(face_index)
        .copied()
        .ok_or_else(|| Error::new(format!("topology is missing face range {face_index}")))?;
    let mut loops = Vec::with_capacity(range.count);
    for offset in range.offset..range.offset + range.count {
        loops.push(BrepFaceLoop {
            wire_index: topology
                .face_wire_indices
                .get(offset)
                .copied()
                .ok_or_else(|| Error::new(format!("topology is missing face-wire index {offset}")))?,
            orientation: topology
                .face_wire_orientations
                .get(offset)
                .copied()
                .ok_or_else(|| Error::new(format!("topology is missing face-wire orientation {offset}")))?,
            role: topology
                .face_wire_roles
                .get(offset)
                .copied()
                .ok_or_else(|| Error::new(format!("topology is missing face-wire role {offset}")))?,
        });
    }
    Ok(loops)
}

fn face_adjacent_face_indices(
    topology: &TopologySnapshot,
    wires: &[BrepWire],
    face_index: usize,
) -> Result<Vec<usize>, Error> {
    let loops = face_loops(topology, face_index)?;
    let mut adjacent = BTreeSet::new();
    for face_loop in loops {
        let wire = wires
            .get(face_loop.wire_index)
            .ok_or_else(|| Error::new(format!("topology is missing wire index {}", face_loop.wire_index)))?;
        for &edge_index in &wire.edge_indices {
            let range = topology.edge_faces.get(edge_index).copied().ok_or_else(|| {
                Error::new(format!("topology is missing edge-face range {edge_index}"))
            })?;
            for &candidate in &topology.edge_face_indices[range.offset..range.offset + range.count] {
                if candidate != face_index {
                    adjacent.insert(candidate);
                }
            }
        }
    }
    Ok(adjacent.into_iter().collect())
}
