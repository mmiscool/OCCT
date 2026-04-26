use super::edge_topology::{oriented_edge_geometry, RootEdgeTopology};
use super::face_topology::{single_face_topology_with_route, FaceSurfaceRoute, SingleFaceTopology};
use super::*;

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
            ported_extrusion_face_surface(context, face_shape, face_geometry, &topology)
        }
        crate::SurfaceKind::Revolution => {
            ported_revolution_face_surface(context, face_shape, face_geometry, &topology)
        }
        _ => Ok(None),
    }
}

fn ported_extrusion_face_surface(
    context: &Context,
    face_shape: &Shape,
    face_geometry: FaceGeometry,
    topology: &SingleFaceTopology,
) -> Result<Option<PortedSweptSurface>, Error> {
    let orientation = context.shape_orientation(face_shape)?;
    for basis_kind in SUPPORTED_SWEPT_BASIS_KINDS {
        let candidates = match face_curve_candidates(
            &topology.loops,
            &topology.wires,
            &topology.edges,
            basis_kind,
        ) {
            Some(candidates) => candidates,
            None => continue,
        };
        let basis_geometry = match candidates.first() {
            Some(candidate) => candidate.geometry,
            None => continue,
        };
        let basis_on_u = basis_parameter_on_u(face_geometry, basis_geometry);
        let direction = match extrusion_direction_from_face_samples(
            context,
            face_shape,
            face_geometry,
            basis_on_u,
        )? {
            Some(direction) => direction,
            None => continue,
        };
        let basis = match select_swept_face_basis_curve(
            candidates,
            face_geometry,
            SweptBasisSelection::Extrusion { direction },
        ) {
            Some(basis) => basis,
            None => continue,
        };
        let payload = crate::ExtrusionSurfacePayload {
            direction,
            basis_curve_kind: basis.geometry.kind,
        };

        if validates_extrusion_face_surface(
            context,
            face_shape,
            face_geometry,
            orientation,
            payload,
            basis.curve,
            basis.geometry,
        )? {
            return Ok(Some(PortedSweptSurface::Extrusion {
                payload,
                basis_curve: basis.curve,
                basis_geometry: basis.geometry,
            }));
        }
    }

    Ok(None)
}

