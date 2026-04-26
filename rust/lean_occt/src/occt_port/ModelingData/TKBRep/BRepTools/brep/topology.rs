use super::edge_topology::root_edge_topology;
use super::face_snapshot::{
    load_ported_face_snapshot, PreparedFaceShape, TopologySnapshotFaceFields,
};
use super::wire_topology::{pack_wire_topology, root_wire_topology, PreparedRootWireShape};
use super::*;

pub(super) struct PreparedShellShape {
    pub(super) shell_shape: Shape,
    pub(super) shell_vertex_shapes: Vec<Shape>,
    pub(super) shell_edge_shapes: Vec<Shape>,
    pub(super) shell_face_shapes: Vec<Shape>,
}

struct TopologySnapshotRootFields {
    vertex_positions: Vec<[f64; 3]>,
    edge_shapes: Vec<Shape>,
    prepared_shell_shapes: Vec<PreparedShellShape>,
    face_shapes: Vec<Shape>,
    prepared_face_shapes: Vec<PreparedFaceShape>,
    edges: Vec<crate::TopologyEdge>,
    root_edges: Vec<super::edge_topology::RootEdgeTopology>,
    root_wires: Vec<super::wire_topology::RootWireTopology>,
    wires: Vec<crate::TopologyRange>,
    wire_edge_indices: Vec<usize>,
    wire_edge_orientations: Vec<Orientation>,
    wire_vertices: Vec<crate::TopologyRange>,
    wire_vertex_indices: Vec<usize>,
}

pub(super) struct LoadedPortedTopology {
    pub(super) topology: TopologySnapshot,
    pub(super) edge_shapes: Vec<Shape>,
    pub(super) prepared_shell_shapes: Vec<PreparedShellShape>,
    pub(super) face_shapes: Vec<Shape>,
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

    let prepared_wire_shapes = context
        .subshapes_occt(shape, ShapeKind::Wire)?
        .into_iter()
        .map(|wire_shape| {
            Ok(PreparedRootWireShape {
                wire_edge_shapes: context.subshapes_occt(&wire_shape, ShapeKind::Edge)?,
                wire_shape,
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    let mut root_wires = Vec::with_capacity(prepared_wire_shapes.len());
    for prepared_wire_shape in &prepared_wire_shapes {
        let Some(topology) =
            root_wire_topology(context, prepared_wire_shape, &vertex_positions, &root_edges)?
        else {
            return Ok(None);
        };
        root_wires.push(topology);
    }
    let (wires, wire_edge_indices, wire_edge_orientations, wire_vertices, wire_vertex_indices) =
        pack_wire_topology(&root_wires);
    let prepared_shell_shapes = context
        .subshapes_occt(shape, ShapeKind::Shell)?
        .into_iter()
        .map(|shell_shape| {
            Ok(PreparedShellShape {
                shell_vertex_shapes: context.subshapes_occt(&shell_shape, ShapeKind::Vertex)?,
                shell_edge_shapes: context.subshapes_occt(&shell_shape, ShapeKind::Edge)?,
                shell_face_shapes: context.subshapes_occt(&shell_shape, ShapeKind::Face)?,
                shell_shape,
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    let face_shapes = context.subshapes_occt(shape, ShapeKind::Face)?;
    let prepared_face_shapes = face_shapes
        .iter()
        .enumerate()
        .map(|(face_index, face_shape)| {
            Ok(PreparedFaceShape {
                face_index,
                face_wire_shapes: context
                    .subshapes_occt(face_shape, ShapeKind::Wire)?
                    .into_iter()
                    .map(|face_wire_shape| {
                        Ok(PreparedRootWireShape {
                            wire_edge_shapes: context
                                .subshapes_occt(&face_wire_shape, ShapeKind::Edge)?,
                            wire_shape: face_wire_shape,
                        })
                    })
                    .collect::<Result<Vec<_>, Error>>()?,
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;

    Ok(Some(TopologySnapshotRootFields {
        vertex_positions,
        edge_shapes,
        prepared_shell_shapes,
        face_shapes,
        prepared_face_shapes,
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

pub(super) fn load_ported_topology(
    context: &Context,
    shape: &Shape,
) -> Result<Option<LoadedPortedTopology>, Error> {
    let Some(TopologySnapshotRootFields {
        vertex_positions,
        edge_shapes,
        prepared_shell_shapes,
        face_shapes,
        prepared_face_shapes,
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
        &prepared_face_shapes,
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

    Ok(Some(LoadedPortedTopology {
        topology: TopologySnapshot {
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
        },
        edge_shapes,
        prepared_shell_shapes,
        face_shapes,
    }))
}

pub(super) fn ported_topology_snapshot(
    context: &Context,
    shape: &Shape,
) -> Result<Option<TopologySnapshot>, Error> {
    Ok(load_ported_topology(context, shape)?.map(|loaded| loaded.topology))
}
