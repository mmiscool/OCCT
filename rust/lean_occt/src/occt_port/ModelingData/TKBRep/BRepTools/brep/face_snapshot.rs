use super::edge_topology::{oriented_edge_geometry, RootEdgeTopology};
use super::swept_face::append_root_edge_sample_points;
use super::wire_topology::{root_wire_topology, PreparedRootWireShape, RootWireTopology};
use super::*;

pub(super) struct TopologySnapshotFaceFields {
    pub(super) edge_faces: Vec<crate::TopologyRange>,
    pub(super) edge_face_indices: Vec<usize>,
    pub(super) faces: Vec<crate::TopologyRange>,
    pub(super) face_wire_indices: Vec<usize>,
    pub(super) face_wire_orientations: Vec<Orientation>,
    pub(super) face_wire_roles: Vec<LoopRole>,
}

pub(super) struct PreparedFaceShape {
    pub(super) face_index: usize,
    pub(super) face_wire_shapes: Vec<PreparedRootWireShape>,
}

struct PreparedFaceTopology {
    face_wire_indices: Vec<usize>,
    face_wire_orientations: Vec<Orientation>,
    wire_roles: Vec<LoopRole>,
    used_edges: BTreeSet<usize>,
}

struct PreparedFaceTopologyBuilder {
    used_root_wire_indices: BTreeSet<usize>,
    face_wire_indices: Vec<usize>,
    face_wire_orientations: Vec<Orientation>,
    face_wire_areas: Vec<f64>,
    used_edges: BTreeSet<usize>,
}

impl PreparedFaceTopologyBuilder {
    fn build(
        context: &Context,
        prepared_face_shape: &PreparedFaceShape,
        face_shapes: &[Shape],
        root_wires: &[RootWireTopology],
        root_edges: &[RootEdgeTopology],
        edge_shapes: &[Shape],
        vertex_positions: &[[f64; 3]],
    ) -> Result<Option<PreparedFaceTopology>, Error> {
        if root_wires.is_empty() || prepared_face_shape.face_wire_shapes.is_empty() {
            return Ok(None);
        }

        let face_wire_shapes = &prepared_face_shape.face_wire_shapes;
        let face_shape = face_shapes
            .get(prepared_face_shape.face_index)
            .ok_or_else(|| {
                Error::new(format!(
                    "prepared face index {} was outside the root face inventory",
                    prepared_face_shape.face_index
                ))
            })?;
        let planar_face = if face_wire_shapes.len() <= 1 {
            None
        } else {
            let face_geometry = match context.face_geometry(face_shape) {
                Ok(geometry) => geometry,
                Err(_) => context.face_geometry_occt(face_shape)?,
            };
            if face_geometry.kind != crate::SurfaceKind::Plane {
                return Ok(None);
            }
            let plane = match context.face_plane_payload(face_shape) {
                Ok(payload) => payload,
                Err(_) => match context.face_plane_payload_occt(face_shape) {
                    Ok(payload) => payload,
                    Err(_) => return Ok(None),
                },
            };
            Some((plane, face_geometry))
        };
        let mut builder = Self {
            used_root_wire_indices: BTreeSet::new(),
            face_wire_indices: Vec::with_capacity(face_wire_shapes.len()),
            face_wire_orientations: Vec::with_capacity(face_wire_shapes.len()),
            face_wire_areas: Vec::new(),
            used_edges: BTreeSet::new(),
        };

        for face_wire_shape in face_wire_shapes {
            let Some(face_wire_topology) =
                root_wire_topology(context, face_wire_shape, vertex_positions, root_edges)?
            else {
                return Ok(None);
            };
            let Some(root_wire_index) = root_wires
                .iter()
                .enumerate()
                .find(|(index, root_wire)| {
                    !builder.used_root_wire_indices.contains(index)
                        && *root_wire == &face_wire_topology
                })
                .map(|(index, _)| index)
            else {
                return Ok(None);
            };
            let orientation = context.shape_orientation(&face_wire_shape.wire_shape)?;
            let used_edges = face_wire_topology.edge_indices;

            let wire_area = if let Some((plane, face_geometry)) = planar_face {
                let wire = &root_wires[root_wire_index];
                let mut curve_segments = Vec::with_capacity(wire.edge_indices.len());
                let mut sampled_points = Vec::new();

                for (&edge_index, &edge_orientation) in
                    wire.edge_indices.iter().zip(&wire.edge_orientations)
                {
                    let Some(root_edge) = root_edges.get(edge_index) else {
                        return Ok(None);
                    };
                    let Some(edge_shape) = edge_shapes.get(edge_index) else {
                        return Ok(None);
                    };

                    let oriented_geometry =
                        oriented_edge_geometry(root_edge.geometry, edge_orientation);
                    if let Some(curve) = match PortedCurve::from_context_with_ported_payloads(
                        context,
                        edge_shape,
                        root_edge.geometry,
                    ) {
                        Ok(curve) => curve,
                        Err(_) => PortedCurve::from_context_with_geometry(
                            context,
                            edge_shape,
                            root_edge.geometry,
                        )?,
                    } {
                        curve_segments.push((curve, oriented_geometry));
                    }

                    append_root_edge_sample_points(
                        context,
                        edge_shape,
                        root_edge,
                        oriented_geometry,
                        &mut sampled_points,
                    )?;
                }

                let wire_area = if curve_segments.len() == wire.edge_indices.len() {
                    planar_wire_signed_area(plane, &curve_segments).abs()
                } else {
                    let Some(wire_area) = analytic_sampled_wire_signed_area(
                        PortedSurface::Plane(plane),
                        face_geometry,
                        &sampled_points,
                    ) else {
                        return Ok(None);
                    };
                    wire_area.abs()
                };
                Some(wire_area)
            } else {
                None
            };
            builder.used_root_wire_indices.insert(root_wire_index);
            builder.used_edges.extend(used_edges);
            builder.face_wire_indices.push(root_wire_index);
            builder.face_wire_orientations.push(orientation);
            if let Some(wire_area) = wire_area {
                builder.face_wire_areas.push(wire_area);
            }
        }

        let Self {
            face_wire_indices,
            face_wire_orientations,
            face_wire_areas,
            used_edges,
            ..
        } = builder;
        let wire_roles = match face_wire_indices.len() {
            1 => vec![LoopRole::Outer],
            _ => {
                let Some((outer_offset, outer_area)) = face_wire_areas
                    .iter()
                    .copied()
                    .enumerate()
                    .max_by(|(_, lhs), (_, rhs)| lhs.total_cmp(rhs))
                else {
                    return Ok(None);
                };
                if outer_area <= 1.0e-9 {
                    return Ok(None);
                }

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
                    .collect()
            }
        };

        Ok(Some(PreparedFaceTopology {
            face_wire_indices,
            face_wire_orientations,
            wire_roles,
            used_edges,
        }))
    }
}

