use super::edge_topology::{
    root_edge_topology, topology_edge_length, topology_edge_query, RootEdgeTopology,
};
use super::face_snapshot::{
    load_ported_face_snapshot, PreparedFaceShape, TopologySnapshotFaceFields,
};
use super::wire_topology::{
    match_vertex_index, pack_wire_topology, root_wire_topology, PreparedRootWireShape,
};
use super::*;
use crate::{OffsetSurfaceFaceMetadata, OffsetSurfacePayload};

const OFFSET_METADATA_MATCH_UV_SAMPLES: [[f64; 2]; 4] =
    [[0.23, 0.31], [0.37, 0.61], [0.58, 0.47], [0.79, 0.73]];
const OFFSET_METADATA_MATCH_SCORE_TOLERANCE: f64 = 1.0e-12;

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
    solid_shapes: Vec<Shape>,
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
    pub(super) solid_shapes: Vec<Shape>,
    pub(super) wire_shapes: Vec<Shape>,
    pub(super) prepared_shell_shapes: Vec<PreparedShellShape>,
    pub(super) face_shapes: Vec<Shape>,
}

#[derive(Clone, Copy)]
enum RootAssemblyKind {
    Compound,
    CompSolid,
}

enum RootAssemblyTopologyInventory {
    Supported(RootAssemblyKind),
    Unsupported,
    NotAssembly,
}

const ROOT_ASSEMBLY_MAX_DEPTH: usize = 16;

pub(super) fn root_assembly_requires_ported_topology(
    context: &Context,
    shape: &Shape,
) -> Result<bool, Error> {
    Ok(matches!(
        root_assembly_topology_inventory_required(context, shape)?,
        RootAssemblyTopologyInventory::Supported(_)
    ))
}

