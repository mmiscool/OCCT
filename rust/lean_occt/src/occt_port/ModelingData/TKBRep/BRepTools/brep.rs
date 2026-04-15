use std::collections::{BTreeMap, BTreeSet};
use std::f64::consts::PI;

mod face_metrics;
mod face_surface;
mod summary;
mod swept_face;
mod topology;

use self::face_metrics::{
    analytic_face_area, analytic_offset_face_area, analytic_ported_swept_face_area,
};
use self::face_surface::ported_face_surface_descriptor_from_surface;
pub(crate) use self::face_surface::{ported_face_area, ported_face_surface_descriptor};
use self::summary::{classify_root_kind, mesh_face_properties, ported_shape_summary};
use self::topology::{
    adjacent_face_indices, edge_points, face_adjacent_face_indices, face_loops,
    optional_vertex_position, ported_topology_snapshot, topology_edge,
};

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
                let geometry = match self.edge_geometry(edge_shape) {
                    Ok(geometry) => geometry,
                    Err(_) => self.edge_geometry_occt(edge_shape)?,
                };
                let ported_curve = match PortedCurve::from_context_with_ported_payloads(
                    self, edge_shape, geometry,
                ) {
                    Ok(ported_curve) => ported_curve,
                    Err(_) => PortedCurve::from_context_with_geometry(self, edge_shape, geometry)?,
                };
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
