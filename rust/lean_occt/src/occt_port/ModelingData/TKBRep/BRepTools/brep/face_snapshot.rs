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

struct MatchedFaceWires {
    face_wire_indices: Vec<usize>,
    face_wire_orientations: Vec<Orientation>,
    face_wire_areas: Vec<f64>,
    used_edges: BTreeSet<usize>,
}

struct PreparedFaceTopology {
    matched_face_wires: MatchedFaceWires,
    wire_roles: Vec<LoopRole>,
}

impl PreparedFaceTopology {
    fn load(
        context: &Context,
        face_shape: &Shape,
        root_wires: &[RootWireTopology],
        root_edges: &[RootEdgeTopology],
        edge_shapes: &[Shape],
        vertex_positions: &[[f64; 3]],
    ) -> Result<Option<Self>, Error> {
        let face_wire_shapes = context.subshapes_occt(face_shape, ShapeKind::Wire)?;
        if root_wires.is_empty() || face_wire_shapes.is_empty() {
            return Ok(None);
        }

        let Some(planar_face) =
            Self::load_planar_face(context, face_shape, face_wire_shapes.len())?
        else {
            return Ok(None);
        };

        let Some(matched_face_wires) = Self::collect_matched_face_wires(
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
        let Some(wire_roles) = Self::classify_wire_roles(&matched_face_wires) else {
            return Ok(None);
        };

        Ok(Some(Self {
            matched_face_wires,
            wire_roles,
        }))
    }

    fn load_planar_face(
        context: &Context,
        face_shape: &Shape,
        face_wire_count: usize,
    ) -> Result<Option<Option<(PlanePayload, FaceGeometry)>>, Error> {
        if face_wire_count <= 1 {
            return Ok(Some(None));
        }
        if !multi_wire_face_is_planar(context, face_shape, face_wire_count)? {
            return Ok(None);
        }

        Ok(Some(Some((
            context.face_plane_payload_occt(face_shape)?,
            context.face_geometry_occt(face_shape)?,
        ))))
    }

    fn collect_matched_face_wires(
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

    fn classify_wire_roles(matched_face_wires: &MatchedFaceWires) -> Option<Vec<LoopRole>> {
        match matched_face_wires.face_wire_indices.len() {
            1 => Some(vec![LoopRole::Outer]),
            _ => {
                let (outer_offset, outer_area) = matched_face_wires
                    .face_wire_areas
                    .iter()
                    .copied()
                    .enumerate()
                    .max_by(|(_, lhs), (_, rhs)| lhs.total_cmp(rhs))?;
                if outer_area <= 1.0e-9 {
                    return None;
                }

                Some(
                    matched_face_wires
                        .face_wire_areas
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
}

struct FaceSnapshotAccumulator {
    edge_face_lists: Vec<Vec<usize>>,
    faces: Vec<crate::TopologyRange>,
    face_wire_indices: Vec<usize>,
    face_wire_orientations: Vec<Orientation>,
    face_wire_roles: Vec<LoopRole>,
}

impl FaceSnapshotAccumulator {
    fn new(edge_count: usize, face_count: usize) -> Self {
        Self {
            edge_face_lists: vec![Vec::new(); edge_count],
            faces: Vec::with_capacity(face_count),
            face_wire_indices: Vec::new(),
            face_wire_orientations: Vec::new(),
            face_wire_roles: Vec::new(),
        }
    }

    fn into_fields(self) -> TopologySnapshotFaceFields {
        let (edge_faces, edge_face_indices) = Self::flatten_edge_face_lists(self.edge_face_lists);
        TopologySnapshotFaceFields {
            edge_faces,
            edge_face_indices,
            faces: self.faces,
            face_wire_indices: self.face_wire_indices,
            face_wire_orientations: self.face_wire_orientations,
            face_wire_roles: self.face_wire_roles,
        }
    }

    fn flatten_edge_face_lists(
        edge_face_lists: Vec<Vec<usize>>,
    ) -> (Vec<crate::TopologyRange>, Vec<usize>) {
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

        (edge_faces, edge_face_indices)
    }

    fn append_face_topology_outputs(
        &mut self,
        face_index: usize,
        face_wire_offset: usize,
        matched_face_wires: MatchedFaceWires,
        wire_roles: Vec<LoopRole>,
    ) -> Option<()> {
        let face_wire_count = matched_face_wires.face_wire_indices.len();
        self.face_wire_indices
            .extend(matched_face_wires.face_wire_indices);
        self.face_wire_orientations
            .extend(matched_face_wires.face_wire_orientations);
        self.faces.push(crate::TopologyRange {
            offset: face_wire_offset,
            count: face_wire_count,
        });
        self.face_wire_roles.extend(wire_roles);

        for edge_index in matched_face_wires.used_edges {
            let edge_faces = self.edge_face_lists.get_mut(edge_index)?;
            edge_faces.push(face_index);
        }

        Some(())
    }
}

fn append_ported_face_topology(
    context: &Context,
    face_index: usize,
    face_shape: &Shape,
    root_wires: &[RootWireTopology],
    root_edges: &[RootEdgeTopology],
    edge_shapes: &[Shape],
    vertex_positions: &[[f64; 3]],
    accumulator: &mut FaceSnapshotAccumulator,
) -> Result<Option<()>, Error> {
    let face_wire_offset = accumulator.face_wire_indices.len();
    let Some(prepared_face_topology) = PreparedFaceTopology::load(
        context,
        face_shape,
        root_wires,
        root_edges,
        edge_shapes,
        vertex_positions,
    )?
    else {
        return Ok(None);
    };

    let Some(()) = accumulator.append_face_topology_outputs(
        face_index,
        face_wire_offset,
        prepared_face_topology.matched_face_wires,
        prepared_face_topology.wire_roles,
    ) else {
        return Ok(None);
    };

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
    let mut accumulator = FaceSnapshotAccumulator::new(edge_count, face_shapes.len());

    for (face_index, face_shape) in face_shapes.iter().enumerate() {
        let Some(()) = append_ported_face_topology(
            context,
            face_index,
            face_shape,
            root_wires,
            root_edges,
            edge_shapes,
            vertex_positions,
            &mut accumulator,
        )?
        else {
            return Ok(None);
        };
    }

    Ok(Some(accumulator.into_fields()))
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
