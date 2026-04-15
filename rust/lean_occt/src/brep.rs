use std::collections::{BTreeMap, BTreeSet};
use std::f64::consts::PI;

use crate::ported_geometry::{
    analytic_sampled_wire_signed_area, analytic_sampled_wire_signed_volume, extrusion_swept_area,
    planar_wire_signed_area, revolution_swept_area, PortedFaceSurface, PortedOffsetBasisSurface,
    PortedOffsetSurface, PortedSweptSurface,
};
use crate::{
    ConePayload, Context, CylinderPayload, EdgeEndpoints, EdgeGeometry, Error, FaceGeometry,
    FaceSample, LoopRole, Mesh, MeshParams, Orientation, PlanePayload, PortedCurve, PortedSurface,
    Shape, ShapeKind, ShapeSummary, SpherePayload, TopologySnapshot, TorusPayload,
};

const SUMMARY_VOLUME_MESH_PARAMS: MeshParams = MeshParams {
    linear_deflection: 0.01,
    angular_deflection: 0.05,
    is_relative: false,
};

const SUMMARY_BBOX_MESH_PARAMS: MeshParams = MeshParams {
    linear_deflection: 0.01,
    angular_deflection: 0.05,
    is_relative: false,
};

const UNSUPPORTED_FACE_AREA_MESH_PARAMS: MeshParams = MeshParams {
    linear_deflection: 0.01,
    angular_deflection: 0.05,
    is_relative: false,
};

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
    pub ported_face_surface: Option<PortedFaceSurface>,
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

#[derive(Clone, Copy, Debug)]
struct ExactPrimitiveSummary {
    surface_area: f64,
    volume: f64,
    bbox: Option<([f64; 3], [f64; 3])>,
}

#[derive(Clone, Copy, Debug)]
struct ShapeCounts {
    compound_count: usize,
    compsolid_count: usize,
    solid_count: usize,
    shell_count: usize,
    face_count: usize,
    wire_count: usize,
    edge_count: usize,
    vertex_count: usize,
}

#[derive(Clone, Copy, Debug)]
struct FaceCurveCandidate {
    curve: PortedCurve,
    geometry: EdgeGeometry,
    midpoint: [f64; 3],
}

#[derive(Clone, Copy, Debug)]
struct CurveDifferential {
    position: [f64; 3],
    first_derivative: [f64; 3],
    second_derivative: [f64; 3],
}

#[derive(Clone, Copy, Debug)]
struct OffsetCurveDifferential {
    position: [f64; 3],
    derivative: [f64; 3],
}

#[derive(Clone, Copy, Debug)]
struct MeshFaceProperties {
    area: f64,
    sample: FaceSample,
}

#[derive(Clone, Copy, Debug)]
struct RootEdgeTopology {
    geometry: EdgeGeometry,
    start_vertex: Option<usize>,
    end_vertex: Option<usize>,
    length: f64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RootWireTopology {
    edge_indices: Vec<usize>,
    edge_orientations: Vec<Orientation>,
    vertex_indices: Vec<usize>,
}

#[derive(Clone, Copy, Debug)]
struct WireOccurrence {
    edge_index: usize,
    orientation: Orientation,
    start_vertex: usize,
    end_vertex: usize,
}

struct SingleFaceTopology {
    loops: Vec<BrepFaceLoop>,
    wires: Vec<BrepWire>,
    edges: Vec<BrepEdge>,
    edge_shapes: Vec<Shape>,
}

impl Context {
    pub fn ported_topology(&self, shape: &Shape) -> Result<Option<TopologySnapshot>, Error> {
        ported_topology_snapshot(self, shape)
    }

