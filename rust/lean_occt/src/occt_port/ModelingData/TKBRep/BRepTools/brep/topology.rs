use super::*;

use super::summary::{classify_root_kind, shape_counts};
use super::swept_face::append_root_edge_sample_points;

#[derive(Clone, Copy, Debug)]
pub(super) struct RootEdgeTopology {
    pub(super) geometry: EdgeGeometry,
    start_vertex: Option<usize>,
    end_vertex: Option<usize>,
    length: f64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RootWireTopology {
    edge_indices: Vec<usize>,
    edge_orientations: Vec<Orientation>,
    vertex_indices: Vec<usize>,
}

#[derive(Clone, Copy, Debug)]
struct WireOccurrence {
    edge_index: usize,
    orientation: Orientation,
    start_vertex: usize,
    end_vertex: usize,
}

struct PortedFaceTopology {
    edge_indices: BTreeSet<usize>,
    face_wire_indices: Vec<usize>,
    face_wire_orientations: Vec<Orientation>,
    face_wire_roles: Vec<LoopRole>,
}

pub(super) fn ported_topology_snapshot(
    context: &Context,
    shape: &Shape,
) -> Result<Option<TopologySnapshot>, Error> {
    let face_shapes = context.subshapes_occt(shape, ShapeKind::Face)?;
    for face_shape in &face_shapes {
        let face_wire_shapes = context.subshapes_occt(face_shape, ShapeKind::Wire)?;
        let geometry = match context.face_geometry(face_shape) {
            Ok(geometry) => geometry,
            Err(_) => context.face_geometry_occt(face_shape)?,
        };
        if face_wire_shapes.len() > 1 && geometry.kind != crate::SurfaceKind::Plane {
            return Ok(None);
        }
    }

    let vertex_shapes = context.subshapes_occt(shape, ShapeKind::Vertex)?;
    let vertex_positions = vertex_shapes
        .iter()
        .map(|vertex_shape| context.vertex_point_occt(vertex_shape))
        .collect::<Result<Vec<_>, Error>>()?;

    let edge_shapes = context.subshapes_occt(shape, ShapeKind::Edge)?;
    let root_edges = edge_shapes
        .iter()
        .map(|edge_shape| root_edge_topology(context, edge_shape, &vertex_positions))
        .collect::<Result<Vec<_>, Error>>()?;
    let edges = root_edges
        .iter()
        .map(|edge| crate::TopologyEdge {
            start_vertex: edge.start_vertex,
            end_vertex: edge.end_vertex,
            length: edge.length,
        })
        .collect::<Vec<_>>();

    let wire_shapes = context.subshapes_occt(shape, ShapeKind::Wire)?;
    let mut root_wires = Vec::with_capacity(wire_shapes.len());
    for wire_shape in &wire_shapes {
        let Some(topology) =
            root_wire_topology(context, wire_shape, &vertex_positions, &root_edges)?
        else {
            return Ok(None);
        };
        root_wires.push(topology);
    }
    let (wires, wire_edge_indices, wire_edge_orientations, wire_vertices, wire_vertex_indices) =
        pack_wire_topology(&root_wires);
    let mut edge_face_lists = vec![Vec::new(); edges.len()];
    let mut faces = Vec::with_capacity(face_shapes.len());
    let mut face_wire_indices = Vec::new();
    let mut face_wire_orientations = Vec::new();
    let mut face_wire_roles = Vec::new();

    for (face_index, face_shape) in face_shapes.iter().enumerate() {
        let Some(face_topology) = ported_face_topology(
            context,
            face_shape,
            &root_wires,
            &root_edges,
            &edge_shapes,
            &vertex_positions,
        )?
        else {
            return Ok(None);
        };

        faces.push(crate::TopologyRange {
            offset: face_wire_indices.len(),
            count: face_topology.face_wire_indices.len(),
        });
        face_wire_indices.extend(face_topology.face_wire_indices);
        face_wire_orientations.extend(face_topology.face_wire_orientations);
        face_wire_roles.extend(face_topology.face_wire_roles);

        for edge_index in face_topology.edge_indices {
            let Some(edge_faces) = edge_face_lists.get_mut(edge_index) else {
                return Ok(None);
            };
            edge_faces.push(face_index);
        }
    }

    let mut edge_faces = Vec::with_capacity(edges.len());
    let mut edge_face_indices = Vec::new();
    for face_indices in edge_face_lists {
        edge_faces.push(crate::TopologyRange {
            offset: edge_face_indices.len(),
            count: face_indices.len(),
        });
        edge_face_indices.extend(face_indices);
    }

    Ok(Some(TopologySnapshot {
        vertex_positions,
        edges,
        edge_faces,
        edge_face_indices,
        wires,
        wire_edge_indices,
        wire_edge_orientations,
        wire_vertices,
        wire_vertex_indices,
        faces,
        face_wire_indices,
        face_wire_orientations,
        face_wire_roles,
    }))
}

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

pub(super) fn ported_brep_vertices(topology: &TopologySnapshot) -> Vec<BrepVertex> {
    topology
        .vertex_positions
        .iter()
        .copied()
        .enumerate()
        .map(|(index, position)| BrepVertex { index, position })
        .collect()
}

pub(super) fn ported_brep_wires(topology: &TopologySnapshot) -> Vec<BrepWire> {
    topology
        .wires
        .iter()
        .enumerate()
        .map(|(index, range)| {
            let edge_indices =
                topology.wire_edge_indices[range.offset..range.offset + range.count].to_vec();
            let edge_orientations =
                topology.wire_edge_orientations[range.offset..range.offset + range.count].to_vec();
            let vertex_range = topology.wire_vertices[index];
            let vertex_indices = topology.wire_vertex_indices
                [vertex_range.offset..vertex_range.offset + vertex_range.count]
                .to_vec();
            BrepWire {
                index,
                edge_indices,
                edge_orientations,
                vertex_indices,
            }
        })
        .collect()
}

pub(super) fn ported_brep_edges(
    context: &Context,
    shape: &Shape,
    topology: &TopologySnapshot,
) -> Result<(Vec<Shape>, Vec<BrepEdge>), Error> {
    let edge_shapes = context.subshapes_occt(shape, ShapeKind::Edge)?;
    let edges = edge_shapes
        .iter()
        .enumerate()
        .map(|(index, edge_shape)| {
            let topology_edge = topology_edge(topology, index)?;
            let geometry = match context.edge_geometry(edge_shape) {
                Ok(geometry) => geometry,
                Err(_) => context.edge_geometry_occt(edge_shape)?,
            };
            let ported_curve =
                match PortedCurve::from_context_with_ported_payloads(context, edge_shape, geometry)
                {
                    Ok(ported_curve) => ported_curve,
                    Err(_) => {
                        PortedCurve::from_context_with_geometry(context, edge_shape, geometry)?
                    }
                };
            let adjacent_face_indices = adjacent_face_indices(topology, index)?;
            let (start_point, end_point) = edge_points(topology, index);
            let length = match ported_curve {
                Some(curve) => curve.length_with_geometry(geometry),
                None => topology_edge.length,
            };

            Ok(BrepEdge {
                index,
                geometry,
                ported_curve,
                length,
                start_vertex: topology_edge.start_vertex,
                end_vertex: topology_edge.end_vertex,
                start_point,
                end_point,
                adjacent_face_indices,
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;

    Ok((edge_shapes, edges))
}

fn root_edge_topology(
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

fn root_wire_topology(
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

fn pack_wire_topology(
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

fn ported_face_topology(
    context: &Context,
    face_shape: &Shape,
    root_wires: &[RootWireTopology],
    root_edges: &[RootEdgeTopology],
    edge_shapes: &[Shape],
    vertex_positions: &[[f64; 3]],
) -> Result<Option<PortedFaceTopology>, Error> {
    let face_wire_shapes = context.subshapes_occt(face_shape, ShapeKind::Wire)?;
    if root_wires.is_empty() || face_wire_shapes.is_empty() {
        return Ok(None);
    }

    let mut used_root_wire_indices = BTreeSet::new();
    let mut used_edges = BTreeSet::new();
    let mut face_wire_indices = Vec::with_capacity(face_wire_shapes.len());
    let mut face_wire_orientations = Vec::with_capacity(face_wire_shapes.len());
    let mut face_wire_areas = Vec::new();

    let mut planar_face = None;
    if face_wire_shapes.len() > 1 {
        let face_geometry = context.face_geometry_occt(face_shape)?;
        if face_geometry.kind != crate::SurfaceKind::Plane {
            return Ok(None);
        }
        planar_face = Some((context.face_plane_payload_occt(face_shape)?, face_geometry));
    }

    for face_wire_shape in &face_wire_shapes {
        let Some(face_wire_topology) =
            root_wire_topology(context, face_wire_shape, vertex_positions, root_edges)?
        else {
            return Ok(None);
        };
        let Some(root_wire_index) =
            match_root_wire_index(root_wires, &face_wire_topology, &used_root_wire_indices)
        else {
            return Ok(None);
        };
        used_root_wire_indices.insert(root_wire_index);
        used_edges.extend(face_wire_topology.edge_indices.iter().copied());

        face_wire_indices.push(root_wire_index);
        face_wire_orientations.push(context.shape_orientation(face_wire_shape)?);

        if let Some((plane, face_geometry)) = planar_face {
            let Some(wire_area) = planar_wire_area_magnitude(
                context,
                plane,
                face_geometry,
                &root_wires[root_wire_index],
                edge_shapes,
                root_edges,
            )?
            else {
                return Ok(None);
            };
            face_wire_areas.push(wire_area);
        }
    }

    let face_wire_roles = match face_wire_shapes.len() {
        1 => vec![LoopRole::Outer],
        _ => {
            let Some((outer_offset, outer_area)) = face_wire_areas
                .iter()
                .copied()
                .enumerate()
                .max_by(|(_, lhs), (_, rhs)| lhs.total_cmp(rhs))
            else {
                return Ok(None);
            };
            if outer_area <= 1.0e-9 {
                return Ok(None);
            }

            face_wire_areas
                .iter()
                .enumerate()
                .map(|(offset, _)| {
                    if offset == outer_offset {
                        LoopRole::Outer
                    } else {
                        LoopRole::Inner
                    }
                })
                .collect()
        }
    };

    Ok(Some(PortedFaceTopology {
        edge_indices: used_edges,
        face_wire_indices,
        face_wire_orientations,
        face_wire_roles,
    }))
}

fn match_root_wire_index(
    root_wires: &[RootWireTopology],
    face_wire_topology: &RootWireTopology,
    used_root_wire_indices: &BTreeSet<usize>,
) -> Option<usize> {
    root_wires
        .iter()
        .enumerate()
        .find(|(index, root_wire)| {
            !used_root_wire_indices.contains(index) && *root_wire == face_wire_topology
        })
        .map(|(index, _)| index)
}

fn planar_wire_area_magnitude(
    context: &Context,
    plane: PlanePayload,
    face_geometry: FaceGeometry,
    wire: &RootWireTopology,
    edge_shapes: &[Shape],
    root_edges: &[RootEdgeTopology],
) -> Result<Option<f64>, Error> {
    let mut curve_segments = Vec::with_capacity(wire.edge_indices.len());
    let mut sampled_points = Vec::new();

    for (&edge_index, &edge_orientation) in wire.edge_indices.iter().zip(&wire.edge_orientations) {
        let Some(root_edge) = root_edges.get(edge_index) else {
            return Ok(None);
        };
        let Some(edge_shape) = edge_shapes.get(edge_index) else {
            return Ok(None);
        };

        let geometry = oriented_edge_geometry(root_edge.geometry, edge_orientation);
        if let Some(curve) =
            PortedCurve::from_context_with_geometry(context, edge_shape, root_edge.geometry)?
        {
            curve_segments.push((curve, geometry));
        }

        append_root_edge_sample_points(
            context,
            edge_shape,
            root_edge,
            geometry,
            &mut sampled_points,
        )?;
    }

    let area = if curve_segments.len() == wire.edge_indices.len() {
        planar_wire_signed_area(plane, &curve_segments).abs()
    } else {
        let Some(area) = analytic_sampled_wire_signed_area(
            PortedSurface::Plane(plane),
            face_geometry,
            &sampled_points,
        ) else {
            return Ok(None);
        };
        area.abs()
    };
    Ok(Some(area))
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

pub(super) fn face_loops(
    topology: &TopologySnapshot,
    face_index: usize,
) -> Result<Vec<BrepFaceLoop>, Error> {
    let range = topology
        .faces
        .get(face_index)
        .copied()
        .ok_or_else(|| Error::new(format!("topology is missing face range {face_index}")))?;
    let mut loops = Vec::with_capacity(range.count);
    for offset in range.offset..range.offset + range.count {
        loops.push(BrepFaceLoop {
            wire_index: topology
                .face_wire_indices
                .get(offset)
                .copied()
                .ok_or_else(|| {
                    Error::new(format!("topology is missing face-wire index {offset}"))
                })?,
            orientation: topology
                .face_wire_orientations
                .get(offset)
                .copied()
                .ok_or_else(|| {
                    Error::new(format!(
                        "topology is missing face-wire orientation {offset}"
                    ))
                })?,
            role: topology
                .face_wire_roles
                .get(offset)
                .copied()
                .ok_or_else(|| {
                    Error::new(format!("topology is missing face-wire role {offset}"))
                })?,
        });
    }
    Ok(loops)
}

pub(super) fn face_adjacent_face_indices(
    topology: &TopologySnapshot,
    wires: &[BrepWire],
    face_index: usize,
) -> Result<Vec<usize>, Error> {
    let loops = face_loops(topology, face_index)?;
    let mut adjacent = BTreeSet::new();
    for face_loop in loops {
        let wire = wires.get(face_loop.wire_index).ok_or_else(|| {
            Error::new(format!(
                "topology is missing wire index {}",
                face_loop.wire_index
            ))
        })?;
        for &edge_index in &wire.edge_indices {
            let range = topology
                .edge_faces
                .get(edge_index)
                .copied()
                .ok_or_else(|| {
                    Error::new(format!("topology is missing edge-face range {edge_index}"))
                })?;
            for &candidate in &topology.edge_face_indices[range.offset..range.offset + range.count]
            {
                if candidate != face_index {
                    adjacent.insert(candidate);
                }
            }
        }
    }
    Ok(adjacent.into_iter().collect())
}