struct FaceSnapshotAccumulator {
    edge_face_lists: Vec<Vec<usize>>,
    faces: Vec<crate::TopologyRange>,
    face_wire_indices: Vec<usize>,
    face_wire_orientations: Vec<Orientation>,
    face_wire_roles: Vec<LoopRole>,
}

fn pack_ported_face_snapshot(
    context: &Context,
    prepared_face_shapes: &[PreparedFaceShape],
    face_shapes: &[Shape],
    root_wires: &[RootWireTopology],
    root_edges: &[RootEdgeTopology],
    edge_shapes: &[Shape],
    vertex_positions: &[[f64; 3]],
    edge_count: usize,
) -> Result<Option<TopologySnapshotFaceFields>, Error> {
    let mut accumulator = FaceSnapshotAccumulator {
        edge_face_lists: vec![Vec::new(); edge_count],
        faces: Vec::with_capacity(prepared_face_shapes.len()),
        face_wire_indices: Vec::new(),
        face_wire_orientations: Vec::new(),
        face_wire_roles: Vec::new(),
    };

    for (face_index, prepared_face_shape) in prepared_face_shapes.iter().enumerate() {
        let Some(prepared_face_topology) = PreparedFaceTopologyBuilder::build(
            context,
            prepared_face_shape,
            face_shapes,
            root_wires,
            root_edges,
            edge_shapes,
            vertex_positions,
        )?
        else {
            return Ok(None);
        };
        let face_wire_offset = accumulator.face_wire_indices.len();
        let PreparedFaceTopology {
            face_wire_indices,
            face_wire_orientations,
            wire_roles,
            used_edges,
        } = prepared_face_topology;
        let face_wire_count = face_wire_indices.len();
        accumulator.face_wire_indices.extend(face_wire_indices);
        accumulator
            .face_wire_orientations
            .extend(face_wire_orientations);
        accumulator.faces.push(crate::TopologyRange {
            offset: face_wire_offset,
            count: face_wire_count,
        });
        accumulator.face_wire_roles.extend(wire_roles);

        for edge_index in used_edges {
            let Some(edge_faces) = accumulator.edge_face_lists.get_mut(edge_index) else {
                return Ok(None);
            };
            edge_faces.push(face_index);
        }
    }

    let FaceSnapshotAccumulator {
        edge_face_lists,
        faces,
        face_wire_indices,
        face_wire_orientations,
        face_wire_roles,
    } = accumulator;
    let total_edge_face_count = edge_face_lists.iter().map(Vec::len).sum();
    let mut edge_faces = Vec::with_capacity(edge_face_lists.len());
    let mut edge_face_indices = Vec::with_capacity(total_edge_face_count);

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
    prepared_face_shapes: &[PreparedFaceShape],
    face_shapes: &[Shape],
    root_wires: &[RootWireTopology],
    root_edges: &[RootEdgeTopology],
    edge_shapes: &[Shape],
    vertex_positions: &[[f64; 3]],
    edge_count: usize,
) -> Result<Option<TopologySnapshotFaceFields>, Error> {
    for prepared_face_shape in prepared_face_shapes {
        if prepared_face_shape.face_wire_shapes.len() > 1 {
            let face_shape = face_shapes
                .get(prepared_face_shape.face_index)
                .ok_or_else(|| {
                    Error::new(format!(
                        "prepared face index {} was outside the root face inventory",
                        prepared_face_shape.face_index
                    ))
                })?;
            let geometry = match context.face_geometry(face_shape) {
                Ok(geometry) => geometry,
                Err(_) => context.face_geometry_occt(face_shape)?,
            };
            if geometry.kind != crate::SurfaceKind::Plane {
                return Ok(None);
            }
            if context.face_plane_payload(face_shape).is_err()
                && context.face_plane_payload_occt(face_shape).is_err()
            {
                return Ok(None);
            }
        }
    }

    pack_ported_face_snapshot(
        context,
        prepared_face_shapes,
        face_shapes,
        root_wires,
        root_edges,
        edge_shapes,
        vertex_positions,
        edge_count,
    )
}
