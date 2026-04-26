use super::wire_topology::{edge_length, match_vertex_index};
use super::*;

#[derive(Clone, Copy, Debug)]
pub(super) struct RootEdgeTopology {
    pub(super) geometry: EdgeGeometry,
    pub(super) start_vertex: Option<usize>,
    pub(super) end_vertex: Option<usize>,
    pub(super) length: f64,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct TopologyEdgeQuery {
    pub(super) geometry: EdgeGeometry,
    pub(super) endpoints: EdgeEndpoints,
}

pub(super) fn topology_edge_query(
    context: &Context,
    edge_shape: &Shape,
) -> Result<TopologyEdgeQuery, Error> {
    Ok(TopologyEdgeQuery {
        geometry: context.edge_geometry(edge_shape)?,
        endpoints: context.edge_endpoints(edge_shape)?,
    })
}

pub(super) fn root_edge_topology(
    context: &Context,
    edge_shape: &Shape,
    vertex_positions: &[[f64; 3]],
) -> Result<RootEdgeTopology, Error> {
    let query = topology_edge_query(context, edge_shape)?;
    Ok(RootEdgeTopology {
        geometry: query.geometry,
        start_vertex: match_vertex_index(vertex_positions, query.endpoints.start),
        end_vertex: match_vertex_index(vertex_positions, query.endpoints.end),
        length: edge_length(edge_shape),
    })
}

pub(super) fn oriented_edge_geometry(
    mut geometry: EdgeGeometry,
    orientation: Orientation,
) -> EdgeGeometry {
    if matches!(orientation, Orientation::Reversed) {
        std::mem::swap(&mut geometry.start_parameter, &mut geometry.end_parameter);
    }
    geometry
}
