use super::*;

use std::ops::ControlFlow;

use crate::EdgeSample;

use super::face_metrics::{
    analytic_face_volume, analytic_offset_face_volume, analytic_ported_swept_face_volume,
};
use super::topology::PreparedShellShape;

#[derive(Clone, Copy, Debug)]
pub(super) struct MeshFaceProperties {
    pub(super) area: f64,
    pub(super) sample: FaceSample,
}

pub(super) struct LazyMeshFaceFallback<'a> {
    context: &'a Context,
    face_shape: &'a Shape,
    orientation: Orientation,
    properties: Option<MeshFaceProperties>,
    loaded: bool,
}

impl<'a> LazyMeshFaceFallback<'a> {
    pub(super) fn new(
        context: &'a Context,
        face_shape: &'a Shape,
        orientation: Orientation,
        eagerly_load: bool,
    ) -> Self {
        let properties = if eagerly_load {
            mesh_face_properties(context, face_shape, orientation)
        } else {
            None
        };

        Self {
            context,
            face_shape,
            orientation,
            properties,
            loaded: eagerly_load,
        }
    }

    pub(super) fn resolve_sample(
        &mut self,
        sample: Option<FaceSample>,
        index: usize,
        geometry: FaceGeometry,
    ) -> Result<FaceSample, Error> {
        sample
            .or_else(|| self.load().map(|fallback| fallback.sample))
            .ok_or_else(|| {
                Error::new(format!(
                    "failed to derive a Rust-owned sample for face {index} ({:?})",
                    geometry.kind
                ))
            })
    }

    pub(super) fn resolve_area(
        &mut self,
        area: Option<f64>,
        index: usize,
        geometry: FaceGeometry,
    ) -> Result<f64, Error> {
        area.or_else(|| self.load().map(|fallback| fallback.area))
            .ok_or_else(|| {
                Error::new(format!(
                    "failed to derive a Rust-owned area for face {index} ({:?})",
                    geometry.kind
                ))
            })
    }

    fn load(&mut self) -> Option<MeshFaceProperties> {
        if !self.loaded {
            self.properties = mesh_face_properties(self.context, self.face_shape, self.orientation);
            self.loaded = true;
        }

        self.properties
    }
}

