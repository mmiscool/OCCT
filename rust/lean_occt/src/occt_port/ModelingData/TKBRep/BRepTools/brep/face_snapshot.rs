use super::edge_topology::{oriented_edge_geometry, RootEdgeTopology};
use super::swept_face::append_root_edge_sample_points;
use super::wire_topology::{root_wire_topology, RootWireTopology};
use super::*;

pub(super) struct TopologySnapshotFaceFields {
    pub(super) edge_faces: Vec<crate::TopologyRange>,
    pub(super) edge_face_indices: Vec<usize>,
    pub(super) faces: Vec<crate::TopologyRange>,
    pub(super) face_wire_indices: Vec<usize>,
    pub(super) face_wire_orientations: Vec<Orientation>,
    pub(super) face_wire_roles: Vec<LoopRole>,
}

fn multi_wire_face_is_planar(
    context: &Context,
    face_shape: &Shape,
    face_wire_count: usize,
) -> Result<bool, Error> {
    if face_wire_count <= 1 {
        return Ok(true);
    }

    let geometry = match context.face_geometry(face_shape) {
        Ok(geometry) => geometry,
        Err(_) => context.face_geometry_occt(face_shape)?,
    };
    Ok(geometry.kind == crate::SurfaceKind::Plane)
}

fn validate_ported_face_snapshot(context: &Context, face_shapes: &[Shape]) -> Result<bool, Error> {
    for face_shape in face_shapes {
        let face_wire_shapes = context.subshapes_occt(face_shape, ShapeKind::Wire)?;
        if !multi_wire_face_is_planar(context, face_shape, face_wire_shapes.len())? {
            return Ok(false);
        }
    }

    Ok(true)
}

fn classify_face_wire_roles(
    face_wire_count: usize,
    face_wire_areas: &[f64],
) -> Option<Vec<LoopRole>> {
    match face_wire_count {
        1 => Some(vec![LoopRole::Outer]),
        _ => {
            let (outer_offset, outer_area) = face_wire_areas
                .iter()
                .copied()
                .enumerate()
                .max_by(|(_, lhs), (_, rhs)| lhs.total_cmp(rhs))?;
            if outer_area <= 1.0e-9 {
                return None;
            }

            Some(
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
                    .collect(),
            )
        }
    }
}

fn load_planar_multi_wire_face(
    context: &Context,
    face_shape: &Shape,
    face_wire_count: usize,
) -> Result<Option<(PlanePayload, FaceGeometry)>, Error> {
    if face_wire_count <= 1 {
        return Ok(None);
    }
    if !multi_wire_face_is_planar(context, face_shape, face_wire_count)? {
        return Ok(None);
    }

    Ok(Some((
        context.face_plane_payload_occt(face_shape)?,
        context.face_geometry_occt(face_shape)?,
    )))
}

struct MatchedFaceWires {
    face_wire_indices: Vec<usize>,
    face_wire_orientations: Vec<Orientation>,
    face_wire_areas: Vec<f64>,
    used_edges: BTreeSet<usize>,
}

fn append_ported_face_topology(
    context: &Context,
    face_index: usize,
    face_shape: &Shape,
    root_wires: &[RootWireTopology],
    root_edges: &[RootEdgeTopology],
    edge_shapes: &[Shape],
    vertex_positions: &[[f64; 3]],
    edge_face_lists: &mut [Vec<usize>],
    faces: &mut Vec<crate::TopologyRange>,
    face_wire_indices: &mut Vec<usize>,
    face_wire_orientations: &mut Vec<Orientation>,
    face_wire_roles: &mut Vec<LoopRole>,
) -> Result<Option<()>, Error> {
    let face_wire_shapes = context.subshapes_occt(face_shape, ShapeKind::Wire)?;
    if root_wires.is_empty() || face_wire_shapes.is_empty() {
        return Ok(None);
    }

    let face_wire_offset = face_wire_indices.len();
    let planar_face = if face_wire_shapes.len() > 1 {
        let Some(planar_face) =
            load_planar_multi_wire_face(context, face_shape, face_wire_shapes.len())?
        else {
            return Ok(None);
        };
        Some(planar_face)
    } else {
        None
    };

    let Some(matched_face_wires) = collect_face_wire_matches(
        context,
        &face_wire_shapes,
        root_wires,
        root_edges,
        edge_shapes,
        vertex_positions,
        planar_face,
    )?
    else {
        return Ok(None);
    };
    let face_wire_count = matched_face_wires.face_wire_indices.len();

    let Some(wire_roles) =
        classify_face_wire_roles(face_wire_count, &matched_face_wires.face_wire_areas)
    else {
        return Ok(None);
    };

    face_wire_indices.extend(matched_face_wires.face_wire_indices);
    face_wire_orientations.extend(matched_face_wires.face_wire_orientations);
    faces.push(crate::TopologyRange {
        offset: face_wire_offset,
        count: face_wire_count,
    });
    face_wire_roles.extend(wire_roles);

    for edge_index in matched_face_wires.used_edges {
        let Some(edge_faces) = edge_face_lists.get_mut(edge_index) else {
            return Ok(None);
        };
        edge_faces.push(face_index);
    }

    Ok(Some(()))
}

