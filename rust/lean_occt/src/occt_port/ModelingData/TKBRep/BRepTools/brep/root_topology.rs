use super::*;

#[derive(Clone, Copy, Debug)]
pub(super) struct RootEdgeTopology {
    pub(super) geometry: EdgeGeometry,
    pub(super) start_vertex: Option<usize>,
    pub(super) end_vertex: Option<usize>,
    pub(super) length: f64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RootWireTopology {
    pub(super) edge_indices: Vec<usize>,
    pub(super) edge_orientations: Vec<Orientation>,
    pub(super) vertex_indices: Vec<usize>,
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

#[derive(Clone, Copy, Debug)]
struct WireOccurrence {
    edge_index: usize,
    orientation: Orientation,
    start_vertex: usize,
    end_vertex: usize,
}

pub(super) fn root_edge_topology(
    context: &Context,
    edge_shape: &Shape,
    vertex_positions: &[[f64; 3]],
) -> Result<RootEdgeTopology, Error> {
    let geometry = context.edge_geometry_occt(edge_shape)?;
    let endpoints = context.edge_endpoints_occt(edge_shape)?;
    Ok(RootEdgeTopology {
        geometry,
        start_vertex: match_vertex_index(vertex_positions, endpoints.start),
        end_vertex: match_vertex_index(vertex_positions, endpoints.end),
        length: edge_length(edge_shape),
    })
}

pub(super) fn root_wire_topology(
    context: &Context,
    wire_shape: &Shape,
    vertex_positions: &[[f64; 3]],
    root_edges: &[RootEdgeTopology],
) -> Result<Option<RootWireTopology>, Error> {
    if let Some(topology) =
        root_wire_topology_from_snapshot(context, wire_shape, vertex_positions, root_edges)?
    {
        return Ok(Some(topology));
    }

    let occurrences =
        match ported_wire_occurrences(context, wire_shape, vertex_positions, root_edges)? {
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

fn ported_wire_occurrences(
    context: &Context,
    wire_shape: &Shape,
    vertex_positions: &[[f64; 3]],
    root_edges: &[RootEdgeTopology],
) -> Result<Option<Vec<WireOccurrence>>, Error> {
    let mut occurrences = Vec::new();
    for edge_shape in context.subshapes_occt(wire_shape, ShapeKind::Edge)? {
        let Some(occurrence) = wire_occurrence(context, &edge_shape, vertex_positions, root_edges)?
        else {
            return Ok(None);
        };
        occurrences.push(occurrence);
    }
    Ok(Some(occurrences))
}

fn root_wire_topology_from_snapshot(
    context: &Context,
    wire_shape: &Shape,
    vertex_positions: &[[f64; 3]],
    root_edges: &[RootEdgeTopology],
) -> Result<Option<RootWireTopology>, Error> {
    let topology = context.topology_occt(wire_shape)?;
    if !topology.faces.is_empty() || topology.wires.len() != 1 {
        return Ok(None);
    }

    let wire_range = topology.wires[0];
    let vertex_range = topology.wire_vertices[0];
    if wire_range.count == 0 || vertex_range.count != wire_range.count + 1 {
        return Ok(None);
    }

    let local_edge_shapes = context.subshapes_occt(wire_shape, ShapeKind::Edge)?;
    let mut edge_indices = Vec::with_capacity(wire_range.count);
    let mut edge_orientations = Vec::with_capacity(wire_range.count);
    let mut ordered_vertices = Vec::with_capacity(vertex_range.count);

    for occurrence_offset in 0..wire_range.count {
        let wire_edge_offset = wire_range.offset + occurrence_offset;
        let local_edge_index = *topology
            .wire_edge_indices
            .get(wire_edge_offset)
            .ok_or_else(|| {
                Error::new(format!(
                    "wire topology is missing edge occurrence {wire_edge_offset}"
                ))
            })?;
        let orientation = *topology
            .wire_edge_orientations
            .get(wire_edge_offset)
            .ok_or_else(|| {
                Error::new(format!(
                    "wire topology is missing edge orientation {wire_edge_offset}"
                ))
            })?;
        let local_edge_shape = local_edge_shapes.get(local_edge_index).ok_or_else(|| {
            Error::new(format!(
                "wire topology referenced local edge index {local_edge_index} outside the edge map"
            ))
        })?;

        let local_start_index = *topology
            .wire_vertex_indices
            .get(vertex_range.offset + occurrence_offset)
            .ok_or_else(|| {
                Error::new(format!(
                    "wire topology is missing start vertex occurrence {}",
                    vertex_range.offset + occurrence_offset
                ))
            })?;
        let local_end_index = *topology
            .wire_vertex_indices
            .get(vertex_range.offset + occurrence_offset + 1)
            .ok_or_else(|| {
                Error::new(format!(
                    "wire topology is missing end vertex occurrence {}",
                    vertex_range.offset + occurrence_offset + 1
                ))
            })?;

        let start_vertex = topology_vertex_match(
            &topology.vertex_positions,
            vertex_positions,
            local_start_index,
        );
        let end_vertex = topology_vertex_match(
            &topology.vertex_positions,
            vertex_positions,
            local_end_index,
        );

        let geometry =
            oriented_edge_geometry(context.edge_geometry_occt(local_edge_shape)?, orientation);
        let length = edge_length(local_edge_shape);
        let matches = root_edges
            .iter()
            .enumerate()
            .filter_map(|(root_edge_index, root_edge)| {
                if root_edge.geometry.kind != geometry.kind
                    || !approx_eq(root_edge.length, length, 1.0e-6, 1.0e-6)
                {
                    return None;
                }
                if let (Some(start_vertex), Some(end_vertex)) = (start_vertex, end_vertex) {
                    if !matches_edge_vertices(root_edge, start_vertex, end_vertex) {
                        return None;
                    }
                }
                Some(root_edge_index)
            })
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            return Ok(None);
        }

        let matched_edge = &root_edges[matches[0]];
        let start_vertex = start_vertex.or_else(|| {
            oriented_root_edge_vertices(matched_edge, orientation)
                .map(|(start_vertex, _)| start_vertex)
        });
        let end_vertex = end_vertex.or_else(|| {
            oriented_root_edge_vertices(matched_edge, orientation).map(|(_, end_vertex)| end_vertex)
        });
        let (Some(start_vertex), Some(end_vertex)) = (start_vertex, end_vertex) else {
            return Ok(None);
        };

        edge_indices.push(matches[0]);
        edge_orientations.push(orientation);
        if ordered_vertices.is_empty() {
            ordered_vertices.push(start_vertex);
        } else if *ordered_vertices.last().unwrap_or(&start_vertex) != start_vertex {
            return Ok(None);
        }
        ordered_vertices.push(end_vertex);
    }

    Ok(Some(RootWireTopology {
        edge_indices,
        edge_orientations,
        vertex_indices: ordered_vertices,
    }))
}

fn wire_occurrence(
    context: &Context,
    edge_shape: &Shape,
    vertex_positions: &[[f64; 3]],
    root_edges: &[RootEdgeTopology],
) -> Result<Option<WireOccurrence>, Error> {
    let geometry = context.edge_geometry_occt(edge_shape)?;
    let endpoints = context.edge_endpoints_occt(edge_shape)?;
    let Some(mut start_vertex) = match_vertex_index(vertex_positions, endpoints.start) else {
        return Ok(None);
    };
    let Some(mut end_vertex) = match_vertex_index(vertex_positions, endpoints.end) else {
        return Ok(None);
    };
    let orientation = context.shape_orientation(edge_shape)?;
    if matches!(orientation, Orientation::Reversed) {
        std::mem::swap(&mut start_vertex, &mut end_vertex);
    }
    let length = edge_length(edge_shape);
    let matches = root_edges
        .iter()
        .enumerate()
        .filter(|(_, root_edge)| {
            root_edge.geometry.kind == geometry.kind
                && approx_eq(root_edge.length, length, 1.0e-6, 1.0e-6)
                && matches_edge_vertices(root_edge, start_vertex, end_vertex)
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Ok(None);
    }

    Ok(Some(WireOccurrence {
        edge_index: matches[0],
        orientation,
        start_vertex,
        end_vertex,
    }))
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

fn oriented_root_edge_vertices(
    root_edge: &RootEdgeTopology,
    orientation: Orientation,
) -> Option<(usize, usize)> {
    let start_vertex = root_edge.start_vertex?;
    let end_vertex = root_edge.end_vertex?;
    Some(match orientation {
        Orientation::Reversed => (end_vertex, start_vertex),
        _ => (start_vertex, end_vertex),
    })
}

fn topology_vertex_match(
    topology_vertices: &[[f64; 3]],
    root_vertices: &[[f64; 3]],
    index: usize,
) -> Option<usize> {
    topology_vertices
        .get(index)
        .copied()
        .and_then(|point| match_vertex_index(root_vertices, point))
}

fn edge_length(edge_shape: &Shape) -> f64 {
    edge_shape.linear_length()
}

fn match_vertex_index(vertex_positions: &[[f64; 3]], point: [f64; 3]) -> Option<usize> {
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

pub(super) fn oriented_edge_geometry(
    mut geometry: EdgeGeometry,
    orientation: Orientation,
) -> EdgeGeometry {
    if matches!(orientation, Orientation::Reversed) {
        std::mem::swap(&mut geometry.start_parameter, &mut geometry.end_parameter);
    }
    geometry
}
