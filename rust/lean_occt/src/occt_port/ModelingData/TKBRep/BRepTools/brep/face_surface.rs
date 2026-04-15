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