fn pack_ported_face_snapshot(
    context: &Context,
    face_shapes: &[Shape],
    root_wires: &[RootWireTopology],
    root_edges: &[RootEdgeTopology],
    edge_shapes: &[Shape],
    vertex_positions: &[[f64; 3]],
    edge_count: usize,
) -> Result<Option<TopologySnapshotFaceFields>, Error> {
    let mut edge_face_lists = vec![Vec::new(); edge_count];
    let mut faces = Vec::with_capacity(face_shapes.len());
    let mut face_wire_indices = Vec::new();
    let mut face_wire_orientations = Vec::new();
    let mut face_wire_roles = Vec::new();

    for (face_index, face_shape) in face_shapes.iter().enumerate() {
        let Some(()) = append_ported_face_topology(
            context,
            face_index,
            face_shape,
            root_wires,
            root_edges,
            edge_shapes,
            vertex_positions,
            &mut edge_face_lists,
            &mut faces,
            &mut face_wire_indices,
            &mut face_wire_orientations,
            &mut face_wire_roles,
        )?
        else {
            return Ok(None);
        };
    }

    let mut edge_faces = Vec::with_capacity(edge_count);
    let mut edge_face_indices = Vec::new();
    for face_indices in edge_face_lists {
        edge_faces.push(crate::TopologyRange {
            offset: edge_face_indices.len(),
            count: face_indices.len(),
        });
        edge_face_indices.extend(face_indices);
    }

    Ok(Some(TopologySnapshotFaceFields {
        edge_faces,
        edge_face_indices,
        faces,
        face_wire_indices,
        face_wire_orientations,
        face_wire_roles,
    }))
}

pub(super) fn load_ported_face_snapshot(
    context: &Context,
    shape: &Shape,
    root_wires: &[RootWireTopology],
    root_edges: &[RootEdgeTopology],
    edge_shapes: &[Shape],
    vertex_positions: &[[f64; 3]],
    edge_count: usize,
) -> Result<Option<TopologySnapshotFaceFields>, Error> {
    let face_shapes = context.subshapes_occt(shape, ShapeKind::Face)?;
    if !validate_ported_face_snapshot(context, &face_shapes)? {
        return Ok(None);
    }

    pack_ported_face_snapshot(
        context,
        &face_shapes,
        root_wires,
        root_edges,
        edge_shapes,
        vertex_positions,
        edge_count,
    )
}

fn collect_face_wire_matches(
    context: &Context,
    face_wire_shapes: &[Shape],
    root_wires: &[RootWireTopology],
    root_edges: &[RootEdgeTopology],
    edge_shapes: &[Shape],
    vertex_positions: &[[f64; 3]],
    planar_face: Option<(PlanePayload, FaceGeometry)>,
) -> Result<Option<MatchedFaceWires>, Error> {
    let mut used_root_wire_indices = BTreeSet::new();
    let mut used_edges = BTreeSet::new();
    let mut face_wire_indices = Vec::with_capacity(face_wire_shapes.len());
    let mut face_wire_orientations = Vec::with_capacity(face_wire_shapes.len());
    let mut face_wire_areas = Vec::new();

    for face_wire_shape in face_wire_shapes {
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

    Ok(Some(MatchedFaceWires {
        face_wire_indices,
        face_wire_orientations,
        face_wire_areas,
        used_edges,
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
