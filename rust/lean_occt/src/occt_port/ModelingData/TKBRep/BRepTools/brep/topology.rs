use super::face_snapshot::load_ported_face_snapshot;
use super::root_topology::{load_root_topology_snapshot, RootTopologySnapshot};
use super::*;

pub(super) struct TopologySnapshotFaceFields {
    pub(super) edge_faces: Vec<crate::TopologyRange>,
    pub(super) edge_face_indices: Vec<usize>,
    pub(super) faces: Vec<crate::TopologyRange>,
    pub(super) face_wire_indices: Vec<usize>,
    pub(super) face_wire_orientations: Vec<Orientation>,
    pub(super) face_wire_roles: Vec<LoopRole>,
}

pub(super) fn ported_topology_snapshot(
    context: &Context,
    shape: &Shape,
) -> Result<Option<TopologySnapshot>, Error> {
    let Some(root_topology) = load_root_topology_snapshot(context, shape)? else {
        return Ok(None);
    };
    let Some(face_topology) = load_ported_face_snapshot(
        context,
        shape,
        &root_topology.root_wires,
        &root_topology.root_edges,
        &root_topology.edge_shapes,
        &root_topology.vertex_positions,
        root_topology.edges.len(),
    )?
    else {
        return Ok(None);
    };

    Ok(Some(build_ported_topology_snapshot(
        root_topology,
        face_topology,
    )))
}

fn build_ported_topology_snapshot(
    root_topology: RootTopologySnapshot,
    face_topology: TopologySnapshotFaceFields,
) -> TopologySnapshot {
    TopologySnapshot {
        vertex_positions: root_topology.vertex_positions,
        edges: root_topology.edges,
        edge_faces: face_topology.edge_faces,
        edge_face_indices: face_topology.edge_face_indices,
        wires: root_topology.wires,
        wire_edge_indices: root_topology.wire_edge_indices,
        wire_edge_orientations: root_topology.wire_edge_orientations,
        wire_vertices: root_topology.wire_vertices,
        wire_vertex_indices: root_topology.wire_vertex_indices,
        faces: face_topology.faces,
        face_wire_indices: face_topology.face_wire_indices,
        face_wire_orientations: face_topology.face_wire_orientations,
        face_wire_roles: face_topology.face_wire_roles,
    }
}