fn ported_revolution_face_surface(
    context: &Context,
    face_shape: &Shape,
    face_geometry: FaceGeometry,
    topology: &SingleFaceTopology,
) -> Result<Option<PortedSweptSurface>, Error> {
    let orientation = context.shape_orientation(face_shape)?;
    for basis_kind in SUPPORTED_SWEPT_BASIS_KINDS {
        let candidates = match face_curve_candidates(
            &topology.loops,
            &topology.wires,
            &topology.edges,
            basis_kind,
        ) {
            Some(candidates) => candidates,
            None => continue,
        };
        let basis_geometry = match candidates.first() {
            Some(candidate) => candidate.geometry,
            None => continue,
        };
        let basis_on_u = basis_parameter_on_u(face_geometry, basis_geometry);
        let (axis_origin, axis_direction) = match revolution_axis_from_face_samples(
            context,
            face_shape,
            face_geometry,
            basis_on_u,
        )? {
            Some(axis) => axis,
            None => continue,
        };
        let basis = match select_swept_face_basis_curve(
            candidates,
            face_geometry,
            SweptBasisSelection::Revolution {
                axis_origin,
                axis_direction,
            },
        ) {
            Some(basis) => basis,
            None => continue,
        };
        let payload = crate::RevolutionSurfacePayload {
            axis_origin,
            axis_direction,
            basis_curve_kind: basis.geometry.kind,
        };

        if validates_revolution_face_surface(
            context,
            face_shape,
            face_geometry,
            orientation,
            payload,
            basis.curve,
            basis.geometry,
        )? {
            return Ok(Some(PortedSweptSurface::Revolution {
                payload,
                basis_curve: basis.curve,
                basis_geometry: basis.geometry,
            }));
        }
    }

    Ok(None)
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

const SUPPORTED_SWEPT_BASIS_KINDS: [crate::CurveKind; 3] = [
    crate::CurveKind::Line,
    crate::CurveKind::Circle,
    crate::CurveKind::Ellipse,
];

fn extrusion_direction_from_face_samples(
    context: &Context,
    face_shape: &Shape,
    face_geometry: FaceGeometry,
    basis_on_u: bool,
) -> Result<Option<[f64; 3]>, Error> {
    let (curve_start, curve_end) = curve_parameter_range(face_geometry, basis_on_u);
    let curve_mid = 0.5 * (curve_start + curve_end);
    let (sweep_start, sweep_end) = sweep_parameter_range(face_geometry, basis_on_u);
    let sweep_span = sweep_end - sweep_start;
    if sweep_span.abs() <= 1.0e-12 {
        return Ok(None);
    }

    let derivative_at_curve = |curve_parameter| -> Result<[f64; 3], Error> {
        let start = context.face_sample_occt(
            face_shape,
            swept_uv(basis_on_u, curve_parameter, sweep_start),
        )?;
        let end = context
            .face_sample_occt(face_shape, swept_uv(basis_on_u, curve_parameter, sweep_end))?;
        Ok(scale3(
            subtract3(end.position, start.position),
            1.0 / sweep_span,
        ))
    };

    let direction0 = normalize3(derivative_at_curve(curve_start)?);
    let direction1 = normalize3(derivative_at_curve(curve_mid)?);
    if norm3(direction0) <= 1.0e-12 || norm3(direction1) <= 1.0e-12 {
        return Ok(None);
    }
    if dot3(direction0, direction1) < 1.0 - 1.0e-7 {
        return Ok(None);
    }

    let direction = normalize3(add3(direction0, direction1));
    if norm3(direction) <= 1.0e-12 {
        return Ok(None);
    }
    Ok(Some(direction))
}

fn revolution_axis_from_face_samples(
    context: &Context,
    face_shape: &Shape,
    face_geometry: FaceGeometry,
    basis_on_u: bool,
) -> Result<Option<([f64; 3], [f64; 3])>, Error> {
    let (curve_start, curve_end) = curve_parameter_range(face_geometry, basis_on_u);
    let (sweep_start, sweep_end) = sweep_parameter_range(face_geometry, basis_on_u);
    let sweep_parameters = trigonometric_curve_probe_parameters(sweep_start, sweep_end);
    let sweep_parameters = match select_trigonometric_curve_parameters(sweep_parameters) {
        Some(parameters) => parameters,
        None => return Ok(None),
    };

    let mut axis: Option<([f64; 3], [f64; 3])> = None;
    for fraction in [0.0, 0.33, 0.67, 1.0] {
        let curve_parameter = curve_start + (curve_end - curve_start) * fraction;
        let (origin, mut direction) = match revolution_axis_components_at_curve_parameter(
            context,
            face_shape,
            basis_on_u,
            curve_parameter,
            sweep_parameters,
        )? {
            Some(axis) => axis,
            None => continue,
        };

        if let Some((reference_origin, reference_direction)) = axis {
            if dot3(reference_direction, direction) < 0.0 {
                direction = scale3(direction, -1.0);
            }
            if dot3(reference_direction, direction) < 1.0 - 1.0e-7 {
                return Ok(None);
            }
            let center_delta = subtract3(origin, reference_origin);
            if norm3(center_delta) > 1.0e-8
                && norm3(cross3(center_delta, reference_direction)) > 1.0e-6 * norm3(center_delta)
            {
                return Ok(None);
            }
        } else {
            axis = Some((origin, direction));
        }
    }

    Ok(axis.map(|(origin, direction)| (canonical_axis_origin(origin, direction), direction)))
}

fn revolution_axis_components_at_curve_parameter(
    context: &Context,
    face_shape: &Shape,
    basis_on_u: bool,
    curve_parameter: f64,
    sweep_parameters: [f64; 3],
) -> Result<Option<([f64; 3], [f64; 3])>, Error> {
    let positions = [
        context
            .face_sample_occt(
                face_shape,
                swept_uv(basis_on_u, curve_parameter, sweep_parameters[0]),
            )?
            .position,
        context
            .face_sample_occt(
                face_shape,
                swept_uv(basis_on_u, curve_parameter, sweep_parameters[1]),
            )?
            .position,
        context
            .face_sample_occt(
                face_shape,
                swept_uv(basis_on_u, curve_parameter, sweep_parameters[2]),
            )?
            .position,
    ];
    let (center, x_component, y_component) =
        match solve_trigonometric_curve_components(sweep_parameters, positions) {
            Some(value) => value,
            None => return Ok(None),
        };
    let axis_direction = normalize3(cross3(x_component, y_component));
    if norm3(axis_direction) <= 1.0e-12
        || norm3(x_component) <= 1.0e-12
        || norm3(y_component) <= 1.0e-12
    {
        return Ok(None);
    }

    Ok(Some((center, axis_direction)))
}

fn canonical_axis_origin(axis_point: [f64; 3], axis_direction: [f64; 3]) -> [f64; 3] {
    subtract3(
        axis_point,
        scale3(axis_direction, dot3(axis_point, axis_direction)),
    )
}

fn validates_extrusion_face_surface(
    context: &Context,
    face_shape: &Shape,
    face_geometry: FaceGeometry,
    orientation: Orientation,
    payload: crate::ExtrusionSurfacePayload,
    basis_curve: PortedCurve,
    basis_geometry: EdgeGeometry,
) -> Result<bool, Error> {
    for uv_t in SWEPT_VALIDATION_UVS {
        let expected = sample_extrusion_surface_normalized(
            basis_curve,
            face_geometry,
            basis_geometry,
            uv_t,
            payload.direction,
            orientation,
        );
        let actual = context.face_sample_normalized_occt(face_shape, uv_t)?;
        if !approx_points_eq(expected.position, actual.position, 1.0e-6)
            || !approx_points_eq(expected.normal, actual.normal, 1.0e-6)
        {
            return Ok(false);
        }
    }

    Ok(true)
}

fn validates_revolution_face_surface(
    context: &Context,
    face_shape: &Shape,
    face_geometry: FaceGeometry,
    orientation: Orientation,
    payload: crate::RevolutionSurfacePayload,
    basis_curve: PortedCurve,
    basis_geometry: EdgeGeometry,
) -> Result<bool, Error> {
    for uv_t in SWEPT_VALIDATION_UVS {
        let expected = sample_revolution_surface_normalized(
            basis_curve,
            face_geometry,
            basis_geometry,
            uv_t,
            payload.axis_origin,
            payload.axis_direction,
            orientation,
        );
        let actual = context.face_sample_normalized_occt(face_shape, uv_t)?;
        if !approx_points_eq(expected.position, actual.position, 1.0e-6)
            || !approx_points_eq(expected.normal, actual.normal, 1.0e-6)
        {
            return Ok(false);
        }
    }

    Ok(true)
}

const SWEPT_VALIDATION_UVS: [[f64; 2]; 5] = [
    [0.0, 0.0],
    [0.25, 0.75],
    [0.5, 0.5],
    [0.75, 0.25],
    [1.0, 1.0],
];

fn curve_parameter_range(face_geometry: FaceGeometry, basis_on_u: bool) -> (f64, f64) {
    if basis_on_u {
        (face_geometry.u_min, face_geometry.u_max)
    } else {
        (face_geometry.v_min, face_geometry.v_max)
    }
}

fn sweep_parameter_range(face_geometry: FaceGeometry, basis_on_u: bool) -> (f64, f64) {
    if basis_on_u {
        (face_geometry.v_min, face_geometry.v_max)
    } else {
        (face_geometry.u_min, face_geometry.u_max)
    }
}

fn swept_uv(basis_on_u: bool, curve_parameter: f64, sweep_parameter: f64) -> [f64; 2] {
    if basis_on_u {
        [curve_parameter, sweep_parameter]
    } else {
        [sweep_parameter, curve_parameter]
    }
}

fn trigonometric_curve_probe_parameters(start: f64, end: f64) -> [f64; 5] {
    [0.0, 0.25, 0.5, 0.75, 1.0].map(|fraction| start + (end - start) * fraction)
}

fn select_trigonometric_curve_parameters(candidates: [f64; 5]) -> Option<[f64; 3]> {
    let mut best: Option<([f64; 3], f64)> = None;
    for i in 0..candidates.len() {
        for j in (i + 1)..candidates.len() {
            for k in (j + 1)..candidates.len() {
                let selection = [candidates[i], candidates[j], candidates[k]];
                let determinant = trigonometric_curve_determinant(selection).abs();
                if best
                    .as_ref()
                    .map(|(_, best_determinant)| determinant > *best_determinant)
                    .unwrap_or(true)
                {
                    best = Some((selection, determinant));
                }
            }
        }
    }

    best.filter(|(_, determinant)| *determinant > 1.0e-6)
        .map(|(selection, _)| selection)
}

fn solve_trigonometric_curve_components(
    parameters: [f64; 3],
    positions: [[f64; 3]; 3],
) -> Option<([f64; 3], [f64; 3], [f64; 3])> {
    let determinant = trigonometric_curve_determinant(parameters);
    if determinant.abs() <= 1.0e-12 {
        return None;
    }

    let cosines = parameters.map(f64::cos);
    let sines = parameters.map(f64::sin);
    let delta10 = subtract3(positions[1], positions[0]);
    let delta20 = subtract3(positions[2], positions[0]);
    let x_component = scale3(
        subtract3(
            scale3(delta10, sines[2] - sines[0]),
            scale3(delta20, sines[1] - sines[0]),
        ),
        1.0 / determinant,
    );
    let y_component = scale3(
        add3(
            scale3(delta10, cosines[0] - cosines[2]),
            scale3(delta20, cosines[1] - cosines[0]),
        ),
        1.0 / determinant,
    );
    let center = subtract3(
        positions[0],
        add3(
            scale3(x_component, cosines[0]),
            scale3(y_component, sines[0]),
        ),
    );
    Some((center, x_component, y_component))
}

fn trigonometric_curve_determinant(parameters: [f64; 3]) -> f64 {
    let cosines = parameters.map(f64::cos);
    let sines = parameters.map(f64::sin);
    (cosines[1] - cosines[0]) * (sines[2] - sines[0])
        - (cosines[2] - cosines[0]) * (sines[1] - sines[0])
}

fn approx_points_eq(lhs: [f64; 3], rhs: [f64; 3], tolerance: f64) -> bool {
    lhs.iter()
        .zip(rhs)
        .all(|(lhs, rhs)| (*lhs - rhs).abs() <= tolerance)
}

pub(super) fn basis_parameter_on_u(
    face_geometry: FaceGeometry,
    basis_geometry: EdgeGeometry,
) -> bool {
    let basis_span = (basis_geometry.end_parameter - basis_geometry.start_parameter).abs();
    let u_span = (face_geometry.u_max - face_geometry.u_min).abs();
    let v_span = (face_geometry.v_max - face_geometry.v_min).abs();
    let u_delta = (u_span - basis_span).abs();
    let v_delta = (v_span - basis_span).abs();
    if (u_delta - v_delta).abs() <= 1.0e-9 {
        return match face_geometry.kind {
            crate::SurfaceKind::Revolution => false,
            crate::SurfaceKind::Extrusion => true,
            _ => true,
        };
    }
    u_delta < v_delta
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
    let ported_curve =
        PortedCurve::from_context_with_ported_payloads(context, edge_shape, edge.geometry)?;
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