#[derive(Clone, Copy, Debug)]
struct ExactPrimitiveSummary {
    surface_area: f64,
    volume: f64,
    bbox: Option<([f64; 3], [f64; 3])>,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ShapeCounts {
    compound_count: usize,
    compsolid_count: usize,
    solid_count: usize,
    shell_count: usize,
    face_count: usize,
    wire_count: usize,
    edge_count: usize,
    vertex_count: usize,
}

pub(super) fn ported_shape_summary(
    context: &Context,
    shape: &Shape,
    vertices: &[BrepVertex],
    topology: &TopologySnapshot,
    wires: &[BrepWire],
    edges: &[BrepEdge],
    faces: &[BrepFace],
    vertex_shapes: &[Shape],
    prepared_shell_shapes: &[PreparedShellShape],
    face_shapes: &[Shape],
    edge_shapes: &[Shape],
) -> Result<ShapeSummary, Error> {
    let counts = shape_counts(context, shape, topology)?;
    let root_kind = classify_root_kind(counts);
    let primary_kind = classify_primary_kind(counts);
    let exact_primitive =
        exact_primitive_shape_summary(primary_kind, counts.solid_count, vertices, edges, faces);
    let closed_volume_topology = has_closed_volume_topology(faces, edges);
    let fallback_summary = || context.describe_shape_occt(shape).ok();
    let contains_offset_faces = faces
        .iter()
        .any(|face| matches!(face.ported_face_surface, Some(PortedFaceSurface::Offset(_))));
    let offset_non_solid =
        contains_offset_faces && counts.solid_count == 0 && counts.compsolid_count == 0;
    let offset_margin = offset_face_margin(faces);
    let (bbox_min, bbox_max) = exact_primitive
        .and_then(|summary| summary.bbox)
        .or_else(|| ported_shape_bbox(vertices, edges, faces))
        .or_else(|| {
            if offset_non_solid {
                offset_faces_bbox(context, shape, offset_margin, vertices, edges, face_shapes)
            } else {
                None
            }
        })
        .or_else(|| {
            if offset_non_solid {
                offset_shape_bbox_occt(
                    context,
                    shape,
                    faces,
                    vertex_shapes,
                    face_shapes,
                    edge_shapes,
                )
            } else {
                None
            }
        })
        .or_else(|| {
            if offset_non_solid {
                let shape_occt_bbox = shape_bbox_occt(context, shape)?;
                validated_mesh_bbox(context, shape, shape_occt_bbox, offset_margin)
            } else {
                None
            }
        })
        .or_else(|| {
            if contains_offset_faces && (counts.solid_count > 0 || counts.compsolid_count > 0) {
                offset_solid_shell_bbox(context, faces, prepared_shell_shapes).or_else(|| {
                    fallback_summary().map(|summary| (summary.bbox_min, summary.bbox_max))
                })
            } else {
                None
            }
        })
        .or_else(|| {
            if offset_non_solid {
                None
            } else {
                mesh_shape_bbox(context, shape)
            }
        })
        .or_else(|| fallback_summary().map(|summary| (summary.bbox_min, summary.bbox_max)))
        .unwrap_or(([0.0; 3], [0.0; 3]));

    Ok(ShapeSummary {
        root_kind,
        primary_kind,
        compound_count: counts.compound_count,
        compsolid_count: counts.compsolid_count,
        solid_count: counts.solid_count,
        shell_count: counts.shell_count,
        face_count: counts.face_count,
        wire_count: counts.wire_count,
        edge_count: counts.edge_count,
        vertex_count: counts.vertex_count,
        // Match OCCT's whole-shape linear properties semantics: when wires are present,
        // length is accumulated over wire-edge occurrences rather than unique topological edges.
        linear_length: if topology.wire_edge_indices.is_empty() {
            edges.iter().map(|edge| edge.length).sum()
        } else {
            topology
                .wire_edge_indices
                .iter()
                .filter_map(|&edge_index| edges.get(edge_index))
                .map(|edge| edge.length)
                .sum()
        },
        surface_area: exact_primitive
            .map(|summary| summary.surface_area)
            .unwrap_or_else(|| faces.iter().map(|face| face.area).sum()),
        volume: exact_primitive
            .map(|summary| summary.volume)
            .or_else(|| {
                if closed_volume_topology {
                    analytic_shape_volume(context, wires, edges, faces, face_shapes, edge_shapes)
                } else {
                    None
                }
            })
            .or_else(|| {
                if closed_volume_topology {
                    mesh_shape_volume(context, shape, counts)
                } else {
                    None
                }
            })
            .or_else(|| fallback_summary().map(|summary| summary.volume))
            .unwrap_or(0.0),
        bbox_min,
        bbox_max,
    })
}

fn has_closed_volume_topology(faces: &[BrepFace], edges: &[BrepEdge]) -> bool {
    faces.is_empty()
        || edges
            .iter()
            .all(|edge| edge.adjacent_face_indices.len() == 2)
}

fn analytic_shape_volume(
    context: &Context,
    wires: &[BrepWire],
    edges: &[BrepEdge],
    faces: &[BrepFace],
    face_shapes: &[Shape],
    edge_shapes: &[Shape],
) -> Option<f64> {
    if !has_closed_volume_topology(faces, edges) {
        return None;
    }

    if faces.is_empty() {
        return Some(0.0);
    }

    let mut volume = 0.0;
    for face in faces {
        let face_shape = face_shapes.get(face.index)?;
        let analytic_contribution = match face.ported_face_surface {
            Some(PortedFaceSurface::Analytic(surface)) => analytic_face_volume(
                context,
                face,
                surface,
                face.geometry,
                &face.loops,
                wires,
                edges,
                edge_shapes,
            ),
            Some(PortedFaceSurface::Swept(surface)) => {
                analytic_ported_swept_face_volume(face, face.geometry, surface)
            }
            Some(PortedFaceSurface::Offset(surface)) => analytic_offset_face_volume(
                context,
                face,
                surface,
                face.geometry,
                &face.loops,
                wires,
                edges,
                edge_shapes,
            ),
            None => None,
        };
        let contribution =
            analytic_contribution.or_else(|| mesh_face_volume(context, face_shape, face))?;
        volume += contribution;
    }
    Some(volume.abs())
}

fn exact_primitive_shape_summary(
    primary_kind: ShapeKind,
    solid_count: usize,
    vertices: &[BrepVertex],
    edges: &[BrepEdge],
    faces: &[BrepFace],
) -> Option<ExactPrimitiveSummary> {
    if primary_kind != ShapeKind::Solid || solid_count != 1 {
        return None;
    }

    exact_box_summary(vertices, edges, faces)
        .or_else(|| exact_cylinder_summary(faces))
        .or_else(|| exact_cone_summary(faces))
        .or_else(|| exact_sphere_summary(faces))
        .or_else(|| exact_torus_summary(faces))
        .or_else(|| exact_translational_solid_summary(faces, edges))
}

fn exact_box_summary(
    vertices: &[BrepVertex],
    edges: &[BrepEdge],
    faces: &[BrepFace],
) -> Option<ExactPrimitiveSummary> {
    if vertices.len() != 8 || edges.len() != 12 || faces.len() != 6 {
        return None;
    }
    if !edges
        .iter()
        .all(|edge| matches!(edge.ported_curve, Some(PortedCurve::Line(_))))
    {
        return None;
    }
    if !faces
        .iter()
        .all(|face| matches!(face.ported_surface, Some(PortedSurface::Plane(_))))
    {
        return None;
    }
    let bbox = bbox_from_points(vertices.iter().map(|vertex| vertex.position).collect())?;

    for vertex in vertices {
        let incident = edges
            .iter()
            .filter_map(|edge| incident_edge_vector(edge, vertex.index))
            .collect::<Vec<_>>();
        if incident.len() < 3 {
            continue;
        }

        for i in 0..incident.len() {
            for j in i + 1..incident.len() {
                for k in j + 1..incident.len() {
                    let a = incident[i];
                    let b = incident[j];
                    let c = incident[k];
                    let volume = dot3(a, cross3(b, c)).abs();
                    if volume <= 1.0e-9 {
                        continue;
                    }
                    let surface_area =
                        2.0 * (norm3(cross3(a, b)) + norm3(cross3(a, c)) + norm3(cross3(b, c)));
                    return Some(ExactPrimitiveSummary {
                        surface_area,
                        volume,
                        bbox: Some(bbox),
                    });
                }
            }
        }
    }

    None
}

fn exact_cylinder_summary(faces: &[BrepFace]) -> Option<ExactPrimitiveSummary> {
    if faces.len() != 3 {
        return None;
    }

    let (payload, _) = single_cylinder_face(faces)?;
    let axis = normalize3(payload.axis);
    if norm3(axis) <= 1.0e-12 {
        return None;
    }
    let caps = aligned_plane_faces(faces, axis);
    if caps.len() != 2 {
        return None;
    }

    let height = (dot3(subtract3(caps[0].origin, payload.origin), axis)
        - dot3(subtract3(caps[1].origin, payload.origin), axis))
    .abs();
    let radius = payload.radius.abs();
    let bbox = circular_sections_bbox(
        axis,
        &caps
            .iter()
            .map(|plane| {
                let axial = dot3(subtract3(plane.origin, payload.origin), axis);
                (add3(payload.origin, scale3(axis, axial)), radius)
            })
            .collect::<Vec<_>>(),
    );
    Some(ExactPrimitiveSummary {
        surface_area: 2.0 * PI * radius * (height + radius),
        volume: PI * radius * radius * height,
        bbox,
    })
}

fn exact_cone_summary(faces: &[BrepFace]) -> Option<ExactPrimitiveSummary> {
    if !(2..=3).contains(&faces.len()) {
        return None;
    }

    let (payload, _) = single_cone_face(faces)?;
    let (axis, axial_radii) = exact_cone_axial_radii(payload, faces)?;
    if axial_radii.len() != 2 {
        return None;
    }

    let (axial0, radius0) = axial_radii[0];
    let (axial1, radius1) = axial_radii[1];
    let height = (axial0 - axial1).abs();
    let slant = ((radius0 - radius1).powi(2) + height.powi(2)).sqrt();
    let bbox = circular_sections_bbox(
        axis,
        &axial_radii
            .iter()
            .map(|&(axial, radius)| (add3(payload.origin, scale3(axis, axial)), radius))
            .collect::<Vec<_>>(),
    );

    Some(ExactPrimitiveSummary {
        surface_area: PI * (radius0 + radius1) * slant
            + PI * (radius0 * radius0 + radius1 * radius1),
        volume: PI * height * (radius0 * radius0 + radius0 * radius1 + radius1 * radius1) / 3.0,
        bbox,
    })
}

fn exact_sphere_summary(faces: &[BrepFace]) -> Option<ExactPrimitiveSummary> {
    let (payload, _) = single_sphere_face(faces)?;
    let radius = payload.radius.abs();
    Some(ExactPrimitiveSummary {
        surface_area: 4.0 * PI * radius * radius,
        volume: 4.0 * PI * radius * radius * radius / 3.0,
        bbox: Some((
            [
                payload.center[0] - radius,
                payload.center[1] - radius,
                payload.center[2] - radius,
            ],
            [
                payload.center[0] + radius,
                payload.center[1] + radius,
                payload.center[2] + radius,
            ],
        )),
    })
}

fn exact_torus_summary(faces: &[BrepFace]) -> Option<ExactPrimitiveSummary> {
    let (payload, _) = single_torus_face(faces)?;
    let major_radius = payload.major_radius.abs();
    let minor_radius = payload.minor_radius.abs();
    Some(ExactPrimitiveSummary {
        surface_area: 4.0 * PI * PI * major_radius * minor_radius,
        volume: 2.0 * PI * PI * major_radius * minor_radius * minor_radius,
        bbox: None,
    })
}

fn exact_translational_solid_summary(
    faces: &[BrepFace],
    edges: &[BrepEdge],
) -> Option<ExactPrimitiveSummary> {
    let plane_faces = faces
        .iter()
        .filter_map(|face| match face.ported_surface {
            Some(PortedSurface::Plane(payload)) => Some((face, payload)),
            _ => None,
        })
        .collect::<Vec<_>>();
    if plane_faces.len() < 2 {
        return None;
    }

    for i in 0..plane_faces.len() {
        for j in i + 1..plane_faces.len() {
            let (lhs_face, lhs_plane) = plane_faces[i];
            let (rhs_face, rhs_plane) = plane_faces[j];
            let lhs_normal = normalize3(lhs_plane.normal);
            let rhs_normal = normalize3(rhs_plane.normal);
            if dot3(lhs_normal, rhs_normal) > -1.0 + 1.0e-6 {
                continue;
            }
            if !approx_eq(lhs_face.area, rhs_face.area, 1.0e-6, 1.0e-6) {
                continue;
            }
            if !pair_forms_translational_caps(faces, lhs_face.index, rhs_face.index) {
                continue;
            }

            let span = dot3(subtract3(rhs_plane.origin, lhs_plane.origin), lhs_normal).abs();
            if span <= 1.0e-9 {
                continue;
            }

            return Some(ExactPrimitiveSummary {
                surface_area: faces.iter().map(|face| face.area).sum(),
                volume: lhs_face.area * span,
                bbox: analytic_edges_bbox(edges),
            });
        }
    }

    None
}

fn exact_cone_axial_radii(
    payload: ConePayload,
    faces: &[BrepFace],
) -> Option<([f64; 3], Vec<(f64, f64)>)> {
    let axis = normalize3(payload.axis);
    if norm3(axis) <= 1.0e-12 {
        return None;
    }
    let caps = aligned_plane_faces(faces, axis);
    if caps.is_empty() || caps.len() > 2 || caps.len() + 1 != faces.len() {
        return None;
    }

    let tan_angle = payload.semi_angle.tan();
    if tan_angle.abs() <= 1.0e-12 {
        return None;
    }

    let mut axial_radii = caps
        .iter()
        .map(|plane| {
            let axial = dot3(subtract3(plane.origin, payload.origin), axis);
            let radius = (payload.reference_radius + axial * tan_angle).abs();
            (axial, radius)
        })
        .collect::<Vec<_>>();
    if axial_radii.len() == 1 {
        axial_radii.push((-payload.reference_radius / tan_angle, 0.0));
    }

    (axial_radii.len() == 2).then_some((axis, axial_radii))
}

fn circular_sections_bbox(
    axis: [f64; 3],
    sections: &[([f64; 3], f64)],
) -> Option<([f64; 3], [f64; 3])> {
    if sections.is_empty() {
        return None;
    }

    let axis = normalize3(axis);
    if norm3(axis) <= 1.0e-12 {
        return None;
    }

    let mut bbox_min = [f64::INFINITY; 3];
    let mut bbox_max = [f64::NEG_INFINITY; 3];
    for (center, radius) in sections {
        for coordinate in 0..3 {
            let radial_extent =
                radius.abs() * (1.0 - axis[coordinate] * axis[coordinate]).max(0.0).sqrt();
            bbox_min[coordinate] = bbox_min[coordinate].min(center[coordinate] - radial_extent);
            bbox_max[coordinate] = bbox_max[coordinate].max(center[coordinate] + radial_extent);
        }
    }
    Some((bbox_min, bbox_max))
}

fn single_cylinder_face(faces: &[BrepFace]) -> Option<(CylinderPayload, usize)> {
    single_surface_face(faces, |surface| match surface {
        PortedSurface::Cylinder(payload) => Some(payload),
        _ => None,
    })
}

fn single_cone_face(faces: &[BrepFace]) -> Option<(ConePayload, usize)> {
    single_surface_face(faces, |surface| match surface {
        PortedSurface::Cone(payload) => Some(payload),
        _ => None,
    })
}

fn single_sphere_face(faces: &[BrepFace]) -> Option<(SpherePayload, usize)> {
    if faces.len() != 1 {
        return None;
    }
    single_surface_face(faces, |surface| match surface {
        PortedSurface::Sphere(payload) => Some(payload),
        _ => None,
    })
}

fn single_torus_face(faces: &[BrepFace]) -> Option<(TorusPayload, usize)> {
    if faces.len() != 1 {
        return None;
    }
    single_surface_face(faces, |surface| match surface {
        PortedSurface::Torus(payload) => Some(payload),
        _ => None,
    })
}

fn single_surface_face<T>(
    faces: &[BrepFace],
    extract: impl Fn(PortedSurface) -> Option<T>,
) -> Option<(T, usize)> {
    let mut found = None;
    for face in faces {
        let Some(surface) = face.ported_surface else {
            return None;
        };
        let Some(payload) = extract(surface) else {
            continue;
        };
        if found.is_some() {
            return None;
        }
        found = Some((payload, face.index));
    }
    found
}

fn aligned_plane_faces(faces: &[BrepFace], axis: [f64; 3]) -> Vec<PlanePayload> {
    faces
        .iter()
        .filter_map(|face| match face.ported_surface {
            Some(PortedSurface::Plane(payload))
                if dot3(normalize3(payload.normal), axis).abs() >= 1.0 - 1.0e-6 =>
            {
                Some(payload)
            }
            _ => None,
        })
        .collect()
}

fn pair_forms_translational_caps(faces: &[BrepFace], lhs_index: usize, rhs_index: usize) -> bool {
    faces.iter().all(|candidate| {
        if candidate.index == lhs_index || candidate.index == rhs_index {
            return true;
        }
        candidate.adjacent_face_indices.contains(&lhs_index)
            && candidate.adjacent_face_indices.contains(&rhs_index)
    })
}

fn incident_edge_vector(edge: &BrepEdge, vertex_index: usize) -> Option<[f64; 3]> {
    match (
        edge.start_vertex,
        edge.end_vertex,
        edge.start_point,
        edge.end_point,
    ) {
        (Some(start), Some(_), Some(start_point), Some(end_point)) if start == vertex_index => {
            Some(subtract3(end_point, start_point))
        }
        (Some(_), Some(end), Some(start_point), Some(end_point)) if end == vertex_index => {
            Some(subtract3(start_point, end_point))
        }
        _ => None,
    }
}

fn mesh_shape_volume(context: &Context, shape: &Shape, counts: ShapeCounts) -> Option<f64> {
    if counts.solid_count == 0 && counts.compsolid_count == 0 {
        return Some(0.0);
    }

    let mesh = context.mesh(shape, SUMMARY_VOLUME_MESH_PARAMS).ok()?;
    polyhedral_mesh_volume(&mesh)
}

fn ported_shape_bbox(
    vertices: &[BrepVertex],
    edges: &[BrepEdge],
    faces: &[BrepFace],
) -> Option<([f64; 3], [f64; 3])> {
    topological_shape_bbox(vertices, edges, faces)
}

pub(super) fn shape_counts(
    context: &Context,
    shape: &Shape,
    topology: &TopologySnapshot,
) -> Result<ShapeCounts, Error> {
    Ok(ShapeCounts {
        compound_count: context.subshape_count_occt(shape, ShapeKind::Compound)?,
        compsolid_count: context.subshape_count_occt(shape, ShapeKind::CompSolid)?,
        solid_count: context.subshape_count_occt(shape, ShapeKind::Solid)?,
        shell_count: context.subshape_count_occt(shape, ShapeKind::Shell)?,
        face_count: topology.faces.len(),
        wire_count: topology.wires.len(),
        edge_count: topology.edges.len(),
        vertex_count: topology.vertex_positions.len(),
    })
}

pub(super) fn classify_root_kind(counts: ShapeCounts) -> ShapeKind {
    if counts.compound_count > 0 {
        ShapeKind::Compound
    } else if counts.compsolid_count > 0 {
        ShapeKind::CompSolid
    } else if counts.solid_count > 0 {
        ShapeKind::Solid
    } else if counts.shell_count > 0 {
        ShapeKind::Shell
    } else if counts.face_count > 0 {
        ShapeKind::Face
    } else if counts.wire_count > 0 {
        ShapeKind::Wire
    } else if counts.edge_count > 0 {
        ShapeKind::Edge
    } else if counts.vertex_count > 0 {
        ShapeKind::Vertex
    } else {
        ShapeKind::Unknown
    }
}

fn classify_primary_kind(counts: ShapeCounts) -> ShapeKind {
    if counts.solid_count > 0 {
        ShapeKind::Solid
    } else if counts.shell_count > 0 {
        ShapeKind::Shell
    } else if counts.face_count > 0 {
        ShapeKind::Face
    } else if counts.wire_count > 0 {
        ShapeKind::Wire
    } else if counts.edge_count > 0 {
        ShapeKind::Edge
    } else if counts.vertex_count > 0 {
        ShapeKind::Vertex
    } else if counts.compsolid_count > 0 {
        ShapeKind::CompSolid
    } else if counts.compound_count > 0 {
        ShapeKind::Compound
    } else {
        ShapeKind::Unknown
    }
}

fn topological_shape_bbox(
    vertices: &[BrepVertex],
    edges: &[BrepEdge],
    faces: &[BrepFace],
) -> Option<([f64; 3], [f64; 3])> {
    if faces.is_empty() {
        return boundary_shape_bbox(vertices, edges);
    }

    if faces_use_analytic_edge_bbox(edges, faces) {
        return boundary_shape_bbox(vertices, edges);
    }

    None
}

fn offset_faces_bbox(
    context: &Context,
    shape: &Shape,
    offset_margin: Option<f64>,
    vertices: &[BrepVertex],
    edges: &[BrepEdge],
    face_shapes: &[Shape],
) -> Option<([f64; 3], [f64; 3])> {
    let shape_occt_bbox = shape_bbox_occt(context, shape);
    if let Some(shape_occt_bbox) = shape_occt_bbox {
        if let Some(mesh_bbox) = validated_mesh_bbox(context, shape, shape_occt_bbox, offset_margin)
        {
            return Some(mesh_bbox);
        }
    }

    let boundary_bbox = boundary_shape_bbox(vertices, edges);
    let rust_face_bbox = (face_shapes.len() > 1)
        .then(|| face_breps_bbox(context, face_shapes))
        .flatten()
        .map(|face_bbox| match boundary_bbox {
            Some(boundary_bbox) => union_bbox(boundary_bbox, face_bbox),
            None => face_bbox,
        });
    if let Some(rust_face_bbox) = rust_face_bbox {
        if let Some(shape_occt_bbox) = shape_occt_bbox {
            if bbox_matches(rust_face_bbox, shape_occt_bbox) {
                return Some(rust_face_bbox);
            }
        }
    }

    let face_bbox = face_bboxes_occt(context, face_shapes)?;
    let face_bbox = match boundary_bbox {
        Some(boundary_bbox) => union_bbox(boundary_bbox, face_bbox),
        None => face_bbox,
    };
    match shape_occt_bbox {
        Some(shape_occt_bbox) if bbox_matches(face_bbox, shape_occt_bbox) => Some(face_bbox),
        Some(_) => None,
        None => Some(face_bbox),
    }
}

fn face_breps_bbox(context: &Context, face_shapes: &[Shape]) -> Option<([f64; 3], [f64; 3])> {
    let mut bbox = None;
    for face_shape in face_shapes {
        let face_bbox = validated_face_brep_bbox(context, face_shape)?;
        bbox = Some(match bbox {
            Some(accumulated) => union_bbox(accumulated, face_bbox),
            None => face_bbox,
        });
    }
    bbox
}

fn validated_face_brep_bbox(context: &Context, face_shape: &Shape) -> Option<([f64; 3], [f64; 3])> {
    let face_occt_bbox = shape_bbox_occt(context, face_shape)?;
    let brep = context.ported_brep(face_shape).ok()?;
    let summary_bbox = (brep.summary.bbox_min, brep.summary.bbox_max);
    if bbox_matches(summary_bbox, face_occt_bbox) {
        return Some(summary_bbox);
    }

    if let Some(expanded_bbox) = offset_expanded_brep_bbox(&brep) {
        if bbox_matches(expanded_bbox, face_occt_bbox) {
            return Some(expanded_bbox);
        }
    }

    validated_mesh_bbox(
        context,
        face_shape,
        face_occt_bbox,
        offset_brep_margin(&brep),
    )
}

fn offset_expanded_brep_bbox(brep: &BrepShape) -> Option<([f64; 3], [f64; 3])> {
    let offset = offset_brep_margin(brep)?;
    Some(expand_bbox(
        (brep.summary.bbox_min, brep.summary.bbox_max),
        offset,
    ))
}

fn offset_face_margin(faces: &[BrepFace]) -> Option<f64> {
    faces
        .iter()
        .filter_map(|face| match face.ported_face_surface {
            Some(PortedFaceSurface::Offset(surface)) => Some(surface.payload.offset_value.abs()),
            _ => None,
        })
        .reduce(f64::max)
}

fn offset_brep_margin(brep: &BrepShape) -> Option<f64> {
    offset_face_margin(&brep.faces)
}

fn face_bboxes_occt(context: &Context, face_shapes: &[Shape]) -> Option<([f64; 3], [f64; 3])> {
    let mut bbox = None;
    for face_shape in face_shapes {
        let summary = context.describe_shape_occt(face_shape).ok()?;
        let face_bbox = (summary.bbox_min, summary.bbox_max);
        bbox = Some(match bbox {
            Some(accumulated) => union_bbox(accumulated, face_bbox),
            None => face_bbox,
        });
    }
    bbox
}

fn expand_bbox(bbox: ([f64; 3], [f64; 3]), margin: f64) -> ([f64; 3], [f64; 3]) {
    let mut min = bbox.0;
    let mut max = bbox.1;
    for axis in 0..3 {
        min[axis] -= margin;
        max[axis] += margin;
    }
    (min, max)
}

fn boundary_shape_bbox(
    vertices: &[BrepVertex],
    edges: &[BrepEdge],
) -> Option<([f64; 3], [f64; 3])> {
    analytic_edges_bbox(edges)
        .or_else(|| line_segment_points_bbox(vertices, edges))
        .or_else(|| {
            if edges.is_empty() {
                bbox_from_points(vertices.iter().map(|vertex| vertex.position).collect())
            } else {
                None
            }
        })
}

fn faces_use_analytic_edge_bbox(edges: &[BrepEdge], faces: &[BrepFace]) -> bool {
    !edges.is_empty()
        && edges
            .iter()
            .all(|edge| matches!(edge.ported_curve, Some(_)))
        && faces.iter().all(|face| {
            matches!(
                face.ported_face_surface,
                Some(
                    PortedFaceSurface::Analytic(
                        PortedSurface::Plane(_)
                            | PortedSurface::Cylinder(_)
                            | PortedSurface::Cone(_)
                    ) | PortedFaceSurface::Swept(_)
                )
            )
        })
}

fn offset_shape_bbox_occt(
    context: &Context,
    shape: &Shape,
    faces: &[BrepFace],
    vertex_shapes: &[Shape],
    face_shapes: &[Shape],
    edge_shapes: &[Shape],
) -> Option<([f64; 3], [f64; 3])> {
    if faces.is_empty()
        || face_shapes.is_empty()
        || !faces
            .iter()
            .any(|face| matches!(face.ported_face_surface, Some(PortedFaceSurface::Offset(_))))
    {
        return None;
    }

    let bbox = union_shape_bboxes_occt(
        context,
        vertex_shapes
            .iter()
            .chain(face_shapes.iter())
            .chain(edge_shapes.iter()),
    )?;
    match shape_bbox_occt(context, shape) {
        Some(shape_occt_bbox) if bbox_matches(bbox, shape_occt_bbox) => Some(bbox),
        Some(_) => None,
        None => Some(bbox),
    }
}

fn offset_solid_shell_bbox(
    context: &Context,
    faces: &[BrepFace],
    prepared_shell_shapes: &[PreparedShellShape],
) -> Option<([f64; 3], [f64; 3])> {
    if prepared_shell_shapes.is_empty()
        || !faces
            .iter()
            .any(|face| matches!(face.ported_face_surface, Some(PortedFaceSurface::Offset(_))))
    {
        return None;
    }

    let mut bbox = None;
    for prepared_shell_shape in prepared_shell_shapes
        .iter()
        .filter(|prepared_shell_shape| !prepared_shell_shape.shell_face_shapes.is_empty())
    {
        let shell_bbox = offset_shell_bbox(context, prepared_shell_shape)?;
        bbox = Some(match bbox {
            Some(accumulated) => union_bbox(accumulated, shell_bbox),
            None => shell_bbox,
        });
    }
    bbox
}

fn offset_shell_bbox(
    context: &Context,
    prepared_shell_shape: &PreparedShellShape,
) -> Option<([f64; 3], [f64; 3])> {
    let shell_occt_bbox = shape_bbox_occt(context, &prepared_shell_shape.shell_shape)?;
    offset_shell_face_brep_bbox(context, prepared_shell_shape)
        .filter(|&ported_bbox| bbox_matches(ported_bbox, shell_occt_bbox))
        .or_else(|| validated_shell_boundary_bbox(context, prepared_shell_shape, shell_occt_bbox))
        .or_else(|| validated_shell_mesh_bbox(context, prepared_shell_shape, shell_occt_bbox))
        .or_else(|| validated_shell_brep_bbox(context, prepared_shell_shape, shell_occt_bbox))
        .or(Some(shell_occt_bbox))
}

fn offset_shell_face_brep_bbox(
    context: &Context,
    prepared_shell_shape: &PreparedShellShape,
) -> Option<([f64; 3], [f64; 3])> {
    let mut bbox = None;
    for face_shape in &prepared_shell_shape.shell_face_shapes {
        let face_bbox = validated_face_brep_bbox(context, face_shape)?;
        bbox = Some(match bbox {
            Some(accumulated) => union_bbox(accumulated, face_bbox),
            None => face_bbox,
        });
    }

    context
        .subshape_count_occt(&prepared_shell_shape.shell_shape, ShapeKind::Face)
        .ok()
        .filter(|&count| count == prepared_shell_shape.shell_face_shapes.len())?;
    bbox
}

fn validated_shell_brep_bbox(
    context: &Context,
    prepared_shell_shape: &PreparedShellShape,
    shell_occt_bbox: ([f64; 3], [f64; 3]),
) -> Option<([f64; 3], [f64; 3])> {
    let brep = context
        .ported_brep(&prepared_shell_shape.shell_shape)
        .ok()?;
    let summary_bbox = (brep.summary.bbox_min, brep.summary.bbox_max);
    if bbox_matches(summary_bbox, shell_occt_bbox) {
        return Some(summary_bbox);
    }

    if let Some(expanded_bbox) = offset_expanded_brep_bbox(&brep) {
        if bbox_matches(expanded_bbox, shell_occt_bbox) {
            return Some(expanded_bbox);
        }
    }

    None
}

fn validated_shell_boundary_bbox(
    context: &Context,
    prepared_shell_shape: &PreparedShellShape,
    shell_occt_bbox: ([f64; 3], [f64; 3]),
) -> Option<([f64; 3], [f64; 3])> {
    let boundary_bbox = shell_boundary_shape_bbox(context, prepared_shell_shape)?;
    bbox_matches(boundary_bbox, shell_occt_bbox).then_some(boundary_bbox)
}

fn validated_shell_mesh_bbox(
    context: &Context,
    prepared_shell_shape: &PreparedShellShape,
    shell_occt_bbox: ([f64; 3], [f64; 3]),
) -> Option<([f64; 3], [f64; 3])> {
    let offset_margin = context
        .ported_brep(&prepared_shell_shape.shell_shape)
        .ok()
        .and_then(|brep| offset_brep_margin(&brep));
    validated_mesh_bbox(
        context,
        &prepared_shell_shape.shell_shape,
        shell_occt_bbox,
        offset_margin,
    )
}

fn validated_mesh_bbox(
    context: &Context,
    shape: &Shape,
    expected_bbox: ([f64; 3], [f64; 3]),
    expand_margin: Option<f64>,
) -> Option<([f64; 3], [f64; 3])> {
    let mesh_bbox = mesh_shape_bbox(context, shape)?;
    if bbox_matches(mesh_bbox, expected_bbox) {
        return Some(mesh_bbox);
    }

    let expanded_bbox = expand_margin.map(|margin| expand_bbox(mesh_bbox, margin))?;
    bbox_matches(expanded_bbox, expected_bbox).then_some(expanded_bbox)
}

fn shell_boundary_shape_bbox(
    context: &Context,
    prepared_shell_shape: &PreparedShellShape,
) -> Option<([f64; 3], [f64; 3])> {
    let mut bbox = vertex_shapes_bbox(context, &prepared_shell_shape.shell_vertex_shapes);
    for edge_shape in &prepared_shell_shape.shell_edge_shapes {
        let Some(edge_bbox) = boundary_edge_shape_bbox(context, edge_shape) else {
            continue;
        };
        bbox = Some(match bbox {
            Some(accumulated) => union_bbox(accumulated, edge_bbox),
            None => edge_bbox,
        });
    }
    bbox
}

fn shape_bbox_occt(context: &Context, shape: &Shape) -> Option<([f64; 3], [f64; 3])> {
    context
        .describe_shape_occt(shape)
        .ok()
        .map(|summary| (summary.bbox_min, summary.bbox_max))
}

fn bbox_matches(lhs: ([f64; 3], [f64; 3]), rhs: ([f64; 3], [f64; 3])) -> bool {
    lhs.0
        .iter()
        .chain(lhs.1.iter())
        .zip(rhs.0.iter().chain(rhs.1.iter()))
        .all(|(lhs_coordinate, rhs_coordinate)| {
            approx_eq(*lhs_coordinate, *rhs_coordinate, 1.0e-6, 1.0e-6)
        })
}

fn union_shape_bboxes_occt<'a, I>(context: &Context, shapes: I) -> Option<([f64; 3], [f64; 3])>
where
    I: IntoIterator<Item = &'a Shape>,
{
    let mut bbox_min = [f64::INFINITY; 3];
    let mut bbox_max = [f64::NEG_INFINITY; 3];
    let mut any_shapes = false;
    for shape in shapes {
        any_shapes = true;
        let shape_bbox = shape_bbox_occt(context, shape)?;
        for coordinate in 0..3 {
            bbox_min[coordinate] = bbox_min[coordinate].min(shape_bbox.0[coordinate]);
            bbox_max[coordinate] = bbox_max[coordinate].max(shape_bbox.1[coordinate]);
        }
    }

    if any_shapes
        && bbox_min
            .iter()
            .zip(bbox_max.iter())
            .all(|(min, max)| min.is_finite() && max.is_finite())
    {
        Some((bbox_min, bbox_max))
    } else {
        None
    }
}

fn line_segment_points_bbox(
    vertices: &[BrepVertex],
    edges: &[BrepEdge],
) -> Option<([f64; 3], [f64; 3])> {
    if edges.is_empty()
        || !edges.iter().all(|edge| {
            matches!(edge.ported_curve, Some(PortedCurve::Line(_)))
                && edge.start_point.is_some()
                && edge.end_point.is_some()
        })
    {
        return None;
    }

    let mut points = vertices
        .iter()
        .map(|vertex| vertex.position)
        .collect::<Vec<_>>();
    for edge in edges {
        if let Some(start_point) = edge.start_point {
            points.push(start_point);
        }
        if let Some(end_point) = edge.end_point {
            points.push(end_point);
        }
    }

    bbox_from_points(points)
}

fn analytic_edges_bbox(edges: &[BrepEdge]) -> Option<([f64; 3], [f64; 3])> {
    let mut bbox = None;
    for edge in edges {
        let edge_bbox = analytic_edge_bbox(edge)?;
        bbox = Some(match bbox {
            Some(accumulated) => union_bbox(accumulated, edge_bbox),
            None => edge_bbox,
        });
    }
    bbox
}

fn analytic_edge_bbox(edge: &BrepEdge) -> Option<([f64; 3], [f64; 3])> {
    ported_curve_bbox(
        edge.ported_curve?,
        edge.geometry.start_parameter,
        edge.geometry.end_parameter,
    )
}

fn analytic_edge_shape_bbox(context: &Context, edge_shape: &Shape) -> Option<([f64; 3], [f64; 3])> {
    let geometry = context.edge_geometry(edge_shape).ok()?;
    let curve =
        PortedCurve::from_context_with_ported_payloads(context, edge_shape, geometry).ok()??;
    ported_curve_bbox(curve, geometry.start_parameter, geometry.end_parameter)
}

fn boundary_edge_shape_bbox(context: &Context, edge_shape: &Shape) -> Option<([f64; 3], [f64; 3])> {
    analytic_edge_shape_bbox(context, edge_shape)
        .or_else(|| line_segment_edge_shape_bbox(context, edge_shape))
        .or_else(|| sampled_edge_shape_bbox(context, edge_shape))
}

fn ported_curve_bbox(curve: PortedCurve, start: f64, end: f64) -> Option<([f64; 3], [f64; 3])> {
    match curve {
        PortedCurve::Line(_) => bbox_from_points(vec![
            curve.evaluate(start).position,
            curve.evaluate(end).position,
        ]),
        PortedCurve::Circle(payload) => periodic_curve_bbox(
            start,
            end,
            2.0 * PI,
            |axis| {
                (
                    payload.center[axis],
                    payload.radius * payload.x_direction[axis],
                    payload.radius * payload.y_direction[axis],
                )
            },
            |parameter| curve.evaluate(parameter).position,
        ),
        PortedCurve::Ellipse(payload) => periodic_curve_bbox(
            start,
            end,
            2.0 * PI,
            |axis| {
                (
                    payload.center[axis],
                    payload.major_radius * payload.x_direction[axis],
                    payload.minor_radius * payload.y_direction[axis],
                )
            },
            |parameter| curve.evaluate(parameter).position,
        ),
    }
}

fn vertex_shapes_bbox(context: &Context, vertex_shapes: &[Shape]) -> Option<([f64; 3], [f64; 3])> {
    bbox_from_points(
        vertex_shapes
            .iter()
            .map(|vertex_shape| context.vertex_point(vertex_shape).ok())
            .collect::<Option<Vec<_>>>()?,
    )
}

fn line_segment_edge_shape_bbox(
    context: &Context,
    edge_shape: &Shape,
) -> Option<([f64; 3], [f64; 3])> {
    let geometry = context.edge_geometry(edge_shape).ok()?;
    if !matches!(geometry.kind, crate::CurveKind::Line) {
        return None;
    }

    let endpoints = context.edge_endpoints(edge_shape).ok()?;
    bbox_from_points(vec![endpoints.start, endpoints.end])
}

fn sampled_edge_shape_bbox(context: &Context, edge_shape: &Shape) -> Option<([f64; 3], [f64; 3])> {
    let geometry = context.edge_geometry(edge_shape).ok()?;
    let (sample_count, refinement_depth) = boundary_edge_sampling_plan(geometry.kind)?;
    let mut points = Vec::with_capacity(sample_count);
    let mut samples = Vec::with_capacity(sample_count);
    for sample_index in 0..sample_count {
        let t = sample_index as f64 / (sample_count.saturating_sub(1)) as f64;
        let sample = context.edge_sample(edge_shape, t).ok()?;
        points.push(sample.position);
        samples.push(NormalizedEdgeSample { t, sample });
    }

    let coarse_samples = samples.clone();
    for window in coarse_samples.windows(2) {
        refine_sampled_edge_interval(
            context,
            edge_shape,
            &window[0],
            &window[1],
            refinement_depth,
            &mut points,
            &mut samples,
        )?;
    }

    samples.sort_by(|lhs, rhs| lhs.t.total_cmp(&rhs.t));
    append_axis_turning_edge_samples(context, edge_shape, &samples, &mut points)?;
    append_near_flat_axis_edge_samples(context, edge_shape, &samples, &mut points)?;
    append_axis_position_extremum_samples(context, edge_shape, &samples, &mut points)?;
    append_seeded_axis_position_extremum_samples(context, edge_shape, &samples, &mut points)?;

    bbox_from_points(points)
}

#[derive(Clone, Copy)]
struct NormalizedEdgeSample {
    t: f64,
    sample: EdgeSample,
}

#[derive(Clone, Copy)]
enum AxisExtremumKind {
    Minimum,
    Maximum,
}

fn boundary_edge_sampling_plan(kind: crate::CurveKind) -> Option<(usize, usize)> {
    match kind {
        crate::CurveKind::Line | crate::CurveKind::Circle | crate::CurveKind::Ellipse => None,
        crate::CurveKind::Hyperbola | crate::CurveKind::Parabola => Some((33, 4)),
        crate::CurveKind::Bezier
        | crate::CurveKind::BSpline
        | crate::CurveKind::Offset
        | crate::CurveKind::Other
        | crate::CurveKind::Unknown => Some((65, 4)),
    }
}

fn refine_sampled_edge_interval(
    context: &Context,
    edge_shape: &Shape,
    start: &NormalizedEdgeSample,
    end: &NormalizedEdgeSample,
    remaining_depth: usize,
    points: &mut Vec<[f64; 3]>,
    samples: &mut Vec<NormalizedEdgeSample>,
) -> Option<()> {
    if remaining_depth == 0 {
        return Some(());
    }

    let midpoint_t = 0.5 * (start.t + end.t);
    if approx_eq(midpoint_t, start.t, 1.0e-12, 1.0e-12)
        || approx_eq(midpoint_t, end.t, 1.0e-12, 1.0e-12)
    {
        return Some(());
    }

    let midpoint_sample = NormalizedEdgeSample {
        t: midpoint_t,
        sample: context.edge_sample(edge_shape, midpoint_t).ok()?,
    };

    let needs_refinement = if sampled_edge_interval_needs_refinement(start, &midpoint_sample, end) {
        true
    } else {
        sampled_edge_interval_needs_probe_refinement(
            context,
            edge_shape,
            start,
            &midpoint_sample,
            end,
        )?
    };

    if !needs_refinement {
        return Some(());
    }

    points.push(midpoint_sample.sample.position);
    samples.push(midpoint_sample);
    refine_sampled_edge_interval(
        context,
        edge_shape,
        start,
        &midpoint_sample,
        remaining_depth - 1,
        points,
        samples,
    )?;
    refine_sampled_edge_interval(
        context,
        edge_shape,
        &midpoint_sample,
        end,
        remaining_depth - 1,
        points,
        samples,
    )
}

fn sampled_edge_interval_needs_probe_refinement(
    context: &Context,
    edge_shape: &Shape,
    start: &NormalizedEdgeSample,
    midpoint: &NormalizedEdgeSample,
    end: &NormalizedEdgeSample,
) -> Option<bool> {
    EARLY_PROBE_REFINEMENT_STAGES.needs_refinement(context, edge_shape, start, midpoint, end)
}

#[derive(Clone, Copy)]
struct MidpointEdgeProbeSpanLayout {
    start: usize,
    end: usize,
}

impl MidpointEdgeProbeSpanLayout {
    const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    fn probe<const SOURCE_N: usize>(
        self,
        context: &Context,
        edge_shape: &Shape,
        source: &[NormalizedEdgeSample; SOURCE_N],
    ) -> Option<MidpointEdgeProbeOutcome> {
        midpoint_edge_probe(context, edge_shape, &source[self.start], &source[self.end])
    }
}

#[derive(Clone, Copy)]
struct MidpointEdgeProbePairRequestLayout {
    first: MidpointEdgeProbeSpanLayout,
    second: MidpointEdgeProbeSpanLayout,
}

impl MidpointEdgeProbePairRequestLayout {
    const fn new(first: MidpointEdgeProbeSpanLayout, second: MidpointEdgeProbeSpanLayout) -> Self {
        Self { first, second }
    }

