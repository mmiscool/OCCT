use std::collections::{BTreeMap, BTreeSet};
use std::f64::consts::PI;

mod face_metrics;
mod face_surface;
mod math;
mod mesh;
mod summary;
mod swept_face;
mod topology;

use self::face_metrics::{
    analytic_face_area, analytic_offset_face_area, analytic_ported_swept_face_area,
};
use self::face_surface::ported_face_surface_descriptor_from_surface;
pub(crate) use self::face_surface::{ported_face_area, ported_face_surface_descriptor};
use self::math::{add3, approx_eq, cross3, dot3, norm3, normalize3, scale3, subtract3};
use self::mesh::{
    bbox_from_points, mesh_bbox, polyhedral_mesh_area, polyhedral_mesh_sample,
    polyhedral_mesh_volume, union_bbox,
};
use self::summary::{classify_root_kind, mesh_face_properties, ported_shape_summary, shape_counts};
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
        let counts = shape_counts(self, shape, &topology)?;
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
        let counts = shape_counts(self, shape, &topology)?;
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
