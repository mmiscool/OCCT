use super::*;

use super::topology::{
    oriented_edge_geometry, single_face_topology_with_route, FaceSurfaceRoute, RootEdgeTopology,
    SingleFaceTopology,
};

#[derive(Clone, Copy, Debug)]
pub(super) struct FaceCurveCandidate {
    pub(super) curve: PortedCurve,
    pub(super) geometry: EdgeGeometry,
    midpoint: [f64; 3],
}

pub(super) fn face_curve_candidates(
    loops: &[BrepFaceLoop],
    wires: &[BrepWire],
    edges: &[BrepEdge],
    basis_kind: crate::CurveKind,
) -> Option<Vec<FaceCurveCandidate>> {
    let mut seen = BTreeSet::new();
    let mut candidates = Vec::new();

    for face_loop in loops {
        let wire = wires.get(face_loop.wire_index)?;
        for (edge_index, edge_orientation) in oriented_wire_edges(wire, face_loop.orientation) {
            if !seen.insert(edge_index) {
                continue;
            }
            let edge = edges.get(edge_index)?;
            if edge.geometry.kind != basis_kind {
                continue;
            }
            let curve = edge.ported_curve?;

            let geometry = oriented_edge_geometry(edge.geometry, edge_orientation);
            let midpoint_parameter = 0.5 * (geometry.start_parameter + geometry.end_parameter);
            let midpoint = curve
                .sample_with_geometry(geometry, midpoint_parameter)
                .position;
            candidates.push(FaceCurveCandidate {
                curve,
                geometry,
                midpoint,
            });
        }
    }

    if candidates.is_empty() {
        None
    } else {
        Some(candidates)
    }
}

