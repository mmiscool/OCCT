use super::*;

use super::face_metrics::{
    analytic_face_volume, analytic_offset_face_volume, analytic_ported_swept_face_volume,
};

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
    face_shapes: &[Shape],
    edge_shapes: &[Shape],
) -> Result<ShapeSummary, Error> {
    let counts = shape_counts(context, shape, topology)?;
    let root_kind = classify_root_kind(counts);
    let primary_kind = classify_primary_kind(counts);
    let exact_primitive =
        exact_primitive_shape_summary(primary_kind, counts.solid_count, vertices, edges, faces);
    let fallback_summary = || context.describe_shape_occt(shape).ok();
    let (bbox_min, bbox_max) = exact_primitive
        .and_then(|summary| summary.bbox)
        .or_else(|| ported_shape_bbox(context, shape, vertices, edges, faces))
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
                analytic_shape_volume(context, wires, edges, faces, face_shapes, edge_shapes)
            })
            .or_else(|| mesh_shape_volume(context, shape, counts))
            .or_else(|| fallback_summary().map(|summary| summary.volume))
            .unwrap_or(0.0),
        bbox_min,
        bbox_max,
    })
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

fn analytic_shape_volume(
    context: &Context,
    wires: &[BrepWire],
    edges: &[BrepEdge],
    faces: &[BrepFace],
    face_shapes: &[Shape],
    edge_shapes: &[Shape],
) -> Option<f64> {
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

fn mesh_shape_volume(context: &Context, shape: &Shape, counts: ShapeCounts) -> Option<f64> {
    if counts.solid_count == 0 && counts.compsolid_count == 0 {
        return Some(0.0);
    }

    let mesh = context.mesh(shape, SUMMARY_VOLUME_MESH_PARAMS).ok()?;
    polyhedral_mesh_volume(&mesh)
}

fn ported_shape_bbox(
    context: &Context,
    shape: &Shape,
    vertices: &[BrepVertex],
    edges: &[BrepEdge],
    faces: &[BrepFace],
) -> Option<([f64; 3], [f64; 3])> {
    topological_shape_bbox(vertices, edges, faces).or_else(|| mesh_shape_bbox(context, shape))
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
        return analytic_edges_bbox(edges)
            .or_else(|| line_segment_points_bbox(vertices, edges))
            .or_else(|| {
                if edges.is_empty() {
                    bbox_from_points(vertices.iter().map(|vertex| vertex.position).collect())
                } else {
                    None
                }
            });
    }

    if faces
        .iter()
        .all(|face| matches!(face.ported_surface, Some(PortedSurface::Plane(_))))
    {
        return analytic_edges_bbox(edges).or_else(|| line_segment_points_bbox(vertices, edges));
    }

    None
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
    let curve = edge.ported_curve?;
    let start = edge.geometry.start_parameter;
    let end = edge.geometry.end_parameter;

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
