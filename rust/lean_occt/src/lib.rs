use std::ffi::{CStr, CString};
use std::fmt;
use std::os::raw::c_char;
use std::path::Path;
use std::ptr::NonNull;

use serde::{Deserialize, Serialize};

mod brep;
mod document;
mod high_level;
mod occt_port;
mod pipeline;
mod ported_geometry;
mod recipes;
mod schema;
pub use brep::{
    BrepEdge, BrepFace, BrepFaceLoop, BrepShape, BrepVertex, BrepWire, OffsetFaceBboxSource,
    OffsetShellBboxSource, SummaryBboxSource, SummaryVolumeSource,
};
pub use document::{
    EdgeDescriptor, EdgeSelector, FaceDescriptor, FaceSelector, ModelDocument, OperationRecord,
};
pub use high_level::{ModelKernel, ShapeReport, ThroughHoleCut};
pub use pipeline::{
    FeatureBuildSource, FeatureId, FeatureOperation, FeaturePersistentData, FeaturePipeline,
    FeaturePipelineBuild, FeatureRecord, FeatureRuntimeState,
};
pub use ported_geometry::{
    PortedCurve, PortedFaceSurface, PortedOffsetBasisSurface, PortedOffsetSurface, PortedSurface,
    PortedSweptSurface,
};
pub use recipes::{
    DrilledBlockRecipe, RecipeBuildResult, RoundedDrilledBlockRecipe,
    SelectorDrivenRoundedBlockRecipe,
};
pub use schema::{
    feature_definitions, FeatureDefinition, FeatureInputDefinition, FeatureParamDefinition,
    FeatureParamKind, FeatureSpec, FeatureType,
};

mod ffi {
    use super::c_char;

    #[repr(C)]
    pub struct LeanOcctContext {
        _private: [u8; 0],
    }

    #[repr(C)]
    pub struct LeanOcctShape {
        _private: [u8; 0],
    }

    #[repr(C)]
    pub struct LeanOcctMesh {
        _private: [u8; 0],
    }

