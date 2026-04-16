use std::collections::{BTreeMap, BTreeSet};
use std::f64::consts::PI;

mod brep_materialize;
mod edge_topology;
mod face_metrics;
mod face_prepare;
mod face_queries;
mod face_snapshot;
mod face_surface;
mod face_topology;
mod math;
mod mesh;
mod shape_queries;
mod summary;
mod swept_face;
mod topology;
mod topology_access;
mod wire_topology;

use self::brep_materialize::{ported_brep_edges, ported_brep_vertices, ported_brep_wires};
pub(crate) use self::face_queries::{ported_face_area, ported_face_surface_descriptor};
use self::face_surface::ported_brep_faces;
use self::face_topology::FaceSurfaceRoute;
use self::math::{add3, approx_eq, cross3, dot3, norm3, normalize3, scale3, subtract3};
use self::mesh::{
    bbox_from_points, mesh_bbox, polyhedral_mesh_area, polyhedral_mesh_sample,
    polyhedral_mesh_volume, union_bbox,
};
use self::shape_queries::{
    ported_edge_endpoints, ported_subshape, ported_subshapes, ported_vertex_point,
};
use self::summary::ported_shape_summary;
use self::topology::{load_ported_topology, ported_topology_snapshot};

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
        let (topology, vertex_shapes, edge_shapes, face_shapes, face_route) =
            match load_ported_topology(self, shape)? {
                Some(loaded) => (
                    loaded.topology,
                    loaded.vertex_shapes,
                    loaded.edge_shapes,
                    loaded.face_shapes,
                    FaceSurfaceRoute::Public,
                ),
                None => (
                    self.topology_occt(shape)?,
                    self.subshapes_occt(shape, ShapeKind::Vertex)?,
                    self.subshapes_occt(shape, ShapeKind::Edge)?,
                    self.subshapes_occt(shape, ShapeKind::Face)?,
                    FaceSurfaceRoute::Raw,
                ),
            };
        let vertices = ported_brep_vertices(&topology);
        let wires = ported_brep_wires(&topology);
        let edges = ported_brep_edges(self, &edge_shapes, &topology)?;
        let faces = ported_brep_faces(
            self,
            &face_shapes,
            &topology,
            &wires,
            &edges,
            &edge_shapes,
            face_route,
        )?;
        let summary = ported_shape_summary(
            self,
            shape,
            &vertices,
            &topology,
            &wires,
            &edges,
            &faces,
            &vertex_shapes,
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

    pub(crate) fn ported_subshape(
        &self,
        shape: &Shape,
        kind: ShapeKind,
        index: usize,
    ) -> Result<Option<Shape>, Error> {
        ported_subshape(self, shape, kind, index)
    }

    pub(crate) fn ported_subshapes(
        &self,
        shape: &Shape,
        kind: ShapeKind,
    ) -> Result<Option<Vec<Shape>>, Error> {
        ported_subshapes(self, shape, kind)
    }
}