fn load_root_topology_snapshot(
    context: &Context,
    shape: &Shape,
) -> Result<Option<TopologySnapshotRootFields>, Error> {
    if let Some(root_edge_fields) = load_root_edge_topology_snapshot(context, shape)? {
        return Ok(Some(root_edge_fields));
    }
    if root_vertex_topology_inventory_required(context, shape)? {
        return load_root_vertex_topology_snapshot(context, shape);
    }
    if root_wire_topology_inventory_required(context, shape)? {
        return load_root_wire_topology_snapshot(context, shape);
    }
    if root_face_topology_inventory_required(context, shape)? {
        return load_root_face_topology_snapshot(context, shape);
    }
    if root_shell_topology_inventory_required(context, shape)? {
        return load_root_shell_topology_snapshot(context, shape);
    }
    if root_solid_topology_inventory_required(context, shape)? {
        return load_root_solid_topology_snapshot(context, shape);
    }
    match root_assembly_topology_inventory_required(context, shape)? {
        RootAssemblyTopologyInventory::Supported(root_assembly_kind) => {
            return load_root_assembly_topology_snapshot(context, shape, root_assembly_kind);
        }
        RootAssemblyTopologyInventory::Unsupported => return Ok(None),
        RootAssemblyTopologyInventory::NotAssembly => {}
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
    let solid_shapes = context.subshapes_occt(shape, ShapeKind::Solid)?;

    let prepared_wire_shapes = context
        .subshapes_occt(shape, ShapeKind::Wire)?
        .into_iter()
        .map(|wire_shape| {
            Ok(PreparedRootWireShape {
                wire_edge_occurrence_shapes: context.wire_edge_occurrences_occt(&wire_shape)?,
                wire_shape,
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    let mut root_wires = Vec::with_capacity(prepared_wire_shapes.len());
    for prepared_wire_shape in &prepared_wire_shapes {
        let Some(topology) = root_wire_topology(
            context,
            prepared_wire_shape,
            &vertex_positions,
            &edge_shapes,
            &root_edges,
        )?
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
                            wire_edge_occurrence_shapes: context
                                .wire_edge_occurrences_occt(&face_wire_shape)?,
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
        solid_shapes,
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

fn root_vertex_topology_inventory_required(
    context: &Context,
    shape: &Shape,
) -> Result<bool, Error> {
    Ok(context.describe_shape_occt(shape)?.root_kind == ShapeKind::Vertex)
}

fn load_root_vertex_topology_snapshot(
    context: &Context,
    shape: &Shape,
) -> Result<Option<TopologySnapshotRootFields>, Error> {
    let vertex_position = match context.root_vertex_point_seed_occt(shape) {
        Ok(point) => point,
        Err(_) => return Ok(None),
    };
    let vertex_shape = match context.duplicate_shape_occt(shape) {
        Ok(vertex_shape) => vertex_shape,
        Err(_) => return Ok(None),
    };

    Ok(Some(TopologySnapshotRootFields {
        vertex_shapes: vec![vertex_shape],
        vertex_positions: vec![vertex_position],
        edge_shapes: Vec::new(),
        solid_shapes: Vec::new(),
        wire_shapes: Vec::new(),
        prepared_shell_shapes: Vec::new(),
        face_shapes: Vec::new(),
        prepared_face_shapes: Vec::new(),
        edges: Vec::new(),
        root_edges: Vec::new(),
        root_wires: Vec::new(),
        wires: Vec::new(),
        wire_edge_indices: Vec::new(),
        wire_edge_orientations: Vec::new(),
        wire_vertices: Vec::new(),
        wire_vertex_indices: Vec::new(),
    }))
}

fn load_root_edge_topology_snapshot(
    context: &Context,
    shape: &Shape,
) -> Result<Option<TopologySnapshotRootFields>, Error> {
    if context.describe_shape_occt(shape)?.root_kind != ShapeKind::Edge {
        return Ok(None);
    }

    let Some(endpoints) = ported_edge_endpoints(context, shape)? else {
        return Ok(None);
    };
    let geometry = context.edge_geometry(shape)?;
    if !matches!(
        geometry.kind,
        CurveKind::Line | CurveKind::Circle | CurveKind::Ellipse
    ) {
        return Ok(None);
    }

    let (vertex_shapes, vertex_positions, start_vertex, end_vertex) =
        root_edge_vertices_from_ported_seed(context, shape, endpoints)?;
    let edge_shape = context.duplicate_shape_occt(shape)?;
    let length = topology_edge_length(context, shape, geometry)?;
    let root_edges = vec![RootEdgeTopology {
        geometry,
        start_vertex,
        end_vertex,
        length,
    }];
    let edges = vec![crate::TopologyEdge {
        start_vertex,
        end_vertex,
        length,
    }];

    Ok(Some(TopologySnapshotRootFields {
        vertex_shapes,
        vertex_positions,
        edge_shapes: vec![edge_shape],
        solid_shapes: Vec::new(),
        wire_shapes: Vec::new(),
        prepared_shell_shapes: Vec::new(),
        face_shapes: Vec::new(),
        prepared_face_shapes: Vec::new(),
        edges,
        root_edges,
        root_wires: Vec::new(),
        wires: Vec::new(),
        wire_edge_indices: Vec::new(),
        wire_edge_orientations: Vec::new(),
        wire_vertices: Vec::new(),
        wire_vertex_indices: Vec::new(),
    }))
}

fn root_edge_vertices_from_ported_seed(
    context: &Context,
    shape: &Shape,
    endpoints: EdgeEndpoints,
) -> Result<(Vec<Shape>, Vec<[f64; 3]>, Option<usize>, Option<usize>), Error> {
    let start_shape = context.root_edge_vertex_shape_occt(shape, 0)?;
    let end_shape = context.root_edge_vertex_shape_occt(shape, 1)?;
    if context.shape_is_same_occt(&start_shape, &end_shape)? {
        return Ok((vec![start_shape], vec![endpoints.start], Some(0), Some(0)));
    }

    Ok((
        vec![start_shape, end_shape],
        vec![endpoints.start, endpoints.end],
        Some(0),
        Some(1),
    ))
}

fn root_wire_topology_inventory_required(context: &Context, shape: &Shape) -> Result<bool, Error> {
    let summary = context.describe_shape_occt(shape)?;
    Ok(summary.root_kind == ShapeKind::Wire && summary.face_count == 0 && summary.edge_count > 0)
}

fn load_root_wire_topology_snapshot(
    context: &Context,
    shape: &Shape,
) -> Result<Option<TopologySnapshotRootFields>, Error> {
    let wire_shape = context.duplicate_shape_occt(shape)?;
    let wire_edge_occurrence_shapes = context.wire_edge_occurrences_occt(&wire_shape)?;
    if wire_edge_occurrence_shapes.len() != wire_shape.edge_count() {
        return Ok(None);
    }

    let mut vertex_shapes = Vec::new();
    let mut vertex_positions = Vec::new();
    let mut edge_shapes = Vec::new();
    let mut root_edges = Vec::new();

    if !append_root_wire_inventory_from_ported_occurrences(
        context,
        &wire_edge_occurrence_shapes,
        &mut vertex_shapes,
        &mut vertex_positions,
        &mut edge_shapes,
        &mut root_edges,
    )? {
        return Ok(None);
    }

    let edges = root_edges
        .iter()
        .map(|edge| crate::TopologyEdge {
            start_vertex: edge.start_vertex,
            end_vertex: edge.end_vertex,
            length: edge.length,
        })
        .collect::<Vec<_>>();

    let prepared_wire_shape = PreparedRootWireShape {
        wire_shape,
        wire_edge_occurrence_shapes,
    };
    let Some(root_wire) = root_wire_topology(
        context,
        &prepared_wire_shape,
        &vertex_positions,
        &edge_shapes,
        &root_edges,
    )?
    else {
        return Ok(None);
    };
    let root_wires = vec![root_wire];
    let (wires, wire_edge_indices, wire_edge_orientations, wire_vertices, wire_vertex_indices) =
        pack_wire_topology(&root_wires);
    let wire_shapes = vec![prepared_wire_shape.wire_shape];

    Ok(Some(TopologySnapshotRootFields {
        vertex_shapes,
        vertex_positions,
        edge_shapes,
        solid_shapes: Vec::new(),
        wire_shapes,
        prepared_shell_shapes: Vec::new(),
        face_shapes: Vec::new(),
        prepared_face_shapes: Vec::new(),
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

fn root_face_topology_inventory_required(context: &Context, shape: &Shape) -> Result<bool, Error> {
    let summary = context.describe_shape_occt(shape)?;
    Ok(summary.root_kind == ShapeKind::Face && summary.face_count == 1 && summary.shell_count == 0)
}

fn load_root_face_topology_snapshot(
    context: &Context,
    shape: &Shape,
) -> Result<Option<TopologySnapshotRootFields>, Error> {
    let face_shapes = attach_offset_result_face_metadata(
        context,
        shape,
        vec![context.duplicate_shape_occt(shape)?],
    )?;
    if face_shapes.len() != 1 {
        return Ok(None);
    }

    let vertex_shapes = match context.root_face_vertex_shapes_occt(&face_shapes[0]) {
        Ok(vertex_shapes) => vertex_shapes,
        Err(_) => return Ok(None),
    };
    let vertex_positions = vertex_shapes
        .iter()
        .map(|vertex_shape| context.root_vertex_point_seed_occt(vertex_shape))
        .collect::<Result<Vec<_>, Error>>()?;
    let edge_shapes = match context.root_face_edge_shapes_occt(&face_shapes[0]) {
        Ok(edge_shapes) => edge_shapes,
        Err(_) => return Ok(None),
    };
    let root_edges = edge_shapes
        .iter()
        .map(|edge_shape| root_edge_topology(context, edge_shape, &vertex_positions))
        .collect::<Result<Vec<_>, Error>>()?;
    let prepared_face_wire_shapes = match prepare_root_face_wire_shapes(context, &face_shapes[0])? {
        Some(prepared) => prepared,
        None => return Ok(None),
    };
    let wire_shapes = duplicate_prepared_wire_shapes(context, &prepared_face_wire_shapes)?;

    let edges = root_edges
        .iter()
        .map(|edge| crate::TopologyEdge {
            start_vertex: edge.start_vertex,
            end_vertex: edge.end_vertex,
            length: edge.length,
        })
        .collect::<Vec<_>>();

    let mut root_wires = Vec::with_capacity(prepared_face_wire_shapes.len());
    for prepared_wire_shape in &prepared_face_wire_shapes {
        let Some(root_wire) = root_wire_topology(
            context,
            prepared_wire_shape,
            &vertex_positions,
            &edge_shapes,
            &root_edges,
        )?
        else {
            return Ok(None);
        };
        root_wires.push(root_wire);
    }
    let (wires, wire_edge_indices, wire_edge_orientations, wire_vertices, wire_vertex_indices) =
        pack_wire_topology(&root_wires);
    let prepared_face_shapes = vec![PreparedFaceShape {
        face_index: 0,
        face_wire_shapes: prepared_face_wire_shapes,
    }];

    Ok(Some(TopologySnapshotRootFields {
        vertex_shapes,
        vertex_positions,
        edge_shapes,
        solid_shapes: Vec::new(),
        wire_shapes,
        prepared_shell_shapes: Vec::new(),
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

fn root_shell_topology_inventory_required(context: &Context, shape: &Shape) -> Result<bool, Error> {
    let summary = context.describe_shape_occt(shape)?;
    Ok(summary.root_kind == ShapeKind::Shell
        && summary.shell_count == 1
        && summary.solid_count == 0
        && summary.compsolid_count == 0
        && summary.compound_count == 0
        && summary.face_count > 0)
}

fn load_root_shell_topology_snapshot(
    context: &Context,
    shape: &Shape,
) -> Result<Option<TopologySnapshotRootFields>, Error> {
    let shell_shape = context.duplicate_shape_occt(shape)?;
    let shell_shape = match shape.multi_face_offset_result_metadata() {
        Some(metadata) => shell_shape.with_multi_face_offset_result_metadata(metadata.to_vec()),
        None => shell_shape,
    };
    let face_shapes = match context.root_shell_face_shapes_occt(&shell_shape) {
        Ok(face_shapes) => attach_offset_result_face_metadata(context, shape, face_shapes)?,
        Err(_) => return Ok(None),
    };
    if face_shapes.is_empty() {
        return Ok(None);
    }

    let vertex_shapes = match context.root_shell_vertex_shapes_occt(&shell_shape) {
        Ok(vertex_shapes) => vertex_shapes,
        Err(_) => return Ok(None),
    };
    let vertex_positions = vertex_shapes
        .iter()
        .map(|vertex_shape| context.root_vertex_point_seed_occt(vertex_shape))
        .collect::<Result<Vec<_>, Error>>()?;
    let edge_shapes = match context.root_shell_edge_shapes_occt(&shell_shape) {
        Ok(edge_shapes) => edge_shapes,
        Err(_) => return Ok(None),
    };
    let root_edges = edge_shapes
        .iter()
        .map(|edge_shape| root_edge_topology(context, edge_shape, &vertex_positions))
        .collect::<Result<Vec<_>, Error>>()?;
    let root_shell_wire_shapes = match context.root_shell_wire_shapes_occt(&shell_shape) {
        Ok(wire_shapes) => wire_shapes,
        Err(_) => return Ok(None),
    };
    let mut prepared_wire_shapes = Vec::with_capacity(root_shell_wire_shapes.len());
    for wire_shape in root_shell_wire_shapes {
        let Some(prepared_wire_shape) = prepare_root_wire_shape(context, wire_shape)? else {
            return Ok(None);
        };
        prepared_wire_shapes.push(prepared_wire_shape);
    }
    let wire_shapes = duplicate_prepared_wire_shapes(context, &prepared_wire_shapes)?;

    let edges = root_edges
        .iter()
        .map(|edge| crate::TopologyEdge {
            start_vertex: edge.start_vertex,
            end_vertex: edge.end_vertex,
            length: edge.length,
        })
        .collect::<Vec<_>>();

    let mut root_wires = Vec::with_capacity(prepared_wire_shapes.len());
    for prepared_wire_shape in &prepared_wire_shapes {
        let Some(root_wire) = root_wire_topology(
            context,
            prepared_wire_shape,
            &vertex_positions,
            &edge_shapes,
            &root_edges,
        )?
        else {
            return Ok(None);
        };
        root_wires.push(root_wire);
    }
    let (wires, wire_edge_indices, wire_edge_orientations, wire_vertices, wire_vertex_indices) =
        pack_wire_topology(&root_wires);

    let mut prepared_face_shapes = Vec::with_capacity(face_shapes.len());
    for (face_index, face_shape) in face_shapes.iter().enumerate() {
        let Some(face_wire_shapes) = prepare_root_face_wire_shapes(context, face_shape)? else {
            return Ok(None);
        };
        prepared_face_shapes.push(PreparedFaceShape {
            face_index,
            face_wire_shapes,
        });
    }

    let prepared_shell_shapes = vec![PreparedShellShape {
        shell_vertex_shapes: match context.root_shell_vertex_shapes_occt(&shell_shape) {
            Ok(vertex_shapes) => vertex_shapes,
            Err(_) => return Ok(None),
        },
        shell_edge_shapes: match context.root_shell_edge_shapes_occt(&shell_shape) {
            Ok(edge_shapes) => edge_shapes,
            Err(_) => return Ok(None),
        },
        shell_face_shapes: match context.root_shell_face_shapes_occt(&shell_shape) {
            Ok(face_shapes) => attach_offset_result_face_metadata(context, shape, face_shapes)?,
            Err(_) => return Ok(None),
        },
        shell_shape,
    }];

    Ok(Some(TopologySnapshotRootFields {
        vertex_shapes,
        vertex_positions,
        edge_shapes,
        solid_shapes: Vec::new(),
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

fn root_solid_topology_inventory_required(context: &Context, shape: &Shape) -> Result<bool, Error> {
    let summary = context.describe_shape_occt(shape)?;
    Ok(summary.root_kind == ShapeKind::Solid
        && summary.solid_count == 1
        && summary.compsolid_count == 0
        && summary.compound_count == 0
        && summary.shell_count > 0
        && summary.face_count > 0)
}

fn load_root_solid_topology_snapshot(
    context: &Context,
    shape: &Shape,
) -> Result<Option<TopologySnapshotRootFields>, Error> {
    let solid_shape = context.duplicate_shape_occt(shape)?;
    let solid_shape = match shape.multi_face_offset_result_metadata() {
        Some(metadata) => solid_shape.with_multi_face_offset_result_metadata(metadata.to_vec()),
        None => solid_shape,
    };
    let face_shapes = match context.root_solid_face_shapes_occt(&solid_shape) {
        Ok(face_shapes) => attach_offset_result_face_metadata(context, shape, face_shapes)?,
        Err(_) => return Ok(None),
    };
    if face_shapes.is_empty() {
        return Ok(None);
    }

    let vertex_shapes = match context.root_solid_vertex_shapes_occt(&solid_shape) {
        Ok(vertex_shapes) => vertex_shapes,
        Err(_) => return Ok(None),
    };
    let vertex_positions = vertex_shapes
        .iter()
        .map(|vertex_shape| context.root_vertex_point_seed_occt(vertex_shape))
        .collect::<Result<Vec<_>, Error>>()?;
    let edge_shapes = match context.root_solid_edge_shapes_occt(&solid_shape) {
        Ok(edge_shapes) => edge_shapes,
        Err(_) => return Ok(None),
    };
    let root_edges = edge_shapes
        .iter()
        .map(|edge_shape| root_edge_topology(context, edge_shape, &vertex_positions))
        .collect::<Result<Vec<_>, Error>>()?;
    let root_solid_wire_shapes = match context.root_solid_wire_shapes_occt(&solid_shape) {
        Ok(wire_shapes) => wire_shapes,
        Err(_) => return Ok(None),
    };
    let mut prepared_wire_shapes = Vec::with_capacity(root_solid_wire_shapes.len());
    for wire_shape in root_solid_wire_shapes {
        let Some(prepared_wire_shape) = prepare_root_wire_shape(context, wire_shape)? else {
            return Ok(None);
        };
        prepared_wire_shapes.push(prepared_wire_shape);
    }
    let wire_shapes = duplicate_prepared_wire_shapes(context, &prepared_wire_shapes)?;

    let edges = root_edges
        .iter()
        .map(|edge| crate::TopologyEdge {
            start_vertex: edge.start_vertex,
            end_vertex: edge.end_vertex,
            length: edge.length,
        })
        .collect::<Vec<_>>();

    let mut root_wires = Vec::with_capacity(prepared_wire_shapes.len());
    for prepared_wire_shape in &prepared_wire_shapes {
        let Some(root_wire) = root_wire_topology(
            context,
            prepared_wire_shape,
            &vertex_positions,
            &edge_shapes,
            &root_edges,
        )?
        else {
            return Ok(None);
        };
        root_wires.push(root_wire);
    }
    let (wires, wire_edge_indices, wire_edge_orientations, wire_vertices, wire_vertex_indices) =
        pack_wire_topology(&root_wires);

    let mut prepared_face_shapes = Vec::with_capacity(face_shapes.len());
    for (face_index, face_shape) in face_shapes.iter().enumerate() {
        let Some(face_wire_shapes) = prepare_root_face_wire_shapes(context, face_shape)? else {
            return Ok(None);
        };
        prepared_face_shapes.push(PreparedFaceShape {
            face_index,
            face_wire_shapes,
        });
    }

    let root_solid_shell_shapes = match context.root_solid_shell_shapes_occt(&solid_shape) {
        Ok(shell_shapes) => shell_shapes,
        Err(_) => return Ok(None),
    };
    if root_solid_shell_shapes.is_empty() {
        return Ok(None);
    }
    let multi_face_offset_metadata = shape
        .multi_face_offset_result_metadata()
        .map(|metadata| metadata.to_vec());
    let mut prepared_shell_shapes = Vec::with_capacity(root_solid_shell_shapes.len());
    for shell_shape in root_solid_shell_shapes {
        let shell_shape = match &multi_face_offset_metadata {
            Some(metadata) => shell_shape.with_multi_face_offset_result_metadata(metadata.clone()),
            None => shell_shape,
        };
        let shell_vertex_shapes = match context.root_shell_vertex_shapes_occt(&shell_shape) {
            Ok(vertex_shapes) => vertex_shapes,
            Err(_) => return Ok(None),
        };
        let shell_edge_shapes = match context.root_shell_edge_shapes_occt(&shell_shape) {
            Ok(edge_shapes) => edge_shapes,
            Err(_) => return Ok(None),
        };
        let shell_face_shapes = match context.root_shell_face_shapes_occt(&shell_shape) {
            Ok(face_shapes) => attach_offset_result_face_metadata(context, shape, face_shapes)?,
            Err(_) => return Ok(None),
        };
        prepared_shell_shapes.push(PreparedShellShape {
            shell_shape,
            shell_vertex_shapes,
            shell_edge_shapes,
            shell_face_shapes,
        });
    }

    Ok(Some(TopologySnapshotRootFields {
        vertex_shapes,
        vertex_positions,
        edge_shapes,
        solid_shapes: vec![solid_shape],
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

fn root_assembly_topology_inventory_required(
    context: &Context,
    shape: &Shape,
) -> Result<RootAssemblyTopologyInventory, Error> {
    let summary = context.describe_shape_occt(shape)?;

    match summary.root_kind {
        ShapeKind::Compound => root_compound_topology_inventory_required(context, shape, 0),
        ShapeKind::CompSolid => {
            if summary.compsolid_count != 1
                || summary.compound_count != 0
                || summary.solid_count == 0
                || summary.shell_count == 0
                || summary.face_count == 0
            {
                return Ok(RootAssemblyTopologyInventory::Unsupported);
            }
            root_compsolid_topology_inventory_required(context, shape, 0)
        }
        _ => Ok(RootAssemblyTopologyInventory::NotAssembly),
    }
}

fn root_compound_topology_inventory_required(
    context: &Context,
    shape: &Shape,
    depth: usize,
) -> Result<RootAssemblyTopologyInventory, Error> {
    if depth >= ROOT_ASSEMBLY_MAX_DEPTH {
        return Ok(RootAssemblyTopologyInventory::Unsupported);
    }
    let child_shapes = match context.root_compound_child_shapes_occt(shape) {
        Ok(child_shapes) => child_shapes,
        Err(_) => return Ok(RootAssemblyTopologyInventory::Unsupported),
    };
    if child_shapes.is_empty() {
        return Ok(RootAssemblyTopologyInventory::Unsupported);
    }

    for child_shape in &child_shapes {
        if !root_assembly_child_topology_supported(context, child_shape, depth)? {
            return Ok(RootAssemblyTopologyInventory::Unsupported);
        }
    }

    Ok(RootAssemblyTopologyInventory::Supported(
        RootAssemblyKind::Compound,
    ))
}

fn root_compsolid_topology_inventory_required(
    context: &Context,
    shape: &Shape,
    depth: usize,
) -> Result<RootAssemblyTopologyInventory, Error> {
    if depth >= ROOT_ASSEMBLY_MAX_DEPTH {
        return Ok(RootAssemblyTopologyInventory::Unsupported);
    }
    let child_shapes = match context.root_compsolid_child_shapes_occt(shape) {
        Ok(child_shapes) => child_shapes,
        Err(_) => return Ok(RootAssemblyTopologyInventory::Unsupported),
    };
    if child_shapes.is_empty() {
        return Ok(RootAssemblyTopologyInventory::Unsupported);
    }

    for child_shape in &child_shapes {
        let child_summary = context.describe_shape_occt(child_shape)?;
        if child_summary.root_kind != ShapeKind::Solid
            || child_summary.solid_count != 1
            || child_summary.shell_count == 0
            || child_summary.face_count == 0
        {
            return Ok(RootAssemblyTopologyInventory::Unsupported);
        }
    }

    Ok(RootAssemblyTopologyInventory::Supported(
        RootAssemblyKind::CompSolid,
    ))
}

fn root_assembly_child_topology_supported(
    context: &Context,
    child_shape: &Shape,
    depth: usize,
) -> Result<bool, Error> {
    let child_summary = context.describe_shape_occt(child_shape)?;
    match child_summary.root_kind {
        ShapeKind::Solid => Ok(child_summary.solid_count == 1
            && child_summary.shell_count > 0
            && child_summary.face_count > 0),
        ShapeKind::Shell => Ok(child_summary.solid_count == 0
            && child_summary.shell_count >= 1
            && child_summary.face_count > 0),
        ShapeKind::Face => Ok(root_face_topology_inventory_required(context, child_shape)?
            && load_root_face_topology_snapshot(context, child_shape)?.is_some()),
        ShapeKind::Wire => Ok(root_wire_topology_inventory_required(context, child_shape)?
            && load_root_wire_topology_snapshot(context, child_shape)?.is_some()),
        ShapeKind::Edge => Ok(load_root_edge_topology_snapshot(context, child_shape)?.is_some()),
        ShapeKind::Vertex => Ok(
            root_vertex_topology_inventory_required(context, child_shape)?
                && load_root_vertex_topology_snapshot(context, child_shape)?.is_some(),
        ),
        ShapeKind::Compound => Ok(matches!(
            root_compound_topology_inventory_required(context, child_shape, depth + 1)?,
            RootAssemblyTopologyInventory::Supported(RootAssemblyKind::Compound)
        )),
        ShapeKind::CompSolid => Ok(matches!(
            root_compsolid_topology_inventory_required(context, child_shape, depth + 1)?,
            RootAssemblyTopologyInventory::Supported(RootAssemblyKind::CompSolid)
        )),
        _ => Ok(false),
    }
}

fn load_root_assembly_topology_snapshot(
    context: &Context,
    shape: &Shape,
    assembly_kind: RootAssemblyKind,
) -> Result<Option<TopologySnapshotRootFields>, Error> {
    let root_shape = context.duplicate_shape_occt(shape)?;
    let solid_shapes =
        match root_assembly_subshapes_occt(context, &root_shape, assembly_kind, ShapeKind::Solid) {
            Ok(solid_shapes) => solid_shapes,
            Err(_) => return Ok(None),
        };
    let face_shapes =
        match root_assembly_subshapes_occt(context, &root_shape, assembly_kind, ShapeKind::Face) {
            Ok(face_shapes) => attach_offset_result_face_metadata(context, shape, face_shapes)?,
            Err(_) => return Ok(None),
        };

    let vertex_shapes = match root_assembly_subshapes_occt(
        context,
        &root_shape,
        assembly_kind,
        ShapeKind::Vertex,
    ) {
        Ok(vertex_shapes) => vertex_shapes,
        Err(_) => return Ok(None),
    };
    let vertex_positions = vertex_shapes
        .iter()
        .map(|vertex_shape| context.root_vertex_point_seed_occt(vertex_shape))
        .collect::<Result<Vec<_>, Error>>()?;
    let edge_shapes =
        match root_assembly_subshapes_occt(context, &root_shape, assembly_kind, ShapeKind::Edge) {
            Ok(edge_shapes) => edge_shapes,
            Err(_) => return Ok(None),
        };
    let root_edges = edge_shapes
        .iter()
        .map(|edge_shape| root_edge_topology(context, edge_shape, &vertex_positions))
        .collect::<Result<Vec<_>, Error>>()?;
    let root_assembly_wire_shapes =
        match root_assembly_subshapes_occt(context, &root_shape, assembly_kind, ShapeKind::Wire) {
            Ok(wire_shapes) => wire_shapes,
            Err(_) => return Ok(None),
        };
    let mut prepared_wire_shapes = Vec::with_capacity(root_assembly_wire_shapes.len());
    for wire_shape in root_assembly_wire_shapes {
        let Some(prepared_wire_shape) = prepare_root_wire_shape(context, wire_shape)? else {
            return Ok(None);
        };
        prepared_wire_shapes.push(prepared_wire_shape);
    }
    let wire_shapes = duplicate_prepared_wire_shapes(context, &prepared_wire_shapes)?;

    let edges = root_edges
        .iter()
        .map(|edge| crate::TopologyEdge {
            start_vertex: edge.start_vertex,
            end_vertex: edge.end_vertex,
            length: edge.length,
        })
        .collect::<Vec<_>>();

    let mut root_wires = Vec::with_capacity(prepared_wire_shapes.len());
    for prepared_wire_shape in &prepared_wire_shapes {
        let Some(root_wire) = root_wire_topology(
            context,
            prepared_wire_shape,
            &vertex_positions,
            &edge_shapes,
            &root_edges,
        )?
        else {
            return Ok(None);
        };
        root_wires.push(root_wire);
    }
    let (wires, wire_edge_indices, wire_edge_orientations, wire_vertices, wire_vertex_indices) =
        pack_wire_topology(&root_wires);

    let mut prepared_face_shapes = Vec::with_capacity(face_shapes.len());
    for (face_index, face_shape) in face_shapes.iter().enumerate() {
        let Some(face_wire_shapes) = prepare_root_face_wire_shapes(context, face_shape)? else {
            return Ok(None);
        };
        prepared_face_shapes.push(PreparedFaceShape {
            face_index,
            face_wire_shapes,
        });
    }

    let root_assembly_shell_shapes =
        match root_assembly_subshapes_occt(context, &root_shape, assembly_kind, ShapeKind::Shell) {
            Ok(shell_shapes) => shell_shapes,
            Err(_) => return Ok(None),
        };
    let multi_face_offset_metadata = shape
        .multi_face_offset_result_metadata()
        .map(|metadata| metadata.to_vec());
    let mut prepared_shell_shapes = Vec::with_capacity(root_assembly_shell_shapes.len());
    for shell_shape in root_assembly_shell_shapes {
        let shell_shape = match &multi_face_offset_metadata {
            Some(metadata) => shell_shape.with_multi_face_offset_result_metadata(metadata.clone()),
            None => shell_shape,
        };
        let shell_vertex_shapes = match context.root_shell_vertex_shapes_occt(&shell_shape) {
            Ok(vertex_shapes) => vertex_shapes,
            Err(_) => return Ok(None),
        };
        let shell_edge_shapes = match context.root_shell_edge_shapes_occt(&shell_shape) {
            Ok(edge_shapes) => edge_shapes,
            Err(_) => return Ok(None),
        };
        let shell_face_shapes = match context.root_shell_face_shapes_occt(&shell_shape) {
            Ok(face_shapes) => attach_offset_result_face_metadata(context, shape, face_shapes)?,
            Err(_) => return Ok(None),
        };
        prepared_shell_shapes.push(PreparedShellShape {
            shell_shape,
            shell_vertex_shapes,
            shell_edge_shapes,
            shell_face_shapes,
        });
    }

    Ok(Some(TopologySnapshotRootFields {
        vertex_shapes,
        vertex_positions,
        edge_shapes,
        solid_shapes,
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

fn root_assembly_subshapes_occt(
    context: &Context,
    root_shape: &Shape,
    assembly_kind: RootAssemblyKind,
    kind: ShapeKind,
) -> Result<Vec<Shape>, Error> {
    match assembly_kind {
        RootAssemblyKind::Compound => context.root_compound_subshapes_occt(root_shape, kind),
        RootAssemblyKind::CompSolid => context.root_compsolid_subshapes_occt(root_shape, kind),
    }
}

fn prepare_root_face_wire_shapes(
    context: &Context,
    face_shape: &Shape,
) -> Result<Option<Vec<PreparedRootWireShape>>, Error> {
    let root_face_wire_shapes = match context.root_face_wire_shapes_occt(face_shape) {
        Ok(wire_shapes) => wire_shapes,
        Err(_) => return Ok(None),
    };
    let mut prepared_wire_shapes = Vec::with_capacity(root_face_wire_shapes.len());
    for face_wire_shape in root_face_wire_shapes {
        let Some(prepared_wire_shape) = prepare_root_wire_shape(context, face_wire_shape)? else {
            return Ok(None);
        };
        prepared_wire_shapes.push(prepared_wire_shape);
    }
    Ok(Some(prepared_wire_shapes))
}

fn prepare_root_wire_shape(
    context: &Context,
    wire_shape: Shape,
) -> Result<Option<PreparedRootWireShape>, Error> {
    let wire_edge_occurrence_shapes = context.wire_edge_occurrences_occt(&wire_shape)?;
    if wire_edge_occurrence_shapes.len() != wire_shape.edge_count() {
        return Ok(None);
    }

    Ok(Some(PreparedRootWireShape {
        wire_shape,
        wire_edge_occurrence_shapes,
    }))
}

fn duplicate_prepared_wire_shapes(
    context: &Context,
    prepared_wire_shapes: &[PreparedRootWireShape],
) -> Result<Vec<Shape>, Error> {
    prepared_wire_shapes
        .iter()
        .map(|prepared_wire_shape| context.duplicate_shape_occt(&prepared_wire_shape.wire_shape))
        .collect()
}

fn append_root_wire_inventory_from_ported_occurrences(
    context: &Context,
    wire_edge_occurrence_shapes: &[Shape],
    vertex_shapes: &mut Vec<Shape>,
    vertex_positions: &mut Vec<[f64; 3]>,
    edge_shapes: &mut Vec<Shape>,
    root_edges: &mut Vec<RootEdgeTopology>,
) -> Result<bool, Error> {
    for edge_shape in wire_edge_occurrence_shapes {
        let query = topology_edge_query(context, edge_shape)?;
        let Some((start_vertex, end_vertex)) = root_wire_edge_vertex_indices_from_ported_seed(
            context,
            edge_shape,
            query.endpoints.start,
            query.endpoints.end,
            vertex_shapes,
            vertex_positions,
        )?
        else {
            return Ok(false);
        };
        let length = topology_edge_length(context, edge_shape, query.geometry)?;
        if root_wire_existing_edge_index(context, edge_shape, edge_shapes)?.is_none() {
            edge_shapes.push(context.duplicate_shape_occt(edge_shape)?);
            root_edges.push(RootEdgeTopology {
                geometry: query.geometry,
                start_vertex: Some(start_vertex),
                end_vertex: Some(end_vertex),
                length,
            });
        }
    }

    Ok(true)
}

fn root_wire_edge_vertex_indices_from_ported_seed(
    context: &Context,
    edge_shape: &Shape,
    start_endpoint: [f64; 3],
    end_endpoint: [f64; 3],
    vertex_shapes: &mut Vec<Shape>,
    vertex_positions: &mut Vec<[f64; 3]>,
) -> Result<Option<(usize, usize)>, Error> {
    for vertex_index in 0..2 {
        let vertex_shape = match context.edge_vertex_inventory_shape_occt(edge_shape, vertex_index)
        {
            Ok(vertex_shape) => vertex_shape,
            Err(_) if vertex_index > 0 => break,
            Err(_) => return Ok(None),
        };
        if root_wire_vertex_index_from_inventory_shape(
            context,
            vertex_shape,
            vertex_shapes,
            vertex_positions,
        )?
        .is_none()
        {
            return Ok(None);
        }
    }

    let first_endpoint_shape = match context.root_edge_vertex_shape_occt(edge_shape, 0) {
        Ok(vertex_shape) => vertex_shape,
        Err(_) => return Ok(None),
    };
    let last_endpoint_shape = match context.root_edge_vertex_shape_occt(edge_shape, 1) {
        Ok(vertex_shape) => vertex_shape,
        Err(_) => return Ok(None),
    };
    let Some(first_vertex) =
        root_wire_existing_vertex_index(context, &first_endpoint_shape, vertex_shapes)?
    else {
        return Ok(None);
    };
    let Some(last_vertex) =
        root_wire_existing_vertex_index(context, &last_endpoint_shape, vertex_shapes)?
    else {
        return Ok(None);
    };
    Ok(root_wire_oriented_vertex_indices(
        start_endpoint,
        end_endpoint,
        first_vertex,
        last_vertex,
        vertex_positions,
    ))
}

fn root_wire_vertex_index_from_inventory_shape(
    context: &Context,
    vertex_shape: Shape,
    vertex_shapes: &mut Vec<Shape>,
    vertex_positions: &mut Vec<[f64; 3]>,
) -> Result<Option<usize>, Error> {
    let vertex_position = match context.root_vertex_point_seed_occt(&vertex_shape) {
        Ok(position) => position,
        Err(_) => return Ok(None),
    };
    for (index, existing_shape) in vertex_shapes.iter().enumerate() {
        if context.shape_is_same_occt(&vertex_shape, existing_shape)? {
            if root_wire_endpoint_matches_position(vertex_positions[index], vertex_position) {
                return Ok(Some(index));
            }
            return Ok(None);
        }
    }
    if match_vertex_index(vertex_positions, vertex_position).is_some() {
        return Ok(None);
    }

    vertex_shapes.push(vertex_shape);
    vertex_positions.push(vertex_position);
    Ok(Some(vertex_positions.len() - 1))
}

fn root_wire_existing_vertex_index(
    context: &Context,
    vertex_shape: &Shape,
    vertex_shapes: &[Shape],
) -> Result<Option<usize>, Error> {
    for (index, existing_shape) in vertex_shapes.iter().enumerate() {
        if context.shape_is_same_occt(vertex_shape, existing_shape)? {
            return Ok(Some(index));
        }
    }
    Ok(None)
}

fn root_wire_oriented_vertex_indices(
    start_endpoint: [f64; 3],
    end_endpoint: [f64; 3],
    first_vertex: usize,
    last_vertex: usize,
    vertex_positions: &[[f64; 3]],
) -> Option<(usize, usize)> {
    let first_position = *vertex_positions.get(first_vertex)?;
    let last_position = *vertex_positions.get(last_vertex)?;
    if root_wire_endpoint_matches_position(first_position, start_endpoint)
        && root_wire_endpoint_matches_position(last_position, end_endpoint)
    {
        return Some((first_vertex, last_vertex));
    }
    if root_wire_endpoint_matches_position(first_position, end_endpoint)
        && root_wire_endpoint_matches_position(last_position, start_endpoint)
    {
        return Some((last_vertex, first_vertex));
    }
    None
}

fn root_wire_existing_edge_index(
    context: &Context,
    edge_shape: &Shape,
    edge_shapes: &[Shape],
) -> Result<Option<usize>, Error> {
    for (index, existing_shape) in edge_shapes.iter().enumerate() {
        if context.shape_is_same_occt(edge_shape, existing_shape)? {
            return Ok(Some(index));
        }
    }
    Ok(None)
}

fn root_wire_endpoint_matches_position(lhs: [f64; 3], rhs: [f64; 3]) -> bool {
    (lhs[0] - rhs[0]).abs() <= 1.0e-7
        && (lhs[1] - rhs[1]).abs() <= 1.0e-7
        && (lhs[2] - rhs[2]).abs() <= 1.0e-7
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

            let mut best_match = None;
            let mut tied_best_match = false;
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
                    if let Some(score) =
                        offset_metadata_match_score(context, &face_shape, signed_candidate)?
                    {
                        match best_match {
                            None => {
                                best_match = Some((score, signed_candidate));
                                tied_best_match = false;
                            }
                            Some((best_score, _))
                                if score + OFFSET_METADATA_MATCH_SCORE_TOLERANCE < best_score =>
                            {
                                best_match = Some((score, signed_candidate));
                                tied_best_match = false;
                            }
                            Some((best_score, _))
                                if (score - best_score).abs()
                                    <= OFFSET_METADATA_MATCH_SCORE_TOLERANCE =>
                            {
                                tied_best_match = true;
                            }
                            Some(_) => {}
                        }
                    }
                }
            }

            if let Some((_, matched)) = best_match.filter(|_| !tied_best_match) {
                Ok(face_shape.with_offset_surface_face_metadata(matched))
            } else {
                Ok(face_shape)
            }
        })
        .collect()
}

fn offset_metadata_match_score(
    context: &Context,
    face_shape: &Shape,
    metadata: OffsetSurfaceFaceMetadata,
) -> Result<Option<f64>, Error> {
    match context.ported_offset_surface_from_metadata(face_shape, metadata) {
        Ok(Some(_)) => {}
        Ok(None) | Err(_) => return Ok(None),
    }

    let orientation = context.shape_orientation(face_shape)?;
    let generated_normal_sign = if matches!(orientation, Orientation::Reversed) {
        -1.0
    } else {
        1.0
    };
    let source_basis = PortedOffsetSurface {
        payload: OffsetSurfacePayload {
            offset_value: 0.0,
            basis_surface_kind: metadata.basis_geometry.kind,
        },
        basis_geometry: metadata.basis_geometry,
        basis: metadata.basis,
    };

    let mut score = 0.0;
    for uv_t in OFFSET_METADATA_MATCH_UV_SAMPLES {
        let generated_sample = context.face_sample_normalized_occt(face_shape, uv_t)?;
        let generated_basis_normal = scale3(generated_sample.normal, generated_normal_sign);
        let generated_basis_position = subtract3(
            generated_sample.position,
            scale3(generated_basis_normal, metadata.offset_value),
        );
        let source_sample = source_basis.sample_normalized(uv_t);
        let delta = subtract3(generated_basis_position, source_sample.position);
        score += dot3(delta, delta);
    }

    Ok(Some(score))
}

pub(super) fn load_ported_topology(
    context: &Context,
    shape: &Shape,
) -> Result<Option<LoadedPortedTopology>, Error> {
    let Some(TopologySnapshotRootFields {
        vertex_shapes,
        vertex_positions,
        edge_shapes,
        solid_shapes,
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
        solid_shapes,
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
