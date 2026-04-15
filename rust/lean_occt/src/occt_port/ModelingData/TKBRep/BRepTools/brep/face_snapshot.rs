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

struct PreparedFaceShape {
    face_shape: Shape,
    face_wire_shapes: Vec<Shape>,
}

impl PreparedFaceShape {
    fn load_all(context: &Context, shape: &Shape) -> Result<Option<Vec<Self>>, Error> {
        let face_shapes = context.subshapes_occt(shape, ShapeKind::Face)?;
        let mut prepared_face_shapes = Vec::with_capacity(face_shapes.len());

        for face_shape in face_shapes {
            let Some(prepared_face_shape) = Self::load(context, face_shape)? else {
                return Ok(None);
            };
            prepared_face_shapes.push(prepared_face_shape);
        }

        Ok(Some(prepared_face_shapes))
    }

    fn load(context: &Context, face_shape: Shape) -> Result<Option<Self>, Error> {
        let face_wire_shapes = context.subshapes_occt(&face_shape, ShapeKind::Wire)?;
        if !Self::multi_wire_face_is_planar(context, &face_shape, face_wire_shapes.len())? {
            return Ok(None);
        }

        Ok(Some(Self {
            face_shape,
            face_wire_shapes,
        }))
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

    fn is_empty(&self) -> bool {
        self.face_wire_shapes.is_empty()
    }

    fn wire_shapes(&self) -> &[Shape] {
        &self.face_wire_shapes
    }

    fn planar_face(
        &self,
        context: &Context,
    ) -> Result<Option<(PlanePayload, FaceGeometry)>, Error> {
        if self.face_wire_shapes.len() <= 1 {
            return Ok(None);
        }

        Ok(Some((
            context.face_plane_payload_occt(&self.face_shape)?,
            context.face_geometry_occt(&self.face_shape)?,
        )))
    }
}

struct PreparedFaceTopology {
    face_wire_indices: Vec<usize>,
    face_wire_orientations: Vec<Orientation>,
    wire_roles: Vec<LoopRole>,
    used_edges: BTreeSet<usize>,
}

impl PreparedFaceTopology {
    fn load(
        context: &Context,
        prepared_face_shape: &PreparedFaceShape,
        root_wires: &[RootWireTopology],
        root_edges: &[RootEdgeTopology],
        edge_shapes: &[Shape],
        vertex_positions: &[[f64; 3]],
    ) -> Result<Option<Self>, Error> {
        if root_wires.is_empty() || prepared_face_shape.is_empty() {
            return Ok(None);
        }

        let planar_face = prepared_face_shape.planar_face(context)?;

        Self::collect_matched_face_wires(
            context,
            prepared_face_shape.wire_shapes(),
            root_wires,
            root_edges,
            edge_shapes,
            vertex_positions,
            planar_face,
        )
    }

    fn collect_matched_face_wires(
        context: &Context,
        face_wire_shapes: &[Shape],
        root_wires: &[RootWireTopology],
        root_edges: &[RootEdgeTopology],
        edge_shapes: &[Shape],
        vertex_positions: &[[f64; 3]],
        planar_face: Option<(PlanePayload, FaceGeometry)>,
    ) -> Result<Option<Self>, Error> {
        let mut used_root_wire_indices = BTreeSet::new();
        let mut face_wire_indices = Vec::with_capacity(face_wire_shapes.len());
        let mut face_wire_orientations = Vec::with_capacity(face_wire_shapes.len());
        let mut face_wire_areas = Vec::new();
        let mut used_edges = BTreeSet::new();

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

        let Some(wire_roles) = Self::classify_wire_roles(face_wire_indices.len(), &face_wire_areas)
        else {
            return Ok(None);
        };

        Ok(Some(Self {
            face_wire_indices,
            face_wire_orientations,
            wire_roles,
            used_edges,
        }))
    }

    fn classify_wire_roles(
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
        face_wire_indices: Vec<usize>,
        face_wire_orientations: Vec<Orientation>,
        used_edges: BTreeSet<usize>,
        wire_roles: Vec<LoopRole>,
    ) -> Option<()> {
        let face_wire_count = face_wire_indices.len();
        self.face_wire_indices.extend(face_wire_indices);
        self.face_wire_orientations.extend(face_wire_orientations);
        self.faces.push(crate::TopologyRange {
            offset: face_wire_offset,
            count: face_wire_count,
        });
        self.face_wire_roles.extend(wire_roles);

        for edge_index in used_edges {
            let edge_faces = self.edge_face_lists.get_mut(edge_index)?;
            edge_faces.push(face_index);
        }

        Some(())
    }

    fn append_prepared_face_topology(
        &mut self,
        face_index: usize,
        prepared_face_topology: PreparedFaceTopology,
    ) -> Option<()> {
        let face_wire_offset = self.face_wire_indices.len();
        self.append_face_topology_outputs(
            face_index,
            face_wire_offset,
            prepared_face_topology.face_wire_indices,
            prepared_face_topology.face_wire_orientations,
            prepared_face_topology.used_edges,
            prepared_face_topology.wire_roles,
        )
    }
}

fn pack_ported_face_snapshot(
    context: &Context,
    prepared_face_shapes: &[PreparedFaceShape],
    root_wires: &[RootWireTopology],
    root_edges: &[RootEdgeTopology],
    edge_shapes: &[Shape],
    vertex_positions: &[[f64; 3]],
    edge_count: usize,
) -> Result<Option<TopologySnapshotFaceFields>, Error> {
    let mut accumulator = FaceSnapshotAccumulator::new(edge_count, prepared_face_shapes.len());

    for (face_index, prepared_face_shape) in prepared_face_shapes.iter().enumerate() {
        let Some(prepared_face_topology) = PreparedFaceTopology::load(
            context,
            prepared_face_shape,
            root_wires,
            root_edges,
            edge_shapes,
            vertex_positions,
        )?
        else {
            return Ok(None);
        };
        let Some(()) =
            accumulator.append_prepared_face_topology(face_index, prepared_face_topology)
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
    let Some(prepared_face_shapes) = PreparedFaceShape::load_all(context, shape)? else {
        return Ok(None);
    };

    pack_ported_face_snapshot(
        context,
        &prepared_face_shapes,
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
