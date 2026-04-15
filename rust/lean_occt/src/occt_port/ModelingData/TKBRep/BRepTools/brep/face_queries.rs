use super::face_metrics::ported_face_area_from_surface;
use super::face_prepare::{prepare_face_surface_with_geometry, prepare_face_surface_with_route};
use super::face_topology::{single_face_topology_with_route, FaceSurfaceRoute};
use super::*;

pub(crate) fn ported_face_surface_descriptor(
    context: &Context,
    face_shape: &Shape,
    face_geometry: FaceGeometry,
) -> Result<Option<PortedFaceSurface>, Error> {
    Ok(prepare_face_surface_with_geometry(
        context,
        face_shape,
        face_geometry,
        FaceSurfaceRoute::Public,
    )?
    .ported_face_surface)
}

pub(crate) fn ported_face_area(
    context: &Context,
    face_shape: &Shape,
) -> Result<Option<f64>, Error> {
    let topology =
        match single_face_topology_with_route(context, face_shape, FaceSurfaceRoute::Public)? {
            Some(topology) => topology,
            None => return Ok(None),
        };

    let prepared = prepare_face_surface_with_route(context, face_shape, FaceSurfaceRoute::Public)?;
    let face_geometry = prepared.geometry;
    Ok(ported_face_area_from_surface(
        context,
        prepared.ported_face_surface,
        face_geometry,
        &topology.loops,
        &topology.wires,
        &topology.edges,
        &topology.edge_shapes,
    ))
}