pub(super) fn select_swept_face_basis_curve(
    candidates: Vec<FaceCurveCandidate>,
    face_geometry: FaceGeometry,
    selection: SweptBasisSelection,
) -> Option<FaceCurveCandidate> {
    let basis_geometry = candidates.first()?.geometry;
    let use_u_for_basis = basis_parameter_on_u(face_geometry, basis_geometry);
    let (sweep_min, sweep_max) = if use_u_for_basis {
        (face_geometry.v_min, face_geometry.v_max)
    } else {
        (face_geometry.u_min, face_geometry.u_max)
    };
    let target_is_min = sweep_min.abs() <= sweep_max.abs();

    match selection {
        SweptBasisSelection::Extrusion { direction } => {
            let direction = normalize3(direction);
            if target_is_min {
                candidates.into_iter().min_by(|lhs, rhs| {
                    dot3(lhs.midpoint, direction)
                        .partial_cmp(&dot3(rhs.midpoint, direction))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
            } else {
                candidates.into_iter().max_by(|lhs, rhs| {
                    dot3(lhs.midpoint, direction)
                        .partial_cmp(&dot3(rhs.midpoint, direction))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
            }
        }
        SweptBasisSelection::Revolution {
            axis_origin,
            axis_direction,
        } => {
            if periodic_face_span(face_geometry).is_some() {
                return candidates.into_iter().next();
            }
            let axis_direction = normalize3(axis_direction);
            let reference_radial = candidates.iter().find_map(|candidate| {
                radial_direction(candidate.midpoint, axis_origin, axis_direction)
            })?;
            let tangent = normalize3(cross3(axis_direction, reference_radial));
            let angular_candidates = candidates
                .into_iter()
                .filter_map(|candidate| {
                    let radial = radial_direction(candidate.midpoint, axis_origin, axis_direction)?;
                    Some((
                        candidate,
                        dot3(radial, tangent).atan2(dot3(radial, reference_radial)),
                    ))
                })
                .collect::<Vec<_>>();

            if target_is_min {
                angular_candidates
                    .into_iter()
                    .min_by(|lhs, rhs| {
                        lhs.1
                            .partial_cmp(&rhs.1)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|(candidate, _)| candidate)
            } else {
                angular_candidates
                    .into_iter()
                    .max_by(|lhs, rhs| {
                        lhs.1
                            .partial_cmp(&rhs.1)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|(candidate, _)| candidate)
            }
        }
    }
}

pub(super) fn ported_swept_face_surface_with_route(
    context: &Context,
    face_shape: &Shape,
    face_geometry: FaceGeometry,
    route: FaceSurfaceRoute,
) -> Result<Option<PortedSweptSurface>, Error> {
    let topology = match single_face_topology_with_route(context, face_shape, route)? {
        Some(topology) => topology,
        None => return Ok(None),
    };

    ported_swept_face_surface_from_topology(context, face_shape, face_geometry, topology)
}

fn ported_swept_face_surface_from_topology(
    context: &Context,
    face_shape: &Shape,
    face_geometry: FaceGeometry,
    topology: SingleFaceTopology,
) -> Result<Option<PortedSweptSurface>, Error> {
    match face_geometry.kind {
        crate::SurfaceKind::Extrusion => {
            ported_extrusion_face_surface(context, face_shape, face_geometry, &topology).map(Some)
        }
        crate::SurfaceKind::Revolution => {
            ported_revolution_face_surface(context, face_shape, face_geometry, &topology).map(Some)
        }
        _ => Ok(None),
    }
}

fn ported_extrusion_face_surface(
    context: &Context,
    face_shape: &Shape,
    face_geometry: FaceGeometry,
    topology: &SingleFaceTopology,
) -> Result<PortedSweptSurface, Error> {
    let payload = context.face_extrusion_payload_occt(face_shape)?;
    let basis = select_swept_face_basis(
        topology,
        face_geometry,
        payload.basis_curve_kind,
        SweptBasisSelection::Extrusion {
            direction: payload.direction,
        },
        "extrusion",
    )?;

    Ok(PortedSweptSurface::Extrusion {
        payload,
        basis_curve: basis.curve,
        basis_geometry: basis.geometry,
    })
}

fn ported_revolution_face_surface(
    context: &Context,
    face_shape: &Shape,
    face_geometry: FaceGeometry,
    topology: &SingleFaceTopology,
) -> Result<PortedSweptSurface, Error> {
    let payload = context.face_revolution_payload_occt(face_shape)?;
    let basis = select_swept_face_basis(
        topology,
        face_geometry,
        payload.basis_curve_kind,
        SweptBasisSelection::Revolution {
            axis_origin: payload.axis_origin,
            axis_direction: payload.axis_direction,
        },
        "revolution",
    )?;

    Ok(PortedSweptSurface::Revolution {
        payload,
        basis_curve: basis.curve,
        basis_geometry: basis.geometry,
    })
}

fn select_swept_face_basis(
    topology: &SingleFaceTopology,
    face_geometry: FaceGeometry,
    basis_curve_kind: crate::CurveKind,
    selection: SweptBasisSelection,
    face_kind: &'static str,
) -> Result<FaceCurveCandidate, Error> {
    let candidates = face_curve_candidates(
        &topology.loops,
        &topology.wires,
        &topology.edges,
        basis_curve_kind,
    )
    .ok_or_else(|| {
        Error::new(format!(
            "failed to identify a Rust-owned basis curve for {face_kind} face"
        ))
    })?;

    select_swept_face_basis_curve(candidates, face_geometry, selection).ok_or_else(|| {
        Error::new(format!(
            "failed to select a Rust-owned basis curve for {face_kind} face"
        ))
    })
}

#[derive(Clone, Copy)]
pub(super) enum SweptBasisSelection {
    Extrusion {
        direction: [f64; 3],
    },
    Revolution {
        axis_origin: [f64; 3],
        axis_direction: [f64; 3],
    },
}

pub(super) fn basis_parameter_on_u(
    face_geometry: FaceGeometry,
    basis_geometry: EdgeGeometry,
) -> bool {
    let basis_span = (basis_geometry.end_parameter - basis_geometry.start_parameter).abs();
    let u_span = (face_geometry.u_max - face_geometry.u_min).abs();
    let v_span = (face_geometry.v_max - face_geometry.v_min).abs();
    (u_span - basis_span).abs() <= (v_span - basis_span).abs()
}

fn radial_direction(
    point: [f64; 3],
    axis_origin: [f64; 3],
    axis_direction: [f64; 3],
) -> Option<[f64; 3]> {
    let radial = subtract3(
        point,
        add3(
            axis_origin,
            scale3(
                axis_direction,
                dot3(subtract3(point, axis_origin), axis_direction),
            ),
        ),
    );
    (norm3(radial) > 1.0e-9).then_some(normalize3(radial))
}

pub(super) fn rotate_point_about_axis(
    point: [f64; 3],
    axis_origin: [f64; 3],
    axis_direction: [f64; 3],
    angle: f64,
) -> [f64; 3] {
    add3(
        axis_origin,
        rotate_vector_about_axis(subtract3(point, axis_origin), axis_direction, angle),
    )
}

pub(super) fn rotate_vector_about_axis(
    vector: [f64; 3],
    axis_direction: [f64; 3],
    angle: f64,
) -> [f64; 3] {
    let axis_direction = normalize3(axis_direction);
    let cos_angle = angle.cos();
    let sin_angle = angle.sin();
    add3(
        add3(
            scale3(vector, cos_angle),
            scale3(cross3(axis_direction, vector), sin_angle),
        ),
        scale3(
            axis_direction,
            dot3(axis_direction, vector) * (1.0 - cos_angle),
        ),
    )
}

pub(super) fn revolution_surface_dv(
    position: [f64; 3],
    axis_origin: [f64; 3],
    axis_direction: [f64; 3],
) -> [f64; 3] {
    cross3(normalize3(axis_direction), subtract3(position, axis_origin))
}

pub(super) fn signed_scalar_integral<F>(start: f64, end: f64, integrand: F) -> f64
where
    F: Fn(f64) -> f64,
{
    if (end - start).abs() <= 1.0e-15 {
        return 0.0;
    }

    let (a, b, sign) = if start <= end {
        (start, end, 1.0)
    } else {
        (end, start, -1.0)
    };
    let fa = integrand(a);
    let fm = integrand(0.5 * (a + b));
    let fb = integrand(b);
    sign * adaptive_simpson(&integrand, a, b, fa, fm, fb, 1.0e-8, 12)
}

pub(super) fn positive_scalar_integral<F>(start: f64, end: f64, integrand: F) -> f64
where
    F: Fn(f64) -> f64,
{
    signed_scalar_integral(start, end, |value| integrand(value).abs()).abs()
}

fn adaptive_simpson<F>(
    integrand: &F,
    a: f64,
    b: f64,
    fa: f64,
    fm: f64,
    fb: f64,
    tolerance: f64,
    depth: u32,
) -> f64
where
    F: Fn(f64) -> f64,
{
    let midpoint = 0.5 * (a + b);
    let left_mid = 0.5 * (a + midpoint);
    let right_mid = 0.5 * (midpoint + b);
    let flm = integrand(left_mid);
    let frm = integrand(right_mid);

    let whole = simpson_step(a, b, fa, fm, fb);
    let left = simpson_step(a, midpoint, fa, flm, fm);
    let right = simpson_step(midpoint, b, fm, frm, fb);
    let delta = left + right - whole;

    if depth == 0 || delta.abs() <= 15.0 * tolerance {
        return left + right + delta / 15.0;
    }

    adaptive_simpson(
        integrand,
        a,
        midpoint,
        fa,
        flm,
        fm,
        0.5 * tolerance,
        depth - 1,
    ) + adaptive_simpson(
        integrand,
        midpoint,
        b,
        fm,
        frm,
        fb,
        0.5 * tolerance,
        depth - 1,
    )
}

fn simpson_step(a: f64, b: f64, fa: f64, fm: f64, fb: f64) -> f64 {
    (b - a) * (fa + 4.0 * fm + fb) / 6.0
}

fn periodic_face_span(face_geometry: FaceGeometry) -> Option<f64> {
    if face_geometry.is_u_periodic && !face_geometry.is_v_periodic {
        let span = face_geometry.u_max - face_geometry.u_min;
        return (span.abs() > 1.0e-9).then_some(span.abs());
    }
    if face_geometry.is_v_periodic && !face_geometry.is_u_periodic {
        let span = face_geometry.v_max - face_geometry.v_min;
        return (span.abs() > 1.0e-9).then_some(span.abs());
    }
    None
}

pub(super) fn append_edge_sample_points(
    context: &Context,
    edge_shape: &Shape,
    edge: &BrepEdge,
    geometry: EdgeGeometry,
    out_points: &mut Vec<[f64; 3]>,
) -> Result<(), Error> {
    let segment_count = edge_sample_count(edge, geometry);
    for step in 0..=segment_count {
        if !out_points.is_empty() && step == 0 {
            continue;
        }
        let t = step as f64 / segment_count as f64;
        let parameter = interpolate_range(geometry.start_parameter, geometry.end_parameter, t);
        let position = match edge.ported_curve {
            Some(curve) => curve.sample_with_geometry(geometry, parameter).position,
            None => {
                context
                    .edge_sample_at_parameter_occt(edge_shape, parameter)?
                    .position
            }
        };
        out_points.push(position);
    }
    Ok(())
}

pub(super) fn append_root_edge_sample_points(
    context: &Context,
    edge_shape: &Shape,
    edge: &RootEdgeTopology,
    geometry: EdgeGeometry,
    out_points: &mut Vec<[f64; 3]>,
) -> Result<(), Error> {
    let ported_curve = PortedCurve::from_context_with_geometry(context, edge_shape, edge.geometry)?;
    let segment_count = root_edge_sample_count(edge.geometry.kind, geometry);
    for step in 0..=segment_count {
        if !out_points.is_empty() && step == 0 {
            continue;
        }
        let t = step as f64 / segment_count as f64;
        let parameter = interpolate_range(geometry.start_parameter, geometry.end_parameter, t);
        let position = match ported_curve {
            Some(curve) => curve.sample_with_geometry(geometry, parameter).position,
            None => {
                context
                    .edge_sample_at_parameter_occt(edge_shape, parameter)?
                    .position
            }
        };
        out_points.push(position);
    }
    Ok(())
}

pub(super) fn oriented_wire_edges(
    wire: &BrepWire,
    wire_orientation: Orientation,
) -> Vec<(usize, Orientation)> {
    let reverse_wire = matches!(wire_orientation, Orientation::Reversed);
    let mut uses = wire
        .edge_indices
        .iter()
        .copied()
        .zip(wire.edge_orientations.iter().copied())
        .collect::<Vec<_>>();
    if reverse_wire {
        uses.reverse();
        for (_, orientation) in &mut uses {
            *orientation = reversed_orientation(*orientation);
        }
    }
    uses
}

fn reversed_orientation(orientation: Orientation) -> Orientation {
    match orientation {
        Orientation::Forward => Orientation::Reversed,
        Orientation::Reversed => Orientation::Forward,
        other => other,
    }
}

fn edge_sample_count(edge: &BrepEdge, geometry: EdgeGeometry) -> usize {
    let span = (geometry.end_parameter - geometry.start_parameter).abs();
    let base = match edge.geometry.kind {
        crate::CurveKind::Line => 8,
        crate::CurveKind::Circle | crate::CurveKind::Ellipse => {
            (span / (std::f64::consts::TAU / 32.0)).ceil() as usize
        }
        _ => 48,
    };
    base.clamp(8, 256)
}

fn root_edge_sample_count(kind: crate::CurveKind, geometry: EdgeGeometry) -> usize {
    let span = (geometry.end_parameter - geometry.start_parameter).abs();
    let base = match kind {
        crate::CurveKind::Line => 8,
        crate::CurveKind::Circle | crate::CurveKind::Ellipse => {
            (span / (std::f64::consts::TAU / 32.0)).ceil() as usize
        }
        _ => 48,
    };
    base.clamp(8, 256)
}

fn interpolate_range(start: f64, end: f64, t: f64) -> f64 {
    start + (end - start) * t
}
