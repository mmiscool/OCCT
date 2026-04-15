use super::face_snapshot::{pack_ported_face_snapshot, validate_ported_face_snapshot};
use super::root_topology::{pack_wire_topology, root_edge_topology, root_wire_topology};
use super::*;

pub(super) fn ported_topology_snapshot(
    context: &Context,
    shape: &Shape,
) -> Result<Option<TopologySnapshot>, Error> {
    let face_shapes = context.subshapes_occt(shape, ShapeKind::Face)?;
    if !validate_ported_face_snapshot(context, &face_shapes)? {
        return Ok(None);
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
    let Some(face_topology) = pack_ported_face_snapshot(
        context,
        &face_shapes,
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
        edge_faces: face_topology.edge_faces,
        edge_face_indices: face_topology.edge_face_indices,
        wires,
        wire_edge_indices,
        wire_edge_orientations,
        wire_vertices,
        wire_vertex_indices,
        faces: face_topology.faces,
        face_wire_indices: face_topology.face_wire_indices,
        face_wire_orientations: face_topology.face_wire_orientations,
        face_wire_roles: face_topology.face_wire_roles,
    }))
}
