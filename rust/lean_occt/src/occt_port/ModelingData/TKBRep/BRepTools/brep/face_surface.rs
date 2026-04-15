use super::face_metrics::{
    analytic_face_area, analytic_offset_face_area, analytic_ported_swept_face_area,
};
use super::summary::mesh_face_properties;
use super::swept_face::{
    face_curve_candidates, select_swept_face_basis_curve, SweptBasisSelection,
};
use super::topology::{face_adjacent_face_indices, face_loops, ported_brep_wires};
use super::*;

struct SingleFaceTopology {
    loops: Vec<BrepFaceLoop>,
    wires: Vec<BrepWire>,
    edges: Vec<BrepEdge>,
    edge_shapes: Vec<Shape>,
}

type SingleFaceTopologyBuilder = fn(&Context, &Shape) -> Result<Option<SingleFaceTopology>, Error>;

pub(crate) fn ported_face_surface_descriptor(
    context: &Context,
    face_shape: &Shape,
    face_geometry: FaceGeometry,
) -> Result<Option<PortedFaceSurface>, Error> {
    let ported_surface =
        PortedSurface::from_context_with_geometry(context, face_shape, face_geometry)?;
    ported_face_surface_descriptor_from_surface_public(
        context,
        face_shape,
        face_geometry,
        ported_surface,
    )
}

