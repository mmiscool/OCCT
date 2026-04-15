use super::face_metrics::ported_face_area_from_surface;
use super::summary::LazyMeshFaceFallback;
use super::swept_face::ported_swept_face_surface_with_route;
use super::topology::{
    face_adjacent_face_indices, face_loops, single_face_topology_with_route, FaceSurfaceRoute,
};
use super::*;

struct PreparedFaceSurface {
    geometry: FaceGeometry,
    ported_surface: Option<PortedSurface>,
    ported_face_surface: Option<PortedFaceSurface>,
}

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
            ported_brep_face(
                context,
                topology,
                wires,
                edges,
                edge_shapes,
                index,
                face_shape,
            )
        })
        .collect::<Result<Vec<_>, Error>>()?;

    Ok((face_shapes, faces))
}

fn ported_brep_face(
    context: &Context,
    topology: &TopologySnapshot,
    wires: &[BrepWire],
    edges: &[BrepEdge],
    edge_shapes: &[Shape],
    index: usize,
    face_shape: &Shape,
) -> Result<BrepFace, Error> {
    let prepared = prepare_face_surface_with_route(context, face_shape, FaceSurfaceRoute::Raw)?;
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

fn prepare_face_surface_with_route(
    context: &Context,
    face_shape: &Shape,
    route: FaceSurfaceRoute,
) -> Result<PreparedFaceSurface, Error> {
    let face_geometry = face_geometry_with_route(context, face_shape, route)?;
    prepare_face_surface_with_geometry(context, face_shape, face_geometry, route)
}

fn prepare_face_surface_with_geometry(
    context: &Context,
    face_shape: &Shape,
    face_geometry: FaceGeometry,
    route: FaceSurfaceRoute,
) -> Result<PreparedFaceSurface, Error> {
    let ported_surface =
        PortedSurface::from_context_with_geometry(context, face_shape, face_geometry)?;
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
