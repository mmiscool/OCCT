use super::summary::{classify_root_kind, shape_counts};
use super::topology::{load_ported_topology, LoadedPortedTopology};
use super::topology_access::optional_vertex_position;
use super::*;

fn topology_backed_subshape_count(
    topology: &TopologySnapshot,
    loaded: Option<&LoadedPortedTopology>,
    kind: ShapeKind,
) -> Option<usize> {
    match kind {
        ShapeKind::Face => Some(topology.faces.len()),
        ShapeKind::Wire => Some(topology.wires.len()),
        ShapeKind::Edge => Some(topology.edges.len()),
        ShapeKind::Vertex => Some(topology.vertex_positions.len()),
        ShapeKind::Shell => loaded.map(|loaded| loaded.prepared_shell_shapes.len()),
        ShapeKind::Solid => loaded.map(|loaded| loaded.solid_shapes.len()),
        _ => None,
    }
}

pub(super) fn ported_vertex_point(
    context: &Context,
    shape: &Shape,
) -> Result<Option<[f64; 3]>, Error> {
    let Some(topology) = context.ported_topology(shape)? else {
        return Ok(None);
    };
    let counts = shape_counts(context, shape, &topology)?;
    if classify_root_kind(counts) != ShapeKind::Vertex {
        return Ok(None);
    }

    let [point] = topology.vertex_positions.as_slice() else {
        return Err(Error::new(format!(
            "expected exactly one vertex in vertex topology, found {}",
            topology.vertex_positions.len()
        )));
    };
    Ok(Some(*point))
}

pub(super) fn ported_edge_endpoints(
    context: &Context,
    shape: &Shape,
) -> Result<Option<EdgeEndpoints>, Error> {
    match root_edge_endpoints_from_raw_endpoint_seed(context, shape)? {
        RootEdgeEndpointSeed::Seeded(endpoints) => return Ok(Some(endpoints)),
        RootEdgeEndpointSeed::UnsupportedRootEdge => return Ok(None),
        RootEdgeEndpointSeed::NotRootEdge => {}
    }

    let Some(topology) = context.ported_topology(shape)? else {
        return Ok(None);
    };
    let counts = shape_counts(context, shape, &topology)?;
    if classify_root_kind(counts) != ShapeKind::Edge {
        return Ok(None);
    }

    let [edge] = topology.edges.as_slice() else {
        return Err(Error::new(format!(
            "expected exactly one edge in edge topology, found {}",
            topology.edges.len()
        )));
    };
    let (Some(start), Some(end)) = (
        optional_vertex_position(&topology, edge.start_vertex),
        optional_vertex_position(&topology, edge.end_vertex),
    ) else {
        return Err(Error::new("Edge did not contain two endpoint vertices."));
    };
    Ok(Some(EdgeEndpoints { start, end }))
}

enum RootEdgeEndpointSeed {
    NotRootEdge,
    UnsupportedRootEdge,
    Seeded(EdgeEndpoints),
}

fn root_edge_endpoints_from_raw_endpoint_seed(
    context: &Context,
    shape: &Shape,
) -> Result<RootEdgeEndpointSeed, Error> {
    if context.describe_shape_occt(shape)?.root_kind != ShapeKind::Edge {
        return Ok(RootEdgeEndpointSeed::NotRootEdge);
    }

    let geometry = context.edge_geometry_occt(shape)?;
    if !matches!(
        geometry.kind,
        CurveKind::Line | CurveKind::Circle | CurveKind::Ellipse
    ) {
        return Ok(RootEdgeEndpointSeed::UnsupportedRootEdge);
    }

    Ok(RootEdgeEndpointSeed::Seeded(
        context.edge_endpoints_occt(shape)?,
    ))
}

pub(super) fn ported_subshape_count(
    context: &Context,
    shape: &Shape,
    kind: ShapeKind,
) -> Result<Option<usize>, Error> {
    let Some(loaded) = load_ported_topology(context, shape)? else {
        return Ok(None);
    };
    Ok(topology_backed_subshape_count(
        &loaded.topology,
        Some(&loaded),
        kind,
    ))
}

pub(super) fn ported_subshape(
    context: &Context,
    shape: &Shape,
    kind: ShapeKind,
    index: usize,
) -> Result<Option<Shape>, Error> {
    let Some(mut shapes) = ported_subshapes(context, shape, kind)? else {
        return Ok(None);
    };
    if index >= shapes.len() {
        return Err(Error::new(format!(
            "subshape index {index} out of bounds for {kind:?} count {}",
            shapes.len()
        )));
    }
    Ok(Some(shapes.remove(index)))
}

pub(super) fn ported_subshapes(
    context: &Context,
    shape: &Shape,
    kind: ShapeKind,
) -> Result<Option<Vec<Shape>>, Error> {
    let Some(loaded) = load_ported_topology(context, shape)? else {
        return Ok(None);
    };
    let Some(expected_count) =
        topology_backed_subshape_count(&loaded.topology, Some(&loaded), kind)
    else {
        return Ok(None);
    };
    let shapes = match kind {
        ShapeKind::Face => loaded.face_shapes,
        ShapeKind::Wire => loaded.wire_shapes,
        ShapeKind::Edge => loaded.edge_shapes,
        ShapeKind::Vertex => loaded.vertex_shapes,
        ShapeKind::Solid => loaded.solid_shapes,
        ShapeKind::Shell => loaded
            .prepared_shell_shapes
            .into_iter()
            .map(|prepared_shell| prepared_shell.shell_shape)
            .collect::<Vec<_>>(),
        _ => return Ok(None),
    };
    if shapes.len() != expected_count {
        return Err(Error::new(format!(
            "expected {expected_count} {kind:?} subshape handle(s) from ported topology, found {}",
            shapes.len()
        )));
    }
    Ok(Some(shapes))
}
