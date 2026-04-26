use super::edge_topology::root_edge_topology;
use super::face_snapshot::{
    load_ported_face_snapshot, PreparedFaceShape, TopologySnapshotFaceFields,
};
use super::wire_topology::{pack_wire_topology, root_wire_topology, PreparedRootWireShape};
use super::*;
use crate::OffsetSurfaceFaceMetadata;

pub(super) struct PreparedShellShape {
    pub(super) shell_shape: Shape,
    pub(super) shell_vertex_shapes: Vec<Shape>,
    pub(super) shell_edge_shapes: Vec<Shape>,
    pub(super) shell_face_shapes: Vec<Shape>,
}

struct TopologySnapshotRootFields {
    vertex_shapes: Vec<Shape>,
    vertex_positions: Vec<[f64; 3]>,
    edge_shapes: Vec<Shape>,
    wire_shapes: Vec<Shape>,
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
    pub(super) vertex_shapes: Vec<Shape>,
    pub(super) edge_shapes: Vec<Shape>,
    pub(super) wire_shapes: Vec<Shape>,
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
    let wire_shapes = prepared_wire_shapes
        .into_iter()
        .map(|prepared_wire_shape| prepared_wire_shape.wire_shape)
        .collect::<Vec<_>>();
    let multi_face_offset_metadata = shape
        .multi_face_offset_result_metadata()
        .map(|metadata| metadata.to_vec());
    let prepared_shell_shapes = context
        .subshapes_occt(shape, ShapeKind::Shell)?
        .into_iter()
        .map(|shell_shape| {
            let shell_face_shapes = attach_offset_result_face_metadata(
                context,
                shape,
                context.subshapes_occt(&shell_shape, ShapeKind::Face)?,
            )?;
            let shell_shape = match &multi_face_offset_metadata {
                Some(metadata) => {
                    shell_shape.with_multi_face_offset_result_metadata(metadata.clone())
                }
                None => shell_shape,
            };
            Ok(PreparedShellShape {
                shell_vertex_shapes: context.subshapes_occt(&shell_shape, ShapeKind::Vertex)?,
                shell_edge_shapes: context.subshapes_occt(&shell_shape, ShapeKind::Edge)?,
                shell_face_shapes,
                shell_shape,
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    let face_shapes = attach_offset_result_face_metadata(
        context,
        shape,
        context.subshapes_occt(shape, ShapeKind::Face)?,
    )?;
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
        vertex_shapes,
        vertex_positions,
        edge_shapes,
        wire_shapes,
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

fn attach_offset_result_face_metadata(
    context: &Context,
    shape: &Shape,
    face_shapes: Vec<Shape>,
) -> Result<Vec<Shape>, Error> {
    if let Some(metadata) = shape.single_face_offset_result_metadata() {
        return attach_single_face_offset_metadata(context, metadata, face_shapes);
    }

    if let Some(metadata) = shape.multi_face_offset_result_metadata() {
        return attach_multi_face_offset_metadata(context, metadata, face_shapes);
    }

    Ok(face_shapes)
}

fn attach_single_face_offset_metadata(
    context: &Context,
    metadata: OffsetSurfaceFaceMetadata,
    face_shapes: Vec<Shape>,
) -> Result<Vec<Shape>, Error> {
    if face_shapes.len() != 1 {
        return Ok(face_shapes);
    }
    let mut face_shapes = face_shapes;
    let face_shape = face_shapes
        .pop()
        .expect("length was checked before popping single offset face");
    if context.face_geometry_occt(&face_shape)?.kind != crate::SurfaceKind::Offset {
        return Ok(vec![face_shape]);
    }

    Ok(vec![face_shape.with_offset_surface_face_metadata(metadata)])
}

fn attach_multi_face_offset_metadata(
    context: &Context,
    metadata: &[OffsetSurfaceFaceMetadata],
    face_shapes: Vec<Shape>,
) -> Result<Vec<Shape>, Error> {
    if metadata.is_empty() {
        return Ok(face_shapes);
    }

    face_shapes
        .into_iter()
        .map(|face_shape| {
            if context.face_geometry_occt(&face_shape)?.kind != crate::SurfaceKind::Offset {
                return Ok(face_shape);
            }

            let mut matched = None;
            let mut match_count = 0usize;
            for candidate in metadata.iter().copied() {
                let signed_candidates = [
                    candidate,
                    OffsetSurfaceFaceMetadata {
                        offset_value: -candidate.offset_value,
                        ..candidate
                    },
                ];
                for (variant_index, signed_candidate) in signed_candidates.into_iter().enumerate() {
                    if variant_index == 1 && candidate.offset_value.abs() <= 1.0e-12 {
                        continue;
                    }
                    match context.ported_offset_surface_from_metadata(&face_shape, signed_candidate)
                    {
                        Ok(Some(_)) => {
                            matched = Some(signed_candidate);
                            match_count += 1;
                        }
                        Ok(None) | Err(_) => {}
                    }
                }
            }

            if match_count == 1 {
                Ok(face_shape.with_offset_surface_face_metadata(
                    matched.expect("single validated offset metadata match"),
                ))
            } else {
                Ok(face_shape)
            }
        })
        .collect()
}

pub(super) fn load_ported_topology(
    context: &Context,
    shape: &Shape,
) -> Result<Option<LoadedPortedTopology>, Error> {
    let Some(TopologySnapshotRootFields {
        vertex_shapes,
        vertex_positions,
        edge_shapes,
        wire_shapes,
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
        vertex_shapes,
        edge_shapes,
        wire_shapes,
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
