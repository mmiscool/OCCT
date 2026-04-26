use super::brep_materialize::{ported_brep_edge_geometry_and_curve, ported_brep_wires};
use super::topology::load_ported_topology;
use super::*;

#[derive(Clone, Copy)]
pub(super) enum FaceSurfaceRoute {
    Raw,
    Public,
}

pub(super) struct SingleFaceTopology {
    pub(super) loops: Vec<BrepFaceLoop>,
    pub(super) wires: Vec<BrepWire>,
    pub(super) edges: Vec<BrepEdge>,
    pub(super) edge_shapes: Vec<Shape>,
}

struct SingleFaceTopologySnapshot {
    topology: TopologySnapshot,
    edge_shapes: Vec<Shape>,
}

pub(super) fn single_face_topology_with_route(
    context: &Context,
    face_shape: &Shape,
    route: FaceSurfaceRoute,
) -> Result<Option<SingleFaceTopology>, Error> {
    let snapshot = match single_face_topology_snapshot(context, face_shape)? {
        Some(snapshot) => snapshot,
        None => return Ok(None),
    };

    let topology = snapshot.topology;
    let wires = ported_brep_wires(&topology);
    let edge_shapes = snapshot.edge_shapes;
    let edges = edge_shapes
        .iter()
        .enumerate()
        .map(|(index, edge_shape)| single_face_edge_with_route(context, index, edge_shape, route))
        .collect::<Result<Vec<_>, Error>>()?;

    Ok(Some(SingleFaceTopology {
        loops: face_loops(&topology, 0)?,
        wires,
        edges,
        edge_shapes,
    }))
}

fn single_face_topology_snapshot(
    context: &Context,
    face_shape: &Shape,
) -> Result<Option<SingleFaceTopologySnapshot>, Error> {
    let loaded = match load_ported_topology(context, face_shape)? {
        Some(loaded) => loaded,
        None => return Ok(None),
    };
    if loaded.topology.faces.len() != 1 {
        return Ok(None);
    }
    Ok(Some(SingleFaceTopologySnapshot {
        topology: loaded.topology,
        edge_shapes: loaded.edge_shapes,
    }))
}

fn single_face_edge_with_route(
    context: &Context,
    index: usize,
    edge_shape: &Shape,
    _route: FaceSurfaceRoute,
) -> Result<BrepEdge, Error> {
    let (geometry, ported_curve) = ported_brep_edge_geometry_and_curve(context, edge_shape)?;
    Ok(single_face_edge(index, geometry, ported_curve))
}

fn single_face_edge(
    index: usize,
    geometry: EdgeGeometry,
    ported_curve: Option<PortedCurve>,
) -> BrepEdge {
    BrepEdge {
        index,
        geometry,
        ported_curve,
        length: 0.0,
        start_vertex: None,
        end_vertex: None,
        start_point: None,
        end_point: None,
        adjacent_face_indices: Vec::new(),
    }
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