    pub fn ported_brep(&self, shape: &Shape) -> Result<BrepShape, Error> {
        let topology = match self.ported_topology(shape)? {
            Some(topology) => topology,
            None => self.topology_occt(shape)?,
        };
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

        let edge_shapes = self.subshapes_occt(shape, ShapeKind::Edge)?;
        let edges = edge_shapes
            .iter()
            .enumerate()
            .map(|(index, edge_shape)| {
                let topology_edge = topology_edge(&topology, index)?;
                let geometry = self.edge_geometry_occt(edge_shape)?;
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

        let face_shapes = self.subshapes_occt(shape, ShapeKind::Face)?;
        let faces = face_shapes
            .iter()
            .enumerate()
            .map(|(index, face_shape)| {
                let geometry = self.face_geometry_occt(face_shape)?;
                let ported_surface =
                    PortedSurface::from_context_with_geometry(self, face_shape, geometry)?;
                let ported_face_surface = ported_face_surface_descriptor_from_surface(
                    self,
                    face_shape,
                    geometry,
                    ported_surface,
                )?;
                let orientation = self.shape_orientation(face_shape)?;
                let loops = face_loops(&topology, index)?;
                let mut mesh_fallback = if ported_face_surface.is_none() {
                    mesh_face_properties(self, face_shape, orientation)
                } else {
                    None
                };
                let mut mesh_fallback_loaded = ported_face_surface.is_none();
                let mut load_mesh_fallback = || {
                    if !mesh_fallback_loaded {
                        mesh_fallback = mesh_face_properties(self, face_shape, orientation);
                        mesh_fallback_loaded = true;
                    }
                    mesh_fallback
                };
                let sample = ported_face_surface
                    .map(|surface| {
                        surface.sample_normalized_with_orientation(
                            geometry,
                            [0.5, 0.5],
                            orientation,
                        )
                    })
                    .or_else(|| load_mesh_fallback().map(|fallback| fallback.sample))
                    .ok_or_else(|| {
                        Error::new(format!(
                            "failed to derive a Rust-owned sample for face {index} ({:?})",
                            geometry.kind
                        ))
                    })?;
                let area = match ported_face_surface {
                    Some(PortedFaceSurface::Analytic(surface)) => analytic_face_area(
                        self,
                        surface,
                        geometry,
                        &loops,
                        &wires,
                        &edges,
                        &edge_shapes,
                    )
                    .or_else(|| load_mesh_fallback().map(|fallback| fallback.area))
                    .ok_or_else(|| {
                        Error::new(format!(
                            "failed to derive a Rust-owned area for face {index} ({:?})",
                            geometry.kind
                        ))
                    })?,
                    Some(PortedFaceSurface::Offset(surface)) => analytic_offset_face_area(
                        self,
                        surface,
                        geometry,
                        &loops,
                        &wires,
                        &edges,
                        &edge_shapes,
                    )
                    .or_else(|| load_mesh_fallback().map(|fallback| fallback.area))
                    .ok_or_else(|| {
                        Error::new(format!(
                            "failed to derive a Rust-owned area for face {index} ({:?})",
                            geometry.kind
                        ))
                    })?,
                    Some(PortedFaceSurface::Swept(surface)) => {
                        analytic_ported_swept_face_area(surface, geometry)
                            .or_else(|| load_mesh_fallback().map(|fallback| fallback.area))
                            .ok_or_else(|| {
                                Error::new(format!(
                                    "failed to derive a Rust-owned area for face {index} ({:?})",
                                    geometry.kind
                                ))
                            })?
                    }
                    None => load_mesh_fallback()
                        .map(|fallback| fallback.area)
                        .ok_or_else(|| {
                            Error::new(format!(
                                "failed to derive a Rust-owned area for face {index} ({:?})",
                                geometry.kind
                            ))
                        })?,
                };
                let adjacent_face_indices = face_adjacent_face_indices(&topology, &wires, index)?;

                Ok(BrepFace {
                    index,
                    geometry,
                    ported_surface,
                    ported_face_surface,
                    orientation,
                    area,
                    sample,
                    loops,
                    adjacent_face_indices,
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;

        let summary = ported_shape_summary(
            self,
            shape,
            &vertices,
            &topology,
            &wires,
            &edges,
            &faces,
            &face_shapes,
            &edge_shapes,
        )?;

        Ok(BrepShape {
            summary,
            topology,
            vertices,
            wires,
            edges,
            faces,
        })
    }

    pub fn ported_vertex_point(&self, shape: &Shape) -> Result<Option<[f64; 3]>, Error> {
        let topology = self.topology(shape)?;
        let counts = ShapeCounts {
            compound_count: self.subshape_count_occt(shape, ShapeKind::Compound)?,
            compsolid_count: self.subshape_count_occt(shape, ShapeKind::CompSolid)?,
            solid_count: self.subshape_count_occt(shape, ShapeKind::Solid)?,
            shell_count: self.subshape_count_occt(shape, ShapeKind::Shell)?,
            face_count: topology.faces.len(),
            wire_count: topology.wires.len(),
            edge_count: topology.edges.len(),
            vertex_count: topology.vertex_positions.len(),
        };
        if classify_root_kind(counts) != ShapeKind::Vertex {
            return Ok(None);
        }

        let [point] = topology.vertex_positions.as_slice() else {
            return Err(Error::new(format!(
                "expected exactly one vertex in vertex topology, found {}",
                topology.vertex_positions.len()
            )));
        };
        Ok(Some(*point))
    }

    pub fn ported_edge_endpoints(&self, shape: &Shape) -> Result<Option<EdgeEndpoints>, Error> {
        let topology = self.topology(shape)?;
        let counts = ShapeCounts {
            compound_count: self.subshape_count_occt(shape, ShapeKind::Compound)?,
            compsolid_count: self.subshape_count_occt(shape, ShapeKind::CompSolid)?,
            solid_count: self.subshape_count_occt(shape, ShapeKind::Solid)?,
            shell_count: self.subshape_count_occt(shape, ShapeKind::Shell)?,
            face_count: topology.faces.len(),
            wire_count: topology.wires.len(),
            edge_count: topology.edges.len(),
            vertex_count: topology.vertex_positions.len(),
        };
        if classify_root_kind(counts) != ShapeKind::Edge {
            return Ok(None);
        }

        let [edge] = topology.edges.as_slice() else {
            return Err(Error::new(format!(
                "expected exactly one edge in edge topology, found {}",
                topology.edges.len()
            )));
        };
        let (Some(start), Some(end)) = (
            optional_vertex_position(&topology, edge.start_vertex),
            optional_vertex_position(&topology, edge.end_vertex),
        ) else {
            return Err(Error::new("Edge did not contain two endpoint vertices."));
        };
        Ok(Some(EdgeEndpoints { start, end }))
    }
}

fn ported_topology_snapshot(
    context: &Context,
    shape: &Shape,
) -> Result<Option<TopologySnapshot>, Error> {
    let face_shapes = context.subshapes_occt(shape, ShapeKind::Face)?;
    for face_shape in &face_shapes {
        let face_wire_shapes = context.subshapes_occt(face_shape, ShapeKind::Wire)?;
        if face_wire_shapes.len() > 1
            && context.face_geometry_occt(face_shape)?.kind != crate::SurfaceKind::Plane
        {
            return Ok(None);
        }
    }

    let vertex_shapes = context.subshapes_occt(shape, ShapeKind::Vertex)?;
    let vertex_positions = vertex_shapes
        .iter()
        .map(|vertex_shape| context.vertex_point_occt(vertex_shape))
        .collect::<Result<Vec<_>, Error>>()?;

    let edge_shapes = context.subshapes_occt(shape, ShapeKind::Edge)?;
    let root_edges = edge_shapes
        .iter()
        .map(|edge_shape| root_edge_topology(context, edge_shape, &vertex_positions))
        .collect::<Result<Vec<_>, Error>>()?;
    let edges = root_edges
        .iter()
        .map(|edge| crate::TopologyEdge {
            start_vertex: edge.start_vertex,
            end_vertex: edge.end_vertex,
            length: edge.length,
        })
        .collect::<Vec<_>>();

    let wire_shapes = context.subshapes_occt(shape, ShapeKind::Wire)?;
    let mut root_wires = Vec::with_capacity(wire_shapes.len());
    for wire_shape in &wire_shapes {
        let Some(topology) =
            root_wire_topology(context, wire_shape, &vertex_positions, &root_edges)?
        else {
            return Ok(None);
        };
        root_wires.push(topology);
    }
    let (wires, wire_edge_indices, wire_edge_orientations, wire_vertices, wire_vertex_indices) =
        pack_wire_topology(&root_wires);
    let mut edge_face_lists = vec![Vec::new(); edges.len()];
    let mut faces = Vec::with_capacity(face_shapes.len());
    let mut face_wire_indices = Vec::new();
    let mut face_wire_orientations = Vec::new();
    let mut face_wire_roles = Vec::new();

    for (face_index, face_shape) in face_shapes.iter().enumerate() {
        let Some(face_topology) = ported_face_topology(
            context,
            face_shape,
            &root_wires,
            &root_edges,
            &edge_shapes,
            &vertex_positions,
        )?
        else {
            return Ok(None);
        };

        faces.push(crate::TopologyRange {
            offset: face_wire_indices.len(),
            count: face_topology.face_wire_indices.len(),
        });
        face_wire_indices.extend(face_topology.face_wire_indices);
        face_wire_orientations.extend(face_topology.face_wire_orientations);
        face_wire_roles.extend(face_topology.face_wire_roles);

        for edge_index in face_topology.edge_indices {
            let Some(edge_faces) = edge_face_lists.get_mut(edge_index) else {
                return Ok(None);
            };
            edge_faces.push(face_index);
        }
    }

    let mut edge_faces = Vec::with_capacity(edges.len());
    let mut edge_face_indices = Vec::new();
    for face_indices in edge_face_lists {
        edge_faces.push(crate::TopologyRange {
            offset: edge_face_indices.len(),
            count: face_indices.len(),
        });
        edge_face_indices.extend(face_indices);
    }

    Ok(Some(TopologySnapshot {
        vertex_positions,
        edges,
        edge_faces,
        edge_face_indices,
        wires,
        wire_edge_indices,
        wire_edge_orientations,
        wire_vertices,
        wire_vertex_indices,
        faces,
        face_wire_indices,
        face_wire_orientations,
        face_wire_roles,
    }))
}

fn root_edge_topology(
    context: &Context,
    edge_shape: &Shape,
    vertex_positions: &[[f64; 3]],
) -> Result<RootEdgeTopology, Error> {
    let geometry = context.edge_geometry_occt(edge_shape)?;
    let endpoints = context.edge_endpoints_occt(edge_shape)?;
    Ok(RootEdgeTopology {
        geometry,
        start_vertex: match_vertex_index(vertex_positions, endpoints.start),
        end_vertex: match_vertex_index(vertex_positions, endpoints.end),
        length: edge_length(edge_shape),
    })
}

fn ported_wire_occurrences(
    context: &Context,
    wire_shape: &Shape,
    vertex_positions: &[[f64; 3]],
    root_edges: &[RootEdgeTopology],
) -> Result<Option<Vec<WireOccurrence>>, Error> {
    let mut occurrences = Vec::new();
    for edge_shape in context.subshapes_occt(wire_shape, ShapeKind::Edge)? {
        let Some(occurrence) = wire_occurrence(context, &edge_shape, vertex_positions, root_edges)?
        else {
            return Ok(None);
        };
        occurrences.push(occurrence);
    }
    Ok(Some(occurrences))
}

fn root_wire_topology(
    context: &Context,
    wire_shape: &Shape,
    vertex_positions: &[[f64; 3]],
    root_edges: &[RootEdgeTopology],
) -> Result<Option<RootWireTopology>, Error> {
    if let Some(topology) =
        root_wire_topology_from_snapshot(context, wire_shape, vertex_positions, root_edges)?
    {
        return Ok(Some(topology));
    }

    let occurrences =
        match ported_wire_occurrences(context, wire_shape, vertex_positions, root_edges)? {
            Some(occurrences) => occurrences,
            None => return Ok(None),
        };
    let (edge_indices, edge_orientations, vertex_indices) =
        match order_wire_occurrences(&occurrences) {
            Some(ordered) => ordered,
            None => return Ok(None),
        };
    Ok(Some(RootWireTopology {
        edge_indices,
        edge_orientations,
        vertex_indices,
    }))
}

fn root_wire_topology_from_snapshot(
    context: &Context,
    wire_shape: &Shape,
    vertex_positions: &[[f64; 3]],
    root_edges: &[RootEdgeTopology],
) -> Result<Option<RootWireTopology>, Error> {
    let topology = context.topology_occt(wire_shape)?;
    if !topology.faces.is_empty() || topology.wires.len() != 1 {
        return Ok(None);
    }

    let wire_range = topology.wires[0];
    let vertex_range = topology.wire_vertices[0];
    if wire_range.count == 0 || vertex_range.count != wire_range.count + 1 {
        return Ok(None);
    }

    let local_edge_shapes = context.subshapes_occt(wire_shape, ShapeKind::Edge)?;
    let mut edge_indices = Vec::with_capacity(wire_range.count);
    let mut edge_orientations = Vec::with_capacity(wire_range.count);
    let mut ordered_vertices = Vec::with_capacity(vertex_range.count);

    for occurrence_offset in 0..wire_range.count {
        let wire_edge_offset = wire_range.offset + occurrence_offset;
        let local_edge_index = *topology
            .wire_edge_indices
            .get(wire_edge_offset)
            .ok_or_else(|| {
                Error::new(format!(
                    "wire topology is missing edge occurrence {wire_edge_offset}"
                ))
            })?;
        let orientation = *topology
            .wire_edge_orientations
            .get(wire_edge_offset)
            .ok_or_else(|| {
                Error::new(format!(
                    "wire topology is missing edge orientation {wire_edge_offset}"
                ))
            })?;
        let local_edge_shape = local_edge_shapes.get(local_edge_index).ok_or_else(|| {
            Error::new(format!(
                "wire topology referenced local edge index {local_edge_index} outside the edge map"
            ))
        })?;

        let local_start_index = *topology
            .wire_vertex_indices
            .get(vertex_range.offset + occurrence_offset)
            .ok_or_else(|| {
                Error::new(format!(
                    "wire topology is missing start vertex occurrence {}",
                    vertex_range.offset + occurrence_offset
                ))
            })?;
        let local_end_index = *topology
            .wire_vertex_indices
            .get(vertex_range.offset + occurrence_offset + 1)
            .ok_or_else(|| {
                Error::new(format!(
                    "wire topology is missing end vertex occurrence {}",
                    vertex_range.offset + occurrence_offset + 1
                ))
            })?;

        let start_vertex = topology_vertex_match(
            &topology.vertex_positions,
            vertex_positions,
            local_start_index,
        );
        let end_vertex = topology_vertex_match(
            &topology.vertex_positions,
            vertex_positions,
            local_end_index,
        );

        let geometry =
            oriented_edge_geometry(context.edge_geometry_occt(local_edge_shape)?, orientation);
        let length = edge_length(local_edge_shape);
        let matches = root_edges
            .iter()
            .enumerate()
            .filter_map(|(root_edge_index, root_edge)| {
                if root_edge.geometry.kind != geometry.kind
                    || !approx_eq(root_edge.length, length, 1.0e-6, 1.0e-6)
                {
                    return None;
                }
                if let (Some(start_vertex), Some(end_vertex)) = (start_vertex, end_vertex) {
                    if !matches_edge_vertices(root_edge, start_vertex, end_vertex) {
                        return None;
                    }
                }
                Some(root_edge_index)
            })
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            return Ok(None);
        }

        let matched_edge = &root_edges[matches[0]];
        let start_vertex = start_vertex.or_else(|| {
            oriented_root_edge_vertices(matched_edge, orientation)
                .map(|(start_vertex, _)| start_vertex)
        });
        let end_vertex = end_vertex.or_else(|| {
            oriented_root_edge_vertices(matched_edge, orientation).map(|(_, end_vertex)| end_vertex)
        });
        let (Some(start_vertex), Some(end_vertex)) = (start_vertex, end_vertex) else {
            return Ok(None);
        };

        edge_indices.push(matches[0]);
        edge_orientations.push(orientation);
        if ordered_vertices.is_empty() {
            ordered_vertices.push(start_vertex);
        } else if *ordered_vertices.last().unwrap_or(&start_vertex) != start_vertex {
            return Ok(None);
        }
        ordered_vertices.push(end_vertex);
    }

    Ok(Some(RootWireTopology {
        edge_indices,
        edge_orientations,
        vertex_indices: ordered_vertices,
    }))
}

fn pack_wire_topology(
    root_wires: &[RootWireTopology],
) -> (
    Vec<crate::TopologyRange>,
    Vec<usize>,
    Vec<Orientation>,
    Vec<crate::TopologyRange>,
    Vec<usize>,
) {
    let mut wires = Vec::with_capacity(root_wires.len());
    let mut wire_edge_indices = Vec::new();
    let mut wire_edge_orientations = Vec::new();
    let mut wire_vertices = Vec::with_capacity(root_wires.len());
    let mut wire_vertex_indices = Vec::new();

    for wire in root_wires {
        wires.push(crate::TopologyRange {
            offset: wire_edge_indices.len(),
            count: wire.edge_indices.len(),
        });
        wire_edge_indices.extend(&wire.edge_indices);
        wire_edge_orientations.extend(&wire.edge_orientations);
        wire_vertices.push(crate::TopologyRange {
            offset: wire_vertex_indices.len(),
            count: wire.vertex_indices.len(),
        });
        wire_vertex_indices.extend(&wire.vertex_indices);
    }

    (
        wires,
        wire_edge_indices,
        wire_edge_orientations,
        wire_vertices,
        wire_vertex_indices,
    )
}

struct PortedFaceTopology {
    edge_indices: BTreeSet<usize>,
    face_wire_indices: Vec<usize>,
    face_wire_orientations: Vec<Orientation>,
    face_wire_roles: Vec<LoopRole>,
}

fn ported_face_topology(
    context: &Context,
    face_shape: &Shape,
    root_wires: &[RootWireTopology],
    root_edges: &[RootEdgeTopology],
    edge_shapes: &[Shape],
    vertex_positions: &[[f64; 3]],
) -> Result<Option<PortedFaceTopology>, Error> {
    let face_wire_shapes = context.subshapes_occt(face_shape, ShapeKind::Wire)?;
    if root_wires.is_empty() || face_wire_shapes.is_empty() {
        return Ok(None);
    }

    let mut used_root_wire_indices = BTreeSet::new();
    let mut used_edges = BTreeSet::new();
    let mut face_wire_indices = Vec::with_capacity(face_wire_shapes.len());
    let mut face_wire_orientations = Vec::with_capacity(face_wire_shapes.len());
    let mut face_wire_areas = Vec::new();

    let mut planar_face = None;
    if face_wire_shapes.len() > 1 {
        let face_geometry = context.face_geometry_occt(face_shape)?;
        if face_geometry.kind != crate::SurfaceKind::Plane {
            return Ok(None);
        }
        planar_face = Some((context.face_plane_payload_occt(face_shape)?, face_geometry));
    }

    for face_wire_shape in &face_wire_shapes {
        let Some(face_wire_topology) =
            root_wire_topology(context, face_wire_shape, vertex_positions, root_edges)?
        else {
            return Ok(None);
        };
        let Some(root_wire_index) =
            match_root_wire_index(root_wires, &face_wire_topology, &used_root_wire_indices)
        else {
            return Ok(None);
        };
        used_root_wire_indices.insert(root_wire_index);
        used_edges.extend(face_wire_topology.edge_indices.iter().copied());

        face_wire_indices.push(root_wire_index);
        face_wire_orientations.push(context.shape_orientation(face_wire_shape)?);

        if let Some((plane, face_geometry)) = planar_face {
            let Some(wire_area) = planar_wire_area_magnitude(
                context,
                plane,
                face_geometry,
                &root_wires[root_wire_index],
                edge_shapes,
                root_edges,
            )?
            else {
                return Ok(None);
            };
            face_wire_areas.push(wire_area);
        }
    }

    let face_wire_roles = match face_wire_shapes.len() {
        1 => vec![LoopRole::Outer],
        _ => {
            let Some((outer_offset, outer_area)) = face_wire_areas
                .iter()
                .copied()
                .enumerate()
                .max_by(|(_, lhs), (_, rhs)| lhs.total_cmp(rhs))
            else {
                return Ok(None);
            };
            if outer_area <= 1.0e-9 {
                return Ok(None);
            }

            face_wire_areas
                .iter()
                .enumerate()
                .map(|(offset, _)| {
                    if offset == outer_offset {
                        LoopRole::Outer
                    } else {
                        LoopRole::Inner
                    }
                })
                .collect()
        }
    };

    Ok(Some(PortedFaceTopology {
        edge_indices: used_edges,
        face_wire_indices,
        face_wire_orientations,
        face_wire_roles,
    }))
}

fn match_root_wire_index(
    root_wires: &[RootWireTopology],
    face_wire_topology: &RootWireTopology,
    used_root_wire_indices: &BTreeSet<usize>,
) -> Option<usize> {
    root_wires
        .iter()
        .enumerate()
        .find(|(index, root_wire)| {
            !used_root_wire_indices.contains(index) && *root_wire == face_wire_topology
        })
        .map(|(index, _)| index)
}

fn planar_wire_area_magnitude(
    context: &Context,
    plane: PlanePayload,
    face_geometry: FaceGeometry,
    wire: &RootWireTopology,
    edge_shapes: &[Shape],
    root_edges: &[RootEdgeTopology],
) -> Result<Option<f64>, Error> {
    let mut curve_segments = Vec::with_capacity(wire.edge_indices.len());
    let mut sampled_points = Vec::new();

    for (&edge_index, &edge_orientation) in wire.edge_indices.iter().zip(&wire.edge_orientations) {
        let Some(root_edge) = root_edges.get(edge_index) else {
            return Ok(None);
        };
        let Some(edge_shape) = edge_shapes.get(edge_index) else {
            return Ok(None);
        };

        let geometry = oriented_edge_geometry(root_edge.geometry, edge_orientation);
        if let Some(curve) =
            PortedCurve::from_context_with_geometry(context, edge_shape, root_edge.geometry)?
        {
            curve_segments.push((curve, geometry));
        }

        append_root_edge_sample_points(
            context,
            edge_shape,
            root_edge,
            geometry,
            &mut sampled_points,
        )?;
    }

    let area = if curve_segments.len() == wire.edge_indices.len() {
        planar_wire_signed_area(plane, &curve_segments).abs()
    } else {
        let Some(area) = analytic_sampled_wire_signed_area(
            PortedSurface::Plane(plane),
            face_geometry,
            &sampled_points,
        ) else {
            return Ok(None);
        };
        area.abs()
    };
    Ok(Some(area))
}

fn wire_occurrence(
    context: &Context,
    edge_shape: &Shape,
    vertex_positions: &[[f64; 3]],
    root_edges: &[RootEdgeTopology],
) -> Result<Option<WireOccurrence>, Error> {
    let geometry = context.edge_geometry_occt(edge_shape)?;
    let endpoints = context.edge_endpoints_occt(edge_shape)?;
    let Some(mut start_vertex) = match_vertex_index(vertex_positions, endpoints.start) else {
        return Ok(None);
    };
    let Some(mut end_vertex) = match_vertex_index(vertex_positions, endpoints.end) else {
        return Ok(None);
    };
    let orientation = context.shape_orientation(edge_shape)?;
    if matches!(orientation, Orientation::Reversed) {
        std::mem::swap(&mut start_vertex, &mut end_vertex);
    }
    let length = edge_length(edge_shape);
    let matches = root_edges
        .iter()
        .enumerate()
        .filter(|(_, root_edge)| {
            root_edge.geometry.kind == geometry.kind
                && approx_eq(root_edge.length, length, 1.0e-6, 1.0e-6)
                && matches_edge_vertices(root_edge, start_vertex, end_vertex)
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Ok(None);
    }

    Ok(Some(WireOccurrence {
        edge_index: matches[0],
        orientation,
        start_vertex,
        end_vertex,
    }))
}

fn order_wire_occurrences(
    occurrences: &[WireOccurrence],
) -> Option<(Vec<usize>, Vec<Orientation>, Vec<usize>)> {
    if occurrences.is_empty() {
        return Some((Vec::new(), Vec::new(), Vec::new()));
    }
    if let Some(vertices) =
        chain_wire_occurrences(occurrences, &(0..occurrences.len()).collect::<Vec<_>>())
    {
        return Some((
            occurrences
                .iter()
                .map(|occurrence| occurrence.edge_index)
                .collect(),
            occurrences
                .iter()
                .map(|occurrence| occurrence.orientation)
                .collect(),
            vertices,
        ));
    }

    let mut outgoing = BTreeMap::<usize, Vec<usize>>::new();
    let mut in_degree = BTreeMap::<usize, usize>::new();
    let mut out_degree = BTreeMap::<usize, usize>::new();
    for (index, occurrence) in occurrences.iter().enumerate() {
        outgoing
            .entry(occurrence.start_vertex)
            .or_default()
            .push(index);
        *out_degree.entry(occurrence.start_vertex).or_default() += 1;
        *in_degree.entry(occurrence.end_vertex).or_default() += 1;
    }

    let start_candidates = outgoing
        .keys()
        .copied()
        .filter(|vertex| {
            let outgoing = out_degree.get(vertex).copied().unwrap_or(0);
            let incoming = in_degree.get(vertex).copied().unwrap_or(0);
            outgoing == incoming + 1
        })
        .collect::<Vec<_>>();
    let start_vertex = match start_candidates.as_slice() {
        [start] => *start,
        [] => occurrences.first()?.start_vertex,
        _ => return None,
    };

    let mut used = vec![false; occurrences.len()];
    let mut ordered = Vec::with_capacity(occurrences.len());
    let mut current_vertex = start_vertex;
    while ordered.len() < occurrences.len() {
        let next = outgoing
            .get(&current_vertex)?
            .iter()
            .copied()
            .filter(|index| !used[*index])
            .collect::<Vec<_>>();
        if next.len() != 1 {
            return None;
        }
        let index = next[0];
        used[index] = true;
        ordered.push(index);
        current_vertex = occurrences[index].end_vertex;
    }

    let ordered_vertices = chain_wire_occurrences(occurrences, &ordered)?;
    Some((
        ordered
            .iter()
            .map(|&index| occurrences[index].edge_index)
            .collect(),
        ordered
            .iter()
            .map(|&index| occurrences[index].orientation)
            .collect(),
        ordered_vertices,
    ))
}

fn chain_wire_occurrences(occurrences: &[WireOccurrence], ordered: &[usize]) -> Option<Vec<usize>> {
    let &first = ordered.first()?;
    let mut vertices = vec![occurrences[first].start_vertex];
    let mut current_vertex = occurrences[first].end_vertex;
    vertices.push(current_vertex);
    for &index in ordered.iter().skip(1) {
        let occurrence = occurrences.get(index)?;
        if occurrence.start_vertex != current_vertex {
            return None;
        }
        current_vertex = occurrence.end_vertex;
        vertices.push(current_vertex);
    }
    Some(vertices)
}

fn matches_edge_vertices(
    root_edge: &RootEdgeTopology,
    start_vertex: usize,
    end_vertex: usize,
) -> bool {
    matches!(
        (root_edge.start_vertex, root_edge.end_vertex),
        (Some(root_start), Some(root_end))
            if (root_start == start_vertex && root_end == end_vertex)
                || (root_start == end_vertex && root_end == start_vertex)
    )
}

fn oriented_root_edge_vertices(
    root_edge: &RootEdgeTopology,
    orientation: Orientation,
) -> Option<(usize, usize)> {
    let start_vertex = root_edge.start_vertex?;
    let end_vertex = root_edge.end_vertex?;
    Some(match orientation {
        Orientation::Reversed => (end_vertex, start_vertex),
        _ => (start_vertex, end_vertex),
    })
}

fn topology_vertex_match(
    topology_vertices: &[[f64; 3]],
    root_vertices: &[[f64; 3]],
    index: usize,
) -> Option<usize> {
    topology_vertices
        .get(index)
        .copied()
        .and_then(|point| match_vertex_index(root_vertices, point))
}

fn edge_length(edge_shape: &Shape) -> f64 {
    edge_shape.linear_length()
}

fn match_vertex_index(vertex_positions: &[[f64; 3]], point: [f64; 3]) -> Option<usize> {
    let mut found = None;
    for (index, vertex_position) in vertex_positions.iter().copied().enumerate() {
        if approx_points_eq(vertex_position, point, 1.0e-7) {
            if found.is_some() {
                return None;
            }
            found = Some(index);
        }
    }
    found
}

pub(crate) fn ported_face_surface_descriptor(
    context: &Context,
    face_shape: &Shape,
    face_geometry: FaceGeometry,
) -> Result<Option<PortedFaceSurface>, Error> {
    let ported_surface =
        PortedSurface::from_context_with_geometry(context, face_shape, face_geometry)?;
    ported_face_surface_descriptor_from_surface(context, face_shape, face_geometry, ported_surface)
}

fn ported_face_surface_descriptor_from_surface(
    context: &Context,
    face_shape: &Shape,
    face_geometry: FaceGeometry,
    ported_surface: Option<PortedSurface>,
) -> Result<Option<PortedFaceSurface>, Error> {
    if let Some(surface) = ported_surface {
        return Ok(Some(PortedFaceSurface::Analytic(surface)));
    }

    if let Some(surface) = context.ported_offset_surface(face_shape)? {
        return Ok(Some(PortedFaceSurface::Offset(surface)));
    }

    Ok(
        ported_swept_face_surface(context, face_shape, face_geometry)?
            .map(PortedFaceSurface::Swept),
    )
}

pub(crate) fn ported_swept_face_surface(
    context: &Context,
    face_shape: &Shape,
    face_geometry: FaceGeometry,
) -> Result<Option<PortedSweptSurface>, Error> {
    let topology = match single_face_topology(context, face_shape)? {
        Some(topology) => topology,
        None => return Ok(None),
    };

    match face_geometry.kind {
        crate::SurfaceKind::Extrusion => {
            let payload = context.face_extrusion_payload_occt(face_shape)?;
            let basis = select_swept_face_basis_curve(
                face_curve_candidates(
                    &topology.loops,
                    &topology.wires,
                    &topology.edges,
                    payload.basis_curve_kind,
                )
                .ok_or_else(|| {
                    Error::new("failed to identify a Rust-owned basis curve for extrusion face")
                })?,
                face_geometry,
                SweptBasisSelection::Extrusion {
                    direction: payload.direction,
                },
            )
            .ok_or_else(|| {
                Error::new("failed to select a Rust-owned basis curve for extrusion face")
            })?;
            Ok(Some(PortedSweptSurface::Extrusion {
                payload,
                basis_curve: basis.curve,
                basis_geometry: basis.geometry,
            }))
        }
        crate::SurfaceKind::Revolution => {
            let payload = context.face_revolution_payload_occt(face_shape)?;
            let basis = select_swept_face_basis_curve(
                face_curve_candidates(
                    &topology.loops,
                    &topology.wires,
                    &topology.edges,
                    payload.basis_curve_kind,
                )
                .ok_or_else(|| {
                    Error::new("failed to identify a Rust-owned basis curve for revolution face")
                })?,
                face_geometry,
                SweptBasisSelection::Revolution {
                    axis_origin: payload.axis_origin,
                    axis_direction: payload.axis_direction,
                },
            )
            .ok_or_else(|| {
                Error::new("failed to select a Rust-owned basis curve for revolution face")
            })?;
            Ok(Some(PortedSweptSurface::Revolution {
                payload,
                basis_curve: basis.curve,
                basis_geometry: basis.geometry,
            }))
        }
        _ => Ok(None),
    }
}

pub(crate) fn ported_face_area(
    context: &Context,
    face_shape: &Shape,
) -> Result<Option<f64>, Error> {
    let face_geometry = context.face_geometry(face_shape)?;
    let topology = match single_face_topology(context, face_shape)? {
        Some(topology) => topology,
        None => return Ok(None),
    };

    let ported_surface =
        PortedSurface::from_context_with_geometry(context, face_shape, face_geometry)?;
    let ported_face_surface = ported_face_surface_descriptor_from_surface(
        context,
        face_shape,
        face_geometry,
        ported_surface,
    )?;

    Ok(match ported_face_surface {
        Some(PortedFaceSurface::Analytic(surface)) => analytic_face_area(
            context,
            surface,
            face_geometry,
            &topology.loops,
            &topology.wires,
            &topology.edges,
            &topology.edge_shapes,
        ),
        Some(PortedFaceSurface::Swept(surface)) => {
            analytic_ported_swept_face_area(surface, face_geometry)
        }
        Some(PortedFaceSurface::Offset(surface)) => analytic_offset_face_area(
            context,
            surface,
            face_geometry,
            &topology.loops,
            &topology.wires,
            &topology.edges,
            &topology.edge_shapes,
        ),
        None => None,
    })
}

fn single_face_topology(
    context: &Context,
    face_shape: &Shape,
) -> Result<Option<SingleFaceTopology>, Error> {
    let topology = match context.ported_topology(face_shape)? {
        Some(topology) => topology,
        None => context.topology_occt(face_shape)?,
    };
    if topology.faces.len() != 1 {
        return Ok(None);
    }

    let wires = topology
        .wires
        .iter()
        .enumerate()
        .map(|(index, range)| {
            let edge_indices =
                topology.wire_edge_indices[range.offset..range.offset + range.count].to_vec();
            let edge_orientations =
                topology.wire_edge_orientations[range.offset..range.offset + range.count].to_vec();
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

    let edge_shapes = context.subshapes_occt(face_shape, ShapeKind::Edge)?;
    let edges = edge_shapes
        .iter()
        .enumerate()
        .map(|(index, edge_shape)| {
            let geometry = context.edge_geometry_occt(edge_shape)?;
            let ported_curve =
                PortedCurve::from_context_with_geometry(context, edge_shape, geometry)?;
            Ok(BrepEdge {
                index,
                geometry,
                ported_curve,
                length: 0.0,
                start_vertex: None,
                end_vertex: None,
                start_point: None,
                end_point: None,
                adjacent_face_indices: Vec::new(),
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;

    Ok(Some(SingleFaceTopology {
        loops: face_loops(&topology, 0)?,
        wires,
        edges,
        edge_shapes,
    }))
}

fn ported_shape_summary(
    context: &Context,
    shape: &Shape,
    vertices: &[BrepVertex],
    topology: &TopologySnapshot,
    wires: &[BrepWire],
    edges: &[BrepEdge],
    faces: &[BrepFace],
    face_shapes: &[Shape],
    edge_shapes: &[Shape],
) -> Result<ShapeSummary, Error> {
    let counts = shape_counts(context, shape, topology)?;
    let root_kind = classify_root_kind(counts);
    let primary_kind = classify_primary_kind(counts);
    let exact_primitive =
        exact_primitive_shape_summary(primary_kind, counts.solid_count, vertices, edges, faces);
    let fallback_summary = || context.describe_shape_occt(shape).ok();
    let (bbox_min, bbox_max) = exact_primitive
        .and_then(|summary| summary.bbox)
        .or_else(|| ported_shape_bbox(context, shape, vertices, edges, faces))
        .or_else(|| fallback_summary().map(|summary| (summary.bbox_min, summary.bbox_max)))
        .unwrap_or(([0.0; 3], [0.0; 3]));

    Ok(ShapeSummary {
        root_kind,
        primary_kind,
        compound_count: counts.compound_count,
        compsolid_count: counts.compsolid_count,
        solid_count: counts.solid_count,
        shell_count: counts.shell_count,
        face_count: counts.face_count,
        wire_count: counts.wire_count,
        edge_count: counts.edge_count,
        vertex_count: counts.vertex_count,
        // Match OCCT's whole-shape linear properties semantics: when wires are present,
        // length is accumulated over wire-edge occurrences rather than unique topological edges.
        linear_length: if topology.wire_edge_indices.is_empty() {
            edges.iter().map(|edge| edge.length).sum()
        } else {
            topology
                .wire_edge_indices
                .iter()
                .filter_map(|&edge_index| edges.get(edge_index))
                .map(|edge| edge.length)
                .sum()
        },
        surface_area: exact_primitive
            .map(|summary| summary.surface_area)
            .unwrap_or_else(|| faces.iter().map(|face| face.area).sum()),
        volume: exact_primitive
            .map(|summary| summary.volume)
            .or_else(|| {
                analytic_shape_volume(context, wires, edges, faces, face_shapes, edge_shapes)
            })
            .or_else(|| mesh_shape_volume(context, shape, counts))
            .or_else(|| fallback_summary().map(|summary| summary.volume))
            .unwrap_or(0.0),
        bbox_min,
        bbox_max,
    })
}

fn exact_primitive_shape_summary(
    primary_kind: ShapeKind,
    solid_count: usize,
    vertices: &[BrepVertex],
    edges: &[BrepEdge],
    faces: &[BrepFace],
) -> Option<ExactPrimitiveSummary> {
    if primary_kind != ShapeKind::Solid || solid_count != 1 {
        return None;
    }

    exact_box_summary(vertices, edges, faces)
        .or_else(|| exact_cylinder_summary(faces))
        .or_else(|| exact_cone_summary(faces))
        .or_else(|| exact_sphere_summary(faces))
        .or_else(|| exact_torus_summary(faces))
        .or_else(|| exact_translational_solid_summary(faces, edges))
}

fn exact_box_summary(
    vertices: &[BrepVertex],
    edges: &[BrepEdge],
    faces: &[BrepFace],
) -> Option<ExactPrimitiveSummary> {
    if vertices.len() != 8 || edges.len() != 12 || faces.len() != 6 {
        return None;
    }
    if !edges
        .iter()
        .all(|edge| matches!(edge.ported_curve, Some(PortedCurve::Line(_))))
    {
        return None;
    }
    if !faces
        .iter()
        .all(|face| matches!(face.ported_surface, Some(PortedSurface::Plane(_))))
    {
        return None;
    }
    let bbox = bbox_from_points(vertices.iter().map(|vertex| vertex.position).collect())?;

    for vertex in vertices {
        let incident = edges
            .iter()
            .filter_map(|edge| incident_edge_vector(edge, vertex.index))
            .collect::<Vec<_>>();
        if incident.len() < 3 {
            continue;
        }

        for i in 0..incident.len() {
            for j in i + 1..incident.len() {
                for k in j + 1..incident.len() {
                    let a = incident[i];
                    let b = incident[j];
                    let c = incident[k];
                    let volume = dot3(a, cross3(b, c)).abs();
                    if volume <= 1.0e-9 {
                        continue;
                    }
                    let surface_area =
                        2.0 * (norm3(cross3(a, b)) + norm3(cross3(a, c)) + norm3(cross3(b, c)));
                    return Some(ExactPrimitiveSummary {
                        surface_area,
                        volume,
                        bbox: Some(bbox),
                    });
                }
            }
        }
    }

    None
}

fn exact_cylinder_summary(faces: &[BrepFace]) -> Option<ExactPrimitiveSummary> {
    if faces.len() != 3 {
        return None;
    }

    let (payload, _) = single_cylinder_face(faces)?;
    let axis = normalize3(payload.axis);
    if norm3(axis) <= 1.0e-12 {
        return None;
    }
    let caps = aligned_plane_faces(faces, axis);
    if caps.len() != 2 {
        return None;
    }

    let height = (dot3(subtract3(caps[0].origin, payload.origin), axis)
        - dot3(subtract3(caps[1].origin, payload.origin), axis))
    .abs();
    let radius = payload.radius.abs();
    let bbox = circular_sections_bbox(
        axis,
        &caps
            .iter()
            .map(|plane| {
                let axial = dot3(subtract3(plane.origin, payload.origin), axis);
                (add3(payload.origin, scale3(axis, axial)), radius)
            })
            .collect::<Vec<_>>(),
    );
    Some(ExactPrimitiveSummary {
        surface_area: 2.0 * PI * radius * (height + radius),
        volume: PI * radius * radius * height,
        bbox,
    })
}

fn exact_cone_summary(faces: &[BrepFace]) -> Option<ExactPrimitiveSummary> {
    if !(2..=3).contains(&faces.len()) {
        return None;
    }

    let (payload, _) = single_cone_face(faces)?;
    let (axis, axial_radii) = exact_cone_axial_radii(payload, faces)?;
    if axial_radii.len() != 2 {
        return None;
    }

    let (axial0, radius0) = axial_radii[0];
    let (axial1, radius1) = axial_radii[1];
    let height = (axial0 - axial1).abs();
    let slant = ((radius0 - radius1).powi(2) + height.powi(2)).sqrt();
    let bbox = circular_sections_bbox(
        axis,
        &axial_radii
            .iter()
            .map(|&(axial, radius)| (add3(payload.origin, scale3(axis, axial)), radius))
            .collect::<Vec<_>>(),
    );

    Some(ExactPrimitiveSummary {
        surface_area: PI * (radius0 + radius1) * slant
            + PI * (radius0 * radius0 + radius1 * radius1),
        volume: PI * height * (radius0 * radius0 + radius0 * radius1 + radius1 * radius1) / 3.0,
        bbox,
    })
}

fn exact_sphere_summary(faces: &[BrepFace]) -> Option<ExactPrimitiveSummary> {
    let (payload, _) = single_sphere_face(faces)?;
    let radius = payload.radius.abs();
    Some(ExactPrimitiveSummary {
        surface_area: 4.0 * PI * radius * radius,
        volume: 4.0 * PI * radius * radius * radius / 3.0,
        bbox: Some((
            [
                payload.center[0] - radius,
                payload.center[1] - radius,
                payload.center[2] - radius,
            ],
            [
                payload.center[0] + radius,
                payload.center[1] + radius,
                payload.center[2] + radius,
            ],
        )),
    })
}

fn exact_torus_summary(faces: &[BrepFace]) -> Option<ExactPrimitiveSummary> {
    let (payload, _) = single_torus_face(faces)?;
    let major_radius = payload.major_radius.abs();
    let minor_radius = payload.minor_radius.abs();
    Some(ExactPrimitiveSummary {
        surface_area: 4.0 * PI * PI * major_radius * minor_radius,
        volume: 2.0 * PI * PI * major_radius * minor_radius * minor_radius,
        bbox: None,
    })
}

fn exact_translational_solid_summary(
    faces: &[BrepFace],
    edges: &[BrepEdge],
) -> Option<ExactPrimitiveSummary> {
    let plane_faces = faces
        .iter()
        .filter_map(|face| match face.ported_surface {
            Some(PortedSurface::Plane(payload)) => Some((face, payload)),
            _ => None,
        })
        .collect::<Vec<_>>();
    if plane_faces.len() < 2 {
        return None;
    }

    for i in 0..plane_faces.len() {
        for j in i + 1..plane_faces.len() {
            let (lhs_face, lhs_plane) = plane_faces[i];
            let (rhs_face, rhs_plane) = plane_faces[j];
            let lhs_normal = normalize3(lhs_plane.normal);
            let rhs_normal = normalize3(rhs_plane.normal);
            if dot3(lhs_normal, rhs_normal) > -1.0 + 1.0e-6 {
                continue;
            }
            if !approx_eq(lhs_face.area, rhs_face.area, 1.0e-6, 1.0e-6) {
                continue;
            }
            if !pair_forms_translational_caps(faces, lhs_face.index, rhs_face.index) {
                continue;
            }

            let span = dot3(subtract3(rhs_plane.origin, lhs_plane.origin), lhs_normal).abs();
            if span <= 1.0e-9 {
                continue;
            }

            return Some(ExactPrimitiveSummary {
                surface_area: faces.iter().map(|face| face.area).sum(),
                volume: lhs_face.area * span,
                bbox: analytic_edges_bbox(edges),
            });
        }
    }

    None
}

fn exact_cone_axial_radii(
    payload: ConePayload,
    faces: &[BrepFace],
) -> Option<([f64; 3], Vec<(f64, f64)>)> {
    let axis = normalize3(payload.axis);
    if norm3(axis) <= 1.0e-12 {
        return None;
    }
    let caps = aligned_plane_faces(faces, axis);
    if caps.is_empty() || caps.len() > 2 || caps.len() + 1 != faces.len() {
        return None;
    }

    let tan_angle = payload.semi_angle.tan();
    if tan_angle.abs() <= 1.0e-12 {
        return None;
    }

    let mut axial_radii = caps
        .iter()
        .map(|plane| {
            let axial = dot3(subtract3(plane.origin, payload.origin), axis);
            let radius = (payload.reference_radius + axial * tan_angle).abs();
            (axial, radius)
        })
        .collect::<Vec<_>>();
    if axial_radii.len() == 1 {
        axial_radii.push((-payload.reference_radius / tan_angle, 0.0));
    }

    (axial_radii.len() == 2).then_some((axis, axial_radii))
}

fn circular_sections_bbox(
    axis: [f64; 3],
    sections: &[([f64; 3], f64)],
) -> Option<([f64; 3], [f64; 3])> {
    if sections.is_empty() {
        return None;
    }

    let axis = normalize3(axis);
    if norm3(axis) <= 1.0e-12 {
        return None;
    }

    let mut bbox_min = [f64::INFINITY; 3];
    let mut bbox_max = [f64::NEG_INFINITY; 3];
    for (center, radius) in sections {
        for coordinate in 0..3 {
            let radial_extent =
                radius.abs() * (1.0 - axis[coordinate] * axis[coordinate]).max(0.0).sqrt();
            bbox_min[coordinate] = bbox_min[coordinate].min(center[coordinate] - radial_extent);
            bbox_max[coordinate] = bbox_max[coordinate].max(center[coordinate] + radial_extent);
        }
    }
    Some((bbox_min, bbox_max))
}

fn single_cylinder_face(faces: &[BrepFace]) -> Option<(CylinderPayload, usize)> {
    single_surface_face(faces, |surface| match surface {
        PortedSurface::Cylinder(payload) => Some(payload),
        _ => None,
    })
}

fn single_cone_face(faces: &[BrepFace]) -> Option<(ConePayload, usize)> {
    single_surface_face(faces, |surface| match surface {
        PortedSurface::Cone(payload) => Some(payload),
        _ => None,
    })
}

fn single_sphere_face(faces: &[BrepFace]) -> Option<(SpherePayload, usize)> {
    if faces.len() != 1 {
        return None;
    }
    single_surface_face(faces, |surface| match surface {
        PortedSurface::Sphere(payload) => Some(payload),
        _ => None,
    })
}

fn single_torus_face(faces: &[BrepFace]) -> Option<(TorusPayload, usize)> {
    if faces.len() != 1 {
        return None;
    }
    single_surface_face(faces, |surface| match surface {
        PortedSurface::Torus(payload) => Some(payload),
        _ => None,
    })
}

fn single_surface_face<T>(
    faces: &[BrepFace],
    extract: impl Fn(PortedSurface) -> Option<T>,
) -> Option<(T, usize)> {
    let mut found = None;
    for face in faces {
        let Some(surface) = face.ported_surface else {
            return None;
        };
        let Some(payload) = extract(surface) else {
            continue;
        };
        if found.is_some() {
            return None;
        }
        found = Some((payload, face.index));
    }
    found
}

fn aligned_plane_faces(faces: &[BrepFace], axis: [f64; 3]) -> Vec<PlanePayload> {
    faces
        .iter()
        .filter_map(|face| match face.ported_surface {
            Some(PortedSurface::Plane(payload))
                if dot3(normalize3(payload.normal), axis).abs() >= 1.0 - 1.0e-6 =>
            {
                Some(payload)
            }
            _ => None,
        })
        .collect()
}

fn pair_forms_translational_caps(faces: &[BrepFace], lhs_index: usize, rhs_index: usize) -> bool {
    faces.iter().all(|candidate| {
        if candidate.index == lhs_index || candidate.index == rhs_index {
            return true;
        }
        candidate.adjacent_face_indices.contains(&lhs_index)
            && candidate.adjacent_face_indices.contains(&rhs_index)
    })
}

fn incident_edge_vector(edge: &BrepEdge, vertex_index: usize) -> Option<[f64; 3]> {
    match (
        edge.start_vertex,
        edge.end_vertex,
        edge.start_point,
        edge.end_point,
    ) {
        (Some(start), Some(_), Some(start_point), Some(end_point)) if start == vertex_index => {
            Some(subtract3(end_point, start_point))
        }
        (Some(_), Some(end), Some(start_point), Some(end_point)) if end == vertex_index => {
            Some(subtract3(start_point, end_point))
        }
        _ => None,
    }
}

fn analytic_shape_volume(
    context: &Context,
    wires: &[BrepWire],
    edges: &[BrepEdge],
    faces: &[BrepFace],
    face_shapes: &[Shape],
    edge_shapes: &[Shape],
) -> Option<f64> {
    if faces.is_empty() {
        return Some(0.0);
    }

    let mut volume = 0.0;
    for face in faces {
        let face_shape = face_shapes.get(face.index)?;
        let analytic_contribution = match face.ported_face_surface {
            Some(PortedFaceSurface::Analytic(surface)) => analytic_face_volume(
                context,
                face,
                surface,
                face.geometry,
                &face.loops,
                wires,
                edges,
                edge_shapes,
            ),
            Some(PortedFaceSurface::Swept(surface)) => {
                analytic_ported_swept_face_volume(face, face.geometry, surface)
            }
            Some(PortedFaceSurface::Offset(surface)) => analytic_offset_face_volume(
                context,
                face,
                surface,
                face.geometry,
                &face.loops,
                wires,
                edges,
                edge_shapes,
            ),
            None => None,
        };
        let contribution =
            analytic_contribution.or_else(|| mesh_face_volume(context, face_shape, face))?;
        volume += contribution;
    }
    Some(volume.abs())
}

fn mesh_shape_volume(context: &Context, shape: &Shape, counts: ShapeCounts) -> Option<f64> {
    if counts.solid_count == 0 && counts.compsolid_count == 0 {
        return Some(0.0);
    }

    let mesh = context.mesh(shape, SUMMARY_VOLUME_MESH_PARAMS).ok()?;
    polyhedral_mesh_volume(&mesh)
}

fn ported_shape_bbox(
    context: &Context,
    shape: &Shape,
    vertices: &[BrepVertex],
    edges: &[BrepEdge],
    faces: &[BrepFace],
) -> Option<([f64; 3], [f64; 3])> {
    topological_shape_bbox(vertices, edges, faces).or_else(|| mesh_shape_bbox(context, shape))
}

fn shape_counts(
    context: &Context,
    shape: &Shape,
    topology: &TopologySnapshot,
) -> Result<ShapeCounts, Error> {
    Ok(ShapeCounts {
        compound_count: context.subshape_count_occt(shape, ShapeKind::Compound)?,
        compsolid_count: context.subshape_count_occt(shape, ShapeKind::CompSolid)?,
        solid_count: context.subshape_count_occt(shape, ShapeKind::Solid)?,
        shell_count: context.subshape_count_occt(shape, ShapeKind::Shell)?,
        face_count: topology.faces.len(),
        wire_count: topology.wires.len(),
        edge_count: topology.edges.len(),
        vertex_count: topology.vertex_positions.len(),
    })
}

fn classify_root_kind(counts: ShapeCounts) -> ShapeKind {
    if counts.compound_count > 0 {
        ShapeKind::Compound
    } else if counts.compsolid_count > 0 {
        ShapeKind::CompSolid
    } else if counts.solid_count > 0 {
        ShapeKind::Solid
    } else if counts.shell_count > 0 {
        ShapeKind::Shell
    } else if counts.face_count > 0 {
        ShapeKind::Face
    } else if counts.wire_count > 0 {
        ShapeKind::Wire
    } else if counts.edge_count > 0 {
        ShapeKind::Edge
    } else if counts.vertex_count > 0 {
        ShapeKind::Vertex
    } else {
        ShapeKind::Unknown
    }
}

fn classify_primary_kind(counts: ShapeCounts) -> ShapeKind {
    if counts.solid_count > 0 {
        ShapeKind::Solid
    } else if counts.shell_count > 0 {
        ShapeKind::Shell
    } else if counts.face_count > 0 {
        ShapeKind::Face
    } else if counts.wire_count > 0 {
        ShapeKind::Wire
    } else if counts.edge_count > 0 {
        ShapeKind::Edge
    } else if counts.vertex_count > 0 {
        ShapeKind::Vertex
    } else if counts.compsolid_count > 0 {
        ShapeKind::CompSolid
    } else if counts.compound_count > 0 {
        ShapeKind::Compound
    } else {
        ShapeKind::Unknown
    }
}

fn topological_shape_bbox(
    vertices: &[BrepVertex],
    edges: &[BrepEdge],
    faces: &[BrepFace],
) -> Option<([f64; 3], [f64; 3])> {
    if faces.is_empty() {
        return analytic_edges_bbox(edges)
            .or_else(|| line_segment_points_bbox(vertices, edges))
            .or_else(|| {
                if edges.is_empty() {
                    bbox_from_points(vertices.iter().map(|vertex| vertex.position).collect())
                } else {
                    None
                }
            });
    }

    if faces
        .iter()
        .all(|face| matches!(face.ported_surface, Some(PortedSurface::Plane(_))))
    {
        return analytic_edges_bbox(edges).or_else(|| line_segment_points_bbox(vertices, edges));
    }

    None
}

fn line_segment_points_bbox(
    vertices: &[BrepVertex],
    edges: &[BrepEdge],
) -> Option<([f64; 3], [f64; 3])> {
    if edges.is_empty()
        || !edges.iter().all(|edge| {
            matches!(edge.ported_curve, Some(PortedCurve::Line(_)))
                && edge.start_point.is_some()
                && edge.end_point.is_some()
        })
    {
        return None;
    }

    let mut points = vertices
        .iter()
        .map(|vertex| vertex.position)
        .collect::<Vec<_>>();
    for edge in edges {
        if let Some(start_point) = edge.start_point {
            points.push(start_point);
        }
        if let Some(end_point) = edge.end_point {
            points.push(end_point);
        }
    }

    bbox_from_points(points)
}

fn analytic_edges_bbox(edges: &[BrepEdge]) -> Option<([f64; 3], [f64; 3])> {
    let mut bbox = None;
    for edge in edges {
        let edge_bbox = analytic_edge_bbox(edge)?;
        bbox = Some(match bbox {
            Some(accumulated) => union_bbox(accumulated, edge_bbox),
            None => edge_bbox,
        });
    }
    bbox
}

fn analytic_edge_bbox(edge: &BrepEdge) -> Option<([f64; 3], [f64; 3])> {
    let curve = edge.ported_curve?;
    let start = edge.geometry.start_parameter;
    let end = edge.geometry.end_parameter;

    match curve {
        PortedCurve::Line(_) => bbox_from_points(vec![
            curve.evaluate(start).position,
            curve.evaluate(end).position,
        ]),
        PortedCurve::Circle(payload) => periodic_curve_bbox(
            start,
            end,
            2.0 * PI,
            |axis| {
                (
                    payload.center[axis],
                    payload.radius * payload.x_direction[axis],
                    payload.radius * payload.y_direction[axis],
                )
            },
            |parameter| curve.evaluate(parameter).position,
        ),
        PortedCurve::Ellipse(payload) => periodic_curve_bbox(
            start,
            end,
            2.0 * PI,
            |axis| {
                (
                    payload.center[axis],
                    payload.major_radius * payload.x_direction[axis],
                    payload.minor_radius * payload.y_direction[axis],
                )
            },
            |parameter| curve.evaluate(parameter).position,
        ),
    }
}

fn periodic_curve_bbox(
    start: f64,
    end: f64,
    period: f64,
    coefficients_for_axis: impl Fn(usize) -> (f64, f64, f64),
    position_at: impl Fn(f64) -> [f64; 3],
) -> Option<([f64; 3], [f64; 3])> {
    let mut parameters = vec![start, end];
    for axis in 0..3 {
        let (_, cos_coefficient, sin_coefficient) = coefficients_for_axis(axis);
        if cos_coefficient.abs() <= 1.0e-12 && sin_coefficient.abs() <= 1.0e-12 {
            continue;
        }
        let critical = sin_coefficient.atan2(cos_coefficient);
        extend_periodic_parameters(&mut parameters, start, end, period, critical);
        extend_periodic_parameters(&mut parameters, start, end, period, critical + PI);
    }
    bbox_from_points(parameters.into_iter().map(position_at).collect())
}

fn extend_periodic_parameters(
    parameters: &mut Vec<f64>,
    start: f64,
    end: f64,
    period: f64,
    base: f64,
) {
    let low = start.min(end);
    let high = start.max(end);
    let first_multiple = ((low - base - 1.0e-12) / period).ceil() as i64;
    let last_multiple = ((high - base + 1.0e-12) / period).floor() as i64;
    for multiple in first_multiple..=last_multiple {
        parameters.push(base + multiple as f64 * period);
    }
}

fn mesh_shape_bbox(context: &Context, shape: &Shape) -> Option<([f64; 3], [f64; 3])> {
    let mesh = context.mesh(shape, SUMMARY_BBOX_MESH_PARAMS).ok()?;
    mesh_bbox(&mesh)
}

fn mesh_face_properties(
    context: &Context,
    face_shape: &Shape,
    orientation: Orientation,
) -> Option<MeshFaceProperties> {
    let mesh = context
        .mesh(face_shape, UNSUPPORTED_FACE_AREA_MESH_PARAMS)
        .ok()?;
    let mut sample = polyhedral_mesh_sample(&mesh)?;
    if matches!(orientation, Orientation::Reversed) {
        sample.normal = scale3(sample.normal, -1.0);
    }
    Some(MeshFaceProperties {
        area: polyhedral_mesh_area(&mesh)?,
        sample,
    })
}

fn mesh_face_volume(context: &Context, face_shape: &Shape, face: &BrepFace) -> Option<f64> {
    let mesh = context.mesh(face_shape, SUMMARY_VOLUME_MESH_PARAMS).ok()?;
    mesh_face_signed_volume(&mesh, face.sample.normal)
}

fn mesh_face_signed_volume(mesh: &Mesh, outward_normal_hint: [f64; 3]) -> Option<f64> {
    if mesh.triangle_indices.is_empty() {
        return Some(0.0);
    }

    let orientation_sign = polyhedral_mesh_sample(mesh)
        .map(|sample| {
            if dot3(sample.normal, outward_normal_hint) >= 0.0 {
                1.0
            } else {
                -1.0
            }
        })
        .unwrap_or(1.0);

    let mut signed_volume = 0.0;
    for triangle in mesh.triangle_indices.chunks_exact(3) {
        let i0 = usize::try_from(triangle[0]).ok()?;
        let i1 = usize::try_from(triangle[1]).ok()?;
        let i2 = usize::try_from(triangle[2]).ok()?;
        let a = *mesh.positions.get(i0)?;
        let b = *mesh.positions.get(i1)?;
        let c = *mesh.positions.get(i2)?;
        signed_volume += dot3(a, cross3(b, c)) / 6.0;
    }

    Some(orientation_sign * signed_volume)
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
    if let Some(area) = exact_closed_face_area(surface, face_geometry) {
        return Some(area);
    }

    if loops.is_empty() {
        return match surface {
            PortedSurface::Sphere(payload) => Some(4.0 * PI * payload.radius.abs().powi(2)),
            PortedSurface::Torus(payload) => {
                Some(4.0 * PI * PI * payload.major_radius.abs() * payload.minor_radius.abs())
            }
            _ => Some(0.0),
        };
    }

    let plane = match surface {
        PortedSurface::Plane(plane) => Some(plane),
        _ => None,
    };

    let mut area = 0.0;
    for face_loop in loops {
        let wire = wires.get(face_loop.wire_index)?;
        let mut curve_segments = Vec::with_capacity(wire.edge_indices.len());
        let mut sampled_points = Vec::new();
        for (edge_index, edge_orientation) in oriented_wire_edges(wire, face_loop.orientation) {
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
            Some(_) => {
                analytic_sampled_wire_signed_area(surface, face_geometry, &sampled_points)?.abs()
            }
            None => {
                analytic_sampled_wire_signed_area(surface, face_geometry, &sampled_points)?.abs()
            }
        };
        match face_loop.role {
            LoopRole::Inner => area -= wire_area,
            LoopRole::Outer | LoopRole::Unknown => area += wire_area,
        }
    }
    Some(area.abs())
}

fn exact_closed_face_area(surface: PortedSurface, face_geometry: FaceGeometry) -> Option<f64> {
    match surface {
        PortedSurface::Sphere(payload)
            if periodic_span_matches(
                face_geometry.is_u_periodic,
                face_geometry.u_period,
                face_geometry.u_max - face_geometry.u_min,
            ) && approx_eq(
                face_geometry.v_max - face_geometry.v_min,
                PI,
                1.0e-6,
                1.0e-6,
            ) =>
        {
            Some(4.0 * PI * payload.radius.abs().powi(2))
        }
        PortedSurface::Torus(payload)
            if periodic_span_matches(
                face_geometry.is_u_periodic,
                face_geometry.u_period,
                face_geometry.u_max - face_geometry.u_min,
            ) && periodic_span_matches(
                face_geometry.is_v_periodic,
                face_geometry.v_period,
                face_geometry.v_max - face_geometry.v_min,
            ) =>
        {
            Some(4.0 * PI * PI * payload.major_radius.abs() * payload.minor_radius.abs())
        }
        _ => None,
    }
}

fn periodic_span_matches(is_periodic: bool, period: f64, span: f64) -> bool {
    is_periodic && approx_eq(span.abs(), period.abs(), 1.0e-6, 1.0e-6)
}

fn analytic_ported_swept_face_area(
    surface: PortedSweptSurface,
    face_geometry: FaceGeometry,
) -> Option<f64> {
    match surface {
        PortedSweptSurface::Extrusion {
            payload,
            basis_curve,
            basis_geometry,
        } => {
            let span = swept_surface_span(face_geometry, basis_geometry)?;
            Some(extrusion_swept_area(
                basis_curve,
                basis_geometry,
                payload.direction,
                span,
            ))
        }
        PortedSweptSurface::Revolution {
            payload,
            basis_curve,
            basis_geometry,
        } => {
            let sweep_angle = swept_surface_span(face_geometry, basis_geometry)?;
            Some(revolution_swept_area(
                basis_curve,
                basis_geometry,
                payload.axis_origin,
                payload.axis_direction,
                sweep_angle,
            ))
        }
    }
}

fn analytic_offset_face_area(
    context: &Context,
    surface: PortedOffsetSurface,
    face_geometry: FaceGeometry,
    loops: &[BrepFaceLoop],
    wires: &[BrepWire],
    edges: &[BrepEdge],
    edge_shapes: &[Shape],
) -> Option<f64> {
    if let Some(equivalent_surface) = surface.equivalent_analytic_surface() {
        return analytic_face_area(
            context,
            equivalent_surface,
            face_geometry,
            loops,
            wires,
            edges,
            edge_shapes,
        );
    }

    match surface.basis {
        PortedOffsetBasisSurface::Analytic(_) => None,
        PortedOffsetBasisSurface::Swept(PortedSweptSurface::Extrusion {
            payload,
            basis_curve,
            basis_geometry,
        }) => offset_extrusion_swept_area(
            surface.payload.offset_value,
            surface.basis_geometry,
            basis_curve,
            basis_geometry,
            payload.direction,
        ),
        PortedOffsetBasisSurface::Swept(PortedSweptSurface::Revolution {
            payload,
            basis_curve,
            basis_geometry,
        }) => offset_revolution_swept_area(
            surface.payload.offset_value,
            surface.basis_geometry,
            basis_curve,
            basis_geometry,
            payload.axis_origin,
            payload.axis_direction,
        ),
    }
}

fn offset_extrusion_swept_area(
    offset: f64,
    surface_geometry: FaceGeometry,
    curve: PortedCurve,
    curve_geometry: EdgeGeometry,
    direction: [f64; 3],
) -> Option<f64> {
    let direction = normalize3(direction);
    if norm3(direction) <= 1.0e-12 {
        return None;
    }

    let sweep_span = swept_surface_span(surface_geometry, curve_geometry)?;
    let basis_on_u = basis_parameter_on_u(surface_geometry, curve_geometry);
    Some(
        sweep_span.abs()
            * positive_scalar_integral(
                curve_geometry.start_parameter,
                curve_geometry.end_parameter,
                |parameter| {
                    let differential = offset_extrusion_curve_differential(
                        curve, direction, offset, basis_on_u, parameter,
                    );
                    norm3(cross3(differential.derivative, direction))
                },
            ),
    )
}

fn offset_revolution_swept_area(
    offset: f64,
    surface_geometry: FaceGeometry,
    curve: PortedCurve,
    curve_geometry: EdgeGeometry,
    axis_origin: [f64; 3],
    axis_direction: [f64; 3],
) -> Option<f64> {
    let axis_direction = normalize3(axis_direction);
    if norm3(axis_direction) <= 1.0e-12 {
        return None;
    }

    let sweep_angle = swept_surface_span(surface_geometry, curve_geometry)?;
    let basis_on_u = basis_parameter_on_u(surface_geometry, curve_geometry);
    Some(
        sweep_angle.abs()
            * positive_scalar_integral(
                curve_geometry.start_parameter,
                curve_geometry.end_parameter,
                |parameter| {
                    let differential = offset_revolution_curve_differential(
                        curve,
                        axis_origin,
                        axis_direction,
                        offset,
                        basis_on_u,
                        parameter,
                    );
                    let sweep_derivative = cross3(
                        axis_direction,
                        subtract3(differential.position, axis_origin),
                    );
                    norm3(cross3(differential.derivative, sweep_derivative))
                },
            ),
    )
}

fn swept_surface_span(surface_geometry: FaceGeometry, curve_geometry: EdgeGeometry) -> Option<f64> {
    let span = if basis_parameter_on_u(surface_geometry, curve_geometry) {
        surface_geometry.v_max - surface_geometry.v_min
    } else {
        surface_geometry.u_max - surface_geometry.u_min
    };
    (span.abs() > 1.0e-12).then_some(span)
}

fn offset_extrusion_curve_differential(
    curve: PortedCurve,
    direction: [f64; 3],
    offset: f64,
    basis_on_u: bool,
    parameter: f64,
) -> OffsetCurveDifferential {
    let differential = curve_differential(curve, parameter);
    let (normal_source, normal_source_derivative) = if basis_on_u {
        (
            cross3(differential.first_derivative, direction),
            cross3(differential.second_derivative, direction),
        )
    } else {
        (
            cross3(direction, differential.first_derivative),
            cross3(direction, differential.second_derivative),
        )
    };
    let normal = normalize3(normal_source);
    let normal_derivative =
        normalized_direction_derivative(normal_source, normal_source_derivative);
    OffsetCurveDifferential {
        position: add3(differential.position, scale3(normal, offset)),
        derivative: add3(
            differential.first_derivative,
            scale3(normal_derivative, offset),
        ),
    }
}

fn offset_revolution_curve_differential(
    curve: PortedCurve,
    axis_origin: [f64; 3],
    axis_direction: [f64; 3],
    offset: f64,
    basis_on_u: bool,
    parameter: f64,
) -> OffsetCurveDifferential {
    let differential = curve_differential(curve, parameter);
    let sweep_derivative = cross3(
        axis_direction,
        subtract3(differential.position, axis_origin),
    );
    let sweep_second_derivative = cross3(axis_direction, differential.first_derivative);
    let (normal_source, normal_source_derivative) = if basis_on_u {
        (
            cross3(differential.first_derivative, sweep_derivative),
            add3(
                cross3(differential.second_derivative, sweep_derivative),
                cross3(differential.first_derivative, sweep_second_derivative),
            ),
        )
    } else {
        (
            cross3(sweep_derivative, differential.first_derivative),
            add3(
                cross3(sweep_second_derivative, differential.first_derivative),
                cross3(sweep_derivative, differential.second_derivative),
            ),
        )
    };
    let normal = normalize3(normal_source);
    let normal_derivative =
        normalized_direction_derivative(normal_source, normal_source_derivative);
    OffsetCurveDifferential {
        position: add3(differential.position, scale3(normal, offset)),
        derivative: add3(
            differential.first_derivative,
            scale3(normal_derivative, offset),
        ),
    }
}

fn curve_differential(curve: PortedCurve, parameter: f64) -> CurveDifferential {
    match curve {
        PortedCurve::Line(payload) => CurveDifferential {
            position: add3(payload.origin, scale3(payload.direction, parameter)),
            first_derivative: payload.direction,
            second_derivative: [0.0; 3],
        },
        PortedCurve::Circle(payload) => {
            let cos_parameter = parameter.cos();
            let sin_parameter = parameter.sin();
            CurveDifferential {
                position: add3(
                    payload.center,
                    add3(
                        scale3(payload.x_direction, payload.radius * cos_parameter),
                        scale3(payload.y_direction, payload.radius * sin_parameter),
                    ),
                ),
                first_derivative: add3(
                    scale3(payload.x_direction, -payload.radius * sin_parameter),
                    scale3(payload.y_direction, payload.radius * cos_parameter),
                ),
                second_derivative: add3(
                    scale3(payload.x_direction, -payload.radius * cos_parameter),
                    scale3(payload.y_direction, -payload.radius * sin_parameter),
                ),
            }
        }
        PortedCurve::Ellipse(payload) => {
            let cos_parameter = parameter.cos();
            let sin_parameter = parameter.sin();
            CurveDifferential {
                position: add3(
                    payload.center,
                    add3(
                        scale3(payload.x_direction, payload.major_radius * cos_parameter),
                        scale3(payload.y_direction, payload.minor_radius * sin_parameter),
                    ),
                ),
                first_derivative: add3(
                    scale3(payload.x_direction, -payload.major_radius * sin_parameter),
                    scale3(payload.y_direction, payload.minor_radius * cos_parameter),
                ),
                second_derivative: add3(
                    scale3(payload.x_direction, -payload.major_radius * cos_parameter),
                    scale3(payload.y_direction, -payload.minor_radius * sin_parameter),
                ),
            }
        }
    }
}

fn normalized_direction_derivative(direction: [f64; 3], derivative: [f64; 3]) -> [f64; 3] {
    let direction_norm = norm3(direction);
    if direction_norm <= 1.0e-12 {
        return [0.0; 3];
    }

    let unit_direction = scale3(direction, direction_norm.recip());
    scale3(
        subtract3(
            derivative,
            scale3(unit_direction, dot3(unit_direction, derivative)),
        ),
        direction_norm.recip(),
    )
}

fn analytic_ported_swept_face_volume(
    face: &BrepFace,
    face_geometry: FaceGeometry,
    surface: PortedSweptSurface,
) -> Option<f64> {
    match surface {
        PortedSweptSurface::Extrusion {
            payload,
            basis_curve,
            basis_geometry,
        } => {
            let direction = normalize3(payload.direction);
            if norm3(direction) <= 1.0e-12 {
                return None;
            }
            let sweep = scale3(
                direction,
                swept_surface_span(face_geometry, basis_geometry)?.abs(),
            );
            let midpoint_parameter =
                0.5 * (basis_geometry.start_parameter + basis_geometry.end_parameter);
            let midpoint = basis_curve.evaluate(midpoint_parameter);
            let midpoint_position = add3(midpoint.position, scale3(sweep, 0.5));
            let midpoint_du = midpoint.derivative;
            let midpoint_dv = sweep;
            let sign = oriented_surface_sign(face, midpoint_position, midpoint_du, midpoint_dv);
            Some(
                sign * signed_scalar_integral(
                    basis_geometry.start_parameter,
                    basis_geometry.end_parameter,
                    |parameter| {
                        let evaluation = basis_curve.evaluate(parameter);
                        dot3(evaluation.position, cross3(evaluation.derivative, sweep)) / 3.0
                    },
                ),
            )
        }
        PortedSweptSurface::Revolution {
            payload,
            basis_curve,
            basis_geometry,
        } => {
            let axis_direction = normalize3(payload.axis_direction);
            if norm3(axis_direction) <= 1.0e-12 {
                return None;
            }
            let sweep_angle = swept_surface_span(face_geometry, basis_geometry)?.abs();
            let midpoint_parameter =
                0.5 * (basis_geometry.start_parameter + basis_geometry.end_parameter);
            let midpoint_evaluation = basis_curve.evaluate(midpoint_parameter);
            let midpoint_position = rotate_point_about_axis(
                midpoint_evaluation.position,
                payload.axis_origin,
                axis_direction,
                0.5 * sweep_angle,
            );
            let midpoint_du = rotate_vector_about_axis(
                midpoint_evaluation.derivative,
                axis_direction,
                0.5 * sweep_angle,
            );
            let midpoint_dv =
                revolution_surface_dv(midpoint_position, payload.axis_origin, axis_direction);
            let sign = oriented_surface_sign(face, midpoint_position, midpoint_du, midpoint_dv);

            Some(
                sign * signed_scalar_integral(
                    basis_geometry.start_parameter,
                    basis_geometry.end_parameter,
                    |parameter| {
                        let evaluation = basis_curve.evaluate(parameter);
                        signed_scalar_integral(0.0, sweep_angle, |angle| {
                            let position = rotate_point_about_axis(
                                evaluation.position,
                                payload.axis_origin,
                                axis_direction,
                                angle,
                            );
                            let du = rotate_vector_about_axis(
                                evaluation.derivative,
                                axis_direction,
                                angle,
                            );
                            let dv = revolution_surface_dv(
                                position,
                                payload.axis_origin,
                                axis_direction,
                            );
                            dot3(position, cross3(du, dv)) / 3.0
                        })
                    },
                ),
            )
        }
    }
}

fn analytic_offset_face_volume(
    context: &Context,
    face: &BrepFace,
    surface: PortedOffsetSurface,
    face_geometry: FaceGeometry,
    loops: &[BrepFaceLoop],
    wires: &[BrepWire],
    edges: &[BrepEdge],
    edge_shapes: &[Shape],
) -> Option<f64> {
    if let Some(equivalent_surface) = surface.equivalent_analytic_surface() {
        return analytic_face_volume(
            context,
            face,
            equivalent_surface,
            face_geometry,
            loops,
            wires,
            edges,
            edge_shapes,
        );
    }

    match surface.basis {
        PortedOffsetBasisSurface::Analytic(_) => None,
        PortedOffsetBasisSurface::Swept(PortedSweptSurface::Extrusion {
            payload,
            basis_curve,
            basis_geometry,
        }) => analytic_offset_extrusion_face_volume(
            face,
            surface.payload.offset_value,
            surface.basis_geometry,
            basis_curve,
            basis_geometry,
            payload.direction,
        ),
        PortedOffsetBasisSurface::Swept(PortedSweptSurface::Revolution {
            payload,
            basis_curve,
            basis_geometry,
        }) => analytic_offset_revolution_face_volume(
            face,
            surface.payload.offset_value,
            surface.basis_geometry,
            basis_curve,
            basis_geometry,
            payload.axis_origin,
            payload.axis_direction,
        ),
    }
}

fn analytic_offset_extrusion_face_volume(
    face: &BrepFace,
    offset: f64,
    surface_geometry: FaceGeometry,
    curve: PortedCurve,
    curve_geometry: EdgeGeometry,
    direction: [f64; 3],
) -> Option<f64> {
    let direction = normalize3(direction);
    if norm3(direction) <= 1.0e-12 {
        return None;
    }

    let sweep = scale3(
        direction,
        swept_surface_span(surface_geometry, curve_geometry)?.abs(),
    );
    let basis_on_u = basis_parameter_on_u(surface_geometry, curve_geometry);
    let midpoint_parameter = 0.5 * (curve_geometry.start_parameter + curve_geometry.end_parameter);
    let midpoint = offset_extrusion_curve_differential(
        curve,
        direction,
        offset,
        basis_on_u,
        midpoint_parameter,
    );
    let midpoint_position = add3(midpoint.position, scale3(sweep, 0.5));
    let midpoint_du = midpoint.derivative;
    let midpoint_dv = sweep;
    let sign = oriented_surface_sign(face, midpoint_position, midpoint_du, midpoint_dv);

    Some(
        sign * signed_scalar_integral(
            curve_geometry.start_parameter,
            curve_geometry.end_parameter,
            |parameter| {
                let differential = offset_extrusion_curve_differential(
                    curve, direction, offset, basis_on_u, parameter,
                );
                dot3(
                    differential.position,
                    cross3(differential.derivative, sweep),
                ) / 3.0
            },
        ),
    )
}

fn analytic_offset_revolution_face_volume(
    face: &BrepFace,
    offset: f64,
    surface_geometry: FaceGeometry,
    curve: PortedCurve,
    curve_geometry: EdgeGeometry,
    axis_origin: [f64; 3],
    axis_direction: [f64; 3],
) -> Option<f64> {
    let axis_direction = normalize3(axis_direction);
    if norm3(axis_direction) <= 1.0e-12 {
        return None;
    }

    let sweep_angle = swept_surface_span(surface_geometry, curve_geometry)?.abs();
    let basis_on_u = basis_parameter_on_u(surface_geometry, curve_geometry);
    let midpoint_parameter = 0.5 * (curve_geometry.start_parameter + curve_geometry.end_parameter);
    let midpoint = offset_revolution_curve_differential(
        curve,
        axis_origin,
        axis_direction,
        offset,
        basis_on_u,
        midpoint_parameter,
    );
    let midpoint_position = rotate_point_about_axis(
        midpoint.position,
        axis_origin,
        axis_direction,
        0.5 * sweep_angle,
    );
    let midpoint_du =
        rotate_vector_about_axis(midpoint.derivative, axis_direction, 0.5 * sweep_angle);
    let midpoint_dv = revolution_surface_dv(midpoint_position, axis_origin, axis_direction);
    let sign = oriented_surface_sign(face, midpoint_position, midpoint_du, midpoint_dv);

    Some(
        sign * signed_scalar_integral(
            curve_geometry.start_parameter,
            curve_geometry.end_parameter,
            |parameter| {
                let differential = offset_revolution_curve_differential(
                    curve,
                    axis_origin,
                    axis_direction,
                    offset,
                    basis_on_u,
                    parameter,
                );
                signed_scalar_integral(0.0, sweep_angle, |angle| {
                    let position = rotate_point_about_axis(
                        differential.position,
                        axis_origin,
                        axis_direction,
                        angle,
                    );
                    let du =
                        rotate_vector_about_axis(differential.derivative, axis_direction, angle);
                    let dv = revolution_surface_dv(position, axis_origin, axis_direction);
                    dot3(position, cross3(du, dv)) / 3.0
                })
            },
        ),
    )
}

fn analytic_face_volume(
    context: &Context,
    face: &BrepFace,
    surface: PortedSurface,
    face_geometry: FaceGeometry,
    loops: &[BrepFaceLoop],
    wires: &[BrepWire],
    edges: &[BrepEdge],
    edge_shapes: &[Shape],
) -> Option<f64> {
    if matches!(surface, PortedSurface::Plane(_)) {
        return Some(face.area * dot3(face.sample.position, face.sample.normal) / 3.0);
    }

    let mut volume = 0.0;
    for face_loop in loops {
        let wire = wires.get(face_loop.wire_index)?;
        let mut sampled_points = Vec::new();
        for (edge_index, edge_orientation) in oriented_wire_edges(wire, face_loop.orientation) {
            let edge = edges.get(edge_index)?;
            let geometry = oriented_edge_geometry(edge.geometry, edge_orientation);
            append_edge_sample_points(
                context,
                edge_shapes.get(edge_index)?,
                edge,
                geometry,
                &mut sampled_points,
            )
            .ok()?;
        }
        let loop_volume =
            analytic_sampled_wire_signed_volume(surface, face_geometry, &sampled_points)?;
        match face_loop.role {
            LoopRole::Inner => volume -= loop_volume,
            LoopRole::Outer | LoopRole::Unknown => volume += loop_volume,
        }
    }
    Some(volume)
}

fn oriented_surface_sign(face: &BrepFace, position: [f64; 3], du: [f64; 3], dv: [f64; 3]) -> f64 {
    let _ = position;
    let normal = normalize3(cross3(du, dv));
    if dot3(normal, face.sample.normal) >= 0.0 {
        1.0
    } else {
        -1.0
    }
}

fn face_curve_candidates(
    loops: &[BrepFaceLoop],
    wires: &[BrepWire],
    edges: &[BrepEdge],
    basis_kind: crate::CurveKind,
) -> Option<Vec<FaceCurveCandidate>> {
    let mut seen = BTreeSet::new();
    let mut candidates = Vec::new();

    for face_loop in loops {
        let wire = wires.get(face_loop.wire_index)?;
        for (edge_index, edge_orientation) in oriented_wire_edges(wire, face_loop.orientation) {
            if !seen.insert(edge_index) {
                continue;
            }
            let edge = edges.get(edge_index)?;
            if edge.geometry.kind != basis_kind {
                continue;
            }
            let curve = edge.ported_curve?;

            let geometry = oriented_edge_geometry(edge.geometry, edge_orientation);
            let midpoint_parameter = 0.5 * (geometry.start_parameter + geometry.end_parameter);
            let midpoint = curve
                .sample_with_geometry(geometry, midpoint_parameter)
                .position;
            candidates.push(FaceCurveCandidate {
                curve,
                geometry,
                midpoint,
            });
        }
    }

    if candidates.is_empty() {
        None
    } else {
        Some(candidates)
    }
}

fn select_swept_face_basis_curve(
    candidates: Vec<FaceCurveCandidate>,
    face_geometry: FaceGeometry,
    selection: SweptBasisSelection,
) -> Option<FaceCurveCandidate> {
    let basis_geometry = candidates.first()?.geometry;
    let use_u_for_basis = basis_parameter_on_u(face_geometry, basis_geometry);
    let (sweep_min, sweep_max) = if use_u_for_basis {
        (face_geometry.v_min, face_geometry.v_max)
    } else {
        (face_geometry.u_min, face_geometry.u_max)
    };
    let target_is_min = sweep_min.abs() <= sweep_max.abs();

    match selection {
        SweptBasisSelection::Extrusion { direction } => {
            let direction = normalize3(direction);
            if target_is_min {
                candidates.into_iter().min_by(|lhs, rhs| {
                    dot3(lhs.midpoint, direction)
                        .partial_cmp(&dot3(rhs.midpoint, direction))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
            } else {
                candidates.into_iter().max_by(|lhs, rhs| {
                    dot3(lhs.midpoint, direction)
                        .partial_cmp(&dot3(rhs.midpoint, direction))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
            }
        }
        SweptBasisSelection::Revolution {
            axis_origin,
            axis_direction,
        } => {
            if periodic_face_span(face_geometry).is_some() {
                return candidates.into_iter().next();
            }
            let axis_direction = normalize3(axis_direction);
            let reference_radial = candidates.iter().find_map(|candidate| {
                radial_direction(candidate.midpoint, axis_origin, axis_direction)
            })?;
            let tangent = normalize3(cross3(axis_direction, reference_radial));
            let angular_candidates = candidates
                .into_iter()
                .filter_map(|candidate| {
                    let radial = radial_direction(candidate.midpoint, axis_origin, axis_direction)?;
                    Some((
                        candidate,
                        dot3(radial, tangent).atan2(dot3(radial, reference_radial)),
                    ))
                })
                .collect::<Vec<_>>();

            if target_is_min {
                angular_candidates
                    .into_iter()
                    .min_by(|lhs, rhs| {
                        lhs.1
                            .partial_cmp(&rhs.1)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|(candidate, _)| candidate)
            } else {
                angular_candidates
                    .into_iter()
                    .max_by(|lhs, rhs| {
                        lhs.1
                            .partial_cmp(&rhs.1)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|(candidate, _)| candidate)
            }
        }
    }
}

#[derive(Clone, Copy)]
enum SweptBasisSelection {
    Extrusion {
        direction: [f64; 3],
    },
    Revolution {
        axis_origin: [f64; 3],
        axis_direction: [f64; 3],
    },
}

fn basis_parameter_on_u(face_geometry: FaceGeometry, basis_geometry: EdgeGeometry) -> bool {
    let basis_span = (basis_geometry.end_parameter - basis_geometry.start_parameter).abs();
    let u_span = (face_geometry.u_max - face_geometry.u_min).abs();
    let v_span = (face_geometry.v_max - face_geometry.v_min).abs();
    (u_span - basis_span).abs() <= (v_span - basis_span).abs()
}

fn radial_direction(
    point: [f64; 3],
    axis_origin: [f64; 3],
    axis_direction: [f64; 3],
) -> Option<[f64; 3]> {
    let radial = subtract3(
        point,
        add3(
            axis_origin,
            scale3(
                axis_direction,
                dot3(subtract3(point, axis_origin), axis_direction),
            ),
        ),
    );
    (norm3(radial) > 1.0e-9).then_some(normalize3(radial))
}

fn rotate_point_about_axis(
    point: [f64; 3],
    axis_origin: [f64; 3],
    axis_direction: [f64; 3],
    angle: f64,
) -> [f64; 3] {
    add3(
        axis_origin,
        rotate_vector_about_axis(subtract3(point, axis_origin), axis_direction, angle),
    )
}

fn rotate_vector_about_axis(vector: [f64; 3], axis_direction: [f64; 3], angle: f64) -> [f64; 3] {
    let axis_direction = normalize3(axis_direction);
    let cos_angle = angle.cos();
    let sin_angle = angle.sin();
    add3(
        add3(
            scale3(vector, cos_angle),
            scale3(cross3(axis_direction, vector), sin_angle),
        ),
        scale3(
            axis_direction,
            dot3(axis_direction, vector) * (1.0 - cos_angle),
        ),
    )
}

fn revolution_surface_dv(
    position: [f64; 3],
    axis_origin: [f64; 3],
    axis_direction: [f64; 3],
) -> [f64; 3] {
    cross3(normalize3(axis_direction), subtract3(position, axis_origin))
}

fn signed_scalar_integral<F>(start: f64, end: f64, integrand: F) -> f64
where
    F: Fn(f64) -> f64,
{
    if (end - start).abs() <= 1.0e-15 {
        return 0.0;
    }

    let (a, b, sign) = if start <= end {
        (start, end, 1.0)
    } else {
        (end, start, -1.0)
    };
    let fa = integrand(a);
    let fm = integrand(0.5 * (a + b));
    let fb = integrand(b);
    sign * adaptive_simpson(&integrand, a, b, fa, fm, fb, 1.0e-8, 12)
}

fn positive_scalar_integral<F>(start: f64, end: f64, integrand: F) -> f64
where
    F: Fn(f64) -> f64,
{
    signed_scalar_integral(start, end, |value| integrand(value).abs()).abs()
}

fn adaptive_simpson<F>(
    integrand: &F,
    a: f64,
    b: f64,
    fa: f64,
    fm: f64,
    fb: f64,
    tolerance: f64,
    depth: u32,
) -> f64
where
    F: Fn(f64) -> f64,
{
    let midpoint = 0.5 * (a + b);
    let left_mid = 0.5 * (a + midpoint);
    let right_mid = 0.5 * (midpoint + b);
    let flm = integrand(left_mid);
    let frm = integrand(right_mid);

    let whole = simpson_step(a, b, fa, fm, fb);
    let left = simpson_step(a, midpoint, fa, flm, fm);
    let right = simpson_step(midpoint, b, fm, frm, fb);
    let delta = left + right - whole;

    if depth == 0 || delta.abs() <= 15.0 * tolerance {
        return left + right + delta / 15.0;
    }

    adaptive_simpson(
        integrand,
        a,
        midpoint,
        fa,
        flm,
        fm,
        0.5 * tolerance,
        depth - 1,
    ) + adaptive_simpson(
        integrand,
        midpoint,
        b,
        fm,
        frm,
        fb,
        0.5 * tolerance,
        depth - 1,
    )
}

fn simpson_step(a: f64, b: f64, fa: f64, fm: f64, fb: f64) -> f64 {
    (b - a) * (fa + 4.0 * fm + fb) / 6.0
}

fn periodic_face_span(face_geometry: FaceGeometry) -> Option<f64> {
    if face_geometry.is_u_periodic && !face_geometry.is_v_periodic {
        let span = face_geometry.u_max - face_geometry.u_min;
        return (span.abs() > 1.0e-9).then_some(span.abs());
    }
    if face_geometry.is_v_periodic && !face_geometry.is_u_periodic {
        let span = face_geometry.v_max - face_geometry.v_min;
        return (span.abs() > 1.0e-9).then_some(span.abs());
    }
    None
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
            None => {
                context
                    .edge_sample_at_parameter_occt(edge_shape, parameter)?
                    .position
            }
        };
        out_points.push(position);
    }
    Ok(())
}

fn append_root_edge_sample_points(
    context: &Context,
    edge_shape: &Shape,
    edge: &RootEdgeTopology,
    geometry: EdgeGeometry,
    out_points: &mut Vec<[f64; 3]>,
) -> Result<(), Error> {
    let ported_curve = PortedCurve::from_context_with_geometry(context, edge_shape, edge.geometry)?;
    let segment_count = root_edge_sample_count(edge.geometry.kind, geometry);
    for step in 0..=segment_count {
        if !out_points.is_empty() && step == 0 {
            continue;
        }
        let t = step as f64 / segment_count as f64;
        let parameter = interpolate_range(geometry.start_parameter, geometry.end_parameter, t);
        let position = match ported_curve {
            Some(curve) => curve.sample_with_geometry(geometry, parameter).position,
            None => {
                context
                    .edge_sample_at_parameter_occt(edge_shape, parameter)?
                    .position
            }
        };
        out_points.push(position);
    }
    Ok(())
}

fn oriented_wire_edges(
    wire: &BrepWire,
    wire_orientation: Orientation,
) -> Vec<(usize, Orientation)> {
    let reverse_wire = matches!(wire_orientation, Orientation::Reversed);
    let mut uses = wire
        .edge_indices
        .iter()
        .copied()
        .zip(wire.edge_orientations.iter().copied())
        .collect::<Vec<_>>();
    if reverse_wire {
        uses.reverse();
        for (_, orientation) in &mut uses {
            *orientation = reversed_orientation(*orientation);
        }
    }
    uses
}

fn reversed_orientation(orientation: Orientation) -> Orientation {
    match orientation {
        Orientation::Forward => Orientation::Reversed,
        Orientation::Reversed => Orientation::Forward,
        other => other,
    }
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

fn root_edge_sample_count(kind: crate::CurveKind, geometry: EdgeGeometry) -> usize {
    let span = (geometry.end_parameter - geometry.start_parameter).abs();
    let base = match kind {
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

fn polyhedral_mesh_volume(mesh: &Mesh) -> Option<f64> {
    if mesh.triangle_indices.is_empty() {
        return Some(0.0);
    }

    let origin = [
        0.5 * (mesh.bbox_min[0] + mesh.bbox_max[0]),
        0.5 * (mesh.bbox_min[1] + mesh.bbox_max[1]),
        0.5 * (mesh.bbox_min[2] + mesh.bbox_max[2]),
    ];
    let mut signed_volume = 0.0;

    for triangle in mesh.triangle_indices.chunks_exact(3) {
        let i0 = usize::try_from(triangle[0]).ok()?;
        let i1 = usize::try_from(triangle[1]).ok()?;
        let i2 = usize::try_from(triangle[2]).ok()?;
        let a = *mesh.positions.get(i0)?;
        let b = *mesh.positions.get(i1)?;
        let c = *mesh.positions.get(i2)?;

        let face_cross = cross3(subtract3(b, a), subtract3(c, a));
        let face_cross_length = dot3(face_cross, face_cross).sqrt();
        if face_cross_length <= 1.0e-12 {
            continue;
        }

        let averaged_normal = add3(
            add3(
                mesh.normals.get(i0).copied().unwrap_or([0.0; 3]),
                mesh.normals.get(i1).copied().unwrap_or([0.0; 3]),
            ),
            mesh.normals.get(i2).copied().unwrap_or([0.0; 3]),
        );
        let outward_normal = if dot3(averaged_normal, averaged_normal) > 1.0e-18 {
            normalize3(averaged_normal)
        } else {
            let centroid = scale3(add3(add3(a, b), c), 1.0 / 3.0);
            let fallback_normal = normalize3(face_cross);
            if dot3(fallback_normal, subtract3(centroid, origin)) >= 0.0 {
                fallback_normal
            } else {
                scale3(fallback_normal, -1.0)
            }
        };
        let centroid = scale3(add3(add3(a, b), c), 1.0 / 3.0);
        let area = 0.5 * face_cross_length;
        signed_volume += area * dot3(subtract3(centroid, origin), outward_normal) / 3.0;
    }

    Some(signed_volume.abs())
}

fn polyhedral_mesh_area(mesh: &Mesh) -> Option<f64> {
    if mesh.triangle_indices.is_empty() {
        return Some(0.0);
    }

    let mut area = 0.0;
    for triangle in mesh.triangle_indices.chunks_exact(3) {
        let i0 = usize::try_from(triangle[0]).ok()?;
        let i1 = usize::try_from(triangle[1]).ok()?;
        let i2 = usize::try_from(triangle[2]).ok()?;
        let a = *mesh.positions.get(i0)?;
        let b = *mesh.positions.get(i1)?;
        let c = *mesh.positions.get(i2)?;
        area += 0.5 * norm3(cross3(subtract3(b, a), subtract3(c, a)));
    }

    Some(area)
}

fn polyhedral_mesh_sample(mesh: &Mesh) -> Option<FaceSample> {
    if mesh.positions.is_empty() {
        return None;
    }

    let mut weighted_area = 0.0;
    let mut weighted_centroid = [0.0; 3];
    let mut weighted_normal = [0.0; 3];

    for triangle in mesh.triangle_indices.chunks_exact(3) {
        let i0 = usize::try_from(triangle[0]).ok()?;
        let i1 = usize::try_from(triangle[1]).ok()?;
        let i2 = usize::try_from(triangle[2]).ok()?;
        let a = *mesh.positions.get(i0)?;
        let b = *mesh.positions.get(i1)?;
        let c = *mesh.positions.get(i2)?;
        let face_cross = cross3(subtract3(b, a), subtract3(c, a));
        let triangle_area = 0.5 * norm3(face_cross);
        if triangle_area <= 1.0e-12 {
            continue;
        }

        let averaged_normal = add3(
            add3(
                mesh.normals.get(i0).copied().unwrap_or([0.0; 3]),
                mesh.normals.get(i1).copied().unwrap_or([0.0; 3]),
            ),
            mesh.normals.get(i2).copied().unwrap_or([0.0; 3]),
        );
        let triangle_normal = if norm3(averaged_normal) > 1.0e-12 {
            normalize3(averaged_normal)
        } else {
            normalize3(face_cross)
        };
        let centroid = scale3(add3(add3(a, b), c), 1.0 / 3.0);
        weighted_area += triangle_area;
        weighted_centroid = add3(weighted_centroid, scale3(centroid, triangle_area));
        weighted_normal = add3(weighted_normal, scale3(triangle_normal, triangle_area));
    }

    if weighted_area > 1.0e-12 {
        return Some(FaceSample {
            position: scale3(weighted_centroid, weighted_area.recip()),
            normal: normalize3(weighted_normal),
        });
    }

    let position = scale3(
        mesh.positions.iter().copied().fold([0.0; 3], add3),
        (mesh.positions.len() as f64).recip(),
    );
    let normal = normalize3(mesh.normals.iter().copied().fold([0.0; 3], add3));
    Some(FaceSample { position, normal })
}

fn mesh_bbox(mesh: &Mesh) -> Option<([f64; 3], [f64; 3])> {
    let mut points = mesh.positions.clone();
    for segment in &mesh.edge_segments {
        points.push(segment[0]);
        points.push(segment[1]);
    }
    bbox_from_points(points)
}

fn bbox_from_points(points: Vec<[f64; 3]>) -> Option<([f64; 3], [f64; 3])> {
    let mut iter = points.into_iter();
    let first = iter.next()?;
    let mut min = first;
    let mut max = first;

    for point in iter {
        for axis in 0..3 {
            min[axis] = min[axis].min(point[axis]);
            max[axis] = max[axis].max(point[axis]);
        }
    }

    Some((min, max))
}

fn union_bbox(lhs: ([f64; 3], [f64; 3]), rhs: ([f64; 3], [f64; 3])) -> ([f64; 3], [f64; 3]) {
    let mut min = lhs.0;
    let mut max = lhs.1;
    for axis in 0..3 {
        min[axis] = min[axis].min(rhs.0[axis]);
        max[axis] = max[axis].max(rhs.1[axis]);
    }
    (min, max)
}

fn add3(lhs: [f64; 3], rhs: [f64; 3]) -> [f64; 3] {
    [lhs[0] + rhs[0], lhs[1] + rhs[1], lhs[2] + rhs[2]]
}

fn subtract3(lhs: [f64; 3], rhs: [f64; 3]) -> [f64; 3] {
    [lhs[0] - rhs[0], lhs[1] - rhs[1], lhs[2] - rhs[2]]
}

fn scale3(vector: [f64; 3], factor: f64) -> [f64; 3] {
    [vector[0] * factor, vector[1] * factor, vector[2] * factor]
}

fn dot3(lhs: [f64; 3], rhs: [f64; 3]) -> f64 {
    lhs[0] * rhs[0] + lhs[1] * rhs[1] + lhs[2] * rhs[2]
}

fn cross3(lhs: [f64; 3], rhs: [f64; 3]) -> [f64; 3] {
    [
        lhs[1] * rhs[2] - lhs[2] * rhs[1],
        lhs[2] * rhs[0] - lhs[0] * rhs[2],
        lhs[0] * rhs[1] - lhs[1] * rhs[0],
    ]
}

fn normalize3(vector: [f64; 3]) -> [f64; 3] {
    let length = dot3(vector, vector).sqrt();
    if length <= 1.0e-18 {
        [0.0; 3]
    } else {
        scale3(vector, length.recip())
    }
}

fn norm3(vector: [f64; 3]) -> f64 {
    dot3(vector, vector).sqrt()
}

fn approx_eq(lhs: f64, rhs: f64, relative_tolerance: f64, absolute_tolerance: f64) -> bool {
    let delta = (lhs - rhs).abs();
    if delta <= absolute_tolerance {
        return true;
    }
    let scale = lhs.abs().max(rhs.abs()).max(1.0);
    delta <= relative_tolerance * scale
}

fn approx_points_eq(lhs: [f64; 3], rhs: [f64; 3], tolerance: f64) -> bool {
    (lhs[0] - rhs[0]).abs() <= tolerance
        && (lhs[1] - rhs[1]).abs() <= tolerance
        && (lhs[2] - rhs[2]).abs() <= tolerance
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

fn adjacent_face_indices(
    topology: &TopologySnapshot,
    edge_index: usize,
) -> Result<Vec<usize>, Error> {
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
                .ok_or_else(|| {
                    Error::new(format!("topology is missing face-wire index {offset}"))
                })?,
            orientation: topology
                .face_wire_orientations
                .get(offset)
                .copied()
                .ok_or_else(|| {
                    Error::new(format!(
                        "topology is missing face-wire orientation {offset}"
                    ))
                })?,
            role: topology
                .face_wire_roles
                .get(offset)
                .copied()
                .ok_or_else(|| {
                    Error::new(format!("topology is missing face-wire role {offset}"))
                })?,
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
        let wire = wires.get(face_loop.wire_index).ok_or_else(|| {
            Error::new(format!(
                "topology is missing wire index {}",
                face_loop.wire_index
            ))
        })?;
        for &edge_index in &wire.edge_indices {
            let range = topology
                .edge_faces
                .get(edge_index)
                .copied()
                .ok_or_else(|| {
                    Error::new(format!("topology is missing edge-face range {edge_index}"))
                })?;
            for &candidate in &topology.edge_face_indices[range.offset..range.offset + range.count]
            {
                if candidate != face_index {
                    adjacent.insert(candidate);
                }
            }
        }
    }
    Ok(adjacent.into_iter().collect())
}
