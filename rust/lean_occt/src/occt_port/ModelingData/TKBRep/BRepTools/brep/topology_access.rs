use super::*;

pub(super) fn topology_edge(
    topology: &TopologySnapshot,
    index: usize,
) -> Result<crate::TopologyEdge, Error> {
    topology
        .edges
        .get(index)
        .copied()
        .ok_or_else(|| Error::new(format!("topology is missing edge index {index}")))
}

pub(super) fn adjacent_face_indices(
    topology: &TopologySnapshot,
    edge_index: usize,
) -> Result<Vec<usize>, Error> {
    let range = topology
        .edge_faces
        .get(edge_index)
        .copied()
        .ok_or_else(|| Error::new(format!("topology is missing edge-face range {edge_index}")))?;
    Ok(topology.edge_face_indices[range.offset..range.offset + range.count].to_vec())
}

pub(super) fn edge_points(
    topology: &TopologySnapshot,
    edge_index: usize,
) -> (Option<[f64; 3]>, Option<[f64; 3]>) {
    let Some(edge) = topology.edges.get(edge_index) else {
        return (None, None);
    };
    (
        optional_vertex_position(topology, edge.start_vertex),
        optional_vertex_position(topology, edge.end_vertex),
    )
}

pub(super) fn optional_vertex_position(
    topology: &TopologySnapshot,
    vertex_index: Option<usize>,
) -> Option<[f64; 3]> {
    vertex_index.and_then(|index| topology.vertex_positions.get(index).copied())
}
