use super::edge_topology::root_edge_topology;
use super::face_snapshot::{load_ported_face_snapshot, TopologySnapshotFaceFields};
use super::wire_topology::{pack_wire_topology, root_wire_topology};
use super::*;

struct TopologySnapshotRootFields {
    vertex_positions: Vec<[f64; 3]>,
    edge_shapes: Vec<Shape>,
    edges: Vec<crate::TopologyEdge>,
    root_edges: Vec<super::edge_topology::RootEdgeTopology>,
    root_wires: Vec<super::wire_topology::RootWireTopology>,
    wires: Vec<crate::TopologyRange>,
    wire_edge_indices: Vec<usize>,
    wire_edge_orientations: Vec<Orientation>,
    wire_vertices: Vec<crate::TopologyRange>,
    wire_vertex_indices: Vec<usize>,
}

fn load_root_topology_snapshot(
    context: &Context,
    shape: &Shape,
) -> Result<Option<TopologySnapshotRootFields>, Error> {
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

    Ok(Some(TopologySnapshotRootFields {
        vertex_positions,
        edge_shapes,
        edges,
        root_edges,
        root_wires,
        wires,
        wire_edge_indices,
        wire_edge_orientations,
        wire_vertices,
        wire_vertex_indices,
    }))
}

pub(super) fn ported_topology_snapshot(
    context: &Context,
    shape: &Shape,
) -> Result<Option<TopologySnapshot>, Error> {
    let Some(TopologySnapshotRootFields {
        vertex_positions,
        edge_shapes,
        edges,
        root_edges,
        root_wires,
        wires,
        wire_edge_indices,
        wire_edge_orientations,
        wire_vertices,
        wire_vertex_indices,
    }) = load_root_topology_snapshot(context, shape)?
    else {
        return Ok(None);
    };
    let Some(TopologySnapshotFaceFields {
        edge_faces,
        edge_face_indices,
        faces,
        face_wire_indices,
        face_wire_orientations,
        face_wire_roles,
    }) = load_ported_face_snapshot(
        context,
        shape,
        &root_wires,
        &root_edges,
        &edge_shapes,
        &vertex_positions,
        edges.len(),
    )?
    else {
        return Ok(None);
    };

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