pub(super) fn ported_brep_faces(
    context: &Context,
    shape: &Shape,
    topology: &TopologySnapshot,
    wires: &[BrepWire],
    edges: &[BrepEdge],
    edge_shapes: &[Shape],
) -> Result<(Vec<Shape>, Vec<BrepFace>), Error> {
    let face_shapes = context.subshapes_occt(shape, ShapeKind::Face)?;
    let faces = face_shapes
        .iter()
        .enumerate()
        .map(|(index, face_shape)| {
            let geometry = context.face_geometry_occt(face_shape)?;
            let ported_surface =
                PortedSurface::from_context_with_geometry(context, face_shape, geometry)?;
            let ported_face_surface = ported_face_surface_descriptor_from_surface(
                context,
                face_shape,
                geometry,
                ported_surface,
            )?;
            let orientation = context.shape_orientation(face_shape)?;
            let loops = face_loops(topology, index)?;
            let mut mesh_fallback = if ported_face_surface.is_none() {
                mesh_face_properties(context, face_shape, orientation)
            } else {
                None
            };
            let mut mesh_fallback_loaded = ported_face_surface.is_none();
            let mut load_mesh_fallback = || {
                if !mesh_fallback_loaded {
                    mesh_fallback = mesh_face_properties(context, face_shape, orientation);
                    mesh_fallback_loaded = true;
                }
                mesh_fallback
            };
            let sample = ported_face_surface
                .map(|surface| {
                    surface.sample_normalized_with_orientation(geometry, [0.5, 0.5], orientation)
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
                    context,
                    surface,
                    geometry,
                    &loops,
                    wires,
                    edges,
                    edge_shapes,
                )
                .or_else(|| load_mesh_fallback().map(|fallback| fallback.area))
                .ok_or_else(|| {
                    Error::new(format!(
                        "failed to derive a Rust-owned area for face {index} ({:?})",
                        geometry.kind
                    ))
                })?,
                Some(PortedFaceSurface::Offset(surface)) => analytic_offset_face_area(
                    context,
                    surface,
                    geometry,
                    &loops,
                    wires,
                    edges,
                    edge_shapes,
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
            let adjacent_face_indices = face_adjacent_face_indices(topology, wires, index)?;

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

    Ok((face_shapes, faces))
}

pub(super) fn ported_face_surface_descriptor_from_surface(
    context: &Context,
    face_shape: &Shape,
    face_geometry: FaceGeometry,
    ported_surface: Option<PortedSurface>,
) -> Result<Option<PortedFaceSurface>, Error> {
    ported_face_surface_descriptor_from_surface_with_topology(
        context,
        face_shape,
        face_geometry,
        ported_surface,
        single_face_topology,
    )
}

fn ported_face_surface_descriptor_from_surface_public(
    context: &Context,
    face_shape: &Shape,
    face_geometry: FaceGeometry,
    ported_surface: Option<PortedSurface>,
) -> Result<Option<PortedFaceSurface>, Error> {
    ported_face_surface_descriptor_from_surface_with_topology(
        context,
        face_shape,
        face_geometry,
        ported_surface,
        single_face_topology_public,
    )
}

fn ported_face_surface_descriptor_from_surface_with_topology(
    context: &Context,
    face_shape: &Shape,
    face_geometry: FaceGeometry,
    ported_surface: Option<PortedSurface>,
    topology_builder: SingleFaceTopologyBuilder,
) -> Result<Option<PortedFaceSurface>, Error> {
    if let Some(surface) = ported_surface {
        return Ok(Some(PortedFaceSurface::Analytic(surface)));
    }

    if let Some(surface) = context.ported_offset_surface(face_shape)? {
        return Ok(Some(PortedFaceSurface::Offset(surface)));
    }

    Ok(ported_swept_face_surface_with_topology(
        context,
        face_shape,
        face_geometry,
        topology_builder,
    )?
    .map(PortedFaceSurface::Swept))
}

fn ported_swept_face_surface_with_topology(
    context: &Context,
    face_shape: &Shape,
    face_geometry: FaceGeometry,
    topology_builder: SingleFaceTopologyBuilder,
) -> Result<Option<PortedSweptSurface>, Error> {
    let topology = match topology_builder(context, face_shape)? {
        Some(topology) => topology,
        None => return Ok(None),
    };

    ported_swept_face_surface_from_topology(context, face_shape, face_geometry, topology)
}

fn ported_swept_face_surface_from_topology(
    context: &Context,
    face_shape: &Shape,
    face_geometry: FaceGeometry,
    topology: SingleFaceTopology,
) -> Result<Option<PortedSweptSurface>, Error> {
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
    let topology = match single_face_topology_public(context, face_shape)? {
        Some(topology) => topology,
        None => return Ok(None),
    };

    let ported_surface =
        PortedSurface::from_context_with_geometry(context, face_shape, face_geometry)?;
    let ported_face_surface = ported_face_surface_descriptor_from_surface_public(
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
    single_face_topology_with_edges(context, face_shape, single_face_edge_raw)
}

fn single_face_topology_public(
    context: &Context,
    face_shape: &Shape,
) -> Result<Option<SingleFaceTopology>, Error> {
    single_face_topology_with_edges(context, face_shape, single_face_edge_public)
}

fn single_face_topology_with_edges(
    context: &Context,
    face_shape: &Shape,
    edge_builder: fn(&Context, usize, &Shape) -> Result<BrepEdge, Error>,
) -> Result<Option<SingleFaceTopology>, Error> {
    let topology = match single_face_topology_snapshot(context, face_shape)? {
        Some(topology) => topology,
        None => return Ok(None),
    };

    let wires = ported_brep_wires(&topology);
    let edge_shapes = context.subshapes_occt(face_shape, ShapeKind::Edge)?;
    let edges = edge_shapes
        .iter()
        .enumerate()
        .map(|(index, edge_shape)| edge_builder(context, index, edge_shape))
        .collect::<Result<Vec<_>, Error>>()?;

    Ok(Some(SingleFaceTopology {
        loops: face_loops(&topology, 0)?,
        wires,
        edges,
        edge_shapes,
    }))
}

fn single_face_topology_snapshot(
    context: &Context,
    face_shape: &Shape,
) -> Result<Option<TopologySnapshot>, Error> {
    let topology = match context.ported_topology(face_shape)? {
        Some(topology) => topology,
        None => context.topology_occt(face_shape)?,
    };
    if topology.faces.len() != 1 {
        return Ok(None);
    }
    Ok(Some(topology))
}

fn single_face_edge_raw(
    context: &Context,
    index: usize,
    edge_shape: &Shape,
) -> Result<BrepEdge, Error> {
    let geometry = context.edge_geometry_occt(edge_shape)?;
    let ported_curve = PortedCurve::from_context_with_geometry(context, edge_shape, geometry)?;
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
}

fn single_face_edge_public(
    context: &Context,
    index: usize,
    edge_shape: &Shape,
) -> Result<BrepEdge, Error> {
    let geometry = match context.edge_geometry(edge_shape) {
        Ok(geometry) => geometry,
        Err(_) => context.edge_geometry_occt(edge_shape)?,
    };
    let ported_curve =
        match PortedCurve::from_context_with_ported_payloads(context, edge_shape, geometry) {
            Ok(ported_curve) => ported_curve,
            Err(_) => PortedCurve::from_context_with_geometry(context, edge_shape, geometry)?,
        };
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
}
