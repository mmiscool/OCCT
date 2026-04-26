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
    ported_edge_endpoints, ported_subshape, ported_subshape_count, ported_subshapes,
    ported_vertex_point,
};
use self::summary::{ported_offset_shell_bbox_sources, ported_shape_summary};
pub(crate) use self::topology::ported_root_edge_geometry;
use self::topology::{
    load_ported_topology, ported_topology_snapshot, root_assembly_requires_ported_topology,
    PreparedShellShape,
};

use crate::ported_geometry::{
    analytic_sampled_wire_signed_area, analytic_sampled_wire_signed_volume, extrusion_swept_area,
    planar_wire_signed_area, ported_swept_face_surface_from_samples, revolution_swept_area,
    sample_extrusion_surface_normalized, sample_revolution_surface_normalized, PortedFaceSurface,
    PortedOffsetBasisSurface, PortedOffsetSurface, PortedSweptSurface,
};
use crate::{
    rust_owned_face_query_required_kind, ConePayload, Context, CurveKind, CylinderPayload,
    EdgeEndpoints, EdgeGeometry, Error, FaceGeometry, FaceSample, LoopRole, Mesh, MeshParams,
    Orientation, PlanePayload, PortedCurve, PortedSurface, Shape, ShapeKind, ShapeSummary,
    SpherePayload, TopologySnapshot, TorusPayload,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OffsetShellBboxSource {
    FaceBrep,
    Boundary,
    Mesh,
    Brep,
    OcctFallback,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OffsetFaceBboxSource {
    ValidatedMesh,
    ValidatedFaceBrep,
    SummaryFaceBrep,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SummaryBboxSource {
    ExactPrimitive,
    PortedBrep,
    OffsetFaceUnion,
    OffsetValidatedMesh,
    OffsetSolidShellUnion,
    Mesh,
    OcctFallback,
    Zero,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SummaryVolumeSource {
    ExactPrimitive,
    FaceContributions,
    WholeShapeMesh,
    OcctFallback,
    Zero,
}

#[derive(Debug)]
pub struct BrepShape {
    pub summary: ShapeSummary,
    pub topology: TopologySnapshot,
    pub vertices: Vec<BrepVertex>,
    pub wires: Vec<BrepWire>,
    pub edges: Vec<BrepEdge>,
    pub faces: Vec<BrepFace>,
    summary_bbox_source: SummaryBboxSource,
    summary_volume_source: SummaryVolumeSource,
    offset_shell_bbox_sources: Vec<OffsetShellBboxSource>,
    offset_face_bbox_source: Option<OffsetFaceBboxSource>,
}

impl BrepShape {
    pub fn summary_bbox_source(&self) -> SummaryBboxSource {
        self.summary_bbox_source
    }

    pub fn summary_volume_source(&self) -> SummaryVolumeSource {
        self.summary_volume_source
    }

    pub fn offset_shell_bbox_sources(&self) -> &[OffsetShellBboxSource] {
        &self.offset_shell_bbox_sources
    }

    pub fn offset_face_bbox_source(&self) -> Option<OffsetFaceBboxSource> {
        self.offset_face_bbox_source
    }
}

fn strict_brep_raw_topology_fallback_allowed(
    context: &Context,
    shape: &Shape,
) -> Result<bool, Error> {
    Ok(!strict_brep_requires_ported_topology(context, shape)?)
}

fn strict_brep_requires_ported_topology(context: &Context, shape: &Shape) -> Result<bool, Error> {
    let summary = context.describe_shape_occt(shape)?;
    match summary.root_kind {
        ShapeKind::Edge => strict_brep_root_edge_requires_ported_topology(context, shape),
        ShapeKind::Wire => Ok(summary.face_count == 0 && summary.edge_count > 0),
        ShapeKind::Compound => {
            if root_assembly_requires_ported_topology(context, shape)? {
                Ok(true)
            } else if summary.face_count == 0 {
                Ok(false)
            } else {
                strict_brep_face_inventory_requires_ported_topology(
                    context,
                    shape,
                    summary.face_count,
                )
            }
        }
        ShapeKind::Face | ShapeKind::Shell | ShapeKind::Solid | ShapeKind::CompSolid => {
            if summary.face_count == 0 {
                Ok(false)
            } else {
                strict_brep_face_inventory_requires_ported_topology(
                    context,
                    shape,
                    summary.face_count,
                )
            }
        }
        ShapeKind::Unknown | ShapeKind::Vertex | ShapeKind::Shape => Ok(false),
    }
}

fn strict_brep_root_edge_requires_ported_topology(
    context: &Context,
    shape: &Shape,
) -> Result<bool, Error> {
    let Some(geometry) = ported_root_edge_geometry(context, shape)? else {
        return Ok(false);
    };
    Ok(matches!(
        geometry.kind,
        CurveKind::Line | CurveKind::Circle | CurveKind::Ellipse
    ))
}

fn strict_brep_face_inventory_requires_ported_topology(
    context: &Context,
    shape: &Shape,
    face_count: usize,
) -> Result<bool, Error> {
    let face_shapes = context.subshapes_occt(shape, ShapeKind::Face)?;
    if face_shapes.len() != face_count {
        return Ok(false);
    }

    for face_shape in face_shapes {
        if rust_owned_face_query_required_kind(context, &face_shape)?.is_none() {
            return Ok(false);
        }
    }

    Ok(face_count > 0)
}

fn strict_brep_missing_ported_topology_error(
    context: &Context,
    shape: &Shape,
) -> Result<Error, Error> {
    let summary = context.describe_shape_occt(shape)?;
    Ok(Error::new(format!(
        "Rust-owned BRep materialization requires ported topology for supported {:?} root \
         (faces={}, wires={}, edges={}, vertices={}), but the topology loader returned no snapshot",
        summary.root_kind,
        summary.face_count,
        summary.wire_count,
        summary.edge_count,
        summary.vertex_count
    )))
}

impl Context {
    pub fn ported_topology(&self, shape: &Shape) -> Result<Option<TopologySnapshot>, Error> {
        ported_topology_snapshot(self, shape)
    }

    pub fn ported_brep(&self, shape: &Shape) -> Result<BrepShape, Error> {
        let (topology, edge_shapes, solid_shapes, prepared_shell_shapes, face_shapes, face_route) =
            match load_ported_topology(self, shape)? {
                Some(loaded) => (
                    loaded.topology,
                    loaded.edge_shapes,
                    loaded.solid_shapes,
                    loaded.prepared_shell_shapes,
                    loaded.face_shapes,
                    FaceSurfaceRoute::Public,
                ),
                None => {
                    if !strict_brep_raw_topology_fallback_allowed(self, shape)? {
                        return Err(strict_brep_missing_ported_topology_error(self, shape)?);
                    }

                    let prepared_shell_shapes = self
                        .subshapes_occt(shape, ShapeKind::Shell)?
                        .into_iter()
                        .map(|shell_shape| {
                            Ok(PreparedShellShape {
                                shell_vertex_shapes: self
                                    .subshapes_occt(&shell_shape, ShapeKind::Vertex)?,
                                shell_edge_shapes: self
                                    .subshapes_occt(&shell_shape, ShapeKind::Edge)?,
                                shell_face_shapes: self
                                    .subshapes_occt(&shell_shape, ShapeKind::Face)?,
                                shell_shape,
                            })
                        })
                        .collect::<Result<Vec<_>, Error>>()?;
                    (
                        self.topology_occt(shape)?,
                        self.subshapes_occt(shape, ShapeKind::Edge)?,
                        self.subshapes_occt(shape, ShapeKind::Solid)?,
                        prepared_shell_shapes,
                        self.subshapes_occt(shape, ShapeKind::Face)?,
                        FaceSurfaceRoute::Raw,
                    )
                }
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
        let (summary, summary_bbox_source, summary_volume_source, offset_face_bbox_source) =
            ported_shape_summary(
                self,
                shape,
                &vertices,
                &topology,
                &wires,
                &edges,
                &faces,
                &prepared_shell_shapes,
                &solid_shapes,
                &face_shapes,
                &edge_shapes,
            )?;
        let offset_shell_bbox_sources = if summary.solid_count > 0 || summary.compsolid_count > 0 {
            ported_offset_shell_bbox_sources(self, &faces, &prepared_shell_shapes)
        } else {
            Vec::new()
        };

        Ok(BrepShape {
            summary,
            topology,
            vertices,
            wires,
            edges,
            faces,
            summary_bbox_source,
            summary_volume_source,
            offset_shell_bbox_sources,
            offset_face_bbox_source,
        })
    }

    pub fn ported_vertex_point(&self, shape: &Shape) -> Result<Option<[f64; 3]>, Error> {
        ported_vertex_point(self, shape)
    }

    pub fn ported_edge_endpoints(&self, shape: &Shape) -> Result<Option<EdgeEndpoints>, Error> {
        ported_edge_endpoints(self, shape)
    }

    pub(crate) fn ported_subshape_count(
        &self,
        shape: &Shape,
        kind: ShapeKind,
    ) -> Result<Option<usize>, Error> {
        ported_subshape_count(self, shape, kind)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        BoxParams, EllipseEdgeParams, HelixParams, OffsetParams, PrismParams, SurfaceKind,
    };
    use std::sync::Mutex;

    static STRICT_BREP_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn strict_brep_requires_ported_topology_for_supported_face_free_compounds() -> Result<(), Error>
    {
        let _guard = STRICT_BREP_TEST_LOCK.lock().unwrap();
        let context = Context::new()?;

        let lhs_wire = context.make_helix(HelixParams {
            origin: [-15.0, 0.0, 0.0],
            axis: [0.0, 0.0, 1.0],
            x_direction: [1.0, 0.0, 0.0],
            radius: 6.0,
            height: 18.0,
            pitch: 6.0,
        })?;
        let rhs_wire = context.make_helix(HelixParams {
            origin: [15.0, 0.0, 0.0],
            axis: [0.0, 0.0, 1.0],
            x_direction: [1.0, 0.0, 0.0],
            radius: 5.0,
            height: 20.0,
            pitch: 5.0,
        })?;
        let child_wire_compound = context.make_compound(&[lhs_wire, rhs_wire])?;
        let root_wire_compound = context.make_compound(&[child_wire_compound])?;
        assert!(strict_brep_requires_ported_topology(
            &context,
            &root_wire_compound
        )?);

        let lhs_edge = context.make_ellipse_edge(EllipseEdgeParams {
            origin: [-18.0, 0.0, 0.0],
            axis: [0.0, 1.0, 0.0],
            x_direction: [1.0, 0.0, 0.0],
            major_radius: 5.0,
            minor_radius: 3.0,
        })?;
        let rhs_edge = context.make_ellipse_edge(EllipseEdgeParams {
            origin: [18.0, 0.0, 0.0],
            axis: [0.0, 1.0, 0.0],
            x_direction: [1.0, 0.0, 0.0],
            major_radius: 6.0,
            minor_radius: 2.5,
        })?;
        let lhs_vertex = context.subshape(&lhs_edge, ShapeKind::Vertex, 0)?;
        let rhs_vertex = context.subshape(&rhs_edge, ShapeKind::Vertex, 0)?;
        let edge_compound = context.make_compound(&[lhs_edge, rhs_edge])?;
        let vertex_compound = context.make_compound(&[lhs_vertex, rhs_vertex])?;
        assert!(strict_brep_requires_ported_topology(
            &context,
            &edge_compound
        )?);
        assert!(strict_brep_requires_ported_topology(
            &context,
            &vertex_compound
        )?);

        let mixed_wire = context.make_helix(HelixParams {
            origin: [0.0, -15.0, 0.0],
            axis: [0.0, 0.0, 1.0],
            x_direction: [1.0, 0.0, 0.0],
            radius: 4.0,
            height: 12.0,
            pitch: 4.0,
        })?;
        let mixed_edge = context.make_ellipse_edge(EllipseEdgeParams {
            origin: [0.0, 15.0, 0.0],
            axis: [0.0, 1.0, 0.0],
            x_direction: [1.0, 0.0, 0.0],
            major_radius: 4.0,
            minor_radius: 2.0,
        })?;
        let mixed_vertex = context.subshape(&mixed_edge, ShapeKind::Vertex, 0)?;
        let mixed_compound = context.make_compound(&[mixed_wire, mixed_edge, mixed_vertex])?;
        assert!(strict_brep_requires_ported_topology(
            &context,
            &mixed_compound
        )?);

        Ok(())
    }

    #[test]
    fn strict_brep_face_inventory_gate_uses_rust_face_support() -> Result<(), Error> {
        let _guard = STRICT_BREP_TEST_LOCK.lock().unwrap();
        let context = Context::new()?;

        let box_shape = context.make_box(BoxParams {
            origin: [-10.0, -10.0, -10.0],
            size: [20.0, 20.0, 20.0],
        })?;
        let box_summary = context.describe_shape_occt(&box_shape)?;
        assert!(strict_brep_face_inventory_requires_ported_topology(
            &context,
            &box_shape,
            box_summary.face_count,
        )?);
        assert!(!strict_brep_face_inventory_requires_ported_topology(
            &context,
            &box_shape,
            box_summary.face_count + 1,
        )?);

        let ellipse = context.make_ellipse_edge(EllipseEdgeParams {
            origin: [30.0, 4.0, -2.0],
            axis: [0.0, 1.0, 0.0],
            x_direction: [1.0, 0.0, 0.0],
            major_radius: 10.0,
            minor_radius: 6.0,
        })?;
        let prism = context.make_prism(
            &ellipse,
            PrismParams {
                direction: [0.0, 24.0, 0.0],
            },
        )?;
        let prism_summary = context.describe_shape_occt(&prism)?;
        assert!(strict_brep_face_inventory_requires_ported_topology(
            &context,
            &prism,
            prism_summary.face_count,
        )?);

        let offset_basis = context
            .subshapes_occt(&box_shape, ShapeKind::Face)?
            .into_iter()
            .find(|face| {
                matches!(
                    context.face_geometry(face).map(|geometry| geometry.kind),
                    Ok(SurfaceKind::Plane)
                )
            })
            .ok_or_else(|| Error::new("expected box to expose a planar basis face"))?;
        let offset_surface = context.make_offset_surface_face(
            &offset_basis,
            OffsetParams {
                offset: 1.25,
                tolerance: 1.0e-4,
            },
        )?;
        let offset_summary = context.describe_shape_occt(&offset_surface)?;
        assert!(strict_brep_face_inventory_requires_ported_topology(
            &context,
            &offset_surface,
            offset_summary.face_count,
        )?);

        Ok(())
    }
}
