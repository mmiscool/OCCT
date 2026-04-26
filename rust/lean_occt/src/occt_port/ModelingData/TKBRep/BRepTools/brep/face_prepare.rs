use super::face_topology::FaceSurfaceRoute;
use super::swept_face::ported_swept_face_surface_with_route;
use super::*;

pub(super) struct PreparedFaceSurface {
    pub(super) geometry: FaceGeometry,
    pub(super) ported_surface: Option<PortedSurface>,
    pub(super) ported_face_surface: Option<PortedFaceSurface>,
}

pub(super) fn prepare_face_surface_with_route(
    context: &Context,
    face_shape: &Shape,
    route: FaceSurfaceRoute,
) -> Result<PreparedFaceSurface, Error> {
    let face_geometry = face_geometry_with_route(context, face_shape, route)?;
    prepare_face_surface_with_geometry(context, face_shape, face_geometry, route)
}

pub(super) fn prepare_face_surface_with_geometry(
    context: &Context,
    face_shape: &Shape,
    face_geometry: FaceGeometry,
    route: FaceSurfaceRoute,
) -> Result<PreparedFaceSurface, Error> {
    let ported_surface =
        PortedSurface::from_context_with_ported_payloads(context, face_shape, face_geometry)?;
    let ported_face_surface = ported_face_surface_descriptor_from_surface_with_route(
        context,
        face_shape,
        face_geometry,
        ported_surface,
        route,
    )?;

    Ok(PreparedFaceSurface {
        geometry: face_geometry,
        ported_surface,
        ported_face_surface,
    })
}

fn ported_face_surface_descriptor_from_surface_with_route(
    context: &Context,
    face_shape: &Shape,
    face_geometry: FaceGeometry,
    ported_surface: Option<PortedSurface>,
    route: FaceSurfaceRoute,
) -> Result<Option<PortedFaceSurface>, Error> {
    if let Some(surface) = ported_surface {
        return Ok(Some(PortedFaceSurface::Analytic(surface)));
    }

    if let Some(surface) = context.ported_offset_surface(face_shape)? {
        return Ok(Some(PortedFaceSurface::Offset(surface)));
    }

    Ok(
        ported_swept_face_surface_with_route(context, face_shape, face_geometry, route)?
            .map(PortedFaceSurface::Swept),
    )
}

fn face_geometry_with_route(
    context: &Context,
    face_shape: &Shape,
    route: FaceSurfaceRoute,
) -> Result<FaceGeometry, Error> {
    match route {
        FaceSurfaceRoute::Raw => context.face_geometry_occt(face_shape),
        FaceSurfaceRoute::Public => context.face_geometry(face_shape),
    }
}
