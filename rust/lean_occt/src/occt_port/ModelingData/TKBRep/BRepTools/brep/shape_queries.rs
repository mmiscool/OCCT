use super::summary::{classify_root_kind, shape_counts};
use super::topology_access::optional_vertex_position;
use super::*;

pub(super) fn ported_vertex_point(
    context: &Context,
    shape: &Shape,
) -> Result<Option<[f64; 3]>, Error> {
    let topology = context.topology(shape)?;
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
    let topology = context.topology(shape)?;
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