    fn probe_pair<const SOURCE_N: usize>(
        self,
        context: &Context,
        edge_shape: &Shape,
        source: [NormalizedEdgeSample; SOURCE_N],
    ) -> Option<MidpointEdgeProbePairOutcome> {
        let first_probe = self.first.probe(context, edge_shape, &source)?;
        let second_probe = self.second.probe(context, edge_shape, &source)?;
        Some(first_probe.pair_with(second_probe))
    }
}

#[derive(Clone, Copy)]
struct EarlyProbeStageLayout<const SOURCE_N: usize, const STAGE_N: usize> {
    probe_request_layout: MidpointEdgeProbePairRequestLayout,
    sample_roles: [EarlyProbeSampleRole; STAGE_N],
}

impl<const SOURCE_N: usize, const STAGE_N: usize> EarlyProbeStageLayout<SOURCE_N, STAGE_N> {
    const fn new(
        probe_request_layout: MidpointEdgeProbePairRequestLayout,
        sample_roles: [EarlyProbeSampleRole; STAGE_N],
    ) -> Self {
        Self {
            probe_request_layout,
            sample_roles,
        }
    }

    fn stage_progress(
        self,
        context: &Context,
        edge_shape: &Shape,
        source: [NormalizedEdgeSample; SOURCE_N],
    ) -> Option<ControlFlow<bool, [NormalizedEdgeSample; STAGE_N]>> {
        let probes = self
            .probe_request_layout
            .probe_pair(context, edge_shape, source)?;
        Some(match probes.refinement_result(source, self.sample_roles) {
            Ok(samples) => ControlFlow::Continue(samples),
            Err(result) => ControlFlow::Break(result),
        })
    }
}

#[derive(Clone, Copy)]
enum EarlyProbeSourcePosition {
    First,
    Second,
    Third,
    Fourth,
    Fifth,
}

impl EarlyProbeSourcePosition {
    fn index(self) -> usize {
        match self {
            Self::First => 0,
            Self::Second => 1,
            Self::Third => 2,
            Self::Fourth => 3,
            Self::Fifth => 4,
        }
    }
}

#[derive(Clone, Copy)]
enum EarlyProbeSampleRole {
    Source(EarlyProbeSourcePosition),
    FirstProbe,
    SecondProbe,
}

impl EarlyProbeSampleRole {
    fn stage_sample<const SOURCE_N: usize>(
        self,
        source: [NormalizedEdgeSample; SOURCE_N],
        probes: &MidpointEdgeProbePair,
    ) -> NormalizedEdgeSample {
        match self {
            Self::Source(position) => source[position.index()],
            Self::FirstProbe => probes.first_probe,
            Self::SecondProbe => probes.second_probe,
        }
    }
}

impl EarlyProbeSampleRole {
    fn stage_samples<const SOURCE_N: usize, const STAGE_N: usize>(
        roles: [EarlyProbeSampleRole; STAGE_N],
        source: [NormalizedEdgeSample; SOURCE_N],
        probes: &MidpointEdgeProbePair,
    ) -> [NormalizedEdgeSample; STAGE_N] {
        roles.map(|role| role.stage_sample(source, probes))
    }
}

#[derive(Clone, Copy)]
struct EarlyProbeRefinementStages {
    midpoint_stage: EarlyProbeStageLayout<3, 5>,
    outer_stage: EarlyProbeStageLayout<5, 7>,
    interval_aware_side_layouts: PreparedIntervalAwareRefinementSideLayouts,
    coarse_refinement_checks_before_adaptive_chase: usize,
}

impl EarlyProbeRefinementStages {
    const fn new(
        midpoint_stage: EarlyProbeStageLayout<3, 5>,
        outer_stage: EarlyProbeStageLayout<5, 7>,
        interval_aware_side_layouts: PreparedIntervalAwareRefinementSideLayouts,
        coarse_refinement_checks_before_adaptive_chase: usize,
    ) -> Self {
        Self {
            midpoint_stage,
            outer_stage,
            interval_aware_side_layouts,
            coarse_refinement_checks_before_adaptive_chase,
        }
    }

