use super::edge_topology::{topology_edge_query, RootEdgeTopology};
use super::*;

pub(super) struct PreparedRootWireShape {
    pub(super) wire_shape: Shape,
    pub(super) wire_edge_occurrence_shapes: Vec<Shape>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RootWireTopology {
    pub(super) edge_indices: Vec<usize>,
    pub(super) edge_orientations: Vec<Orientation>,
    pub(super) vertex_indices: Vec<usize>,
}

#[derive(Clone, Copy, Debug)]
struct WireOccurrence {
    edge_index: usize,
    orientation: Orientation,
    start_vertex: usize,
    end_vertex: usize,
}

pub(super) fn root_wire_topology(
    context: &Context,
    prepared_wire_shape: &PreparedRootWireShape,
    vertex_positions: &[[f64; 3]],
    root_edge_shapes: &[Shape],
    root_edges: &[RootEdgeTopology],
) -> Result<Option<RootWireTopology>, Error> {
    root_wire_topology_from_occurrences(
        context,
        prepared_wire_shape,
        vertex_positions,
        root_edge_shapes,
        root_edges,
    )
}

fn root_wire_topology_from_occurrences(
    context: &Context,
    prepared_wire_shape: &PreparedRootWireShape,
    vertex_positions: &[[f64; 3]],
    root_edge_shapes: &[Shape],
    root_edges: &[RootEdgeTopology],
) -> Result<Option<RootWireTopology>, Error> {
    if prepared_wire_shape.wire_edge_occurrence_shapes.len()
        != prepared_wire_shape.wire_shape.edge_count()
    {
        return Ok(None);
    }

    let occurrences = match ported_wire_occurrences(
        context,
        &prepared_wire_shape.wire_edge_occurrence_shapes,
        vertex_positions,
        root_edge_shapes,
        root_edges,
    )? {
        Some(occurrences) => occurrences,
        None => return Ok(None),
    };
    let (edge_indices, edge_orientations, vertex_indices) =
        match order_wire_occurrences(&occurrences) {
            Some(ordered) => ordered,
            None => return Ok(None),
        };
    Ok(Some(RootWireTopology {
        edge_indices,
        edge_orientations,
        vertex_indices,
    }))
}

pub(super) fn pack_wire_topology(
    root_wires: &[RootWireTopology],
) -> (
    Vec<crate::TopologyRange>,
    Vec<usize>,
    Vec<Orientation>,
    Vec<crate::TopologyRange>,
    Vec<usize>,
) {
    let mut wires = Vec::with_capacity(root_wires.len());
    let mut wire_edge_indices = Vec::new();
    let mut wire_edge_orientations = Vec::new();
    let mut wire_vertices = Vec::with_capacity(root_wires.len());
    let mut wire_vertex_indices = Vec::new();

    for wire in root_wires {
        wires.push(crate::TopologyRange {
            offset: wire_edge_indices.len(),
            count: wire.edge_indices.len(),
        });
        wire_edge_indices.extend(&wire.edge_indices);
        wire_edge_orientations.extend(&wire.edge_orientations);
        wire_vertices.push(crate::TopologyRange {
            offset: wire_vertex_indices.len(),
            count: wire.vertex_indices.len(),
        });
        wire_vertex_indices.extend(&wire.vertex_indices);
    }

    (
        wires,
        wire_edge_indices,
        wire_edge_orientations,
        wire_vertices,
        wire_vertex_indices,
    )
}

fn ported_wire_occurrences(
    context: &Context,
    wire_edge_occurrence_shapes: &[Shape],
    vertex_positions: &[[f64; 3]],
    root_edge_shapes: &[Shape],
    root_edges: &[RootEdgeTopology],
) -> Result<Option<Vec<WireOccurrence>>, Error> {
    let mut occurrences = Vec::new();
    for edge_shape in wire_edge_occurrence_shapes {
        let Some(occurrence) = wire_occurrence(
            context,
            edge_shape,
            vertex_positions,
            root_edge_shapes,
            root_edges,
        )?
        else {
            return Ok(None);
        };
        occurrences.push(occurrence);
    }
    Ok(Some(occurrences))
}

fn wire_occurrence(
    context: &Context,
    edge_shape: &Shape,
    vertex_positions: &[[f64; 3]],
    root_edge_shapes: &[Shape],
    root_edges: &[RootEdgeTopology],
) -> Result<Option<WireOccurrence>, Error> {
    let query = topology_edge_query(context, edge_shape)?;
    let geometry = query.geometry;
    let Some(mut start_vertex) = match_vertex_index(vertex_positions, query.endpoints.start) else {
        return Ok(None);
    };
    let Some(mut end_vertex) = match_vertex_index(vertex_positions, query.endpoints.end) else {
        return Ok(None);
    };
    let orientation = context.shape_orientation(edge_shape)?;
    if matches!(orientation, Orientation::Reversed) {
        std::mem::swap(&mut start_vertex, &mut end_vertex);
    }
    let length = edge_length(edge_shape);
    let Some(edge_index) = matched_root_edge_index(
        context,
        edge_shape,
        root_edge_shapes,
        root_edges,
        geometry,
        length,
        start_vertex,
        end_vertex,
    )?
    else {
        return Ok(None);
    };

    Ok(Some(WireOccurrence {
        edge_index,
        orientation,
        start_vertex,
        end_vertex,
    }))
}

fn matched_root_edge_index(
    context: &Context,
    edge_shape: &Shape,
    root_edge_shapes: &[Shape],
    root_edges: &[RootEdgeTopology],
    geometry: EdgeGeometry,
    length: f64,
    start_vertex: usize,
    end_vertex: usize,
) -> Result<Option<usize>, Error> {
    let mut identity_matches = Vec::new();
    for (index, root_edge_shape) in root_edge_shapes.iter().enumerate() {
        let Some(root_edge) = root_edges.get(index) else {
            return Ok(None);
        };
        if !root_edge_matches_occurrence(root_edge, geometry, length, start_vertex, end_vertex) {
            continue;
        }
        if context.shape_is_same_occt(edge_shape, root_edge_shape)? {
            identity_matches.push(index);
        }
    }
    match identity_matches.as_slice() {
        [index] => return Ok(Some(*index)),
        [] => {}
        _ => return Ok(None),
    }

    let matches = root_edges
        .iter()
        .enumerate()
        .filter(|(_, root_edge)| {
            root_edge_matches_occurrence(root_edge, geometry, length, start_vertex, end_vertex)
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    Ok(match matches.as_slice() {
        [index] => Some(*index),
        _ => None,
    })
}

fn root_edge_matches_occurrence(
    root_edge: &RootEdgeTopology,
    geometry: EdgeGeometry,
    length: f64,
    start_vertex: usize,
    end_vertex: usize,
) -> bool {
    root_edge.geometry.kind == geometry.kind
        && approx_eq(root_edge.length, length, 1.0e-6, 1.0e-6)
        && matches_edge_vertices(root_edge, start_vertex, end_vertex)
}

fn order_wire_occurrences(
    occurrences: &[WireOccurrence],
) -> Option<(Vec<usize>, Vec<Orientation>, Vec<usize>)> {
    if occurrences.is_empty() {
        return Some((Vec::new(), Vec::new(), Vec::new()));
    }
    if let Some(vertices) =
        chain_wire_occurrences(occurrences, &(0..occurrences.len()).collect::<Vec<_>>())
    {
        return Some((
            occurrences
                .iter()
                .map(|occurrence| occurrence.edge_index)
                .collect(),
            occurrences
                .iter()
                .map(|occurrence| occurrence.orientation)
                .collect(),
            vertices,
        ));
    }

    let mut outgoing = BTreeMap::<usize, Vec<usize>>::new();
    let mut in_degree = BTreeMap::<usize, usize>::new();
    let mut out_degree = BTreeMap::<usize, usize>::new();
    for (index, occurrence) in occurrences.iter().enumerate() {
        outgoing
            .entry(occurrence.start_vertex)
            .or_default()
            .push(index);
        *out_degree.entry(occurrence.start_vertex).or_default() += 1;
        *in_degree.entry(occurrence.end_vertex).or_default() += 1;
    }

    let start_candidates = outgoing
        .keys()
        .copied()
        .filter(|vertex| {
            let outgoing = out_degree.get(vertex).copied().unwrap_or(0);
            let incoming = in_degree.get(vertex).copied().unwrap_or(0);
            outgoing == incoming + 1
        })
        .collect::<Vec<_>>();
    let start_vertex = match start_candidates.as_slice() {
        [start] => *start,
        [] => occurrences.first()?.start_vertex,
        _ => return None,
    };

    let mut used = vec![false; occurrences.len()];
    let mut ordered = Vec::with_capacity(occurrences.len());
    let mut current_vertex = start_vertex;
    while ordered.len() < occurrences.len() {
        let next = outgoing
            .get(&current_vertex)?
            .iter()
            .copied()
            .filter(|index| !used[*index])
            .collect::<Vec<_>>();
        if next.len() != 1 {
            return None;
        }
        let index = next[0];
        used[index] = true;
        ordered.push(index);
        current_vertex = occurrences[index].end_vertex;
    }

    let ordered_vertices = chain_wire_occurrences(occurrences, &ordered)?;
    Some((
        ordered
            .iter()
            .map(|&index| occurrences[index].edge_index)
            .collect(),
        ordered
            .iter()
            .map(|&index| occurrences[index].orientation)
            .collect(),
        ordered_vertices,
    ))
}

fn chain_wire_occurrences(occurrences: &[WireOccurrence], ordered: &[usize]) -> Option<Vec<usize>> {
    let &first = ordered.first()?;
    let mut vertices = vec![occurrences[first].start_vertex];
    let mut current_vertex = occurrences[first].end_vertex;
    vertices.push(current_vertex);
    for &index in ordered.iter().skip(1) {
        let occurrence = occurrences.get(index)?;
        if occurrence.start_vertex != current_vertex {
            return None;
        }
        current_vertex = occurrence.end_vertex;
        vertices.push(current_vertex);
    }
    Some(vertices)
}

fn matches_edge_vertices(
    root_edge: &RootEdgeTopology,
    start_vertex: usize,
    end_vertex: usize,
) -> bool {
    matches!(
        (root_edge.start_vertex, root_edge.end_vertex),
        (Some(root_start), Some(root_end))
            if (root_start == start_vertex && root_end == end_vertex)
                || (root_start == end_vertex && root_end == start_vertex)
    )
}

pub(super) fn edge_length(edge_shape: &Shape) -> f64 {
    edge_shape.linear_length()
}

pub(super) fn match_vertex_index(vertex_positions: &[[f64; 3]], point: [f64; 3]) -> Option<usize> {
    let mut found = None;
    for (index, vertex_position) in vertex_positions.iter().copied().enumerate() {
        if approx_points_eq(vertex_position, point, 1.0e-7) {
            if found.is_some() {
                return None;
            }
            found = Some(index);
        }
    }
    found
}

fn approx_points_eq(lhs: [f64; 3], rhs: [f64; 3], tolerance: f64) -> bool {
    (lhs[0] - rhs[0]).abs() <= tolerance
        && (lhs[1] - rhs[1]).abs() <= tolerance
        && (lhs[2] - rhs[2]).abs() <= tolerance
}