    #[repr(C)]
    pub struct LeanOcctTopology {
        _private: [u8; 0],
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct LeanOcctBoxParams {
        pub x: f64,
        pub y: f64,
        pub z: f64,
        pub dx: f64,
        pub dy: f64,
        pub dz: f64,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct LeanOcctCylinderParams {
        pub x: f64,
        pub y: f64,
        pub z: f64,
        pub axis_x: f64,
        pub axis_y: f64,
        pub axis_z: f64,
        pub radius: f64,
        pub height: f64,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct LeanOcctConeParams {
        pub x: f64,
        pub y: f64,
        pub z: f64,
        pub axis_x: f64,
        pub axis_y: f64,
        pub axis_z: f64,
        pub x_dir_x: f64,
        pub x_dir_y: f64,
        pub x_dir_z: f64,
        pub base_radius: f64,
        pub top_radius: f64,
        pub height: f64,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct LeanOcctSphereParams {
        pub x: f64,
        pub y: f64,
        pub z: f64,
        pub axis_x: f64,
        pub axis_y: f64,
        pub axis_z: f64,
        pub x_dir_x: f64,
        pub x_dir_y: f64,
        pub x_dir_z: f64,
        pub radius: f64,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct LeanOcctTorusParams {
        pub x: f64,
        pub y: f64,
        pub z: f64,
        pub axis_x: f64,
        pub axis_y: f64,
        pub axis_z: f64,
        pub x_dir_x: f64,
        pub x_dir_y: f64,
        pub x_dir_z: f64,
        pub major_radius: f64,
        pub minor_radius: f64,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct LeanOcctEllipseEdgeParams {
        pub x: f64,
        pub y: f64,
        pub z: f64,
        pub axis_x: f64,
        pub axis_y: f64,
        pub axis_z: f64,
        pub x_dir_x: f64,
        pub x_dir_y: f64,
        pub x_dir_z: f64,
        pub major_radius: f64,
        pub minor_radius: f64,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct LeanOcctFilletParams {
        pub radius: f64,
        pub edge_index: u32,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct LeanOcctOffsetParams {
        pub offset: f64,
        pub tolerance: f64,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct LeanOcctCylindricalHoleParams {
        pub x: f64,
        pub y: f64,
        pub z: f64,
        pub axis_x: f64,
        pub axis_y: f64,
        pub axis_z: f64,
        pub radius: f64,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct LeanOcctHelixParams {
        pub origin_x: f64,
        pub origin_y: f64,
        pub origin_z: f64,
        pub axis_x: f64,
        pub axis_y: f64,
        pub axis_z: f64,
        pub x_dir_x: f64,
        pub x_dir_y: f64,
        pub x_dir_z: f64,
        pub radius: f64,
        pub height: f64,
        pub pitch: f64,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct LeanOcctPrismParams {
        pub dx: f64,
        pub dy: f64,
        pub dz: f64,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct LeanOcctRevolutionParams {
        pub x: f64,
        pub y: f64,
        pub z: f64,
        pub axis_x: f64,
        pub axis_y: f64,
        pub axis_z: f64,
        pub angle_radians: f64,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct LeanOcctMeshParams {
        pub linear_deflection: f64,
        pub angular_deflection: f64,
        pub is_relative: u8,
    }

    #[repr(C)]
    #[derive(Clone, Copy, PartialEq, Eq)]
    #[allow(dead_code)]
    pub enum LeanOcctResult {
        Ok = 0,
        Error = 1,
    }

    #[repr(C)]
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    #[allow(dead_code)]
    pub enum LeanOcctShapeKind {
        Unknown = 0,
        Compound = 1,
        CompSolid = 2,
        Solid = 3,
        Shell = 4,
        Face = 5,
        Wire = 6,
        Edge = 7,
        Vertex = 8,
        Shape = 9,
    }

    #[repr(C)]
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    #[allow(dead_code)]
    pub enum LeanOcctOrientation {
        Forward = 0,
        Reversed = 1,
        Internal = 2,
        External = 3,
    }

    #[repr(C)]
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    #[allow(dead_code)]
    pub enum LeanOcctLoopRole {
        Unknown = 0,
        Outer = 1,
        Inner = 2,
    }

    #[repr(C)]
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    #[allow(dead_code)]
    pub enum LeanOcctCurveKind {
        Unknown = 0,
        Line = 1,
        Circle = 2,
        Ellipse = 3,
        Hyperbola = 4,
        Parabola = 5,
        Bezier = 6,
        BSpline = 7,
        Offset = 8,
        Other = 9,
    }

    #[repr(C)]
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    #[allow(dead_code)]
    pub enum LeanOcctSurfaceKind {
        Unknown = 0,
        Plane = 1,
        Cylinder = 2,
        Cone = 3,
        Sphere = 4,
        Torus = 5,
        Bezier = 6,
        BSpline = 7,
        Revolution = 8,
        Extrusion = 9,
        Offset = 10,
        Other = 11,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct LeanOcctShapeSummary {
        pub root_kind: LeanOcctShapeKind,
        pub primary_kind: LeanOcctShapeKind,
        pub compound_count: usize,
        pub compsolid_count: usize,
        pub solid_count: usize,
        pub shell_count: usize,
        pub face_count: usize,
        pub wire_count: usize,
        pub edge_count: usize,
        pub vertex_count: usize,
        pub linear_length: f64,
        pub surface_area: f64,
        pub volume: f64,
        pub bbox_min: [f64; 3],
        pub bbox_max: [f64; 3],
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct LeanOcctEdgeSample {
        pub position: [f64; 3],
        pub tangent: [f64; 3],
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct LeanOcctBbox {
        pub min: [f64; 3],
        pub max: [f64; 3],
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct LeanOcctFaceUvBounds {
        pub u_min: f64,
        pub u_max: f64,
        pub v_min: f64,
        pub v_max: f64,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct LeanOcctFaceSample {
        pub position: [f64; 3],
        pub normal: [f64; 3],
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct LeanOcctEdgeGeometry {
        pub kind: LeanOcctCurveKind,
        pub start_parameter: f64,
        pub end_parameter: f64,
        pub is_closed: u8,
        pub is_periodic: u8,
        pub period: f64,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct LeanOcctFaceGeometry {
        pub kind: LeanOcctSurfaceKind,
        pub u_min: f64,
        pub u_max: f64,
        pub v_min: f64,
        pub v_max: f64,
        pub is_u_closed: u8,
        pub is_v_closed: u8,
        pub is_u_periodic: u8,
        pub is_v_periodic: u8,
        pub u_period: f64,
        pub v_period: f64,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct LeanOcctLinePayload {
        pub origin: [f64; 3],
        pub direction: [f64; 3],
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct LeanOcctCirclePayload {
        pub center: [f64; 3],
        pub normal: [f64; 3],
        pub x_direction: [f64; 3],
        pub y_direction: [f64; 3],
        pub radius: f64,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct LeanOcctEllipsePayload {
        pub center: [f64; 3],
        pub normal: [f64; 3],
        pub x_direction: [f64; 3],
        pub y_direction: [f64; 3],
        pub major_radius: f64,
        pub minor_radius: f64,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct LeanOcctPlanePayload {
        pub origin: [f64; 3],
        pub normal: [f64; 3],
        pub x_direction: [f64; 3],
        pub y_direction: [f64; 3],
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct LeanOcctCylinderPayload {
        pub origin: [f64; 3],
        pub axis: [f64; 3],
        pub x_direction: [f64; 3],
        pub y_direction: [f64; 3],
        pub radius: f64,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct LeanOcctConePayload {
        pub origin: [f64; 3],
        pub axis: [f64; 3],
        pub x_direction: [f64; 3],
        pub y_direction: [f64; 3],
        pub reference_radius: f64,
        pub semi_angle: f64,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct LeanOcctSpherePayload {
        pub center: [f64; 3],
        pub normal: [f64; 3],
        pub x_direction: [f64; 3],
        pub y_direction: [f64; 3],
        pub radius: f64,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct LeanOcctTorusPayload {
        pub center: [f64; 3],
        pub axis: [f64; 3],
        pub x_direction: [f64; 3],
        pub y_direction: [f64; 3],
        pub major_radius: f64,
        pub minor_radius: f64,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct LeanOcctRevolutionSurfacePayload {
        pub axis_origin: [f64; 3],
        pub axis_direction: [f64; 3],
        pub basis_curve_kind: LeanOcctCurveKind,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct LeanOcctExtrusionSurfacePayload {
        pub direction: [f64; 3],
        pub basis_curve_kind: LeanOcctCurveKind,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct LeanOcctOffsetSurfacePayload {
        pub offset_value: f64,
        pub basis_surface_kind: LeanOcctSurfaceKind,
    }

    unsafe extern "C" {
        pub fn lean_occt_context_create() -> *mut LeanOcctContext;
        pub fn lean_occt_context_destroy(context: *mut LeanOcctContext);
        pub fn lean_occt_context_last_error(context: *const LeanOcctContext) -> *const c_char;

        pub fn lean_occt_shape_make_box(
            context: *mut LeanOcctContext,
            params: *const LeanOcctBoxParams,
        ) -> *mut LeanOcctShape;
        pub fn lean_occt_shape_make_cylinder(
            context: *mut LeanOcctContext,
            params: *const LeanOcctCylinderParams,
        ) -> *mut LeanOcctShape;
        pub fn lean_occt_shape_make_cone(
            context: *mut LeanOcctContext,
            params: *const LeanOcctConeParams,
        ) -> *mut LeanOcctShape;
        pub fn lean_occt_shape_make_sphere(
            context: *mut LeanOcctContext,
            params: *const LeanOcctSphereParams,
        ) -> *mut LeanOcctShape;
        pub fn lean_occt_shape_make_torus(
            context: *mut LeanOcctContext,
            params: *const LeanOcctTorusParams,
        ) -> *mut LeanOcctShape;
        pub fn lean_occt_shape_make_ellipse_edge(
            context: *mut LeanOcctContext,
            params: *const LeanOcctEllipseEdgeParams,
        ) -> *mut LeanOcctShape;
        pub fn lean_occt_shape_make_fillet(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            params: *const LeanOcctFilletParams,
        ) -> *mut LeanOcctShape;
        pub fn lean_occt_shape_make_offset(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            params: *const LeanOcctOffsetParams,
        ) -> *mut LeanOcctShape;
        pub fn lean_occt_shape_make_offset_surface_face(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            params: *const LeanOcctOffsetParams,
        ) -> *mut LeanOcctShape;
        pub fn lean_occt_shape_make_cylindrical_hole(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            params: *const LeanOcctCylindricalHoleParams,
        ) -> *mut LeanOcctShape;
        pub fn lean_occt_shape_make_helix(
            context: *mut LeanOcctContext,
            params: *const LeanOcctHelixParams,
        ) -> *mut LeanOcctShape;
        pub fn lean_occt_shape_make_prism(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            params: *const LeanOcctPrismParams,
        ) -> *mut LeanOcctShape;
        pub fn lean_occt_shape_make_revolution(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            params: *const LeanOcctRevolutionParams,
        ) -> *mut LeanOcctShape;
        pub fn lean_occt_shape_boolean_cut(
            context: *mut LeanOcctContext,
            lhs: *const LeanOcctShape,
            rhs: *const LeanOcctShape,
        ) -> *mut LeanOcctShape;
        pub fn lean_occt_shape_boolean_fuse(
            context: *mut LeanOcctContext,
            lhs: *const LeanOcctShape,
            rhs: *const LeanOcctShape,
        ) -> *mut LeanOcctShape;
        pub fn lean_occt_shape_boolean_common(
            context: *mut LeanOcctContext,
            lhs: *const LeanOcctShape,
            rhs: *const LeanOcctShape,
        ) -> *mut LeanOcctShape;
        pub fn lean_occt_shape_read_step(
            context: *mut LeanOcctContext,
            path_utf8: *const c_char,
        ) -> *mut LeanOcctShape;
        pub fn lean_occt_shape_write_step(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            path_utf8: *const c_char,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_orientation(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            orientation: *mut LeanOcctOrientation,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_destroy(shape: *mut LeanOcctShape);
        pub fn lean_occt_shape_vertex_point(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            xyz3: *mut f64,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_edge_endpoints(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            start_xyz3: *mut f64,
            end_xyz3: *mut f64,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_edge_sample(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            t: f64,
            sample: *mut LeanOcctEdgeSample,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_edge_sample_at_parameter(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            parameter: f64,
            sample: *mut LeanOcctEdgeSample,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_edge_geometry(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            geometry: *mut LeanOcctEdgeGeometry,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_edge_curve_bbox(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            bbox: *mut LeanOcctBbox,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_edge_line_payload(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            payload: *mut LeanOcctLinePayload,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_edge_circle_payload(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            payload: *mut LeanOcctCirclePayload,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_edge_ellipse_payload(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            payload: *mut LeanOcctEllipsePayload,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_face_uv_bounds(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            bounds: *mut LeanOcctFaceUvBounds,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_face_sample(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            u: f64,
            v: f64,
            sample: *mut LeanOcctFaceSample,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_face_sample_normalized(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            u_t: f64,
            v_t: f64,
            sample: *mut LeanOcctFaceSample,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_face_geometry(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            geometry: *mut LeanOcctFaceGeometry,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_face_pcurve_control_polygon_bbox(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            bbox: *mut LeanOcctBbox,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_face_surface_bbox(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            bbox: *mut LeanOcctBbox,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_face_plane_payload(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            payload: *mut LeanOcctPlanePayload,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_face_cylinder_payload(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            payload: *mut LeanOcctCylinderPayload,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_face_cone_payload(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            payload: *mut LeanOcctConePayload,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_face_sphere_payload(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            payload: *mut LeanOcctSpherePayload,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_face_torus_payload(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            payload: *mut LeanOcctTorusPayload,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_face_revolution_payload(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            payload: *mut LeanOcctRevolutionSurfacePayload,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_face_extrusion_payload(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            payload: *mut LeanOcctExtrusionSurfacePayload,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_face_offset_payload(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            payload: *mut LeanOcctOffsetSurfacePayload,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_face_offset_basis_geometry(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            geometry: *mut LeanOcctFaceGeometry,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_face_offset_basis_plane_payload(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            payload: *mut LeanOcctPlanePayload,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_face_offset_basis_cylinder_payload(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            payload: *mut LeanOcctCylinderPayload,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_face_offset_basis_cone_payload(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            payload: *mut LeanOcctConePayload,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_face_offset_basis_sphere_payload(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            payload: *mut LeanOcctSpherePayload,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_face_offset_basis_torus_payload(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            payload: *mut LeanOcctTorusPayload,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_face_offset_basis_revolution_payload(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            payload: *mut LeanOcctRevolutionSurfacePayload,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_face_offset_basis_extrusion_payload(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            payload: *mut LeanOcctExtrusionSurfacePayload,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_face_offset_basis_curve_geometry(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            geometry: *mut LeanOcctEdgeGeometry,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_face_offset_basis_curve_line_payload(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            payload: *mut LeanOcctLinePayload,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_face_offset_basis_curve_circle_payload(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            payload: *mut LeanOcctCirclePayload,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_face_offset_basis_curve_ellipse_payload(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            payload: *mut LeanOcctEllipsePayload,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_describe(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            summary: *mut LeanOcctShapeSummary,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_subshape_count(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            kind: LeanOcctShapeKind,
            count: *mut usize,
        ) -> LeanOcctResult;
        pub fn lean_occt_shape_subshape(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            kind: LeanOcctShapeKind,
            index: usize,
        ) -> *mut LeanOcctShape;
        pub fn lean_occt_shape_edge_count(shape: *const LeanOcctShape) -> usize;
        pub fn lean_occt_shape_face_count_raw(shape: *const LeanOcctShape) -> usize;
        pub fn lean_occt_shape_solid_count_raw(shape: *const LeanOcctShape) -> usize;
        pub fn lean_occt_shape_linear_length(shape: *const LeanOcctShape) -> f64;
        pub fn lean_occt_shape_topology(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
        ) -> *mut LeanOcctTopology;
        pub fn lean_occt_topology_destroy(topology: *mut LeanOcctTopology);
        pub fn lean_occt_topology_vertex_count(topology: *const LeanOcctTopology) -> usize;
        pub fn lean_occt_topology_edge_count(topology: *const LeanOcctTopology) -> usize;
        pub fn lean_occt_topology_wire_count(topology: *const LeanOcctTopology) -> usize;
        pub fn lean_occt_topology_face_count(topology: *const LeanOcctTopology) -> usize;
        pub fn lean_occt_topology_vertex_positions(topology: *const LeanOcctTopology)
            -> *const f64;
        pub fn lean_occt_topology_edge_vertex_indices(
            topology: *const LeanOcctTopology,
        ) -> *const u32;
        pub fn lean_occt_topology_edge_lengths(topology: *const LeanOcctTopology) -> *const f64;
        pub fn lean_occt_topology_edge_face_ranges(topology: *const LeanOcctTopology)
            -> *const u32;
        pub fn lean_occt_topology_edge_face_indices(
            topology: *const LeanOcctTopology,
        ) -> *const u32;
        pub fn lean_occt_topology_wire_ranges(topology: *const LeanOcctTopology) -> *const u32;
        pub fn lean_occt_topology_wire_edge_indices(
            topology: *const LeanOcctTopology,
        ) -> *const u32;
        pub fn lean_occt_topology_wire_edge_orientations(
            topology: *const LeanOcctTopology,
        ) -> *const u8;
        pub fn lean_occt_topology_wire_vertex_ranges(
            topology: *const LeanOcctTopology,
        ) -> *const u32;
        pub fn lean_occt_topology_wire_vertex_indices(
            topology: *const LeanOcctTopology,
        ) -> *const u32;
        pub fn lean_occt_topology_face_ranges(topology: *const LeanOcctTopology) -> *const u32;
        pub fn lean_occt_topology_face_wire_indices(
            topology: *const LeanOcctTopology,
        ) -> *const u32;
        pub fn lean_occt_topology_face_wire_orientations(
            topology: *const LeanOcctTopology,
        ) -> *const u8;
        pub fn lean_occt_topology_face_wire_roles(topology: *const LeanOcctTopology) -> *const u8;

        pub fn lean_occt_shape_mesh(
            context: *mut LeanOcctContext,
            shape: *const LeanOcctShape,
            params: *const LeanOcctMeshParams,
        ) -> *mut LeanOcctMesh;
        pub fn lean_occt_mesh_destroy(mesh: *mut LeanOcctMesh);
        pub fn lean_occt_mesh_vertex_count(mesh: *const LeanOcctMesh) -> usize;
        pub fn lean_occt_mesh_triangle_count(mesh: *const LeanOcctMesh) -> usize;
        pub fn lean_occt_mesh_edge_segment_count(mesh: *const LeanOcctMesh) -> usize;
        pub fn lean_occt_mesh_face_count(mesh: *const LeanOcctMesh) -> usize;
        pub fn lean_occt_mesh_solid_count(mesh: *const LeanOcctMesh) -> usize;
        pub fn lean_occt_mesh_positions(mesh: *const LeanOcctMesh) -> *const f64;
        pub fn lean_occt_mesh_normals(mesh: *const LeanOcctMesh) -> *const f64;
        pub fn lean_occt_mesh_triangle_indices(mesh: *const LeanOcctMesh) -> *const u32;
        pub fn lean_occt_mesh_edge_positions(mesh: *const LeanOcctMesh) -> *const f64;
        pub fn lean_occt_mesh_bounds(
            mesh: *const LeanOcctMesh,
            min_xyz3: *mut f64,
            max_xyz3: *mut f64,
        );
    }
}

#[derive(Debug, Clone)]
pub struct Error {
    message: String,
}

impl Error {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for Error {}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoxParams {
    pub origin: [f64; 3],
    pub size: [f64; 3],
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CylinderParams {
    pub origin: [f64; 3],
    pub axis: [f64; 3],
    pub radius: f64,
    pub height: f64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConeParams {
    pub origin: [f64; 3],
    pub axis: [f64; 3],
    pub x_direction: [f64; 3],
    pub base_radius: f64,
    pub top_radius: f64,
    pub height: f64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SphereParams {
    pub origin: [f64; 3],
    pub axis: [f64; 3],
    pub x_direction: [f64; 3],
    pub radius: f64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TorusParams {
    pub origin: [f64; 3],
    pub axis: [f64; 3],
    pub x_direction: [f64; 3],
    pub major_radius: f64,
    pub minor_radius: f64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EllipseEdgeParams {
    pub origin: [f64; 3],
    pub axis: [f64; 3],
    pub x_direction: [f64; 3],
    pub major_radius: f64,
    pub minor_radius: f64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FilletParams {
    pub radius: f64,
    pub edge_index: u32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OffsetParams {
    pub offset: f64,
    pub tolerance: f64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CylindricalHoleParams {
    pub origin: [f64; 3],
    pub axis: [f64; 3],
    pub radius: f64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HelixParams {
    pub origin: [f64; 3],
    pub axis: [f64; 3],
    pub x_direction: [f64; 3],
    pub radius: f64,
    pub height: f64,
    pub pitch: f64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrismParams {
    pub direction: [f64; 3],
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RevolutionParams {
    pub origin: [f64; 3],
    pub axis: [f64; 3],
    pub angle_radians: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct MeshParams {
    pub linear_deflection: f64,
    pub angular_deflection: f64,
    pub is_relative: bool,
}

impl Default for MeshParams {
    fn default() -> Self {
        Self {
            linear_deflection: 0.9,
            angular_deflection: 0.35,
            is_relative: false,
        }
    }
}

#[derive(Debug)]
pub struct Mesh {
    pub positions: Vec<[f64; 3]>,
    pub normals: Vec<[f64; 3]>,
    pub triangle_indices: Vec<u32>,
    pub edge_segments: Vec<[[f64; 3]; 2]>,
    pub solid_count: usize,
    pub face_count: usize,
    pub bbox_min: [f64; 3],
    pub bbox_max: [f64; 3],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShapeKind {
    Unknown,
    Compound,
    CompSolid,
    Solid,
    Shell,
    Face,
    Wire,
    Edge,
    Vertex,
    Shape,
}

#[derive(Clone, Copy, Debug)]
pub struct ShapeSummary {
    pub root_kind: ShapeKind,
    pub primary_kind: ShapeKind,
    pub compound_count: usize,
    pub compsolid_count: usize,
    pub solid_count: usize,
    pub shell_count: usize,
    pub face_count: usize,
    pub wire_count: usize,
    pub edge_count: usize,
    pub vertex_count: usize,
    pub linear_length: f64,
    pub surface_area: f64,
    pub volume: f64,
    pub bbox_min: [f64; 3],
    pub bbox_max: [f64; 3],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Orientation {
    Forward,
    Reversed,
    Internal,
    External,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoopRole {
    Unknown,
    Outer,
    Inner,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CurveKind {
    Unknown,
    Line,
    Circle,
    Ellipse,
    Hyperbola,
    Parabola,
    Bezier,
    BSpline,
    Offset,
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SurfaceKind {
    Unknown,
    Plane,
    Cylinder,
    Cone,
    Sphere,
    Torus,
    Bezier,
    BSpline,
    Revolution,
    Extrusion,
    Offset,
    Other,
}

#[derive(Clone, Copy, Debug)]
pub struct EdgeEndpoints {
    pub start: [f64; 3],
    pub end: [f64; 3],
}

#[derive(Clone, Copy, Debug)]
pub struct EdgeSample {
    pub position: [f64; 3],
    pub tangent: [f64; 3],
}

#[derive(Clone, Copy, Debug)]
pub struct EdgeGeometry {
    pub kind: CurveKind,
    pub start_parameter: f64,
    pub end_parameter: f64,
    pub is_closed: bool,
    pub is_periodic: bool,
    pub period: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct LinePayload {
    pub origin: [f64; 3],
    pub direction: [f64; 3],
}

#[derive(Clone, Copy, Debug)]
pub struct CirclePayload {
    pub center: [f64; 3],
    pub normal: [f64; 3],
    pub x_direction: [f64; 3],
    pub y_direction: [f64; 3],
    pub radius: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct EllipsePayload {
    pub center: [f64; 3],
    pub normal: [f64; 3],
    pub x_direction: [f64; 3],
    pub y_direction: [f64; 3],
    pub major_radius: f64,
    pub minor_radius: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct FaceUvBounds {
    pub u_min: f64,
    pub u_max: f64,
    pub v_min: f64,
    pub v_max: f64,
}

impl FaceUvBounds {
    pub fn center(&self) -> [f64; 2] {
        [
            0.5 * (self.u_min + self.u_max),
            0.5 * (self.v_min + self.v_max),
        ]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FaceSample {
    pub position: [f64; 3],
    pub normal: [f64; 3],
}

#[derive(Clone, Copy, Debug)]
pub struct FaceGeometry {
    pub kind: SurfaceKind,
    pub u_min: f64,
    pub u_max: f64,
    pub v_min: f64,
    pub v_max: f64,
    pub is_u_closed: bool,
    pub is_v_closed: bool,
    pub is_u_periodic: bool,
    pub is_v_periodic: bool,
    pub u_period: f64,
    pub v_period: f64,
}

impl FaceGeometry {
    pub fn center_uv(&self) -> [f64; 2] {
        [
            0.5 * (self.u_min + self.u_max),
            0.5 * (self.v_min + self.v_max),
        ]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PlanePayload {
    pub origin: [f64; 3],
    pub normal: [f64; 3],
    pub x_direction: [f64; 3],
    pub y_direction: [f64; 3],
}

#[derive(Clone, Copy, Debug)]
pub struct CylinderPayload {
    pub origin: [f64; 3],
    pub axis: [f64; 3],
    pub x_direction: [f64; 3],
    pub y_direction: [f64; 3],
    pub radius: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct ConePayload {
    pub origin: [f64; 3],
    pub axis: [f64; 3],
    pub x_direction: [f64; 3],
    pub y_direction: [f64; 3],
    pub reference_radius: f64,
    pub semi_angle: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct SpherePayload {
    pub center: [f64; 3],
    pub normal: [f64; 3],
    pub x_direction: [f64; 3],
    pub y_direction: [f64; 3],
    pub radius: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct TorusPayload {
    pub center: [f64; 3],
    pub axis: [f64; 3],
    pub x_direction: [f64; 3],
    pub y_direction: [f64; 3],
    pub major_radius: f64,
    pub minor_radius: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct RevolutionSurfacePayload {
    pub axis_origin: [f64; 3],
    pub axis_direction: [f64; 3],
    pub basis_curve_kind: CurveKind,
}

#[derive(Clone, Copy, Debug)]
pub struct ExtrusionSurfacePayload {
    pub direction: [f64; 3],
    pub basis_curve_kind: CurveKind,
}

#[derive(Clone, Copy, Debug)]
pub struct OffsetSurfacePayload {
    pub offset_value: f64,
    pub basis_surface_kind: SurfaceKind,
}

#[derive(Clone, Copy, Debug)]
pub struct TopologyRange {
    pub offset: usize,
    pub count: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct TopologyEdge {
    pub start_vertex: Option<usize>,
    pub end_vertex: Option<usize>,
    pub length: f64,
}

#[derive(Debug)]
pub struct TopologySnapshot {
    pub vertex_positions: Vec<[f64; 3]>,
    pub edges: Vec<TopologyEdge>,
    pub edge_faces: Vec<TopologyRange>,
    pub edge_face_indices: Vec<usize>,
    pub wires: Vec<TopologyRange>,
    pub wire_edge_indices: Vec<usize>,
    pub wire_edge_orientations: Vec<Orientation>,
    pub wire_vertices: Vec<TopologyRange>,
    pub wire_vertex_indices: Vec<usize>,
    pub faces: Vec<TopologyRange>,
    pub face_wire_indices: Vec<usize>,
    pub face_wire_orientations: Vec<Orientation>,
    pub face_wire_roles: Vec<LoopRole>,
}

pub struct Context {
    raw: NonNull<ffi::LeanOcctContext>,
}

pub struct Shape {
    raw: NonNull<ffi::LeanOcctShape>,
}

struct MeshHandle {
    raw: NonNull<ffi::LeanOcctMesh>,
}

impl MeshHandle {
    fn as_ptr(&self) -> *mut ffi::LeanOcctMesh {
        self.raw.as_ptr()
    }
}

impl Drop for MeshHandle {
    fn drop(&mut self) {
        unsafe { ffi::lean_occt_mesh_destroy(self.raw.as_ptr()) };
    }
}

struct TopologyHandle {
    raw: NonNull<ffi::LeanOcctTopology>,
}

impl TopologyHandle {
    fn as_ptr(&self) -> *mut ffi::LeanOcctTopology {
        self.raw.as_ptr()
    }
}

impl Drop for TopologyHandle {
    fn drop(&mut self) {
        unsafe { ffi::lean_occt_topology_destroy(self.raw.as_ptr()) };
    }
}

fn ffi_slice<'a, T>(ptr: *const T, len: usize, label: &str) -> Result<&'a [T], Error> {
    if len == 0 {
        return Ok(&[]);
    }
    if ptr.is_null() {
        return Err(Error::new(format!(
            "Lean OCCT returned a null {label} buffer for {len} element(s)."
        )));
    }
    Ok(unsafe { std::slice::from_raw_parts(ptr, len) })
}

impl Context {
    pub fn new() -> Result<Self, Error> {
        let raw = unsafe { ffi::lean_occt_context_create() };
        let raw =
            NonNull::new(raw).ok_or_else(|| Error::new("failed to create Lean OCCT context"))?;
        Ok(Self { raw })
    }

    pub fn last_error(&self) -> String {
        unsafe {
            let message = ffi::lean_occt_context_last_error(self.raw.as_ptr());
            if message.is_null() {
                return String::new();
            }
            CStr::from_ptr(message).to_string_lossy().into_owned()
        }
    }

    pub fn make_box(&self, params: BoxParams) -> Result<Shape, Error> {
        let raw_params = ffi::LeanOcctBoxParams {
            x: params.origin[0],
            y: params.origin[1],
            z: params.origin[2],
            dx: params.size[0],
            dy: params.size[1],
            dz: params.size[2],
        };

        let raw = unsafe { ffi::lean_occt_shape_make_box(self.raw.as_ptr(), &raw_params) };
        self.wrap_shape(raw)
    }

    pub fn make_cylinder(&self, params: CylinderParams) -> Result<Shape, Error> {
        let raw_params = ffi::LeanOcctCylinderParams {
            x: params.origin[0],
            y: params.origin[1],
            z: params.origin[2],
            axis_x: params.axis[0],
            axis_y: params.axis[1],
            axis_z: params.axis[2],
            radius: params.radius,
            height: params.height,
        };

        let raw = unsafe { ffi::lean_occt_shape_make_cylinder(self.raw.as_ptr(), &raw_params) };
        self.wrap_shape(raw)
    }

    pub fn make_cone(&self, params: ConeParams) -> Result<Shape, Error> {
        let raw_params = ffi::LeanOcctConeParams {
            x: params.origin[0],
            y: params.origin[1],
            z: params.origin[2],
            axis_x: params.axis[0],
            axis_y: params.axis[1],
            axis_z: params.axis[2],
            x_dir_x: params.x_direction[0],
            x_dir_y: params.x_direction[1],
            x_dir_z: params.x_direction[2],
            base_radius: params.base_radius,
            top_radius: params.top_radius,
            height: params.height,
        };

        let raw = unsafe { ffi::lean_occt_shape_make_cone(self.raw.as_ptr(), &raw_params) };
        self.wrap_shape(raw)
    }

    pub fn make_sphere(&self, params: SphereParams) -> Result<Shape, Error> {
        let raw_params = ffi::LeanOcctSphereParams {
            x: params.origin[0],
            y: params.origin[1],
            z: params.origin[2],
            axis_x: params.axis[0],
            axis_y: params.axis[1],
            axis_z: params.axis[2],
            x_dir_x: params.x_direction[0],
            x_dir_y: params.x_direction[1],
            x_dir_z: params.x_direction[2],
            radius: params.radius,
        };

        let raw = unsafe { ffi::lean_occt_shape_make_sphere(self.raw.as_ptr(), &raw_params) };
        self.wrap_shape(raw)
    }

    pub fn make_torus(&self, params: TorusParams) -> Result<Shape, Error> {
        let raw_params = ffi::LeanOcctTorusParams {
            x: params.origin[0],
            y: params.origin[1],
            z: params.origin[2],
            axis_x: params.axis[0],
            axis_y: params.axis[1],
            axis_z: params.axis[2],
            x_dir_x: params.x_direction[0],
            x_dir_y: params.x_direction[1],
            x_dir_z: params.x_direction[2],
            major_radius: params.major_radius,
            minor_radius: params.minor_radius,
        };

        let raw = unsafe { ffi::lean_occt_shape_make_torus(self.raw.as_ptr(), &raw_params) };
        self.wrap_shape(raw)
    }

    pub fn make_ellipse_edge(&self, params: EllipseEdgeParams) -> Result<Shape, Error> {
        let raw_params = ffi::LeanOcctEllipseEdgeParams {
            x: params.origin[0],
            y: params.origin[1],
            z: params.origin[2],
            axis_x: params.axis[0],
            axis_y: params.axis[1],
            axis_z: params.axis[2],
            x_dir_x: params.x_direction[0],
            x_dir_y: params.x_direction[1],
            x_dir_z: params.x_direction[2],
            major_radius: params.major_radius,
            minor_radius: params.minor_radius,
        };

        let raw = unsafe { ffi::lean_occt_shape_make_ellipse_edge(self.raw.as_ptr(), &raw_params) };
        self.wrap_shape(raw)
    }

    pub fn make_fillet(&self, shape: &Shape, params: FilletParams) -> Result<Shape, Error> {
        let raw_params = ffi::LeanOcctFilletParams {
            radius: params.radius,
            edge_index: params.edge_index,
        };

        let raw = unsafe {
            ffi::lean_occt_shape_make_fillet(self.raw.as_ptr(), shape.raw.as_ptr(), &raw_params)
        };
        self.wrap_shape(raw)
    }

    pub fn make_offset(&self, shape: &Shape, params: OffsetParams) -> Result<Shape, Error> {
        let raw_params = ffi::LeanOcctOffsetParams {
            offset: params.offset,
            tolerance: params.tolerance,
        };

        let raw = unsafe {
            ffi::lean_occt_shape_make_offset(self.raw.as_ptr(), shape.raw.as_ptr(), &raw_params)
        };
        self.wrap_shape(raw)
    }

    pub fn make_offset_surface_face(
        &self,
        basis_face: &Shape,
        params: OffsetParams,
    ) -> Result<Shape, Error> {
        let raw_params = ffi::LeanOcctOffsetParams {
            offset: params.offset,
            tolerance: params.tolerance,
        };

        let raw = unsafe {
            ffi::lean_occt_shape_make_offset_surface_face(
                self.raw.as_ptr(),
                basis_face.raw.as_ptr(),
                &raw_params,
            )
        };
        self.wrap_shape(raw)
    }

    pub fn make_cylindrical_hole(
        &self,
        shape: &Shape,
        params: CylindricalHoleParams,
    ) -> Result<Shape, Error> {
        let raw_params = ffi::LeanOcctCylindricalHoleParams {
            x: params.origin[0],
            y: params.origin[1],
            z: params.origin[2],
            axis_x: params.axis[0],
            axis_y: params.axis[1],
            axis_z: params.axis[2],
            radius: params.radius,
        };

        let raw = unsafe {
            ffi::lean_occt_shape_make_cylindrical_hole(
                self.raw.as_ptr(),
                shape.raw.as_ptr(),
                &raw_params,
            )
        };
        self.wrap_shape(raw)
    }

    pub fn make_helix(&self, params: HelixParams) -> Result<Shape, Error> {
        let raw_params = ffi::LeanOcctHelixParams {
            origin_x: params.origin[0],
            origin_y: params.origin[1],
            origin_z: params.origin[2],
            axis_x: params.axis[0],
            axis_y: params.axis[1],
            axis_z: params.axis[2],
            x_dir_x: params.x_direction[0],
            x_dir_y: params.x_direction[1],
            x_dir_z: params.x_direction[2],
            radius: params.radius,
            height: params.height,
            pitch: params.pitch,
        };

        let raw = unsafe { ffi::lean_occt_shape_make_helix(self.raw.as_ptr(), &raw_params) };
        self.wrap_shape(raw)
    }

    pub fn make_prism(&self, shape: &Shape, params: PrismParams) -> Result<Shape, Error> {
        let raw_params = ffi::LeanOcctPrismParams {
            dx: params.direction[0],
            dy: params.direction[1],
            dz: params.direction[2],
        };

        let raw = unsafe {
            ffi::lean_occt_shape_make_prism(self.raw.as_ptr(), shape.raw.as_ptr(), &raw_params)
        };
        self.wrap_shape(raw)
    }

    pub fn make_revolution(&self, shape: &Shape, params: RevolutionParams) -> Result<Shape, Error> {
        let raw_params = ffi::LeanOcctRevolutionParams {
            x: params.origin[0],
            y: params.origin[1],
            z: params.origin[2],
            axis_x: params.axis[0],
            axis_y: params.axis[1],
            axis_z: params.axis[2],
            angle_radians: params.angle_radians,
        };

        let raw = unsafe {
            ffi::lean_occt_shape_make_revolution(self.raw.as_ptr(), shape.raw.as_ptr(), &raw_params)
        };
        self.wrap_shape(raw)
    }

    pub fn cut(&self, lhs: &Shape, rhs: &Shape) -> Result<Shape, Error> {
        let raw = unsafe {
            ffi::lean_occt_shape_boolean_cut(self.raw.as_ptr(), lhs.raw.as_ptr(), rhs.raw.as_ptr())
        };
        self.wrap_shape(raw)
    }

    pub fn fuse(&self, lhs: &Shape, rhs: &Shape) -> Result<Shape, Error> {
        let raw = unsafe {
            ffi::lean_occt_shape_boolean_fuse(self.raw.as_ptr(), lhs.raw.as_ptr(), rhs.raw.as_ptr())
        };
        self.wrap_shape(raw)
    }

    pub fn common(&self, lhs: &Shape, rhs: &Shape) -> Result<Shape, Error> {
        let raw = unsafe {
            ffi::lean_occt_shape_boolean_common(
                self.raw.as_ptr(),
                lhs.raw.as_ptr(),
                rhs.raw.as_ptr(),
            )
        };
        self.wrap_shape(raw)
    }

    pub fn read_step(&self, path: impl AsRef<Path>) -> Result<Shape, Error> {
        let path = cstring_from_path(path.as_ref())?;
        let raw = unsafe { ffi::lean_occt_shape_read_step(self.raw.as_ptr(), path.as_ptr()) };
        self.wrap_shape(raw)
    }

    pub fn write_step(&self, shape: &Shape, path: impl AsRef<Path>) -> Result<(), Error> {
        let path = cstring_from_path(path.as_ref())?;
        let result = unsafe {
            ffi::lean_occt_shape_write_step(self.raw.as_ptr(), shape.raw.as_ptr(), path.as_ptr())
        };
        if result == ffi::LeanOcctResult::Ok {
            Ok(())
        } else {
            Err(Error::new(self.last_error()))
        }
    }

    pub fn shape_orientation(&self, shape: &Shape) -> Result<Orientation, Error> {
        let mut orientation = ffi::LeanOcctOrientation::Forward;
        let result = unsafe {
            ffi::lean_occt_shape_orientation(
                self.raw.as_ptr(),
                shape.raw.as_ptr(),
                &mut orientation,
            )
        };
        if result == ffi::LeanOcctResult::Ok {
            Ok(Orientation::from_lean_occt(orientation))
        } else {
            Err(Error::new(self.last_error()))
        }
    }

    pub fn vertex_point(&self, shape: &Shape) -> Result<[f64; 3], Error> {
        match self.ported_vertex_point(shape)? {
            Some(point) => Ok(point),
            None => self.vertex_point_occt(shape),
        }
    }

    pub fn vertex_point_occt(&self, shape: &Shape) -> Result<[f64; 3], Error> {
        let mut xyz = [0.0_f64; 3];
        let result = unsafe {
            ffi::lean_occt_shape_vertex_point(
                self.raw.as_ptr(),
                shape.raw.as_ptr(),
                xyz.as_mut_ptr(),
            )
        };
        if result == ffi::LeanOcctResult::Ok {
            Ok(xyz)
        } else {
            Err(Error::new(self.last_error()))
        }
    }

    pub fn edge_endpoints(&self, shape: &Shape) -> Result<EdgeEndpoints, Error> {
        match self.ported_edge_endpoints(shape)? {
            Some(endpoints) => Ok(endpoints),
            None => self.edge_endpoints_occt(shape),
        }
    }

    pub fn edge_endpoints_occt(&self, shape: &Shape) -> Result<EdgeEndpoints, Error> {
        let mut start = [0.0_f64; 3];
        let mut end = [0.0_f64; 3];
        let result = unsafe {
            ffi::lean_occt_shape_edge_endpoints(
                self.raw.as_ptr(),
                shape.raw.as_ptr(),
                start.as_mut_ptr(),
                end.as_mut_ptr(),
            )
        };
        if result == ffi::LeanOcctResult::Ok {
            Ok(EdgeEndpoints { start, end })
        } else {
            Err(Error::new(self.last_error()))
        }
    }

    pub fn edge_sample(&self, shape: &Shape, t: f64) -> Result<EdgeSample, Error> {
        match self.ported_edge_sample(shape, t)? {
            Some(sample) => Ok(sample),
            None => self.edge_sample_occt(shape, t),
        }
    }

    pub fn edge_sample_occt(&self, shape: &Shape, t: f64) -> Result<EdgeSample, Error> {
        let mut sample = ffi::LeanOcctEdgeSample {
            position: [0.0; 3],
            tangent: [0.0; 3],
        };
        let result = unsafe {
            ffi::lean_occt_shape_edge_sample(self.raw.as_ptr(), shape.raw.as_ptr(), t, &mut sample)
        };
        if result == ffi::LeanOcctResult::Ok {
            Ok(EdgeSample {
                position: sample.position,
                tangent: sample.tangent,
            })
        } else {
            Err(Error::new(self.last_error()))
        }
    }

    pub fn edge_sample_at_parameter(
        &self,
        shape: &Shape,
        parameter: f64,
    ) -> Result<EdgeSample, Error> {
        match self.ported_edge_sample_at_parameter(shape, parameter)? {
            Some(sample) => Ok(sample),
            None => self.edge_sample_at_parameter_occt(shape, parameter),
        }
    }

    pub fn edge_sample_at_parameter_occt(
        &self,
        shape: &Shape,
        parameter: f64,
    ) -> Result<EdgeSample, Error> {
        let mut sample = ffi::LeanOcctEdgeSample {
            position: [0.0; 3],
            tangent: [0.0; 3],
        };
        let result = unsafe {
            ffi::lean_occt_shape_edge_sample_at_parameter(
                self.raw.as_ptr(),
                shape.raw.as_ptr(),
                parameter,
                &mut sample,
            )
        };
        if result == ffi::LeanOcctResult::Ok {
            Ok(EdgeSample {
                position: sample.position,
                tangent: sample.tangent,
            })
        } else {
            Err(Error::new(self.last_error()))
        }
    }

    pub fn edge_geometry(&self, shape: &Shape) -> Result<EdgeGeometry, Error> {
        match self.ported_edge_geometry(shape)? {
            Some(geometry) => Ok(geometry),
            None => self.edge_geometry_occt(shape),
        }
    }

    pub fn edge_geometry_occt(&self, shape: &Shape) -> Result<EdgeGeometry, Error> {
        let mut geometry = ffi::LeanOcctEdgeGeometry {
            kind: ffi::LeanOcctCurveKind::Unknown,
            start_parameter: 0.0,
            end_parameter: 0.0,
            is_closed: 0,
            is_periodic: 0,
            period: 0.0,
        };
        let result = unsafe {
            ffi::lean_occt_shape_edge_geometry(self.raw.as_ptr(), shape.raw.as_ptr(), &mut geometry)
        };
        if result == ffi::LeanOcctResult::Ok {
            Ok(EdgeGeometry {
                kind: CurveKind::from(geometry.kind),
                start_parameter: geometry.start_parameter,
                end_parameter: geometry.end_parameter,
                is_closed: geometry.is_closed != 0,
                is_periodic: geometry.is_periodic != 0,
                period: geometry.period,
            })
        } else {
            Err(Error::new(self.last_error()))
        }
    }

    pub fn edge_curve_bbox_occt(&self, shape: &Shape) -> Result<([f64; 3], [f64; 3]), Error> {
        let mut bbox = ffi::LeanOcctBbox {
            min: [0.0; 3],
            max: [0.0; 3],
        };
        let result = unsafe {
            ffi::lean_occt_shape_edge_curve_bbox(self.raw.as_ptr(), shape.raw.as_ptr(), &mut bbox)
        };
        if result == ffi::LeanOcctResult::Ok {
            Ok((bbox.min, bbox.max))
        } else {
            Err(Error::new(self.last_error()))
        }
    }

    pub fn edge_line_payload(&self, shape: &Shape) -> Result<LinePayload, Error> {
        match self.ported_edge_curve(shape)? {
            Some(PortedCurve::Line(payload)) => Ok(payload),
            Some(curve) => Err(mismatched_ported_curve_payload_error(
                CurveKind::Line,
                ported_curve_kind(curve),
            )),
            None => self.edge_line_payload_occt(shape),
        }
    }

    pub fn edge_line_payload_occt(&self, shape: &Shape) -> Result<LinePayload, Error> {
        let mut payload = ffi::LeanOcctLinePayload {
            origin: [0.0; 3],
            direction: [0.0; 3],
        };
        let result = unsafe {
            ffi::lean_occt_shape_edge_line_payload(
                self.raw.as_ptr(),
                shape.raw.as_ptr(),
                &mut payload,
            )
        };
        if result == ffi::LeanOcctResult::Ok {
            Ok(LinePayload {
                origin: payload.origin,
                direction: payload.direction,
            })
        } else {
            Err(Error::new(self.last_error()))
        }
    }

    pub fn edge_circle_payload(&self, shape: &Shape) -> Result<CirclePayload, Error> {
        match self.ported_edge_curve(shape)? {
            Some(PortedCurve::Circle(payload)) => Ok(payload),
            Some(curve) => Err(mismatched_ported_curve_payload_error(
                CurveKind::Circle,
                ported_curve_kind(curve),
            )),
            None => self.edge_circle_payload_occt(shape),
        }
    }

    pub fn edge_circle_payload_occt(&self, shape: &Shape) -> Result<CirclePayload, Error> {
        let mut payload = ffi::LeanOcctCirclePayload {
            center: [0.0; 3],
            normal: [0.0; 3],
            x_direction: [0.0; 3],
            y_direction: [0.0; 3],
            radius: 0.0,
        };
        let result = unsafe {
            ffi::lean_occt_shape_edge_circle_payload(
                self.raw.as_ptr(),
                shape.raw.as_ptr(),
                &mut payload,
            )
        };
        if result == ffi::LeanOcctResult::Ok {
            Ok(CirclePayload {
                center: payload.center,
                normal: payload.normal,
                x_direction: payload.x_direction,
                y_direction: payload.y_direction,
                radius: payload.radius,
            })
        } else {
            Err(Error::new(self.last_error()))
        }
    }

    pub fn edge_ellipse_payload(&self, shape: &Shape) -> Result<EllipsePayload, Error> {
        match self.ported_edge_curve(shape)? {
            Some(PortedCurve::Ellipse(payload)) => Ok(payload),
            Some(curve) => Err(mismatched_ported_curve_payload_error(
                CurveKind::Ellipse,
                ported_curve_kind(curve),
            )),
            None => self.edge_ellipse_payload_occt(shape),
        }
    }

    pub fn edge_ellipse_payload_occt(&self, shape: &Shape) -> Result<EllipsePayload, Error> {
        let mut payload = ffi::LeanOcctEllipsePayload {
            center: [0.0; 3],
            normal: [0.0; 3],
            x_direction: [0.0; 3],
            y_direction: [0.0; 3],
            major_radius: 0.0,
            minor_radius: 0.0,
        };
        let result = unsafe {
            ffi::lean_occt_shape_edge_ellipse_payload(
                self.raw.as_ptr(),
                shape.raw.as_ptr(),
                &mut payload,
            )
        };
        if result == ffi::LeanOcctResult::Ok {
            Ok(EllipsePayload {
                center: payload.center,
                normal: payload.normal,
                x_direction: payload.x_direction,
                y_direction: payload.y_direction,
                major_radius: payload.major_radius,
                minor_radius: payload.minor_radius,
            })
        } else {
            Err(Error::new(self.last_error()))
        }
    }

    pub fn face_uv_bounds(&self, shape: &Shape) -> Result<FaceUvBounds, Error> {
        let geometry = self.face_geometry(shape)?;
        Ok(FaceUvBounds {
            u_min: geometry.u_min,
            u_max: geometry.u_max,
            v_min: geometry.v_min,
            v_max: geometry.v_max,
        })
    }

    pub fn face_uv_bounds_occt(&self, shape: &Shape) -> Result<FaceUvBounds, Error> {
        let mut bounds = ffi::LeanOcctFaceUvBounds {
            u_min: 0.0,
            u_max: 0.0,
            v_min: 0.0,
            v_max: 0.0,
        };
        let result = unsafe {
            ffi::lean_occt_shape_face_uv_bounds(self.raw.as_ptr(), shape.raw.as_ptr(), &mut bounds)
        };
        if result == ffi::LeanOcctResult::Ok {
            Ok(FaceUvBounds {
                u_min: bounds.u_min,
                u_max: bounds.u_max,
                v_min: bounds.v_min,
                v_max: bounds.v_max,
            })
        } else {
            Err(Error::new(self.last_error()))
        }
    }

    pub fn face_sample(&self, shape: &Shape, uv: [f64; 2]) -> Result<FaceSample, Error> {
        match self.ported_face_sample(shape, uv)? {
            Some(sample) => Ok(sample),
            None => self.face_sample_occt(shape, uv),
        }
    }

    pub fn face_sample_occt(&self, shape: &Shape, uv: [f64; 2]) -> Result<FaceSample, Error> {
        let mut sample = ffi::LeanOcctFaceSample {
            position: [0.0; 3],
            normal: [0.0; 3],
        };
        let result = unsafe {
            ffi::lean_occt_shape_face_sample(
                self.raw.as_ptr(),
                shape.raw.as_ptr(),
                uv[0],
                uv[1],
                &mut sample,
            )
        };
        if result == ffi::LeanOcctResult::Ok {
            Ok(FaceSample {
                position: sample.position,
                normal: sample.normal,
            })
        } else {
            Err(Error::new(self.last_error()))
        }
    }

    pub fn face_sample_normalized(
        &self,
        shape: &Shape,
        uv_t: [f64; 2],
    ) -> Result<FaceSample, Error> {
        match self.ported_face_sample_normalized(shape, uv_t)? {
            Some(sample) => Ok(sample),
            None => self.face_sample_normalized_occt(shape, uv_t),
        }
    }

    pub fn face_sample_normalized_occt(
        &self,
        shape: &Shape,
        uv_t: [f64; 2],
    ) -> Result<FaceSample, Error> {
        let mut sample = ffi::LeanOcctFaceSample {
            position: [0.0; 3],
            normal: [0.0; 3],
        };
        let result = unsafe {
            ffi::lean_occt_shape_face_sample_normalized(
                self.raw.as_ptr(),
                shape.raw.as_ptr(),
                uv_t[0],
                uv_t[1],
                &mut sample,
            )
        };
        if result == ffi::LeanOcctResult::Ok {
            Ok(FaceSample {
                position: sample.position,
                normal: sample.normal,
            })
        } else {
            Err(Error::new(self.last_error()))
        }
    }

    pub fn face_geometry(&self, shape: &Shape) -> Result<FaceGeometry, Error> {
        match self.ported_face_geometry(shape)? {
            Some(geometry) => Ok(geometry),
            None => self.face_geometry_occt(shape),
        }
    }

    pub fn face_geometry_occt(&self, shape: &Shape) -> Result<FaceGeometry, Error> {
        let mut geometry = ffi::LeanOcctFaceGeometry {
            kind: ffi::LeanOcctSurfaceKind::Unknown,
            u_min: 0.0,
            u_max: 0.0,
            v_min: 0.0,
            v_max: 0.0,
            is_u_closed: 0,
            is_v_closed: 0,
            is_u_periodic: 0,
            is_v_periodic: 0,
            u_period: 0.0,
            v_period: 0.0,
        };
        let result = unsafe {
            ffi::lean_occt_shape_face_geometry(self.raw.as_ptr(), shape.raw.as_ptr(), &mut geometry)
        };
        if result == ffi::LeanOcctResult::Ok {
            Ok(FaceGeometry {
                kind: SurfaceKind::from(geometry.kind),
                u_min: geometry.u_min,
                u_max: geometry.u_max,
                v_min: geometry.v_min,
                v_max: geometry.v_max,
                is_u_closed: geometry.is_u_closed != 0,
                is_v_closed: geometry.is_v_closed != 0,
                is_u_periodic: geometry.is_u_periodic != 0,
                is_v_periodic: geometry.is_v_periodic != 0,
                u_period: geometry.u_period,
                v_period: geometry.v_period,
            })
        } else {
            Err(Error::new(self.last_error()))
        }
    }

    pub fn face_pcurve_control_polygon_bbox_occt(
        &self,
        shape: &Shape,
    ) -> Result<([f64; 3], [f64; 3]), Error> {
        let mut bbox = ffi::LeanOcctBbox {
            min: [0.0; 3],
            max: [0.0; 3],
        };
        let result = unsafe {
            ffi::lean_occt_shape_face_pcurve_control_polygon_bbox(
                self.raw.as_ptr(),
                shape.raw.as_ptr(),
                &mut bbox,
            )
        };
        if result == ffi::LeanOcctResult::Ok {
            Ok((bbox.min, bbox.max))
        } else {
            Err(Error::new(self.last_error()))
        }
    }

    pub fn face_surface_bbox_occt(&self, shape: &Shape) -> Result<([f64; 3], [f64; 3]), Error> {
        let mut bbox = ffi::LeanOcctBbox {
            min: [0.0; 3],
            max: [0.0; 3],
        };
        let result = unsafe {
            ffi::lean_occt_shape_face_surface_bbox(self.raw.as_ptr(), shape.raw.as_ptr(), &mut bbox)
        };
        if result == ffi::LeanOcctResult::Ok {
            Ok((bbox.min, bbox.max))
        } else {
            Err(Error::new(self.last_error()))
        }
    }

    fn ported_analytic_face_surface_payload(
        &self,
        shape: &Shape,
        expected: SurfaceKind,
    ) -> Result<PortedSurface, Error> {
        if let Some(surface) = self.ported_face_surface(shape)? {
            return Ok(surface);
        }

        let geometry = self.face_geometry_occt(shape)?;
        if geometry.kind != expected {
            return Err(mismatched_ported_surface_payload_error(
                expected,
                geometry.kind,
            ));
        }

        PortedSurface::from_context_with_geometry(self, shape, geometry)?
            .ok_or_else(|| unsupported_ported_surface_payload_error(expected, geometry.kind))
    }

    pub fn face_plane_payload(&self, shape: &Shape) -> Result<PlanePayload, Error> {
        match self.ported_analytic_face_surface_payload(shape, SurfaceKind::Plane)? {
            PortedSurface::Plane(payload) => Ok(payload),
            surface => Err(mismatched_ported_surface_payload_error(
                SurfaceKind::Plane,
                ported_surface_kind(surface),
            )),
        }
    }

    pub fn face_plane_payload_occt(&self, shape: &Shape) -> Result<PlanePayload, Error> {
        let mut payload = ffi::LeanOcctPlanePayload {
            origin: [0.0; 3],
            normal: [0.0; 3],
            x_direction: [0.0; 3],
            y_direction: [0.0; 3],
        };
        let result = unsafe {
            ffi::lean_occt_shape_face_plane_payload(
                self.raw.as_ptr(),
                shape.raw.as_ptr(),
                &mut payload,
            )
        };
        if result == ffi::LeanOcctResult::Ok {
            Ok(PlanePayload {
                origin: payload.origin,
                normal: payload.normal,
                x_direction: payload.x_direction,
                y_direction: payload.y_direction,
            })
        } else {
            Err(Error::new(self.last_error()))
        }
    }

    pub fn face_cylinder_payload(&self, shape: &Shape) -> Result<CylinderPayload, Error> {
        match self.ported_analytic_face_surface_payload(shape, SurfaceKind::Cylinder)? {
            PortedSurface::Cylinder(payload) => Ok(payload),
            surface => Err(mismatched_ported_surface_payload_error(
                SurfaceKind::Cylinder,
                ported_surface_kind(surface),
            )),
        }
    }

    pub fn face_cylinder_payload_occt(&self, shape: &Shape) -> Result<CylinderPayload, Error> {
        let mut payload = ffi::LeanOcctCylinderPayload {
            origin: [0.0; 3],
            axis: [0.0; 3],
            x_direction: [0.0; 3],
            y_direction: [0.0; 3],
            radius: 0.0,
        };
        let result = unsafe {
            ffi::lean_occt_shape_face_cylinder_payload(
                self.raw.as_ptr(),
                shape.raw.as_ptr(),
                &mut payload,
            )
        };
        if result == ffi::LeanOcctResult::Ok {
            Ok(CylinderPayload {
                origin: payload.origin,
                axis: payload.axis,
                x_direction: payload.x_direction,
                y_direction: payload.y_direction,
                radius: payload.radius,
            })
        } else {
            Err(Error::new(self.last_error()))
        }
    }

    pub fn face_cone_payload(&self, shape: &Shape) -> Result<ConePayload, Error> {
        match self.ported_analytic_face_surface_payload(shape, SurfaceKind::Cone)? {
            PortedSurface::Cone(payload) => Ok(payload),
            surface => Err(mismatched_ported_surface_payload_error(
                SurfaceKind::Cone,
                ported_surface_kind(surface),
            )),
        }
    }

    pub fn face_cone_payload_occt(&self, shape: &Shape) -> Result<ConePayload, Error> {
        let mut payload = ffi::LeanOcctConePayload {
            origin: [0.0; 3],
            axis: [0.0; 3],
            x_direction: [0.0; 3],
            y_direction: [0.0; 3],
            reference_radius: 0.0,
            semi_angle: 0.0,
        };
        let result = unsafe {
            ffi::lean_occt_shape_face_cone_payload(
                self.raw.as_ptr(),
                shape.raw.as_ptr(),
                &mut payload,
            )
        };
        if result == ffi::LeanOcctResult::Ok {
            Ok(ConePayload {
                origin: payload.origin,
                axis: payload.axis,
                x_direction: payload.x_direction,
                y_direction: payload.y_direction,
                reference_radius: payload.reference_radius,
                semi_angle: payload.semi_angle,
            })
        } else {
            Err(Error::new(self.last_error()))
        }
    }

    pub fn face_sphere_payload(&self, shape: &Shape) -> Result<SpherePayload, Error> {
        match self.ported_analytic_face_surface_payload(shape, SurfaceKind::Sphere)? {
            PortedSurface::Sphere(payload) => Ok(payload),
            surface => Err(mismatched_ported_surface_payload_error(
                SurfaceKind::Sphere,
                ported_surface_kind(surface),
            )),
        }
    }

    pub fn face_sphere_payload_occt(&self, shape: &Shape) -> Result<SpherePayload, Error> {
        let mut payload = ffi::LeanOcctSpherePayload {
            center: [0.0; 3],
            normal: [0.0; 3],
            x_direction: [0.0; 3],
            y_direction: [0.0; 3],
            radius: 0.0,
        };
        let result = unsafe {
            ffi::lean_occt_shape_face_sphere_payload(
                self.raw.as_ptr(),
                shape.raw.as_ptr(),
                &mut payload,
            )
        };
        if result == ffi::LeanOcctResult::Ok {
            Ok(SpherePayload {
                center: payload.center,
                normal: payload.normal,
                x_direction: payload.x_direction,
                y_direction: payload.y_direction,
                radius: payload.radius,
            })
        } else {
            Err(Error::new(self.last_error()))
        }
    }

    pub fn face_torus_payload(&self, shape: &Shape) -> Result<TorusPayload, Error> {
        match self.ported_analytic_face_surface_payload(shape, SurfaceKind::Torus)? {
            PortedSurface::Torus(payload) => Ok(payload),
            surface => Err(mismatched_ported_surface_payload_error(
                SurfaceKind::Torus,
                ported_surface_kind(surface),
            )),
        }
    }

    pub fn face_torus_payload_occt(&self, shape: &Shape) -> Result<TorusPayload, Error> {
        let mut payload = ffi::LeanOcctTorusPayload {
            center: [0.0; 3],
            axis: [0.0; 3],
            x_direction: [0.0; 3],
            y_direction: [0.0; 3],
            major_radius: 0.0,
            minor_radius: 0.0,
        };
        let result = unsafe {
            ffi::lean_occt_shape_face_torus_payload(
                self.raw.as_ptr(),
                shape.raw.as_ptr(),
                &mut payload,
            )
        };
        if result == ffi::LeanOcctResult::Ok {
            Ok(TorusPayload {
                center: payload.center,
                axis: payload.axis,
                x_direction: payload.x_direction,
                y_direction: payload.y_direction,
                major_radius: payload.major_radius,
                minor_radius: payload.minor_radius,
            })
        } else {
            Err(Error::new(self.last_error()))
        }
    }

    pub fn face_revolution_payload(
        &self,
        shape: &Shape,
    ) -> Result<RevolutionSurfacePayload, Error> {
        match self.ported_face_surface_descriptor(shape)? {
            Some(PortedFaceSurface::Swept(PortedSweptSurface::Revolution { payload, .. })) => {
                Ok(payload)
            }
            Some(surface) => Err(mismatched_ported_surface_payload_error(
                SurfaceKind::Revolution,
                ported_face_surface_descriptor_kind(surface),
            )),
            None => self.face_revolution_payload_occt(shape),
        }
    }

    pub fn face_revolution_payload_occt(
        &self,
        shape: &Shape,
    ) -> Result<RevolutionSurfacePayload, Error> {
        let mut payload = ffi::LeanOcctRevolutionSurfacePayload {
            axis_origin: [0.0; 3],
            axis_direction: [0.0; 3],
            basis_curve_kind: ffi::LeanOcctCurveKind::Unknown,
        };
        let result = unsafe {
            ffi::lean_occt_shape_face_revolution_payload(
                self.raw.as_ptr(),
                shape.raw.as_ptr(),
                &mut payload,
            )
        };
        if result == ffi::LeanOcctResult::Ok {
            Ok(RevolutionSurfacePayload {
                axis_origin: payload.axis_origin,
                axis_direction: payload.axis_direction,
                basis_curve_kind: CurveKind::from(payload.basis_curve_kind),
            })
        } else {
            Err(Error::new(self.last_error()))
        }
    }

    pub fn face_extrusion_payload(&self, shape: &Shape) -> Result<ExtrusionSurfacePayload, Error> {
        match self.ported_face_surface_descriptor(shape)? {
            Some(PortedFaceSurface::Swept(PortedSweptSurface::Extrusion { payload, .. })) => {
                Ok(payload)
            }
            Some(surface) => Err(mismatched_ported_surface_payload_error(
                SurfaceKind::Extrusion,
                ported_face_surface_descriptor_kind(surface),
            )),
            None => self.face_extrusion_payload_occt(shape),
        }
    }

    pub fn face_extrusion_payload_occt(
        &self,
        shape: &Shape,
    ) -> Result<ExtrusionSurfacePayload, Error> {
        let mut payload = ffi::LeanOcctExtrusionSurfacePayload {
            direction: [0.0; 3],
            basis_curve_kind: ffi::LeanOcctCurveKind::Unknown,
        };
        let result = unsafe {
            ffi::lean_occt_shape_face_extrusion_payload(
                self.raw.as_ptr(),
                shape.raw.as_ptr(),
                &mut payload,
            )
        };
        if result == ffi::LeanOcctResult::Ok {
            Ok(ExtrusionSurfacePayload {
                direction: payload.direction,
                basis_curve_kind: CurveKind::from(payload.basis_curve_kind),
            })
        } else {
            Err(Error::new(self.last_error()))
        }
    }

    pub fn face_offset_payload(&self, shape: &Shape) -> Result<OffsetSurfacePayload, Error> {
        match self.ported_offset_surface(shape)? {
            Some(surface) => Ok(surface.payload),
            None => self.face_offset_payload_occt(shape),
        }
    }

    pub fn face_offset_payload_occt(&self, shape: &Shape) -> Result<OffsetSurfacePayload, Error> {
        let mut payload = ffi::LeanOcctOffsetSurfacePayload {
            offset_value: 0.0,
            basis_surface_kind: ffi::LeanOcctSurfaceKind::Unknown,
        };
        let result = unsafe {
            ffi::lean_occt_shape_face_offset_payload(
                self.raw.as_ptr(),
                shape.raw.as_ptr(),
                &mut payload,
            )
        };
        if result == ffi::LeanOcctResult::Ok {
            Ok(OffsetSurfacePayload {
                offset_value: payload.offset_value,
                basis_surface_kind: SurfaceKind::from(payload.basis_surface_kind),
            })
        } else {
            Err(Error::new(self.last_error()))
        }
    }

    pub fn face_offset_basis_geometry(&self, shape: &Shape) -> Result<FaceGeometry, Error> {
        match self.ported_offset_surface(shape)? {
            Some(surface) => Ok(surface.basis_geometry),
            None => self.face_offset_basis_geometry_occt(shape),
        }
    }

    pub fn face_offset_basis_geometry_occt(&self, shape: &Shape) -> Result<FaceGeometry, Error> {
        let mut geometry = ffi::LeanOcctFaceGeometry {
            kind: ffi::LeanOcctSurfaceKind::Unknown,
            u_min: 0.0,
            u_max: 0.0,
            v_min: 0.0,
            v_max: 0.0,
            is_u_closed: 0,
            is_v_closed: 0,
            is_u_periodic: 0,
            is_v_periodic: 0,
            u_period: 0.0,
            v_period: 0.0,
        };
        let result = unsafe {
            ffi::lean_occt_shape_face_offset_basis_geometry(
                self.raw.as_ptr(),
                shape.raw.as_ptr(),
                &mut geometry,
            )
        };
        if result == ffi::LeanOcctResult::Ok {
            Ok(FaceGeometry {
                kind: SurfaceKind::from(geometry.kind),
                u_min: geometry.u_min,
                u_max: geometry.u_max,
                v_min: geometry.v_min,
                v_max: geometry.v_max,
                is_u_closed: geometry.is_u_closed != 0,
                is_v_closed: geometry.is_v_closed != 0,
                is_u_periodic: geometry.is_u_periodic != 0,
                is_v_periodic: geometry.is_v_periodic != 0,
                u_period: geometry.u_period,
                v_period: geometry.v_period,
            })
        } else {
            Err(Error::new(self.last_error()))
        }
    }

    pub fn face_offset_basis_plane_payload(&self, shape: &Shape) -> Result<PlanePayload, Error> {
        match self.ported_offset_surface(shape)? {
            Some(surface) => match surface.basis {
                PortedOffsetBasisSurface::Analytic(PortedSurface::Plane(payload)) => Ok(payload),
                basis => Err(mismatched_ported_offset_basis_payload_error(
                    SurfaceKind::Plane,
                    ported_offset_basis_surface_kind(basis),
                )),
            },
            None => self.face_offset_basis_plane_payload_occt(shape),
        }
    }

    pub fn face_offset_basis_plane_payload_occt(
        &self,
        shape: &Shape,
    ) -> Result<PlanePayload, Error> {
        let mut payload = ffi::LeanOcctPlanePayload {
            origin: [0.0; 3],
            normal: [0.0; 3],
            x_direction: [0.0; 3],
            y_direction: [0.0; 3],
        };
        let result = unsafe {
            ffi::lean_occt_shape_face_offset_basis_plane_payload(
                self.raw.as_ptr(),
                shape.raw.as_ptr(),
                &mut payload,
            )
        };
        if result == ffi::LeanOcctResult::Ok {
            Ok(PlanePayload {
                origin: payload.origin,
                normal: payload.normal,
                x_direction: payload.x_direction,
                y_direction: payload.y_direction,
            })
        } else {
            Err(Error::new(self.last_error()))
        }
    }

    pub fn face_offset_basis_cylinder_payload(
        &self,
        shape: &Shape,
    ) -> Result<CylinderPayload, Error> {
        match self.ported_offset_surface(shape)? {
            Some(surface) => match surface.basis {
                PortedOffsetBasisSurface::Analytic(PortedSurface::Cylinder(payload)) => Ok(payload),
                basis => Err(mismatched_ported_offset_basis_payload_error(
                    SurfaceKind::Cylinder,
                    ported_offset_basis_surface_kind(basis),
                )),
            },
            None => self.face_offset_basis_cylinder_payload_occt(shape),
        }
    }

    pub fn face_offset_basis_cylinder_payload_occt(
        &self,
        shape: &Shape,
    ) -> Result<CylinderPayload, Error> {
        let mut payload = ffi::LeanOcctCylinderPayload {
            origin: [0.0; 3],
            axis: [0.0; 3],
            x_direction: [0.0; 3],
            y_direction: [0.0; 3],
            radius: 0.0,
        };
        let result = unsafe {
            ffi::lean_occt_shape_face_offset_basis_cylinder_payload(
                self.raw.as_ptr(),
                shape.raw.as_ptr(),
                &mut payload,
            )
        };
        if result == ffi::LeanOcctResult::Ok {
            Ok(CylinderPayload {
                origin: payload.origin,
                axis: payload.axis,
                x_direction: payload.x_direction,
                y_direction: payload.y_direction,
                radius: payload.radius,
            })
        } else {
            Err(Error::new(self.last_error()))
        }
    }

    pub fn face_offset_basis_cone_payload(&self, shape: &Shape) -> Result<ConePayload, Error> {
        match self.ported_offset_surface(shape)? {
            Some(surface) => match surface.basis {
                PortedOffsetBasisSurface::Analytic(PortedSurface::Cone(payload)) => Ok(payload),
                basis => Err(mismatched_ported_offset_basis_payload_error(
                    SurfaceKind::Cone,
                    ported_offset_basis_surface_kind(basis),
                )),
            },
            None => self.face_offset_basis_cone_payload_occt(shape),
        }
    }

    pub fn face_offset_basis_cone_payload_occt(&self, shape: &Shape) -> Result<ConePayload, Error> {
        let mut payload = ffi::LeanOcctConePayload {
            origin: [0.0; 3],
            axis: [0.0; 3],
            x_direction: [0.0; 3],
            y_direction: [0.0; 3],
            reference_radius: 0.0,
            semi_angle: 0.0,
        };
        let result = unsafe {
            ffi::lean_occt_shape_face_offset_basis_cone_payload(
                self.raw.as_ptr(),
                shape.raw.as_ptr(),
                &mut payload,
            )
        };
        if result == ffi::LeanOcctResult::Ok {
            Ok(ConePayload {
                origin: payload.origin,
                axis: payload.axis,
                x_direction: payload.x_direction,
                y_direction: payload.y_direction,
                reference_radius: payload.reference_radius,
                semi_angle: payload.semi_angle,
            })
        } else {
            Err(Error::new(self.last_error()))
        }
    }

    pub fn face_offset_basis_sphere_payload(&self, shape: &Shape) -> Result<SpherePayload, Error> {
        match self.ported_offset_surface(shape)? {
            Some(surface) => match surface.basis {
                PortedOffsetBasisSurface::Analytic(PortedSurface::Sphere(payload)) => Ok(payload),
                basis => Err(mismatched_ported_offset_basis_payload_error(
                    SurfaceKind::Sphere,
                    ported_offset_basis_surface_kind(basis),
                )),
            },
            None => self.face_offset_basis_sphere_payload_occt(shape),
        }
    }

    pub fn face_offset_basis_sphere_payload_occt(
        &self,
        shape: &Shape,
    ) -> Result<SpherePayload, Error> {
        let mut payload = ffi::LeanOcctSpherePayload {
            center: [0.0; 3],
            normal: [0.0; 3],
            x_direction: [0.0; 3],
            y_direction: [0.0; 3],
            radius: 0.0,
        };
        let result = unsafe {
            ffi::lean_occt_shape_face_offset_basis_sphere_payload(
                self.raw.as_ptr(),
                shape.raw.as_ptr(),
                &mut payload,
            )
        };
        if result == ffi::LeanOcctResult::Ok {
            Ok(SpherePayload {
                center: payload.center,
                normal: payload.normal,
                x_direction: payload.x_direction,
                y_direction: payload.y_direction,
                radius: payload.radius,
            })
        } else {
            Err(Error::new(self.last_error()))
        }
    }

    pub fn face_offset_basis_torus_payload(&self, shape: &Shape) -> Result<TorusPayload, Error> {
        match self.ported_offset_surface(shape)? {
            Some(surface) => match surface.basis {
                PortedOffsetBasisSurface::Analytic(PortedSurface::Torus(payload)) => Ok(payload),
                basis => Err(mismatched_ported_offset_basis_payload_error(
                    SurfaceKind::Torus,
                    ported_offset_basis_surface_kind(basis),
                )),
            },
            None => self.face_offset_basis_torus_payload_occt(shape),
        }
    }

    pub fn face_offset_basis_torus_payload_occt(
        &self,
        shape: &Shape,
    ) -> Result<TorusPayload, Error> {
        let mut payload = ffi::LeanOcctTorusPayload {
            center: [0.0; 3],
            axis: [0.0; 3],
            x_direction: [0.0; 3],
            y_direction: [0.0; 3],
            major_radius: 0.0,
            minor_radius: 0.0,
        };
        let result = unsafe {
            ffi::lean_occt_shape_face_offset_basis_torus_payload(
                self.raw.as_ptr(),
                shape.raw.as_ptr(),
                &mut payload,
            )
        };
        if result == ffi::LeanOcctResult::Ok {
            Ok(TorusPayload {
                center: payload.center,
                axis: payload.axis,
                x_direction: payload.x_direction,
                y_direction: payload.y_direction,
                major_radius: payload.major_radius,
                minor_radius: payload.minor_radius,
            })
        } else {
            Err(Error::new(self.last_error()))
        }
    }

    pub fn face_offset_basis_revolution_payload(
        &self,
        shape: &Shape,
    ) -> Result<RevolutionSurfacePayload, Error> {
        match self.ported_offset_surface(shape)? {
            Some(surface) => match surface.basis {
                PortedOffsetBasisSurface::Swept(PortedSweptSurface::Revolution {
                    payload, ..
                }) => Ok(payload),
                basis => Err(mismatched_ported_offset_basis_payload_error(
                    SurfaceKind::Revolution,
                    ported_offset_basis_surface_kind(basis),
                )),
            },
            None => self.face_offset_basis_revolution_payload_occt(shape),
        }
    }

    pub fn face_offset_basis_revolution_payload_occt(
        &self,
        shape: &Shape,
    ) -> Result<RevolutionSurfacePayload, Error> {
        let mut payload = ffi::LeanOcctRevolutionSurfacePayload {
            axis_origin: [0.0; 3],
            axis_direction: [0.0; 3],
            basis_curve_kind: ffi::LeanOcctCurveKind::Unknown,
        };
        let result = unsafe {
            ffi::lean_occt_shape_face_offset_basis_revolution_payload(
                self.raw.as_ptr(),
                shape.raw.as_ptr(),
                &mut payload,
            )
        };
        if result == ffi::LeanOcctResult::Ok {
            Ok(RevolutionSurfacePayload {
                axis_origin: payload.axis_origin,
                axis_direction: payload.axis_direction,
                basis_curve_kind: CurveKind::from(payload.basis_curve_kind),
            })
        } else {
            Err(Error::new(self.last_error()))
        }
    }

    pub fn face_offset_basis_extrusion_payload(
        &self,
        shape: &Shape,
    ) -> Result<ExtrusionSurfacePayload, Error> {
        match self.ported_offset_surface(shape)? {
            Some(surface) => match surface.basis {
                PortedOffsetBasisSurface::Swept(PortedSweptSurface::Extrusion {
                    payload, ..
                }) => Ok(payload),
                basis => Err(mismatched_ported_offset_basis_payload_error(
                    SurfaceKind::Extrusion,
                    ported_offset_basis_surface_kind(basis),
                )),
            },
            None => self.face_offset_basis_extrusion_payload_occt(shape),
        }
    }

    pub fn face_offset_basis_extrusion_payload_occt(
        &self,
        shape: &Shape,
    ) -> Result<ExtrusionSurfacePayload, Error> {
        let mut payload = ffi::LeanOcctExtrusionSurfacePayload {
            direction: [0.0; 3],
            basis_curve_kind: ffi::LeanOcctCurveKind::Unknown,
        };
        let result = unsafe {
            ffi::lean_occt_shape_face_offset_basis_extrusion_payload(
                self.raw.as_ptr(),
                shape.raw.as_ptr(),
                &mut payload,
            )
        };
        if result == ffi::LeanOcctResult::Ok {
            Ok(ExtrusionSurfacePayload {
                direction: payload.direction,
                basis_curve_kind: CurveKind::from(payload.basis_curve_kind),
            })
        } else {
            Err(Error::new(self.last_error()))
        }
    }

    pub fn face_offset_basis_curve_geometry(&self, shape: &Shape) -> Result<EdgeGeometry, Error> {
        match self.ported_offset_surface(shape)? {
            Some(surface) => match surface.basis {
                PortedOffsetBasisSurface::Swept(PortedSweptSurface::Revolution {
                    basis_geometry,
                    ..
                })
                | PortedOffsetBasisSurface::Swept(PortedSweptSurface::Extrusion {
                    basis_geometry,
                    ..
                }) => Ok(basis_geometry),
                basis => Err(unsupported_ported_offset_basis_curve_geometry_error(
                    ported_offset_basis_surface_kind(basis),
                )),
            },
            None => self.face_offset_basis_curve_geometry_occt(shape),
        }
    }

    pub fn face_offset_basis_curve_geometry_occt(
        &self,
        shape: &Shape,
    ) -> Result<EdgeGeometry, Error> {
        let mut geometry = ffi::LeanOcctEdgeGeometry {
            kind: ffi::LeanOcctCurveKind::Unknown,
            start_parameter: 0.0,
            end_parameter: 0.0,
            is_closed: 0,
            is_periodic: 0,
            period: 0.0,
        };
        let result = unsafe {
            ffi::lean_occt_shape_face_offset_basis_curve_geometry(
                self.raw.as_ptr(),
                shape.raw.as_ptr(),
                &mut geometry,
            )
        };
        if result == ffi::LeanOcctResult::Ok {
            Ok(EdgeGeometry {
                kind: CurveKind::from(geometry.kind),
                start_parameter: geometry.start_parameter,
                end_parameter: geometry.end_parameter,
                is_closed: geometry.is_closed != 0,
                is_periodic: geometry.is_periodic != 0,
                period: geometry.period,
            })
        } else {
            Err(Error::new(self.last_error()))
        }
    }

    pub fn face_offset_basis_curve_line_payload(
        &self,
        shape: &Shape,
    ) -> Result<LinePayload, Error> {
        match self.ported_offset_surface(shape)? {
            Some(surface) => match surface.basis {
                PortedOffsetBasisSurface::Swept(PortedSweptSurface::Revolution {
                    basis_curve: PortedCurve::Line(payload),
                    ..
                })
                | PortedOffsetBasisSurface::Swept(PortedSweptSurface::Extrusion {
                    basis_curve: PortedCurve::Line(payload),
                    ..
                }) => Ok(payload),
                PortedOffsetBasisSurface::Swept(
                    PortedSweptSurface::Revolution { basis_curve, .. }
                    | PortedSweptSurface::Extrusion { basis_curve, .. },
                ) => Err(mismatched_ported_offset_basis_curve_payload_error(
                    CurveKind::Line,
                    ported_curve_kind(basis_curve),
                )),
                basis => Err(unsupported_ported_offset_basis_curve_payload_error(
                    CurveKind::Line,
                    ported_offset_basis_surface_kind(basis),
                )),
            },
            None => self.face_offset_basis_curve_line_payload_occt(shape),
        }
    }

    pub fn face_offset_basis_curve_line_payload_occt(
        &self,
        shape: &Shape,
    ) -> Result<LinePayload, Error> {
        let mut payload = ffi::LeanOcctLinePayload {
            origin: [0.0; 3],
            direction: [0.0; 3],
        };
        let result = unsafe {
            ffi::lean_occt_shape_face_offset_basis_curve_line_payload(
                self.raw.as_ptr(),
                shape.raw.as_ptr(),
                &mut payload,
            )
        };
        if result == ffi::LeanOcctResult::Ok {
            Ok(LinePayload {
                origin: payload.origin,
                direction: payload.direction,
            })
        } else {
            Err(Error::new(self.last_error()))
        }
    }

    pub fn face_offset_basis_curve_circle_payload(
        &self,
        shape: &Shape,
    ) -> Result<CirclePayload, Error> {
        match self.ported_offset_surface(shape)? {
            Some(surface) => match surface.basis {
                PortedOffsetBasisSurface::Swept(PortedSweptSurface::Revolution {
                    basis_curve: PortedCurve::Circle(payload),
                    ..
                })
                | PortedOffsetBasisSurface::Swept(PortedSweptSurface::Extrusion {
                    basis_curve: PortedCurve::Circle(payload),
                    ..
                }) => Ok(payload),
                PortedOffsetBasisSurface::Swept(
                    PortedSweptSurface::Revolution { basis_curve, .. }
                    | PortedSweptSurface::Extrusion { basis_curve, .. },
                ) => Err(mismatched_ported_offset_basis_curve_payload_error(
                    CurveKind::Circle,
                    ported_curve_kind(basis_curve),
                )),
                basis => Err(unsupported_ported_offset_basis_curve_payload_error(
                    CurveKind::Circle,
                    ported_offset_basis_surface_kind(basis),
                )),
            },
            None => self.face_offset_basis_curve_circle_payload_occt(shape),
        }
    }

    pub fn face_offset_basis_curve_circle_payload_occt(
        &self,
        shape: &Shape,
    ) -> Result<CirclePayload, Error> {
        let mut payload = ffi::LeanOcctCirclePayload {
            center: [0.0; 3],
            normal: [0.0; 3],
            x_direction: [0.0; 3],
            y_direction: [0.0; 3],
            radius: 0.0,
        };
        let result = unsafe {
            ffi::lean_occt_shape_face_offset_basis_curve_circle_payload(
                self.raw.as_ptr(),
                shape.raw.as_ptr(),
                &mut payload,
            )
        };
        if result == ffi::LeanOcctResult::Ok {
            Ok(CirclePayload {
                center: payload.center,
                normal: payload.normal,
                x_direction: payload.x_direction,
                y_direction: payload.y_direction,
                radius: payload.radius,
            })
        } else {
            Err(Error::new(self.last_error()))
        }
    }

    pub fn face_offset_basis_curve_ellipse_payload(
        &self,
        shape: &Shape,
    ) -> Result<EllipsePayload, Error> {
        match self.ported_offset_surface(shape)? {
            Some(surface) => match surface.basis {
                PortedOffsetBasisSurface::Swept(PortedSweptSurface::Revolution {
                    basis_curve: PortedCurve::Ellipse(payload),
                    ..
                })
                | PortedOffsetBasisSurface::Swept(PortedSweptSurface::Extrusion {
                    basis_curve: PortedCurve::Ellipse(payload),
                    ..
                }) => Ok(payload),
                PortedOffsetBasisSurface::Swept(
                    PortedSweptSurface::Revolution { basis_curve, .. }
                    | PortedSweptSurface::Extrusion { basis_curve, .. },
                ) => Err(mismatched_ported_offset_basis_curve_payload_error(
                    CurveKind::Ellipse,
                    ported_curve_kind(basis_curve),
                )),
                basis => Err(unsupported_ported_offset_basis_curve_payload_error(
                    CurveKind::Ellipse,
                    ported_offset_basis_surface_kind(basis),
                )),
            },
            None => self.face_offset_basis_curve_ellipse_payload_occt(shape),
        }
    }

    pub fn face_offset_basis_curve_ellipse_payload_occt(
        &self,
        shape: &Shape,
    ) -> Result<EllipsePayload, Error> {
        let mut payload = ffi::LeanOcctEllipsePayload {
            center: [0.0; 3],
            normal: [0.0; 3],
            x_direction: [0.0; 3],
            y_direction: [0.0; 3],
            major_radius: 0.0,
            minor_radius: 0.0,
        };
        let result = unsafe {
            ffi::lean_occt_shape_face_offset_basis_curve_ellipse_payload(
                self.raw.as_ptr(),
                shape.raw.as_ptr(),
                &mut payload,
            )
        };
        if result == ffi::LeanOcctResult::Ok {
            Ok(EllipsePayload {
                center: payload.center,
                normal: payload.normal,
                x_direction: payload.x_direction,
                y_direction: payload.y_direction,
                major_radius: payload.major_radius,
                minor_radius: payload.minor_radius,
            })
        } else {
            Err(Error::new(self.last_error()))
        }
    }

    pub fn describe_shape(&self, shape: &Shape) -> Result<ShapeSummary, Error> {
        self.ported_brep(shape).map(|brep| brep.summary)
    }

    pub fn describe_shape_occt(&self, shape: &Shape) -> Result<ShapeSummary, Error> {
        let mut raw_summary = ffi::LeanOcctShapeSummary {
            root_kind: ffi::LeanOcctShapeKind::Unknown,
            primary_kind: ffi::LeanOcctShapeKind::Unknown,
            compound_count: 0,
            compsolid_count: 0,
            solid_count: 0,
            shell_count: 0,
            face_count: 0,
            wire_count: 0,
            edge_count: 0,
            vertex_count: 0,
            linear_length: 0.0,
            surface_area: 0.0,
            volume: 0.0,
            bbox_min: [0.0; 3],
            bbox_max: [0.0; 3],
        };

        let result = unsafe {
            ffi::lean_occt_shape_describe(self.raw.as_ptr(), shape.raw.as_ptr(), &mut raw_summary)
        };
        if result != ffi::LeanOcctResult::Ok {
            return Err(Error::new(self.last_error()));
        }

        Ok(ShapeSummary {
            root_kind: ShapeKind::from(raw_summary.root_kind),
            primary_kind: ShapeKind::from(raw_summary.primary_kind),
            compound_count: raw_summary.compound_count,
            compsolid_count: raw_summary.compsolid_count,
            solid_count: raw_summary.solid_count,
            shell_count: raw_summary.shell_count,
            face_count: raw_summary.face_count,
            wire_count: raw_summary.wire_count,
            edge_count: raw_summary.edge_count,
            vertex_count: raw_summary.vertex_count,
            linear_length: raw_summary.linear_length,
            surface_area: raw_summary.surface_area,
            volume: raw_summary.volume,
            bbox_min: raw_summary.bbox_min,
            bbox_max: raw_summary.bbox_max,
        })
    }

    pub fn topology(&self, shape: &Shape) -> Result<TopologySnapshot, Error> {
        match self.ported_topology(shape)? {
            Some(topology) => Ok(topology),
            None => self.topology_occt(shape),
        }
    }

    pub fn topology_occt(&self, shape: &Shape) -> Result<TopologySnapshot, Error> {
        let raw = unsafe { ffi::lean_occt_shape_topology(self.raw.as_ptr(), shape.raw.as_ptr()) };
        let raw = TopologyHandle {
            raw: NonNull::new(raw).ok_or_else(|| Error::new(self.last_error()))?,
        };

        let vertex_count = unsafe { ffi::lean_occt_topology_vertex_count(raw.as_ptr()) };
        let edge_count = unsafe { ffi::lean_occt_topology_edge_count(raw.as_ptr()) };
        let wire_count = unsafe { ffi::lean_occt_topology_wire_count(raw.as_ptr()) };
        let face_count = unsafe { ffi::lean_occt_topology_face_count(raw.as_ptr()) };

        let vertex_positions = ffi_slice(
            unsafe { ffi::lean_occt_topology_vertex_positions(raw.as_ptr()) },
            vertex_count * 3,
            "topology vertex position",
        )?
        .chunks_exact(3)
        .map(|chunk| [chunk[0], chunk[1], chunk[2]])
        .collect::<Vec<_>>();

        let edge_vertex_indices = ffi_slice(
            unsafe { ffi::lean_occt_topology_edge_vertex_indices(raw.as_ptr()) },
            edge_count * 2,
            "topology edge vertex index",
        )?;
        let edge_lengths = ffi_slice(
            unsafe { ffi::lean_occt_topology_edge_lengths(raw.as_ptr()) },
            edge_count,
            "topology edge length",
        )?;
        let edges = (0..edge_count)
            .map(|index| TopologyEdge {
                start_vertex: optional_index(edge_vertex_indices[index * 2]),
                end_vertex: optional_index(edge_vertex_indices[index * 2 + 1]),
                length: edge_lengths[index],
            })
            .collect::<Vec<_>>();

        let edge_face_ranges = ffi_slice(
            unsafe { ffi::lean_occt_topology_edge_face_ranges(raw.as_ptr()) },
            edge_count * 2,
            "topology edge-face range",
        )?;
        let edge_face_total = edge_face_ranges
            .chunks_exact(2)
            .last()
            .map(|chunk| chunk[0] as usize + chunk[1] as usize)
            .unwrap_or(0);
        let edge_face_indices = ffi_slice(
            unsafe { ffi::lean_occt_topology_edge_face_indices(raw.as_ptr()) },
            edge_face_total,
            "topology edge-face index",
        )?
        .iter()
        .map(|&value| value as usize)
        .collect::<Vec<_>>();
        let edge_faces = edge_face_ranges
            .chunks_exact(2)
            .map(|chunk| TopologyRange {
                offset: chunk[0] as usize,
                count: chunk[1] as usize,
            })
            .collect::<Vec<_>>();

        let wire_ranges = ffi_slice(
            unsafe { ffi::lean_occt_topology_wire_ranges(raw.as_ptr()) },
            wire_count * 2,
            "topology wire range",
        )?;
        let wire_edge_total = wire_ranges
            .chunks_exact(2)
            .last()
            .map(|chunk| chunk[0] as usize + chunk[1] as usize)
            .unwrap_or(0);
        let wire_edge_indices = ffi_slice(
            unsafe { ffi::lean_occt_topology_wire_edge_indices(raw.as_ptr()) },
            wire_edge_total,
            "topology wire edge index",
        )?
        .iter()
        .map(|&value| value as usize)
        .collect::<Vec<_>>();
        let wire_edge_orientations = ffi_slice(
            unsafe { ffi::lean_occt_topology_wire_edge_orientations(raw.as_ptr()) },
            wire_edge_indices.len(),
            "topology wire edge orientation",
        )?
        .iter()
        .map(|&value| Orientation::from_ffi(value))
        .collect::<Vec<_>>();
        let wires = wire_ranges
            .chunks_exact(2)
            .map(|chunk| TopologyRange {
                offset: chunk[0] as usize,
                count: chunk[1] as usize,
            })
            .collect::<Vec<_>>();

        let wire_vertex_ranges = ffi_slice(
            unsafe { ffi::lean_occt_topology_wire_vertex_ranges(raw.as_ptr()) },
            wire_count * 2,
            "topology wire vertex range",
        )?;
        let wire_vertex_total = wire_vertex_ranges
            .chunks_exact(2)
            .last()
            .map(|chunk| chunk[0] as usize + chunk[1] as usize)
            .unwrap_or(0);
        let wire_vertex_indices = ffi_slice(
            unsafe { ffi::lean_occt_topology_wire_vertex_indices(raw.as_ptr()) },
            wire_vertex_total,
            "topology wire vertex index",
        )?
        .iter()
        .map(|&value| value as usize)
        .collect::<Vec<_>>();
        let wire_vertices = wire_vertex_ranges
            .chunks_exact(2)
            .map(|chunk| TopologyRange {
                offset: chunk[0] as usize,
                count: chunk[1] as usize,
            })
            .collect::<Vec<_>>();

        let face_ranges = ffi_slice(
            unsafe { ffi::lean_occt_topology_face_ranges(raw.as_ptr()) },
            face_count * 2,
            "topology face range",
        )?;
        let face_wire_total = face_ranges
            .chunks_exact(2)
            .last()
            .map(|chunk| chunk[0] as usize + chunk[1] as usize)
            .unwrap_or(0);
        let face_wire_indices = ffi_slice(
            unsafe { ffi::lean_occt_topology_face_wire_indices(raw.as_ptr()) },
            face_wire_total,
            "topology face wire index",
        )?
        .iter()
        .map(|&value| value as usize)
        .collect::<Vec<_>>();
        let face_wire_orientations = ffi_slice(
            unsafe { ffi::lean_occt_topology_face_wire_orientations(raw.as_ptr()) },
            face_wire_indices.len(),
            "topology face wire orientation",
        )?
        .iter()
        .map(|&value| Orientation::from_ffi(value))
        .collect::<Vec<_>>();
        let face_wire_roles = ffi_slice(
            unsafe { ffi::lean_occt_topology_face_wire_roles(raw.as_ptr()) },
            face_wire_indices.len(),
            "topology face wire role",
        )?
        .iter()
        .map(|&value| LoopRole::from_ffi(value))
        .collect::<Vec<_>>();
        let faces = face_ranges
            .chunks_exact(2)
            .map(|chunk| TopologyRange {
                offset: chunk[0] as usize,
                count: chunk[1] as usize,
            })
            .collect::<Vec<_>>();

        Ok(TopologySnapshot {
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
        })
    }

    pub fn subshape_count(&self, shape: &Shape, kind: ShapeKind) -> Result<usize, Error> {
        match kind {
            ShapeKind::Face | ShapeKind::Wire | ShapeKind::Edge | ShapeKind::Vertex => {
                match self.ported_topology(shape)? {
                    Some(topology) => Ok(match kind {
                        ShapeKind::Face => topology.faces.len(),
                        ShapeKind::Wire => topology.wires.len(),
                        ShapeKind::Edge => topology.edges.len(),
                        ShapeKind::Vertex => topology.vertex_positions.len(),
                        _ => unreachable!("handled by the outer match"),
                    }),
                    None => self.subshape_count_occt(shape, kind),
                }
            }
            ShapeKind::Compound | ShapeKind::CompSolid | ShapeKind::Solid | ShapeKind::Shell => {
                let summary = self.describe_shape(shape)?;
                Ok(match kind {
                    ShapeKind::Compound => summary.compound_count,
                    ShapeKind::CompSolid => summary.compsolid_count,
                    ShapeKind::Solid => summary.solid_count,
                    ShapeKind::Shell => summary.shell_count,
                    _ => unreachable!("handled by the outer match"),
                })
            }
            _ => self.subshape_count_occt(shape, kind),
        }
    }

    pub fn subshape_count_occt(&self, shape: &Shape, kind: ShapeKind) -> Result<usize, Error> {
        let mut count = 0_usize;
        let result = unsafe {
            ffi::lean_occt_shape_subshape_count(
                self.raw.as_ptr(),
                shape.raw.as_ptr(),
                kind.to_ffi(),
                &mut count,
            )
        };
        if result == ffi::LeanOcctResult::Ok {
            Ok(count)
        } else {
            Err(Error::new(self.last_error()))
        }
    }

    pub fn subshape(&self, shape: &Shape, kind: ShapeKind, index: usize) -> Result<Shape, Error> {
        match self.ported_subshape(shape, kind, index)? {
            Some(subshape) => Ok(subshape),
            None => self.subshape_occt(shape, kind, index),
        }
    }

    pub fn subshape_occt(
        &self,
        shape: &Shape,
        kind: ShapeKind,
        index: usize,
    ) -> Result<Shape, Error> {
        let raw = unsafe {
            ffi::lean_occt_shape_subshape(
                self.raw.as_ptr(),
                shape.raw.as_ptr(),
                kind.to_ffi(),
                index,
            )
        };
        self.wrap_shape(raw)
    }

    pub fn subshapes(&self, shape: &Shape, kind: ShapeKind) -> Result<Vec<Shape>, Error> {
        match self.ported_subshapes(shape, kind)? {
            Some(shapes) => Ok(shapes),
            None => self.subshapes_occt(shape, kind),
        }
    }

    pub fn subshapes_occt(&self, shape: &Shape, kind: ShapeKind) -> Result<Vec<Shape>, Error> {
        let count = self.subshape_count_occt(shape, kind)?;
        let mut shapes = Vec::with_capacity(count);
        for index in 0..count {
            shapes.push(self.subshape_occt(shape, kind, index)?);
        }
        Ok(shapes)
    }

    pub fn mesh(&self, shape: &Shape, params: MeshParams) -> Result<Mesh, Error> {
        let raw_params = ffi::LeanOcctMeshParams {
            linear_deflection: params.linear_deflection,
            angular_deflection: params.angular_deflection,
            is_relative: u8::from(params.is_relative),
        };

        let raw = unsafe {
            ffi::lean_occt_shape_mesh(self.raw.as_ptr(), shape.raw.as_ptr(), &raw_params)
        };
        let raw = MeshHandle {
            raw: NonNull::new(raw).ok_or_else(|| Error::new(self.last_error()))?,
        };

        let vertex_count = unsafe { ffi::lean_occt_mesh_vertex_count(raw.as_ptr()) };
        let triangle_count = unsafe { ffi::lean_occt_mesh_triangle_count(raw.as_ptr()) };
        let edge_segment_count = unsafe { ffi::lean_occt_mesh_edge_segment_count(raw.as_ptr()) };
        let face_count = unsafe { ffi::lean_occt_mesh_face_count(raw.as_ptr()) };
        let solid_count = unsafe { ffi::lean_occt_mesh_solid_count(raw.as_ptr()) };

        let positions = ffi_slice(
            unsafe { ffi::lean_occt_mesh_positions(raw.as_ptr()) },
            vertex_count * 3,
            "mesh position",
        )?
        .chunks_exact(3)
        .map(|chunk| [chunk[0], chunk[1], chunk[2]])
        .collect();
        let normals = ffi_slice(
            unsafe { ffi::lean_occt_mesh_normals(raw.as_ptr()) },
            vertex_count * 3,
            "mesh normal",
        )?
        .chunks_exact(3)
        .map(|chunk| [chunk[0], chunk[1], chunk[2]])
        .collect();
        let triangle_indices = ffi_slice(
            unsafe { ffi::lean_occt_mesh_triangle_indices(raw.as_ptr()) },
            triangle_count * 3,
            "mesh triangle index",
        )?
        .to_vec();
        let edge_segments = ffi_slice(
            unsafe { ffi::lean_occt_mesh_edge_positions(raw.as_ptr()) },
            edge_segment_count * 6,
            "mesh edge position",
        )?
        .chunks_exact(6)
        .map(|chunk| {
            [
                [chunk[0], chunk[1], chunk[2]],
                [chunk[3], chunk[4], chunk[5]],
            ]
        })
        .collect();

        let mut bbox_min = [0.0_f64; 3];
        let mut bbox_max = [0.0_f64; 3];
        unsafe {
            ffi::lean_occt_mesh_bounds(raw.as_ptr(), bbox_min.as_mut_ptr(), bbox_max.as_mut_ptr());
        }

        Ok(Mesh {
            positions,
            normals,
            triangle_indices,
            edge_segments,
            solid_count,
            face_count,
            bbox_min,
            bbox_max,
        })
    }

    fn wrap_shape(&self, raw: *mut ffi::LeanOcctShape) -> Result<Shape, Error> {
        let raw = NonNull::new(raw).ok_or_else(|| Error::new(self.last_error()))?;
        Ok(Shape { raw })
    }
}

fn ported_curve_kind(curve: PortedCurve) -> CurveKind {
    match curve {
        PortedCurve::Line(_) => CurveKind::Line,
        PortedCurve::Circle(_) => CurveKind::Circle,
        PortedCurve::Ellipse(_) => CurveKind::Ellipse,
    }
}

fn ported_surface_kind(surface: PortedSurface) -> SurfaceKind {
    match surface {
        PortedSurface::Plane(_) => SurfaceKind::Plane,
        PortedSurface::Cylinder(_) => SurfaceKind::Cylinder,
        PortedSurface::Cone(_) => SurfaceKind::Cone,
        PortedSurface::Sphere(_) => SurfaceKind::Sphere,
        PortedSurface::Torus(_) => SurfaceKind::Torus,
    }
}

fn ported_swept_surface_kind(surface: PortedSweptSurface) -> SurfaceKind {
    match surface {
        PortedSweptSurface::Revolution { .. } => SurfaceKind::Revolution,
        PortedSweptSurface::Extrusion { .. } => SurfaceKind::Extrusion,
    }
}

fn ported_offset_basis_surface_kind(basis: PortedOffsetBasisSurface) -> SurfaceKind {
    match basis {
        PortedOffsetBasisSurface::Analytic(surface) => ported_surface_kind(surface),
        PortedOffsetBasisSurface::Swept(surface) => ported_swept_surface_kind(surface),
    }
}

fn ported_face_surface_descriptor_kind(surface: PortedFaceSurface) -> SurfaceKind {
    match surface {
        PortedFaceSurface::Analytic(surface) => ported_surface_kind(surface),
        PortedFaceSurface::Swept(surface) => ported_swept_surface_kind(surface),
        PortedFaceSurface::Offset(_) => SurfaceKind::Offset,
    }
}

fn mismatched_ported_curve_payload_error(expected: CurveKind, actual: CurveKind) -> Error {
    Error::new(format!(
        "requested {expected:?} payload for ported {actual:?} edge"
    ))
}

fn mismatched_ported_surface_payload_error(expected: SurfaceKind, actual: SurfaceKind) -> Error {
    Error::new(format!(
        "requested {expected:?} payload for ported {actual:?} face"
    ))
}

fn unsupported_ported_surface_payload_error(expected: SurfaceKind, actual: SurfaceKind) -> Error {
    Error::new(format!(
        "Rust-owned {expected:?} payload extraction did not cover {actual:?} face"
    ))
}

fn mismatched_ported_offset_basis_payload_error(
    expected: SurfaceKind,
    actual: SurfaceKind,
) -> Error {
    Error::new(format!(
        "requested {expected:?} offset-basis payload for ported {actual:?} offset basis"
    ))
}

fn unsupported_ported_offset_basis_curve_geometry_error(actual: SurfaceKind) -> Error {
    Error::new(format!(
        "requested offset-basis curve geometry for ported {actual:?} offset basis"
    ))
}

fn mismatched_ported_offset_basis_curve_payload_error(
    expected: CurveKind,
    actual: CurveKind,
) -> Error {
    Error::new(format!(
        "requested {expected:?} offset-basis curve payload for ported {actual:?} offset basis curve"
    ))
}

fn unsupported_ported_offset_basis_curve_payload_error(
    expected: CurveKind,
    actual: SurfaceKind,
) -> Error {
    Error::new(format!(
        "requested {expected:?} offset-basis curve payload for ported {actual:?} offset basis"
    ))
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { ffi::lean_occt_context_destroy(self.raw.as_ptr()) };
    }
}

impl Drop for Shape {
    fn drop(&mut self) {
        unsafe { ffi::lean_occt_shape_destroy(self.raw.as_ptr()) };
    }
}

impl From<ffi::LeanOcctShapeKind> for ShapeKind {
    fn from(value: ffi::LeanOcctShapeKind) -> Self {
        match value {
            ffi::LeanOcctShapeKind::Compound => Self::Compound,
            ffi::LeanOcctShapeKind::CompSolid => Self::CompSolid,
            ffi::LeanOcctShapeKind::Solid => Self::Solid,
            ffi::LeanOcctShapeKind::Shell => Self::Shell,
            ffi::LeanOcctShapeKind::Face => Self::Face,
            ffi::LeanOcctShapeKind::Wire => Self::Wire,
            ffi::LeanOcctShapeKind::Edge => Self::Edge,
            ffi::LeanOcctShapeKind::Vertex => Self::Vertex,
            ffi::LeanOcctShapeKind::Shape => Self::Shape,
            ffi::LeanOcctShapeKind::Unknown => Self::Unknown,
        }
    }
}

impl ShapeKind {
    fn to_ffi(self) -> ffi::LeanOcctShapeKind {
        match self {
            Self::Unknown => ffi::LeanOcctShapeKind::Unknown,
            Self::Compound => ffi::LeanOcctShapeKind::Compound,
            Self::CompSolid => ffi::LeanOcctShapeKind::CompSolid,
            Self::Solid => ffi::LeanOcctShapeKind::Solid,
            Self::Shell => ffi::LeanOcctShapeKind::Shell,
            Self::Face => ffi::LeanOcctShapeKind::Face,
            Self::Wire => ffi::LeanOcctShapeKind::Wire,
            Self::Edge => ffi::LeanOcctShapeKind::Edge,
            Self::Vertex => ffi::LeanOcctShapeKind::Vertex,
            Self::Shape => ffi::LeanOcctShapeKind::Shape,
        }
    }
}

impl Orientation {
    fn from_lean_occt(value: ffi::LeanOcctOrientation) -> Self {
        match value {
            ffi::LeanOcctOrientation::Reversed => Self::Reversed,
            ffi::LeanOcctOrientation::Internal => Self::Internal,
            ffi::LeanOcctOrientation::External => Self::External,
            ffi::LeanOcctOrientation::Forward => Self::Forward,
        }
    }

    fn from_ffi(value: u8) -> Self {
        match value {
            1 => Self::Reversed,
            2 => Self::Internal,
            3 => Self::External,
            _ => Self::Forward,
        }
    }
}

impl LoopRole {
    fn from_ffi(value: u8) -> Self {
        match value {
            1 => Self::Outer,
            2 => Self::Inner,
            _ => Self::Unknown,
        }
    }
}

impl From<ffi::LeanOcctCurveKind> for CurveKind {
    fn from(value: ffi::LeanOcctCurveKind) -> Self {
        match value {
            ffi::LeanOcctCurveKind::Line => Self::Line,
            ffi::LeanOcctCurveKind::Circle => Self::Circle,
            ffi::LeanOcctCurveKind::Ellipse => Self::Ellipse,
            ffi::LeanOcctCurveKind::Hyperbola => Self::Hyperbola,
            ffi::LeanOcctCurveKind::Parabola => Self::Parabola,
            ffi::LeanOcctCurveKind::Bezier => Self::Bezier,
            ffi::LeanOcctCurveKind::BSpline => Self::BSpline,
            ffi::LeanOcctCurveKind::Offset => Self::Offset,
            ffi::LeanOcctCurveKind::Other => Self::Other,
            ffi::LeanOcctCurveKind::Unknown => Self::Unknown,
        }
    }
}

impl From<ffi::LeanOcctSurfaceKind> for SurfaceKind {
    fn from(value: ffi::LeanOcctSurfaceKind) -> Self {
        match value {
            ffi::LeanOcctSurfaceKind::Plane => Self::Plane,
            ffi::LeanOcctSurfaceKind::Cylinder => Self::Cylinder,
            ffi::LeanOcctSurfaceKind::Cone => Self::Cone,
            ffi::LeanOcctSurfaceKind::Sphere => Self::Sphere,
            ffi::LeanOcctSurfaceKind::Torus => Self::Torus,
            ffi::LeanOcctSurfaceKind::Bezier => Self::Bezier,
            ffi::LeanOcctSurfaceKind::BSpline => Self::BSpline,
            ffi::LeanOcctSurfaceKind::Revolution => Self::Revolution,
            ffi::LeanOcctSurfaceKind::Extrusion => Self::Extrusion,
            ffi::LeanOcctSurfaceKind::Offset => Self::Offset,
            ffi::LeanOcctSurfaceKind::Other => Self::Other,
            ffi::LeanOcctSurfaceKind::Unknown => Self::Unknown,
        }
    }
}

impl Shape {
    pub fn edge_count(&self) -> usize {
        unsafe { ffi::lean_occt_shape_edge_count(self.raw.as_ptr()) }
    }

    pub fn face_count_raw(&self) -> usize {
        unsafe { ffi::lean_occt_shape_face_count_raw(self.raw.as_ptr()) }
    }

    pub fn solid_count_raw(&self) -> usize {
        unsafe { ffi::lean_occt_shape_solid_count_raw(self.raw.as_ptr()) }
    }

    pub fn linear_length(&self) -> f64 {
        unsafe { ffi::lean_occt_shape_linear_length(self.raw.as_ptr()) }
    }
}

fn optional_index(value: u32) -> Option<usize> {
    if value == u32::MAX {
        None
    } else {
        Some(value as usize)
    }
}

fn cstring_from_path(path: &Path) -> Result<CString, Error> {
    CString::new(path.to_string_lossy().as_bytes()).map_err(|_| {
        Error::new(format!(
            "path contains an interior NUL byte: {}",
            path.display()
        ))
    })
}
