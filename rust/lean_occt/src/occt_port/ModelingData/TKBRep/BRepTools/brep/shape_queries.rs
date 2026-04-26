use super::summary::{classify_root_kind, shape_counts};
use super::topology_access::optional_vertex_position;
use super::*;

fn topology_backed_subshape_count(topology: &TopologySnapshot, kind: ShapeKind) -> Option<usize> {
    match kind {
        ShapeKind::Face => Some(topology.faces.len()),
        ShapeKind::Wire => Some(topology.wires.len()),
        ShapeKind::Edge => Some(topology.edges.len()),
        ShapeKind::Vertex => Some(topology.vertex_positions.len()),
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

pub(super) fn ported_subshape(
    context: &Context,
    shape: &Shape,
    kind: ShapeKind,
    index: usize,
) -> Result<Option<Shape>, Error> {
    let Some(topology) = context.ported_topology(shape)? else {
        return Ok(None);
    };
    let Some(count) = topology_backed_subshape_count(&topology, kind) else {
        return Ok(None);
    };
    if index >= count {
        return Err(Error::new(format!(
            "subshape index {index} out of bounds for {kind:?} count {count}"
        )));
    }
    Ok(Some(context.subshape_occt(shape, kind, index)?))
}

pub(super) fn ported_subshapes(
    context: &Context,
    shape: &Shape,
    kind: ShapeKind,
) -> Result<Option<Vec<Shape>>, Error> {
    let Some(topology) = context.ported_topology(shape)? else {
        return Ok(None);
    };
    let Some(expected_count) = topology_backed_subshape_count(&topology, kind) else {
        return Ok(None);
    };
    let shapes = context.subshapes_occt(shape, kind)?;
    if shapes.len() != expected_count {
        return Err(Error::new(format!(
            "expected {expected_count} {kind:?} subshape(s) from topology, found {} via OCCT",
            shapes.len()
        )));
    }
    Ok(Some(shapes))
}
