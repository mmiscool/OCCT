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
use self::summary::{mesh_face_properties, ported_shape_summary};
use self::topology::{
    face_adjacent_face_indices, face_loops, ported_brep_edges, ported_brep_vertices,
    ported_brep_wires, ported_edge_endpoints, ported_topology_snapshot, ported_vertex_point,
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
        let vertices = ported_brep_vertices(&topology);
        let wires = ported_brep_wires(&topology);
        let (edge_shapes, edges) = ported_brep_edges(self, shape, &topology)?;

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
        ported_vertex_point(self, shape)
    }

    pub fn ported_edge_endpoints(&self, shape: &Shape) -> Result<Option<EdgeEndpoints>, Error> {
        ported_edge_endpoints(self, shape)
    }
}
