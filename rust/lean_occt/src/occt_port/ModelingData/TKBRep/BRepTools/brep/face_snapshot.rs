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

#[derive(Clone, Copy)]
struct PreparedPlanarFace {
    plane: PlanePayload,
    geometry: FaceGeometry,
}

struct MatchedFaceWire {
    root_wire_index: usize,
    orientation: Orientation,
    used_edges: Vec<usize>,
}

impl MatchedFaceWire {
    fn append_to(
        &self,
        used_root_wire_indices: &mut BTreeSet<usize>,
        face_wire_indices: &mut Vec<usize>,
        face_wire_orientations: &mut Vec<Orientation>,
        used_edges: &mut BTreeSet<usize>,
    ) {
        used_root_wire_indices.insert(self.root_wire_index);
        used_edges.extend(self.used_edges.iter().copied());
        face_wire_indices.push(self.root_wire_index);
        face_wire_orientations.push(self.orientation);
    }

    fn planar_area_magnitude(
        &self,
        context: &Context,
        planar_face: PreparedPlanarFace,
        root_wires: &[RootWireTopology],
        edge_shapes: &[Shape],
        root_edges: &[RootEdgeTopology],
    ) -> Result<Option<f64>, Error> {
        planar_wire_area_magnitude(
            context,
            planar_face.plane,
            planar_face.geometry,
            &root_wires[self.root_wire_index],
            edge_shapes,
            root_edges,
        )
    }
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

    fn planar_face(&self, context: &Context) -> Result<Option<PreparedPlanarFace>, Error> {
        if self.face_wire_shapes.len() <= 1 {
            return Ok(None);
        }

        Ok(Some(PreparedPlanarFace {
            plane: context.face_plane_payload_occt(&self.face_shape)?,
            geometry: context.face_geometry_occt(&self.face_shape)?,
        }))
    }
}

struct PreparedFaceTopology {
    face_wire_indices: Vec<usize>,
    face_wire_orientations: Vec<Orientation>,
    wire_roles: Vec<LoopRole>,
    used_edges: BTreeSet<usize>,
}

impl PreparedFaceTopology {
    fn new(
        face_wire_indices: Vec<usize>,
        face_wire_orientations: Vec<Orientation>,
        wire_roles: Vec<LoopRole>,
        used_edges: BTreeSet<usize>,
    ) -> Self {
        Self {
            face_wire_indices,
            face_wire_orientations,
            wire_roles,
            used_edges,
        }
    }
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
        root_wires: &[RootWireTopology],
        root_edges: &[RootEdgeTopology],
        edge_shapes: &[Shape],
        vertex_positions: &[[f64; 3]],
    ) -> Result<Option<PreparedFaceTopology>, Error> {
        if root_wires.is_empty() || prepared_face_shape.face_wire_shapes.is_empty() {
            return Ok(None);
        }

        let face_wire_shapes = &prepared_face_shape.face_wire_shapes;
        let planar_face = prepared_face_shape.planar_face(context)?;
        let mut builder = Self::new(face_wire_shapes.len());

        for face_wire_shape in face_wire_shapes {
            let Some(face_wire_topology) =
                root_wire_topology(context, face_wire_shape, vertex_positions, root_edges)?
            else {
                return Ok(None);
            };
            let Some(root_wire_index) = match_root_wire_index(
                root_wires,
                &face_wire_topology,
                &builder.used_root_wire_indices,
            ) else {
                return Ok(None);
            };
            let matched_face_wire = MatchedFaceWire {
                root_wire_index,
                orientation: context.shape_orientation(face_wire_shape)?,
                used_edges: face_wire_topology.edge_indices,
            };

            let wire_area = if let Some(planar_face) = planar_face {
                let Some(wire_area) = matched_face_wire.planar_area_magnitude(
                    context,
                    planar_face,
                    root_wires,
                    edge_shapes,
                    root_edges,
                )?
                else {
                    return Ok(None);
                };
                Some(wire_area)
            } else {
                None
            };
            matched_face_wire.append_to(
                &mut builder.used_root_wire_indices,
                &mut builder.face_wire_indices,
                &mut builder.face_wire_orientations,
                &mut builder.used_edges,
            );
            if let Some(wire_area) = wire_area {
                builder.face_wire_areas.push(wire_area);
            }
        }

        Ok(builder.finish())
    }

    fn new(face_wire_count: usize) -> Self {
        Self {
            used_root_wire_indices: BTreeSet::new(),
            face_wire_indices: Vec::with_capacity(face_wire_count),
            face_wire_orientations: Vec::with_capacity(face_wire_count),
            face_wire_areas: Vec::new(),
            used_edges: BTreeSet::new(),
        }
    }

    fn finish(self) -> Option<PreparedFaceTopology> {
        let wire_roles = self.classify_wire_roles()?;
        let Self {
            face_wire_indices,
            face_wire_orientations,
            used_edges,
            ..
        } = self;

        Some(PreparedFaceTopology::new(
            face_wire_indices,
            face_wire_orientations,
            wire_roles,
            used_edges,
        ))
    }

    fn classify_wire_roles(&self) -> Option<Vec<LoopRole>> {
        match self.face_wire_indices.len() {
            1 => Some(vec![LoopRole::Outer]),
            _ => {
                let (outer_offset, outer_area) = self
                    .face_wire_areas
                    .iter()
                    .copied()
                    .enumerate()
                    .max_by(|(_, lhs), (_, rhs)| lhs.total_cmp(rhs))?;
                if outer_area <= 1.0e-9 {
                    return None;
                }

                Some(
                    self.face_wire_areas
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
        prepared_face_topology: PreparedFaceTopology,
    ) -> Option<()> {
        let face_wire_offset = self.face_wire_indices.len();
        let PreparedFaceTopology {
            face_wire_indices,
            face_wire_orientations,
            wire_roles,
            used_edges,
        } = prepared_face_topology;
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
        let Some(prepared_face_topology) = PreparedFaceTopologyBuilder::build(
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
        let Some(()) = accumulator.append_face_topology_outputs(face_index, prepared_face_topology)
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