    fn stage_progress(
        self,
        context: &Context,
        edge_shape: &Shape,
        source: [NormalizedEdgeSample; 3],
    ) -> Option<ControlFlow<bool, [NormalizedEdgeSample; 7]>> {
        Some(
            match self
                .midpoint_stage
                .stage_progress(context, edge_shape, source)?
            {
                ControlFlow::Continue(samples) => self
                    .outer_stage
                    .stage_progress(context, edge_shape, samples)?,
                ControlFlow::Break(result) => ControlFlow::Break(result),
            },
        )
    }

    fn needs_refinement(
        self,
        context: &Context,
        edge_shape: &Shape,
        start: &NormalizedEdgeSample,
        midpoint: &NormalizedEdgeSample,
        end: &NormalizedEdgeSample,
    ) -> Option<bool> {
        self.interval_aware_side_layouts.needs_refinement(
            self.stage_progress(context, edge_shape, [*start, *midpoint, *end])?,
            context,
            edge_shape,
            self.coarse_refinement_checks_before_adaptive_chase,
        )
    }
}

impl PreparedIntervalAwareRefinementSideLayouts {
    fn needs_refinement(
        self,
        stage_progress: ControlFlow<bool, [NormalizedEdgeSample; 7]>,
        context: &Context,
        edge_shape: &Shape,
        coarse_refinement_checks_before_adaptive_chase: usize,
    ) -> Option<bool> {
        let samples = match stage_progress {
            ControlFlow::Continue(samples) => samples,
            ControlFlow::Break(result) => return Some(result),
        };
        let Some((layout, _)) = RefinementSegmentOutcome::choose_stronger_with(
            (self.left, self.left.coarse.refinement_segment(&samples)),
            (self.right, self.right.coarse.refinement_segment(&samples)),
        ) else {
            return Some(false);
        };
        let outer_segment = layout.outer.refinement_segment(&samples);
        let inner_segment = layout
            .inner
            .midpoint_segment(&samples, context, edge_shape)?;
        let Some(probe_segment) =
            RefinementSegmentOutcome::choose_stronger(outer_segment, inner_segment)
        else {
            return Some(false);
        };

        probe_segment.needs_refinement(
            context,
            edge_shape,
            coarse_refinement_checks_before_adaptive_chase,
        )
    }
}

#[derive(Clone, Copy)]
struct PreparedRefinementTripletLayout {
    start: usize,
    end: usize,
    midpoint: usize,
}

impl PreparedRefinementTripletLayout {
    const fn new(start: usize, midpoint: usize, end: usize) -> Self {
        Self {
            start,
            end,
            midpoint,
        }
    }

