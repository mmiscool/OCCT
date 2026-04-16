use super::face_metrics::ported_face_area_from_surface;
use super::face_prepare::prepare_face_surface_with_route;
use super::face_topology::{face_adjacent_face_indices, face_loops, FaceSurfaceRoute};
use super::summary::LazyMeshFaceFallback;
use super::*;

pub(super) fn ported_brep_faces(
    context: &Context,
    face_shapes: &[Shape],
    topology: &TopologySnapshot,
    wires: &[BrepWire],
    edges: &[BrepEdge],
    edge_shapes: &[Shape],
    route: FaceSurfaceRoute,
) -> Result<Vec<BrepFace>, Error> {
    let faces = face_shapes
        .iter()
        .enumerate()
        .map(|(index, face_shape)| {
            ported_brep_face(
                context,
                topology,
                wires,
                edges,
                edge_shapes,
                index,
                face_shape,
                route,
            )
        })
        .collect::<Result<Vec<_>, Error>>()?;

    Ok(faces)
}

/// Build a face inventory for summary derivation from already-computed public faces.
///
/// Analytic, offset, and swept faces can all be reused from the public route for summary
/// derivation. Closed-topology gating in `analytic_shape_volume` preserves the old mesh/OCCT
/// fallback on open or non-manifold solids, and analytic face volume now derives plane
/// contributions from the loop/geometry path instead of Raw-only face area/sample state.
/// Unknown faces are still re-prepared on the `Raw` route.
pub(super) fn ported_brep_summary_faces(
    context: &Context,
    face_shapes: &[Shape],
    topology: &TopologySnapshot,
    wires: &[BrepWire],
    edges: &[BrepEdge],
    edge_shapes: &[Shape],
    public_faces: &[BrepFace],
) -> Result<Vec<BrepFace>, Error> {
    public_faces
        .iter()
        .enumerate()
        .map(|(index, public_face)| {
            let needs_raw = match public_face.ported_face_surface {
                Some(PortedFaceSurface::Analytic(_)) => false,
                Some(PortedFaceSurface::Offset(_)) => {
                    // Offset surface preparation is route-independent: for non-analytic
                    // faces, `ported_face_geometry` returns None and face_geometry falls
                    // back to face_geometry_occt (the same source as the Raw route).
                    // PortedOffsetSurface is also extracted independently of the route.
                    // The public face is therefore identical to what Raw preparation
                    // would produce, so it can be reused directly.
                    false
                }
                Some(PortedFaceSurface::Swept(_)) => {
                    // Swept faces can now reuse the public route because analytic volume
                    // declines non-closed topology before applying divergence-theorem
                    // accumulation, preserving the old mesh/OCCT fallback on open solids.
                    false
                }
                _ => true,
            };
            if needs_raw {
                let face_shape = face_shapes.get(index).ok_or_else(|| {
                    Error::new(format!(
                        "summary face index {index} is outside the face-shape inventory"
                    ))
                })?;
                ported_brep_face(
                    context,
                    topology,
                    wires,
                    edges,
                    edge_shapes,
                    index,
                    face_shape,
                    FaceSurfaceRoute::Raw,
                )
            } else {
                Ok(public_face.clone())
            }
        })
        .collect()
}

fn ported_brep_face(
    context: &Context,
    topology: &TopologySnapshot,
    wires: &[BrepWire],
    edges: &[BrepEdge],
    edge_shapes: &[Shape],
    index: usize,
    face_shape: &Shape,
    route: FaceSurfaceRoute,
) -> Result<BrepFace, Error> {
    let prepared = prepare_face_surface_with_route(context, face_shape, route)?;
    let geometry = prepared.geometry;
    let ported_surface = prepared.ported_surface;
    let ported_face_surface = prepared.ported_face_surface;
    let orientation = context.shape_orientation(face_shape)?;
    let loops = face_loops(topology, index)?;
    let mut mesh_fallback = LazyMeshFaceFallback::new(
        context,
        face_shape,
        orientation,
        ported_face_surface.is_none(),
    );
    let sample = mesh_fallback.resolve_sample(
        ported_face_surface.map(|surface| {
            surface.sample_normalized_with_orientation(geometry, [0.5, 0.5], orientation)
        }),
        index,
        geometry,
    )?;
    let area = ported_face_area_from_surface(
        context,
        ported_face_surface,
        geometry,
        &loops,
        wires,
        edges,
        edge_shapes,
    );
    let area = mesh_fallback.resolve_area(area, index, geometry)?;
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
}
