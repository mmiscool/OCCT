use super::*;

pub(super) struct TopologySnapshotRootFields {
    pub(super) vertex_positions: Vec<[f64; 3]>,
    pub(super) edge_shapes: Vec<Shape>,
    pub(super) edges: Vec<crate::TopologyEdge>,
    pub(super) root_edges: Vec<super::edge_topology::RootEdgeTopology>,
    pub(super) root_wires: Vec<super::wire_topology::RootWireTopology>,
    pub(super) wires: Vec<crate::TopologyRange>,
    pub(super) wire_edge_indices: Vec<usize>,
    pub(super) wire_edge_orientations: Vec<Orientation>,
    pub(super) wire_vertices: Vec<crate::TopologyRange>,
    pub(super) wire_vertex_indices: Vec<usize>,
}

pub(super) struct TopologySnapshotFaceFields {
    pub(super) edge_faces: Vec<crate::TopologyRange>,
    pub(super) edge_face_indices: Vec<usize>,
    pub(super) faces: Vec<crate::TopologyRange>,
    pub(super) face_wire_indices: Vec<usize>,
    pub(super) face_wire_orientations: Vec<Orientation>,
    pub(super) face_wire_roles: Vec<LoopRole>,
}

pub(super) fn build_ported_topology_snapshot(
    root_topology: TopologySnapshotRootFields,
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