    fn refinement_segment(self, samples: &[NormalizedEdgeSample; 7]) -> RefinementSegmentOutcome {
        RefinementSegmentOutcome::from_samples(
            &samples[self.start],
            &samples[self.midpoint],
            &samples[self.end],
        )
    }
}

#[derive(Clone, Copy)]
enum RefinementSegmentOutcome {
    NoSegment,
    Segment(RefinementSegment),
}

impl RefinementSegmentOutcome {
    fn from_samples(
        start: &NormalizedEdgeSample,
        midpoint: &NormalizedEdgeSample,
        end: &NormalizedEdgeSample,
    ) -> Self {
        match RefinementSegment::new(start, midpoint, end) {
            Some(segment) => Self::Segment(segment),
            None => Self::NoSegment,
        }
    }

    fn choose_stronger_with<T: Copy>(
        first: (T, Self),
        second: (T, Self),
    ) -> Option<(T, RefinementSegment)> {
        match (first, second) {
            (
                (first_value, Self::Segment(first_segment)),
                (second_value, Self::Segment(second_segment)),
            ) => {
                if first_segment.score >= second_segment.score {
                    Some((first_value, first_segment))
                } else {
                    Some((second_value, second_segment))
                }
            }
            ((first_value, Self::Segment(first_segment)), (_, Self::NoSegment)) => {
                Some((first_value, first_segment))
            }
            ((_, Self::NoSegment), (second_value, Self::Segment(second_segment))) => {
                Some((second_value, second_segment))
            }
            ((_, Self::NoSegment), (_, Self::NoSegment)) => None,
        }
    }

    fn choose_stronger(first: Self, second: Self) -> Option<RefinementSegment> {
        Self::choose_stronger_with(((), first), ((), second)).map(|(_, segment)| segment)
    }
}

#[derive(Clone, Copy)]
enum EdgeSampleExtremumOutcome {
    NoSample,
    Sample(EdgeSample),
}

#[derive(Clone, Copy)]
enum MidpointEdgeProbeOutcome {
    NoProbe,
    Probe(NormalizedEdgeSample),
}

impl MidpointEdgeProbeOutcome {
    fn refinement_segment(
        self,
        start: &NormalizedEdgeSample,
        end: &NormalizedEdgeSample,
    ) -> RefinementSegmentOutcome {
        match self {
            MidpointEdgeProbeOutcome::NoProbe => RefinementSegmentOutcome::NoSegment,
            MidpointEdgeProbeOutcome::Probe(probe) => {
                RefinementSegmentOutcome::from_samples(start, &probe, end)
            }
        }
    }

    fn pair_with(self, other: Self) -> MidpointEdgeProbePairOutcome {
        match (self, other) {
            (
                MidpointEdgeProbeOutcome::Probe(first_probe),
                MidpointEdgeProbeOutcome::Probe(second_probe),
            ) => MidpointEdgeProbePairOutcome::Pair(MidpointEdgeProbePair {
                first_probe,
                second_probe,
            }),
            _ => MidpointEdgeProbePairOutcome::NoPair,
        }
    }
}

#[derive(Clone, Copy)]
struct PreparedRefinementSpanLayout {
    start: usize,
    end: usize,
}

impl PreparedRefinementSpanLayout {
    const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    fn midpoint_segment(
        self,
        samples: &[NormalizedEdgeSample; 7],
        context: &Context,
        edge_shape: &Shape,
    ) -> Option<RefinementSegmentOutcome> {
        let start = samples[self.start];
        let end = samples[self.end];
        midpoint_refinement_segment(context, edge_shape, &start, &end)
    }
}

#[derive(Clone, Copy)]
struct PreparedIntervalAwareRefinementSideLayout {
    coarse: PreparedRefinementTripletLayout,
    outer: PreparedRefinementTripletLayout,
    inner: PreparedRefinementSpanLayout,
}

impl PreparedIntervalAwareRefinementSideLayout {
    const fn new(
        coarse: PreparedRefinementTripletLayout,
        outer: PreparedRefinementTripletLayout,
        inner: PreparedRefinementSpanLayout,
    ) -> Self {
        Self {
            coarse,
            outer,
            inner,
        }
    }
}

#[derive(Clone, Copy)]
struct PreparedIntervalAwareRefinementSideLayouts {
    left: PreparedIntervalAwareRefinementSideLayout,
    right: PreparedIntervalAwareRefinementSideLayout,
}

impl PreparedIntervalAwareRefinementSideLayouts {
    const fn new(
        left: PreparedIntervalAwareRefinementSideLayout,
        right: PreparedIntervalAwareRefinementSideLayout,
    ) -> Self {
        Self { left, right }
    }
}

const PREPARED_INTERVAL_AWARE_REFINEMENT_SIDE_LAYOUTS: PreparedIntervalAwareRefinementSideLayouts =
    PreparedIntervalAwareRefinementSideLayouts::new(
        PreparedIntervalAwareRefinementSideLayout::new(
            PreparedRefinementTripletLayout::new(0, 2, 3),
            PreparedRefinementTripletLayout::new(0, 1, 2),
            PreparedRefinementSpanLayout::new(2, 3),
        ),
        PreparedIntervalAwareRefinementSideLayout::new(
            PreparedRefinementTripletLayout::new(3, 4, 6),
            PreparedRefinementTripletLayout::new(4, 5, 6),
            PreparedRefinementSpanLayout::new(3, 4),
        ),
    );

const HALF_REFINEMENT_MAX_STEPS: usize = 32;
const HALF_REFINEMENT_RELATIVE_SCORE_FLOOR: f64 = 0.05;
const HALF_REFINEMENT_ABSOLUTE_SCORE_FLOOR: f64 = 2.5e-4;
const HALF_REFINEMENT_RELATIVE_T_SPAN_FLOOR: f64 = 1.0 / 16384.0;
const HALF_REFINEMENT_ABSOLUTE_T_SPAN_FLOOR: f64 = 1.0e-6;
const HALF_REFINEMENT_RELATIVE_CHORD_FLOOR: f64 = 1.0 / 16384.0;
const HALF_REFINEMENT_ABSOLUTE_CHORD_FLOOR: f64 = 1.0e-6;

#[derive(Clone, Copy)]
struct RefinementSegment {
    start: NormalizedEdgeSample,
    midpoint: NormalizedEdgeSample,
    end: NormalizedEdgeSample,
    score: f64,
}

impl RefinementSegment {
    fn new(
        start: &NormalizedEdgeSample,
        midpoint: &NormalizedEdgeSample,
        end: &NormalizedEdgeSample,
    ) -> Option<Self> {
        let score = sampled_edge_interval_refinement_signal_strength(start, midpoint, end);
        if score <= 1.0e-12 {
            None
        } else {
            Some(Self {
                start: *start,
                midpoint: *midpoint,
                end: *end,
                score,
            })
        }
    }

    fn needs_refinement(
        &self,
        context: &Context,
        edge_shape: &Shape,
        coarse_refinement_checks_before_adaptive_chase: usize,
    ) -> Option<bool> {
        if self.needs_local_refinement() {
            return Some(true);
        }

        self.needs_stronger_half_refinement(
            context,
            edge_shape,
            coarse_refinement_checks_before_adaptive_chase,
        )
    }

    fn needs_local_refinement(&self) -> bool {
        sampled_edge_interval_needs_refinement(&self.start, &self.midpoint, &self.end)
    }

    fn needs_stronger_half_refinement(
        &self,
        context: &Context,
        edge_shape: &Shape,
        coarse_refinement_checks_before_adaptive_chase: usize,
    ) -> Option<bool> {
        let mut adaptive_probe = *self;

        for _ in 0..coarse_refinement_checks_before_adaptive_chase {
            let RefinementSegmentOutcome::Segment(probe) =
                adaptive_probe.stronger_half(context, edge_shape)?
            else {
                return Some(false);
            };

            if probe.needs_local_refinement() {
                return Some(true);
            }

            adaptive_probe = probe;
        }

        let RefinementSegmentOutcome::Segment(mut probe) =
            adaptive_probe.stronger_half(context, edge_shape)?
        else {
            return Some(false);
        };

        let initial_score = probe.score;
        let initial_t_span = (probe.end.t - probe.start.t).abs();
        let initial_chord_length = norm3(subtract3(
            probe.end.sample.position,
            probe.start.sample.position,
        ));
        let mut refinement_steps = 1;

        while half_refinement_should_continue(
            &probe,
            initial_score,
            initial_t_span,
            initial_chord_length,
            refinement_steps,
        ) {
            let RefinementSegmentOutcome::Segment(next_probe) =
                probe.stronger_half(context, edge_shape)?
            else {
                break;
            };
            probe = next_probe;
            refinement_steps += 1;
        }

        Some(probe.needs_local_refinement())
    }

