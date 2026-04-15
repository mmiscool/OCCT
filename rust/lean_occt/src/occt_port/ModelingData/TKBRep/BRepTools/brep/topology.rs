use super::face_snapshot::ported_face_topology;
use super::root_topology::{pack_wire_topology, root_edge_topology, root_wire_topology};
use super::*;

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