    fn stronger_half(
        &self,
        context: &Context,
        edge_shape: &Shape,
    ) -> Option<RefinementSegmentOutcome> {
        Some(
            RefinementSegmentOutcome::choose_stronger(
                midpoint_refinement_segment(context, edge_shape, &self.start, &self.midpoint)?,
                midpoint_refinement_segment(context, edge_shape, &self.midpoint, &self.end)?,
            )
            .map_or(
                RefinementSegmentOutcome::NoSegment,
                RefinementSegmentOutcome::Segment,
            ),
        )
    }
}

fn half_refinement_should_continue(
    probe: &RefinementSegment,
    initial_score: f64,
    initial_t_span: f64,
    initial_chord_length: f64,
    refinement_steps: usize,
) -> bool {
    if refinement_steps >= HALF_REFINEMENT_MAX_STEPS {
        return false;
    }

    let signal_floor = HALF_REFINEMENT_ABSOLUTE_SCORE_FLOOR
        .max(HALF_REFINEMENT_RELATIVE_SCORE_FLOOR * initial_score);
    if probe.score <= signal_floor {
        return false;
    }

    let current_t_span = (probe.end.t - probe.start.t).abs();
    let current_chord_length = norm3(subtract3(
        probe.end.sample.position,
        probe.start.sample.position,
    ));
    let t_span_floor = HALF_REFINEMENT_ABSOLUTE_T_SPAN_FLOOR
        .max(HALF_REFINEMENT_RELATIVE_T_SPAN_FLOOR * initial_t_span);
    let chord_floor = HALF_REFINEMENT_ABSOLUTE_CHORD_FLOOR
        .max(HALF_REFINEMENT_RELATIVE_CHORD_FLOOR * initial_chord_length);

    current_t_span > t_span_floor || current_chord_length > chord_floor
}

fn midpoint_edge_probe(
    context: &Context,
    edge_shape: &Shape,
    start: &NormalizedEdgeSample,
    end: &NormalizedEdgeSample,
) -> Option<MidpointEdgeProbeOutcome> {
    let probe_t = 0.5 * (start.t + end.t);
    if approx_eq(probe_t, start.t, 1.0e-12, 1.0e-12) || approx_eq(probe_t, end.t, 1.0e-12, 1.0e-12)
    {
        return Some(MidpointEdgeProbeOutcome::NoProbe);
    }

    Some(MidpointEdgeProbeOutcome::Probe(NormalizedEdgeSample {
        t: probe_t,
        sample: context.edge_sample(edge_shape, probe_t).ok()?,
    }))
}

fn midpoint_refinement_segment(
    context: &Context,
    edge_shape: &Shape,
    start: &NormalizedEdgeSample,
    end: &NormalizedEdgeSample,
) -> Option<RefinementSegmentOutcome> {
    Some(midpoint_edge_probe(context, edge_shape, start, end)?.refinement_segment(start, end))
}

#[derive(Clone, Copy)]
struct MidpointEdgeProbePair {
    first_probe: NormalizedEdgeSample,
    second_probe: NormalizedEdgeSample,
}

#[derive(Clone, Copy)]
enum MidpointEdgeProbePairOutcome {
    NoPair,
    Pair(MidpointEdgeProbePair),
}

impl MidpointEdgeProbePairOutcome {
    fn refinement_result<const SOURCE_N: usize, const STAGE_N: usize>(
        self,
        source: [NormalizedEdgeSample; SOURCE_N],
        sample_roles: [EarlyProbeSampleRole; STAGE_N],
    ) -> Result<[NormalizedEdgeSample; STAGE_N], bool> {
        let MidpointEdgeProbePairOutcome::Pair(probes) = self else {
            return Err(false);
        };

        let samples = EarlyProbeSampleRole::stage_samples(sample_roles, source, &probes);
        if sampled_edge_sample_windows_need_refinement(samples.as_ref()) {
            Err(true)
        } else {
            Ok(samples)
        }
    }
}

const MIDPOINT_EARLY_PROBE_STAGE_LAYOUT: EarlyProbeStageLayout<3, 5> = EarlyProbeStageLayout::new(
    MidpointEdgeProbePairRequestLayout::new(
        MidpointEdgeProbeSpanLayout::new(0, 1),
        MidpointEdgeProbeSpanLayout::new(1, 2),
    ),
    [
        EarlyProbeSampleRole::Source(EarlyProbeSourcePosition::First),
        EarlyProbeSampleRole::FirstProbe,
        EarlyProbeSampleRole::Source(EarlyProbeSourcePosition::Second),
        EarlyProbeSampleRole::SecondProbe,
        EarlyProbeSampleRole::Source(EarlyProbeSourcePosition::Third),
    ],
);

const OUTER_EARLY_PROBE_STAGE_LAYOUT: EarlyProbeStageLayout<5, 7> = EarlyProbeStageLayout::new(
    MidpointEdgeProbePairRequestLayout::new(
        MidpointEdgeProbeSpanLayout::new(0, 1),
        MidpointEdgeProbeSpanLayout::new(3, 4),
    ),
    [
        EarlyProbeSampleRole::Source(EarlyProbeSourcePosition::First),
        EarlyProbeSampleRole::FirstProbe,
        EarlyProbeSampleRole::Source(EarlyProbeSourcePosition::Second),
        EarlyProbeSampleRole::Source(EarlyProbeSourcePosition::Third),
        EarlyProbeSampleRole::Source(EarlyProbeSourcePosition::Fourth),
        EarlyProbeSampleRole::SecondProbe,
        EarlyProbeSampleRole::Source(EarlyProbeSourcePosition::Fifth),
    ],
);

const EARLY_PROBE_REFINEMENT_STAGES: EarlyProbeRefinementStages = EarlyProbeRefinementStages::new(
    MIDPOINT_EARLY_PROBE_STAGE_LAYOUT,
    OUTER_EARLY_PROBE_STAGE_LAYOUT,
    PREPARED_INTERVAL_AWARE_REFINEMENT_SIDE_LAYOUTS,
    3,
);

fn sampled_edge_sample_windows_need_refinement(samples: &[NormalizedEdgeSample]) -> bool {
    samples
        .windows(3)
        .any(|window| sampled_edge_interval_needs_refinement(&window[0], &window[1], &window[2]))
}

fn sampled_edge_interval_refinement_signal_strength(
    start: &NormalizedEdgeSample,
    midpoint: &NormalizedEdgeSample,
    end: &NormalizedEdgeSample,
) -> f64 {
    midpoint_edge_interval_bbox_expansion_amount(
        start.sample.position,
        midpoint.sample.position,
        end.sample.position,
    ) + interval_tangent_turn_strength(
        start.sample.tangent,
        midpoint.sample.tangent,
        end.sample.tangent,
    ) + interval_midpoint_axis_position_shoulder_strength(start, midpoint, end)
        + interval_midpoint_chord_bend_strength(
            start.sample.position,
            midpoint.sample.position,
            end.sample.position,
        )
}

fn midpoint_edge_interval_bbox_expansion_amount(
    start: [f64; 3],
    midpoint: [f64; 3],
    end: [f64; 3],
) -> f64 {
    (0..3)
        .map(|axis| {
            let interval_min = start[axis].min(end[axis]);
            let interval_max = start[axis].max(end[axis]);
            (interval_min - midpoint[axis])
                .max(0.0)
                .max(midpoint[axis] - interval_max)
        })
        .fold(0.0, f64::max)
}

fn interval_tangent_turn_strength(
    start_tangent: [f64; 3],
    midpoint_tangent: [f64; 3],
    end_tangent: [f64; 3],
) -> f64 {
    (0..3)
        .map(|axis| {
            let axis_values = [
                start_tangent[axis].abs(),
                midpoint_tangent[axis].abs(),
                end_tangent[axis].abs(),
            ];
            let scale = axis_values.into_iter().fold(1.0, f64::max);
            let delta = (start_tangent[axis] - midpoint_tangent[axis])
                .abs()
                .max((midpoint_tangent[axis] - end_tangent[axis]).abs())
                .max((start_tangent[axis] - end_tangent[axis]).abs());
            let sign_bonus = if tangent_sign_changes(start_tangent[axis], midpoint_tangent[axis])
                || tangent_sign_changes(midpoint_tangent[axis], end_tangent[axis])
                || tangent_sign_changes(start_tangent[axis], end_tangent[axis])
            {
                1.0
            } else {
                0.0
            };
            sign_bonus + delta / scale
        })
        .fold(0.0, f64::max)
}

fn interval_midpoint_axis_position_shoulder_strength(
    start: &NormalizedEdgeSample,
    midpoint: &NormalizedEdgeSample,
    end: &NormalizedEdgeSample,
) -> f64 {
    let interval_t_span = (end.t - start.t).abs();
    if interval_t_span <= 1.0e-12 {
        return 0.0;
    }

    let interpolation_weight = ((midpoint.t - start.t) / interval_t_span).clamp(0.0, 1.0);
    let chord_length = norm3(subtract3(end.sample.position, start.sample.position));
    (0..3)
        .map(|axis| {
            let expected_axis_value = start.sample.position[axis]
                + interpolation_weight * (end.sample.position[axis] - start.sample.position[axis]);
            let deviation = (midpoint.sample.position[axis] - expected_axis_value).abs();
            let axis_interval_span =
                (end.sample.position[axis] - start.sample.position[axis]).abs();
            let axis_scale = chord_length.max(axis_interval_span).max(1.0);
            deviation / axis_scale
        })
        .fold(0.0, f64::max)
}

fn interval_midpoint_chord_bend_strength(
    start: [f64; 3],
    midpoint: [f64; 3],
    end: [f64; 3],
) -> f64 {
    let chord = subtract3(end, start);
    let chord_length = norm3(chord);
    if chord_length <= 1.0e-12 {
        return norm3(subtract3(midpoint, start));
    }

    let start_to_midpoint = subtract3(midpoint, start);
    let projection = dot3(start_to_midpoint, chord) / dot3(chord, chord).max(1.0e-18);
    let closest_point = add3(start, scale3(chord, projection.clamp(0.0, 1.0)));
    let deviation = norm3(subtract3(midpoint, closest_point));
    deviation / chord_length.max(1.0)
}

fn append_axis_turning_edge_samples(
    context: &Context,
    edge_shape: &Shape,
    samples: &[NormalizedEdgeSample],
    points: &mut Vec<[f64; 3]>,
) -> Option<()> {
    for window in samples.windows(2) {
        for axis in 0..3 {
            let EdgeSampleExtremumOutcome::Sample(extremum_sample) =
                axis_turning_edge_sample(context, edge_shape, &window[0], &window[1], axis)?
            else {
                continue;
            };
            points.push(extremum_sample.position);
        }
    }
    Some(())
}

fn append_near_flat_axis_edge_samples(
    context: &Context,
    edge_shape: &Shape,
    samples: &[NormalizedEdgeSample],
    points: &mut Vec<[f64; 3]>,
) -> Option<()> {
    for window in samples.windows(2) {
        for axis in 0..3 {
            let EdgeSampleExtremumOutcome::Sample(extremum_sample) =
                near_flat_axis_edge_sample(context, edge_shape, &window[0], &window[1], axis)?
            else {
                continue;
            };
            points.push(extremum_sample.position);
        }
    }
    Some(())
}

fn append_axis_position_extremum_samples(
    context: &Context,
    edge_shape: &Shape,
    samples: &[NormalizedEdgeSample],
    points: &mut Vec<[f64; 3]>,
) -> Option<()> {
    for window in samples.windows(3) {
        for axis in 0..3 {
            let EdgeSampleExtremumOutcome::Sample(extremum_sample) =
                axis_position_extremum_edge_sample(
                    context, edge_shape, &window[0], &window[1], &window[2], axis,
                )?
            else {
                continue;
            };
            points.push(extremum_sample.position);
        }
    }
    Some(())
}

fn append_seeded_axis_position_extremum_samples(
    context: &Context,
    edge_shape: &Shape,
    samples: &[NormalizedEdgeSample],
    points: &mut Vec<[f64; 3]>,
) -> Option<()> {
    if samples.len() < 3 {
        return Some(());
    }

    for axis in 0..3 {
        for extremum_kind in [AxisExtremumKind::Minimum, AxisExtremumKind::Maximum] {
            let Some(seed_index) = seeded_axis_extremum_sample_index(samples, axis, extremum_kind)
            else {
                continue;
            };
            let Some((low_index, high_index)) =
                seeded_axis_extremum_sample_range(samples, seed_index, axis, extremum_kind)
            else {
                continue;
            };

            let EdgeSampleExtremumOutcome::Sample(extremum_sample) =
                seeded_axis_position_extremum_edge_sample(
                    context,
                    edge_shape,
                    &samples[low_index],
                    &samples[seed_index],
                    &samples[high_index],
                    axis,
                    extremum_kind,
                )?
            else {
                continue;
            };
            points.push(extremum_sample.position);
        }
    }
    Some(())
}

fn axis_turning_edge_sample(
    context: &Context,
    edge_shape: &Shape,
    start: &NormalizedEdgeSample,
    end: &NormalizedEdgeSample,
    axis: usize,
) -> Option<EdgeSampleExtremumOutcome> {
    if !tangent_sign_changes(start.sample.tangent[axis], end.sample.tangent[axis]) {
        return Some(EdgeSampleExtremumOutcome::NoSample);
    }

    let mut low = *start;
    let mut high = *end;
    for _ in 0..24 {
        let midpoint_t = 0.5 * (low.t + high.t);
        if approx_eq(midpoint_t, low.t, 1.0e-12, 1.0e-12)
            || approx_eq(midpoint_t, high.t, 1.0e-12, 1.0e-12)
        {
            break;
        }

        let midpoint = NormalizedEdgeSample {
            t: midpoint_t,
            sample: context.edge_sample(edge_shape, midpoint_t).ok()?,
        };
        let midpoint_tangent = midpoint.sample.tangent[axis];
        if midpoint_tangent.abs() <= 1.0e-9 {
            return Some(EdgeSampleExtremumOutcome::Sample(midpoint.sample));
        }

        if tangent_sign_changes(low.sample.tangent[axis], midpoint_tangent) {
            high = midpoint;
            continue;
        }
        if tangent_sign_changes(midpoint_tangent, high.sample.tangent[axis]) {
            low = midpoint;
            continue;
        }

        return Some(EdgeSampleExtremumOutcome::Sample(midpoint.sample));
    }

    let midpoint_t = 0.5 * (low.t + high.t);
    if approx_eq(midpoint_t, low.t, 1.0e-12, 1.0e-12)
        || approx_eq(midpoint_t, high.t, 1.0e-12, 1.0e-12)
    {
        return Some(EdgeSampleExtremumOutcome::NoSample);
    }

    Some(EdgeSampleExtremumOutcome::Sample(
        context.edge_sample(edge_shape, midpoint_t).ok()?,
    ))
}

fn near_flat_axis_edge_sample(
    context: &Context,
    edge_shape: &Shape,
    start: &NormalizedEdgeSample,
    end: &NormalizedEdgeSample,
    axis: usize,
) -> Option<EdgeSampleExtremumOutcome> {
    if tangent_sign_changes(start.sample.tangent[axis], end.sample.tangent[axis]) {
        return Some(EdgeSampleExtremumOutcome::NoSample);
    }

    let probe_fractions = [0.25, 0.5, 0.75];
    let mut probes = Vec::with_capacity(probe_fractions.len());
    for fraction in probe_fractions {
        let probe_t = start.t + (end.t - start.t) * fraction;
        if approx_eq(probe_t, start.t, 1.0e-12, 1.0e-12)
            || approx_eq(probe_t, end.t, 1.0e-12, 1.0e-12)
        {
            continue;
        }
        probes.push(NormalizedEdgeSample {
            t: probe_t,
            sample: context.edge_sample(edge_shape, probe_t).ok()?,
        });
    }

    let Some(mut best_probe) = probes.iter().copied().min_by(|lhs, rhs| {
        lhs.sample.tangent[axis]
            .abs()
            .total_cmp(&rhs.sample.tangent[axis].abs())
    }) else {
        return Some(EdgeSampleExtremumOutcome::NoSample);
    };

    if !near_flat_axis_probe_is_promising(start, best_probe, end, axis) {
        return Some(EdgeSampleExtremumOutcome::NoSample);
    }

    let mut low = *start;
    let mut high = *end;
    for _ in 0..12 {
        if best_probe.sample.tangent[axis].abs() <= 1.0e-9 {
            break;
        }

        let left_t = 0.5 * (low.t + best_probe.t);
        let right_t = 0.5 * (best_probe.t + high.t);
        if approx_eq(left_t, low.t, 1.0e-12, 1.0e-12)
            || approx_eq(left_t, best_probe.t, 1.0e-12, 1.0e-12)
            || approx_eq(right_t, best_probe.t, 1.0e-12, 1.0e-12)
            || approx_eq(right_t, high.t, 1.0e-12, 1.0e-12)
        {
            break;
        }

        let left_probe = NormalizedEdgeSample {
            t: left_t,
            sample: context.edge_sample(edge_shape, left_t).ok()?,
        };
        let right_probe = NormalizedEdgeSample {
            t: right_t,
            sample: context.edge_sample(edge_shape, right_t).ok()?,
        };

        let left_abs = left_probe.sample.tangent[axis].abs();
        let best_abs = best_probe.sample.tangent[axis].abs();
        let right_abs = right_probe.sample.tangent[axis].abs();
        if left_abs < best_abs || right_abs < best_abs {
            if left_abs <= right_abs && left_abs < best_abs {
                high = best_probe;
                best_probe = left_probe;
                continue;
            }
            if right_abs < best_abs {
                low = best_probe;
                best_probe = right_probe;
                continue;
            }
        }

        low = left_probe;
        high = right_probe;
    }

    Some(EdgeSampleExtremumOutcome::Sample(best_probe.sample))
}

fn seeded_axis_position_extremum_edge_sample(
    context: &Context,
    edge_shape: &Shape,
    start: &NormalizedEdgeSample,
    seed: &NormalizedEdgeSample,
    end: &NormalizedEdgeSample,
    axis: usize,
    extremum_kind: AxisExtremumKind,
) -> Option<EdgeSampleExtremumOutcome> {
    let mut low = *start;
    let mut high = *end;
    let mut best_probe = *seed;
    for _ in 0..12 {
        let left_t = 0.5 * (low.t + best_probe.t);
        let right_t = 0.5 * (best_probe.t + high.t);
        if approx_eq(left_t, low.t, 1.0e-12, 1.0e-12)
            || approx_eq(left_t, best_probe.t, 1.0e-12, 1.0e-12)
            || approx_eq(right_t, best_probe.t, 1.0e-12, 1.0e-12)
            || approx_eq(right_t, high.t, 1.0e-12, 1.0e-12)
        {
            break;
        }

        let left_probe = NormalizedEdgeSample {
            t: left_t,
            sample: context.edge_sample(edge_shape, left_t).ok()?,
        };
        let right_probe = NormalizedEdgeSample {
            t: right_t,
            sample: context.edge_sample(edge_shape, right_t).ok()?,
        };
        let best_value = best_probe.sample.position[axis];
        let left_value = left_probe.sample.position[axis];
        let right_value = right_probe.sample.position[axis];
        if axis_position_is_better(left_value, best_value, extremum_kind)
            || axis_position_is_better(right_value, best_value, extremum_kind)
        {
            if axis_position_is_better(left_value, right_value, extremum_kind)
                && axis_position_is_better(left_value, best_value, extremum_kind)
            {
                high = best_probe;
                best_probe = left_probe;
                continue;
            }
            if axis_position_is_better(right_value, best_value, extremum_kind) {
                low = best_probe;
                best_probe = right_probe;
                continue;
            }
        }

        low = left_probe;
        high = right_probe;
    }

    if !axis_position_probe_is_promising(seed, best_probe, axis, extremum_kind)
        || approx_eq(best_probe.t, seed.t, 1.0e-12, 1.0e-12)
    {
        return Some(EdgeSampleExtremumOutcome::NoSample);
    }

    Some(EdgeSampleExtremumOutcome::Sample(best_probe.sample))
}

fn axis_position_extremum_edge_sample(
    context: &Context,
    edge_shape: &Shape,
    start: &NormalizedEdgeSample,
    midpoint: &NormalizedEdgeSample,
    end: &NormalizedEdgeSample,
    axis: usize,
) -> Option<EdgeSampleExtremumOutcome> {
    let Some(extremum_kind) = sampled_axis_extremum_kind(
        start.sample.position[axis],
        midpoint.sample.position[axis],
        end.sample.position[axis],
    ) else {
        return Some(EdgeSampleExtremumOutcome::NoSample);
    };

    let mut low = *start;
    let mut high = *end;
    let mut best_probe = *midpoint;
    let candidate_t =
        quadratic_axis_extremum_parameter(start, midpoint, end, axis).filter(|candidate_t| {
            candidate_t.is_finite()
                && *candidate_t > start.t + 1.0e-9
                && *candidate_t < end.t - 1.0e-9
                && !approx_eq(*candidate_t, midpoint.t, 1.0e-12, 1.0e-12)
        });
    if let Some(candidate_t) = candidate_t {
        let candidate_probe = NormalizedEdgeSample {
            t: candidate_t,
            sample: context.edge_sample(edge_shape, candidate_t).ok()?,
        };
        if axis_position_is_better(
            candidate_probe.sample.position[axis],
            best_probe.sample.position[axis],
            extremum_kind,
        ) {
            best_probe = candidate_probe;
        }
    }

    if !axis_position_probe_is_promising(midpoint, best_probe, axis, extremum_kind) {
        return Some(EdgeSampleExtremumOutcome::NoSample);
    }

    for _ in 0..12 {
        let left_t = 0.5 * (low.t + best_probe.t);
        let right_t = 0.5 * (best_probe.t + high.t);
        if approx_eq(left_t, low.t, 1.0e-12, 1.0e-12)
            || approx_eq(left_t, best_probe.t, 1.0e-12, 1.0e-12)
            || approx_eq(right_t, best_probe.t, 1.0e-12, 1.0e-12)
            || approx_eq(right_t, high.t, 1.0e-12, 1.0e-12)
        {
            break;
        }

        let left_probe = NormalizedEdgeSample {
            t: left_t,
            sample: context.edge_sample(edge_shape, left_t).ok()?,
        };
        let right_probe = NormalizedEdgeSample {
            t: right_t,
            sample: context.edge_sample(edge_shape, right_t).ok()?,
        };
        let best_value = best_probe.sample.position[axis];
        let left_value = left_probe.sample.position[axis];
        let right_value = right_probe.sample.position[axis];
        if axis_position_is_better(left_value, best_value, extremum_kind)
            || axis_position_is_better(right_value, best_value, extremum_kind)
        {
            if axis_position_is_better(left_value, right_value, extremum_kind)
                && axis_position_is_better(left_value, best_value, extremum_kind)
            {
                high = best_probe;
                best_probe = left_probe;
                continue;
            }
            if axis_position_is_better(right_value, best_value, extremum_kind) {
                low = best_probe;
                best_probe = right_probe;
                continue;
            }
        }

        low = left_probe;
        high = right_probe;
    }

    if approx_eq(best_probe.t, midpoint.t, 1.0e-12, 1.0e-12) {
        return Some(EdgeSampleExtremumOutcome::NoSample);
    }

    Some(EdgeSampleExtremumOutcome::Sample(best_probe.sample))
}

fn near_flat_axis_probe_is_promising(
    start: &NormalizedEdgeSample,
    best_probe: NormalizedEdgeSample,
    end: &NormalizedEdgeSample,
    axis: usize,
) -> bool {
    let start_abs = start.sample.tangent[axis].abs();
    let end_abs = end.sample.tangent[axis].abs();
    let best_abs = best_probe.sample.tangent[axis].abs();
    if best_abs <= 1.0e-6 {
        return true;
    }

    let endpoint_floor = start_abs.min(end_abs);
    if endpoint_floor <= 1.0e-9 {
        return false;
    }

    if best_abs <= 0.5 * endpoint_floor {
        return true;
    }

    let interval_min = start.sample.position[axis].min(end.sample.position[axis]);
    let interval_max = start.sample.position[axis].max(end.sample.position[axis]);
    best_probe.sample.position[axis] < interval_min - 1.0e-9
        || best_probe.sample.position[axis] > interval_max + 1.0e-9
}

fn sampled_axis_extremum_kind(start: f64, midpoint: f64, end: f64) -> Option<AxisExtremumKind> {
    let tolerance = 1.0e-9;
    let midpoint_is_max = midpoint >= start - tolerance
        && midpoint >= end - tolerance
        && (midpoint > start + tolerance || midpoint > end + tolerance);
    if midpoint_is_max {
        return Some(AxisExtremumKind::Maximum);
    }

    let midpoint_is_min = midpoint <= start + tolerance
        && midpoint <= end + tolerance
        && (midpoint < start - tolerance || midpoint < end - tolerance);
    midpoint_is_min.then_some(AxisExtremumKind::Minimum)
}

fn quadratic_axis_extremum_parameter(
    start: &NormalizedEdgeSample,
    midpoint: &NormalizedEdgeSample,
    end: &NormalizedEdgeSample,
    axis: usize,
) -> Option<f64> {
    let x0 = start.t;
    let x1 = midpoint.t;
    let x2 = end.t;
    let y0 = start.sample.position[axis];
    let y1 = midpoint.sample.position[axis];
    let y2 = end.sample.position[axis];
    let numerator = (x1 - x0).powi(2) * (y1 - y2) - (x1 - x2).powi(2) * (y1 - y0);
    let denominator = 2.0 * ((x1 - x0) * (y1 - y2) - (x1 - x2) * (y1 - y0));
    (denominator.abs() > 1.0e-18).then_some(x1 - numerator / denominator)
}

fn axis_position_is_better(candidate: f64, current: f64, extremum_kind: AxisExtremumKind) -> bool {
    match extremum_kind {
        AxisExtremumKind::Minimum => candidate < current - 1.0e-9,
        AxisExtremumKind::Maximum => candidate > current + 1.0e-9,
    }
}

fn axis_position_probe_is_promising(
    midpoint: &NormalizedEdgeSample,
    best_probe: NormalizedEdgeSample,
    axis: usize,
    extremum_kind: AxisExtremumKind,
) -> bool {
    axis_position_is_better(
        best_probe.sample.position[axis],
        midpoint.sample.position[axis],
        extremum_kind,
    )
}

fn seeded_axis_extremum_sample_index(
    samples: &[NormalizedEdgeSample],
    axis: usize,
    extremum_kind: AxisExtremumKind,
) -> Option<usize> {
    if samples.len() < 3 {
        return None;
    }

    let mut best_index = 1;
    for index in 2..samples.len() - 1 {
        if axis_position_is_better(
            samples[index].sample.position[axis],
            samples[best_index].sample.position[axis],
            extremum_kind,
        ) {
            best_index = index;
        }
    }
    Some(best_index)
}

fn seeded_axis_extremum_sample_range(
    samples: &[NormalizedEdgeSample],
    seed_index: usize,
    axis: usize,
    extremum_kind: AxisExtremumKind,
) -> Option<(usize, usize)> {
    if samples.len() < 3 || seed_index == 0 || seed_index + 1 >= samples.len() {
        return None;
    }

    let tolerance = sampled_axis_run_tolerance(samples, axis);
    let mut low_index = seed_index;
    while low_index > 0
        && sampled_axis_value_stays_on_seed_run(
            samples[low_index - 1].sample.position[axis],
            samples[low_index].sample.position[axis],
            extremum_kind,
            tolerance,
        )
    {
        low_index -= 1;
    }

    let mut high_index = seed_index;
    while high_index + 1 < samples.len()
        && sampled_axis_value_stays_on_seed_run(
            samples[high_index + 1].sample.position[axis],
            samples[high_index].sample.position[axis],
            extremum_kind,
            tolerance,
        )
    {
        high_index += 1;
    }

    if low_index == seed_index || high_index == seed_index {
        return None;
    }

    Some((low_index, high_index))
}

fn sampled_axis_run_tolerance(samples: &[NormalizedEdgeSample], axis: usize) -> f64 {
    let mut min_value = f64::INFINITY;
    let mut max_value = f64::NEG_INFINITY;
    for sample in samples {
        let value = sample.sample.position[axis];
        min_value = min_value.min(value);
        max_value = max_value.max(value);
    }
    1.0e-6 * (max_value - min_value).abs().max(1.0)
}

fn sampled_axis_value_stays_on_seed_run(
    candidate: f64,
    current: f64,
    extremum_kind: AxisExtremumKind,
    tolerance: f64,
) -> bool {
    match extremum_kind {
        AxisExtremumKind::Minimum => candidate >= current - tolerance,
        AxisExtremumKind::Maximum => candidate <= current + tolerance,
    }
}

fn sampled_edge_interval_needs_refinement(
    start: &NormalizedEdgeSample,
    midpoint: &NormalizedEdgeSample,
    end: &NormalizedEdgeSample,
) -> bool {
    midpoint_expands_edge_interval_bbox(
        start.sample.position,
        midpoint.sample.position,
        end.sample.position,
    ) || interval_tangent_indicates_axis_turn(
        start.sample.tangent,
        midpoint.sample.tangent,
        end.sample.tangent,
    ) || interval_midpoint_shows_axis_position_shoulder(start, midpoint, end)
        || interval_midpoint_bends_from_chord(
            start.sample.position,
            midpoint.sample.position,
            end.sample.position,
        )
}

fn midpoint_expands_edge_interval_bbox(start: [f64; 3], midpoint: [f64; 3], end: [f64; 3]) -> bool {
    (0..3).any(|axis| {
        let interval_min = start[axis].min(end[axis]);
        let interval_max = start[axis].max(end[axis]);
        midpoint[axis] < interval_min - 1.0e-9 || midpoint[axis] > interval_max + 1.0e-9
    })
}

fn interval_tangent_indicates_axis_turn(
    start_tangent: [f64; 3],
    midpoint_tangent: [f64; 3],
    end_tangent: [f64; 3],
) -> bool {
    (0..3).any(|axis| {
        tangent_sign_changes(start_tangent[axis], midpoint_tangent[axis])
            || tangent_sign_changes(midpoint_tangent[axis], end_tangent[axis])
            || tangent_sign_changes(start_tangent[axis], end_tangent[axis])
    })
}

fn tangent_sign_changes(lhs: f64, rhs: f64) -> bool {
    lhs.abs() > 1.0e-9 && rhs.abs() > 1.0e-9 && lhs.signum() != rhs.signum()
}

fn interval_midpoint_shows_axis_position_shoulder(
    start: &NormalizedEdgeSample,
    midpoint: &NormalizedEdgeSample,
    end: &NormalizedEdgeSample,
) -> bool {
    let interval_t_span = (end.t - start.t).abs();
    if interval_t_span <= 1.0e-12 {
        return false;
    }

    let interpolation_weight = ((midpoint.t - start.t) / interval_t_span).clamp(0.0, 1.0);
    let chord_length = norm3(subtract3(end.sample.position, start.sample.position));
    (0..3).any(|axis| {
        let expected_axis_value = start.sample.position[axis]
            + interpolation_weight * (end.sample.position[axis] - start.sample.position[axis]);
        let deviation = (midpoint.sample.position[axis] - expected_axis_value).abs();
        let axis_interval_span = (end.sample.position[axis] - start.sample.position[axis]).abs();
        let axis_scale = chord_length.max(axis_interval_span).max(1.0);
        deviation > 5.0e-4 * axis_scale
    })
}

fn interval_midpoint_bends_from_chord(start: [f64; 3], midpoint: [f64; 3], end: [f64; 3]) -> bool {
    let chord = subtract3(end, start);
    let chord_length = norm3(chord);
    if chord_length <= 1.0e-12 {
        return norm3(subtract3(midpoint, start)) > 1.0e-9;
    }

    let start_to_midpoint = subtract3(midpoint, start);
    let projection = dot3(start_to_midpoint, chord) / dot3(chord, chord).max(1.0e-18);
    let closest_point = add3(start, scale3(chord, projection.clamp(0.0, 1.0)));
    let deviation = norm3(subtract3(midpoint, closest_point));
    deviation > 1.0e-3 * chord_length.max(1.0)
}

fn periodic_curve_bbox(
    start: f64,
    end: f64,
    period: f64,
    coefficients_for_axis: impl Fn(usize) -> (f64, f64, f64),
    position_at: impl Fn(f64) -> [f64; 3],
) -> Option<([f64; 3], [f64; 3])> {
    let mut parameters = vec![start, end];
    for axis in 0..3 {
        let (_, cos_coefficient, sin_coefficient) = coefficients_for_axis(axis);
        if cos_coefficient.abs() <= 1.0e-12 && sin_coefficient.abs() <= 1.0e-12 {
            continue;
        }
        let critical = sin_coefficient.atan2(cos_coefficient);
        extend_periodic_parameters(&mut parameters, start, end, period, critical);
        extend_periodic_parameters(&mut parameters, start, end, period, critical + PI);
    }
    bbox_from_points(parameters.into_iter().map(position_at).collect())
}

fn extend_periodic_parameters(
    parameters: &mut Vec<f64>,
    start: f64,
    end: f64,
    period: f64,
    base: f64,
) {
    let low = start.min(end);
    let high = start.max(end);
    let first_multiple = ((low - base - 1.0e-12) / period).ceil() as i64;
    let last_multiple = ((high - base + 1.0e-12) / period).floor() as i64;
    for multiple in first_multiple..=last_multiple {
        parameters.push(base + multiple as f64 * period);
    }
}

fn mesh_shape_bbox(context: &Context, shape: &Shape) -> Option<([f64; 3], [f64; 3])> {
    let mesh = context.mesh(shape, SUMMARY_BBOX_MESH_PARAMS).ok()?;
    mesh_bbox(&mesh)
}

pub(super) fn mesh_face_properties(
    context: &Context,
    face_shape: &Shape,
    orientation: Orientation,
) -> Option<MeshFaceProperties> {
    let mesh = context
        .mesh(face_shape, UNSUPPORTED_FACE_AREA_MESH_PARAMS)
        .ok()?;
    let mut sample = polyhedral_mesh_sample(&mesh)?;
    if matches!(orientation, Orientation::Reversed) {
        sample.normal = scale3(sample.normal, -1.0);
    }
    Some(MeshFaceProperties {
        area: polyhedral_mesh_area(&mesh)?,
        sample,
    })
}

fn mesh_face_volume(context: &Context, face_shape: &Shape, face: &BrepFace) -> Option<f64> {
    let mesh = context.mesh(face_shape, SUMMARY_VOLUME_MESH_PARAMS).ok()?;
    mesh_face_signed_volume(&mesh, face.sample.normal)
}

fn mesh_face_signed_volume(mesh: &Mesh, outward_normal_hint: [f64; 3]) -> Option<f64> {
    if mesh.triangle_indices.is_empty() {
        return Some(0.0);
    }

    let orientation_sign = polyhedral_mesh_sample(mesh)
        .map(|sample| {
            if dot3(sample.normal, outward_normal_hint) >= 0.0 {
                1.0
            } else {
                -1.0
            }
        })
        .unwrap_or(1.0);

    let mut signed_volume = 0.0;
    for triangle in mesh.triangle_indices.chunks_exact(3) {
        let i0 = usize::try_from(triangle[0]).ok()?;
        let i1 = usize::try_from(triangle[1]).ok()?;
        let i2 = usize::try_from(triangle[2]).ok()?;
        let a = *mesh.positions.get(i0)?;
        let b = *mesh.positions.get(i1)?;
        let c = *mesh.positions.get(i2)?;
        signed_volume += dot3(a, cross3(b, c)) / 6.0;
    }

    Some(orientation_sign * signed_volume)
}
