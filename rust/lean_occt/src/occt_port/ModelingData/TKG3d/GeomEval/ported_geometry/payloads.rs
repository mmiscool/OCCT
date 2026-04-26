use std::f64::consts::TAU;

use crate::{
    CirclePayload, ConePayload, Context, CurveKind, CylinderPayload, EdgeEndpoints, EdgeGeometry,
    EllipsePayload, Error, ExtrusionSurfacePayload, FaceGeometry, FaceSample, FaceUvBounds,
    LinePayload, Orientation, PlanePayload, RevolutionSurfacePayload, Shape, SpherePayload,
    SurfaceKind, TorusPayload,
};

use super::{
    add3, cross3, dot3, norm3, normalize3, rotate_point_about_axis, sample_circle, sample_cone,
    sample_cylinder, sample_ellipse, sample_extrusion_surface_normalized, sample_line,
    sample_revolution_surface_normalized, sample_sphere, sample_torus, scale3, subtract3,
    Atan2Components, PortedCurve, PortedSurface, PortedSweptSurface,
};

pub(super) fn ported_line_geometry(
    payload: LinePayload,
    endpoints: EdgeEndpoints,
) -> Option<EdgeGeometry> {
    let start_parameter = line_parameter(payload, endpoints.start)?;
    let end_parameter = line_parameter(payload, endpoints.end)?;
    Some(EdgeGeometry {
        kind: CurveKind::Line,
        start_parameter,
        end_parameter,
        is_closed: approx_points_eq(endpoints.start, endpoints.end, 1.0e-7),
        is_periodic: false,
        period: 0.0,
    })
}

pub(super) fn ported_periodic_curve_geometry<F, H>(
    kind: CurveKind,
    endpoints: EdgeEndpoints,
    edge_length: f64,
    period: f64,
    direction_preference: Option<f64>,
    parameter_at_point: F,
    length_with_parameters: H,
) -> Option<EdgeGeometry>
where
    F: Fn([f64; 3]) -> Option<f64>,
    H: Fn(f64, f64) -> f64,
{
    let start_parameter = parameter_at_point(endpoints.start)?;
    let end_parameter_base = parameter_at_point(endpoints.end)?;
    let closed = approx_points_eq(endpoints.start, endpoints.end, 1.0e-7);
    let end_parameter = select_periodic_end_parameter(
        edge_length,
        period,
        start_parameter,
        end_parameter_base,
        closed,
        direction_preference,
        length_with_parameters,
    )?;
    let direction_sign = if end_parameter >= start_parameter {
        1.0
    } else {
        -1.0
    };
    let (start_parameter, end_parameter) =
        canonicalize_periodic_parameters(start_parameter, end_parameter, period, direction_sign);
    Some(EdgeGeometry {
        kind,
        start_parameter,
        end_parameter,
        is_closed: closed,
        is_periodic: true,
        period,
    })
}

fn select_periodic_end_parameter<H>(
    edge_length: f64,
    period: f64,
    start_parameter: f64,
    end_parameter_base: f64,
    closed: bool,
    direction_preference: Option<f64>,
    length_with_parameters: H,
) -> Option<f64>
where
    H: Fn(f64, f64) -> f64,
{
    let mut best: Option<(f64, f64, f64)> = None;
    let normalized_start = normalize_periodic_parameter(start_parameter, period);
    let normalized_end = normalize_periodic_parameter(end_parameter_base, period);
    let base_delta = normalized_end - normalized_start;
    for turn in -2..=2 {
        let candidate = start_parameter + base_delta + period * f64::from(turn);
        let delta = candidate - start_parameter;
        if delta.abs() <= 1.0e-9 && !(closed && edge_length <= 1.0e-9) {
            continue;
        }
        let length_error = (length_with_parameters(start_parameter, candidate) - edge_length).abs();
        let tie_breaker = periodic_candidate_tie_breaker(delta, direction_preference);
        let replace = best
            .map(|(_, best_error, best_tie_breaker)| {
                length_error < best_error - 1.0e-8
                    || ((length_error - best_error).abs() <= 1.0e-8
                        && tie_breaker < best_tie_breaker)
            })
            .unwrap_or(true);
        if replace {
            best = Some((candidate, length_error, tie_breaker));
        }
    }
    best.map(|(candidate, _, _)| candidate)
}

fn periodic_candidate_tie_breaker(delta: f64, direction_preference: Option<f64>) -> f64 {
    let preference_bias = direction_preference
        .filter(|preference| preference.abs() > 0.0 && delta.abs() > 1.0e-9)
        .map(|preference| {
            if delta.signum() == preference.signum() {
                0.0
            } else {
                1.0e-6
            }
        })
        .unwrap_or(0.0);
    let sign_bias = if delta >= 0.0 { 0.0 } else { 1.0e-12 };
    preference_bias + delta.abs() + sign_bias
}

fn canonicalize_periodic_parameters(
    mut start_parameter: f64,
    mut end_parameter: f64,
    period: f64,
    direction_sign: f64,
) -> (f64, f64) {
    let period = period.abs();

    if direction_sign >= 0.0 {
        while start_parameter < -1.0e-9 {
            start_parameter += period;
            end_parameter += period;
        }
        while start_parameter >= period - 1.0e-9 && end_parameter >= period + 1.0e-9 {
            start_parameter -= period;
            end_parameter -= period;
        }
    } else {
        while end_parameter < -1.0e-9 {
            start_parameter += period;
            end_parameter += period;
        }
        while end_parameter >= period - 1.0e-9 && start_parameter >= period + 1.0e-9 {
            start_parameter -= period;
            end_parameter -= period;
        }
    }

    (
        snap_periodic_parameter(start_parameter, period),
        snap_periodic_parameter(end_parameter, period),
    )
}

pub(super) fn ported_analytic_face_geometry(
    kind: SurfaceKind,
    bounds: FaceUvBounds,
) -> FaceGeometry {
    let (is_u_closed, is_v_closed, is_u_periodic, is_v_periodic, u_period, v_period) = match kind {
        SurfaceKind::Plane => (false, false, false, false, 0.0, 0.0),
        SurfaceKind::Cylinder | SurfaceKind::Cone | SurfaceKind::Sphere => {
            (true, false, true, false, TAU, 0.0)
        }
        SurfaceKind::Torus => (true, true, true, true, TAU, TAU),
        _ => (false, false, false, false, 0.0, 0.0),
    };

    FaceGeometry {
        kind,
        u_min: bounds.u_min,
        u_max: bounds.u_max,
        v_min: bounds.v_min,
        v_max: bounds.v_max,
        is_u_closed,
        is_v_closed,
        is_u_periodic,
        is_v_periodic,
        u_period,
        v_period,
    }
}

pub(super) fn ported_line_payload(
    context: &Context,
    shape: &Shape,
    geometry: EdgeGeometry,
) -> Result<Option<LinePayload>, Error> {
    let endpoints = context.edge_endpoints(shape)?;
    Ok(ported_line_payload_from_endpoints(geometry, endpoints))
}

pub(super) fn ported_line_payload_from_endpoints(
    geometry: EdgeGeometry,
    endpoints: EdgeEndpoints,
) -> Option<LinePayload> {
    if geometry.kind != CurveKind::Line {
        return None;
    }

    let delta_parameter = geometry.end_parameter - geometry.start_parameter;
    if delta_parameter.abs() <= 1.0e-12 {
        return None;
    }

    let direction = scale3(
        subtract3(endpoints.end, endpoints.start),
        1.0 / delta_parameter,
    );
    if norm3(direction) <= 1.0e-12 {
        return None;
    }

    Some(LinePayload {
        origin: subtract3(endpoints.start, scale3(direction, geometry.start_parameter)),
        direction,
    })
}

pub(super) fn ported_circle_payload(
    context: &Context,
    shape: &Shape,
    geometry: EdgeGeometry,
) -> Result<Option<CirclePayload>, Error> {
    if geometry.kind != CurveKind::Circle {
        return Ok(None);
    }

    let mut sample_position = |parameter| {
        Ok(context
            .edge_sample_at_parameter_occt(shape, parameter)?
            .position)
    };
    ported_circle_payload_from_position_samples(geometry, &mut sample_position)
}

fn ported_circle_payload_from_position_samples<F>(
    geometry: EdgeGeometry,
    mut sample_position: F,
) -> Result<Option<CirclePayload>, Error>
where
    F: FnMut(f64) -> Result<[f64; 3], Error>,
{
    if geometry.kind != CurveKind::Circle {
        return Ok(None);
    }

    let parameters =
        trigonometric_curve_probe_parameters(geometry.start_parameter, geometry.end_parameter);
    let [parameter0, parameter1, parameter2] =
        match select_trigonometric_curve_parameters(parameters) {
            Some(selection) => selection,
            None => return Ok(None),
        };

    let (center, x_component, y_component) = match solve_trigonometric_curve_components(
        [parameter0, parameter1, parameter2],
        [
            sample_position(parameter0)?,
            sample_position(parameter1)?,
            sample_position(parameter2)?,
        ],
    ) {
        Some(value) => value,
        None => return Ok(None),
    };

    let radius_x = norm3(x_component);
    let radius_y = norm3(y_component);
    let radius = 0.5 * (radius_x + radius_y);
    if radius.abs() <= 1.0e-12 || (radius_x - radius_y).abs() > 1.0e-7 {
        return Ok(None);
    }

    let x_direction = normalize3(x_component);
    let y_direction = normalize3(y_component);
    let normal = normalize3(cross3(x_direction, y_direction));
    if norm3(x_direction) <= 1.0e-12 || norm3(y_direction) <= 1.0e-12 || norm3(normal) <= 1.0e-12 {
        return Ok(None);
    }

    let payload = CirclePayload {
        center,
        normal,
        x_direction,
        y_direction,
        radius,
    };

    for parameter in parameters {
        if !approx_points_eq(
            sample_circle(payload, parameter).position,
            sample_position(parameter)?,
            1.0e-7,
        ) {
            return Ok(None);
        }
    }

    Ok(Some(payload))
}

pub(super) fn ported_ellipse_payload(
    context: &Context,
    shape: &Shape,
    geometry: EdgeGeometry,
) -> Result<Option<EllipsePayload>, Error> {
    if geometry.kind != CurveKind::Ellipse {
        return Ok(None);
    }

    let mut sample_position = |parameter| {
        Ok(context
            .edge_sample_at_parameter_occt(shape, parameter)?
            .position)
    };
    ported_ellipse_payload_from_position_samples(geometry, &mut sample_position)
}

fn ported_ellipse_payload_from_position_samples<F>(
    geometry: EdgeGeometry,
    mut sample_position: F,
) -> Result<Option<EllipsePayload>, Error>
where
    F: FnMut(f64) -> Result<[f64; 3], Error>,
{
    if geometry.kind != CurveKind::Ellipse {
        return Ok(None);
    }

    let parameters =
        trigonometric_curve_probe_parameters(geometry.start_parameter, geometry.end_parameter);
    let [parameter0, parameter1, parameter2] =
        match select_trigonometric_curve_parameters(parameters) {
            Some(selection) => selection,
            None => return Ok(None),
        };

    let (center, x_component, y_component) = match solve_trigonometric_curve_components(
        [parameter0, parameter1, parameter2],
        [
            sample_position(parameter0)?,
            sample_position(parameter1)?,
            sample_position(parameter2)?,
        ],
    ) {
        Some(value) => value,
        None => return Ok(None),
    };

    let major_radius = norm3(x_component);
    let minor_radius = norm3(y_component);
    if major_radius.abs() <= 1.0e-12 || minor_radius.abs() <= 1.0e-12 {
        return Ok(None);
    }

    let x_direction = normalize3(x_component);
    let y_direction = normalize3(y_component);
    let normal = normalize3(cross3(x_direction, y_direction));
    if norm3(x_direction) <= 1.0e-12 || norm3(y_direction) <= 1.0e-12 || norm3(normal) <= 1.0e-12 {
        return Ok(None);
    }

    let payload = EllipsePayload {
        center,
        normal,
        x_direction,
        y_direction,
        major_radius,
        minor_radius,
    };

    for parameter in parameters {
        if !approx_points_eq(
            sample_ellipse(payload, parameter).position,
            sample_position(parameter)?,
            1.0e-7,
        ) {
            return Ok(None);
        }
    }

    Ok(Some(payload))
}

pub(super) fn ported_plane_payload(
    context: &Context,
    shape: &Shape,
    geometry: FaceGeometry,
) -> Result<Option<PlanePayload>, Error> {
    ported_plane_payload_from_samples(geometry, |uv| context.face_sample_occt(shape, uv))
}

fn ported_plane_payload_from_samples<F>(
    geometry: FaceGeometry,
    mut sample_face: F,
) -> Result<Option<PlanePayload>, Error>
where
    F: FnMut([f64; 2]) -> Result<FaceSample, Error>,
{
    if geometry.kind != SurfaceKind::Plane {
        return Ok(None);
    }

    let u_span = geometry.u_max - geometry.u_min;
    let v_span = geometry.v_max - geometry.v_min;
    if u_span.abs() <= 1.0e-12 || v_span.abs() <= 1.0e-12 {
        return Ok(None);
    }

    let origin_sample = sample_face([geometry.u_min, geometry.v_min])?;
    let u_sample = sample_face([geometry.u_max, geometry.v_min])?;
    let v_sample = sample_face([geometry.u_min, geometry.v_max])?;

    let x_direction = scale3(
        subtract3(u_sample.position, origin_sample.position),
        1.0 / u_span,
    );
    let y_direction = scale3(
        subtract3(v_sample.position, origin_sample.position),
        1.0 / v_span,
    );
    let normal = cross3(x_direction, y_direction);

    if norm3(x_direction) <= 1.0e-12 || norm3(y_direction) <= 1.0e-12 || norm3(normal) <= 1.0e-12 {
        return Ok(None);
    }

    Ok(Some(PlanePayload {
        origin: subtract3(
            origin_sample.position,
            add3(
                scale3(x_direction, geometry.u_min),
                scale3(y_direction, geometry.v_min),
            ),
        ),
        normal: normalize3(normal),
        x_direction,
        y_direction,
    }))
}

pub(super) fn ported_cylinder_payload(
    context: &Context,
    shape: &Shape,
    geometry: FaceGeometry,
) -> Result<Option<CylinderPayload>, Error> {
    let orientation = context.shape_orientation(shape)?;
    ported_cylinder_payload_from_samples(geometry, orientation, |uv| {
        context.face_sample_occt(shape, uv)
    })
}

fn ported_cylinder_payload_from_samples<F>(
    geometry: FaceGeometry,
    orientation: Orientation,
    mut sample_face: F,
) -> Result<Option<CylinderPayload>, Error>
where
    F: FnMut([f64; 2]) -> Result<FaceSample, Error>,
{
    if geometry.kind != SurfaceKind::Cylinder {
        return Ok(None);
    }

    let v_span = geometry.v_max - geometry.v_min;
    if v_span.abs() <= 1.0e-12 {
        return Ok(None);
    }

    let u0 = geometry.u_min;
    let u1 = match select_periodic_probe_parameter(geometry.u_min, geometry.u_max) {
        Some(parameter) => parameter,
        None => return Ok(None),
    };
    let denominator = (u1 - u0).sin();
    if denominator.abs() <= 1.0e-6 {
        return Ok(None);
    }

    let base_sample = sample_face([u0, geometry.v_min])?;
    let axis_sample = sample_face([u0, geometry.v_max])?;
    let probe_sample = sample_face([u1, geometry.v_min])?;
    let probe_top_sample = sample_face([u1, geometry.v_max])?;
    let normal_sign = if matches!(orientation, Orientation::Reversed) {
        -1.0
    } else {
        1.0
    };
    let normal0 = scale3(base_sample.normal, normal_sign);
    let normal1 = scale3(probe_sample.normal, normal_sign);
    let axis = normalize3(scale3(
        subtract3(axis_sample.position, base_sample.position),
        1.0 / v_span,
    ));
    let x_direction = scale3(
        subtract3(scale3(normal0, u1.sin()), scale3(normal1, u0.sin())),
        1.0 / denominator,
    );
    let y_direction = scale3(
        subtract3(scale3(normal1, u0.cos()), scale3(normal0, u1.cos())),
        1.0 / denominator,
    );
    let normal_delta = subtract3(normal1, normal0);
    let normal_delta_norm2 = dot3(normal_delta, normal_delta);
    if norm3(axis) <= 1.0e-12
        || norm3(x_direction) <= 1.0e-12
        || norm3(y_direction) <= 1.0e-12
        || normal_delta_norm2 <= 1.0e-12
    {
        return Ok(None);
    }

    let radius = dot3(
        subtract3(probe_sample.position, base_sample.position),
        normal_delta,
    ) / normal_delta_norm2;
    if radius.abs() <= 1.0e-12 {
        return Ok(None);
    }

    let payload = CylinderPayload {
        origin: subtract3(
            base_sample.position,
            add3(scale3(axis, geometry.v_min), scale3(normal0, radius)),
        ),
        axis,
        x_direction,
        y_direction,
        radius,
    };

    if !approx_points_eq(
        sample_cylinder(payload, [u0, geometry.v_min]).position,
        base_sample.position,
        1.0e-7,
    ) || !approx_points_eq(
        sample_cylinder(payload, [u1, geometry.v_min]).position,
        probe_sample.position,
        1.0e-7,
    ) || !approx_points_eq(
        sample_cylinder(payload, [u0, geometry.v_max]).position,
        axis_sample.position,
        1.0e-7,
    ) || !approx_points_eq(
        sample_cylinder(payload, [u1, geometry.v_max]).position,
        probe_top_sample.position,
        1.0e-7,
    ) {
        return Ok(None);
    }

    Ok(Some(payload))
}

pub(super) fn ported_cone_payload(
    context: &Context,
    shape: &Shape,
    geometry: FaceGeometry,
) -> Result<Option<ConePayload>, Error> {
    let orientation = context.shape_orientation(shape)?;
    ported_cone_payload_from_samples(geometry, orientation, |uv| {
        context.face_sample_occt(shape, uv)
    })
}

fn ported_cone_payload_from_samples<F>(
    geometry: FaceGeometry,
    orientation: Orientation,
    mut sample_face: F,
) -> Result<Option<ConePayload>, Error>
where
    F: FnMut([f64; 2]) -> Result<FaceSample, Error>,
{
    if geometry.kind != SurfaceKind::Cone {
        return Ok(None);
    }

    let v_span = geometry.v_max - geometry.v_min;
    if v_span.abs() <= 1.0e-12 {
        return Ok(None);
    }

    let u0 = geometry.u_min;
    let u1 = match select_periodic_probe_parameter(geometry.u_min, geometry.u_max) {
        Some(parameter) => parameter,
        None => return Ok(None),
    };
    let denominator = (u1 - u0).sin();
    if denominator.abs() <= 1.0e-6 {
        return Ok(None);
    }

    let normal_sign = if matches!(orientation, Orientation::Reversed) {
        -1.0
    } else {
        1.0
    };

    let base_sample = sample_face([u0, geometry.v_min])?;
    let base_top_sample = sample_face([u0, geometry.v_max])?;
    let probe_sample = sample_face([u1, geometry.v_min])?;
    let probe_top_sample = sample_face([u1, geometry.v_max])?;
    let normal0 = scale3(base_sample.normal, normal_sign);
    let normal1 = scale3(probe_sample.normal, normal_sign);
    let generatrix0 = normalize3(scale3(
        subtract3(base_top_sample.position, base_sample.position),
        1.0 / v_span,
    ));
    let generatrix1 = normalize3(scale3(
        subtract3(probe_top_sample.position, probe_sample.position),
        1.0 / v_span,
    ));
    let generatrix_delta = subtract3(generatrix1, generatrix0);
    let normal_delta = subtract3(normal1, normal0);
    let generatrix_delta_norm = norm3(generatrix_delta);
    let normal_delta_norm = norm3(normal_delta);
    if norm3(generatrix0) <= 1.0e-12
        || norm3(generatrix1) <= 1.0e-12
        || normal_delta_norm <= 1.0e-12
    {
        return Ok(None);
    }

    let semi_angle_magnitude = generatrix_delta_norm.atan2(normal_delta_norm);
    let semi_angle_sign = if dot3(generatrix_delta, normal_delta) < 0.0 {
        -1.0
    } else {
        1.0
    };
    let semi_angle = semi_angle_sign * semi_angle_magnitude;
    let sin_angle = semi_angle.sin();
    let cos_angle = semi_angle.cos();
    if cos_angle.abs() <= 1.0e-12 {
        return Ok(None);
    }

    let axis = normalize3(subtract3(
        scale3(generatrix0, cos_angle),
        scale3(normal0, sin_angle),
    ));
    let radial0 = normalize3(add3(
        scale3(generatrix0, sin_angle),
        scale3(normal0, cos_angle),
    ));
    let radial1 = normalize3(add3(
        scale3(generatrix1, sin_angle),
        scale3(normal1, cos_angle),
    ));
    let x_direction = scale3(
        subtract3(scale3(radial0, u1.sin()), scale3(radial1, u0.sin())),
        1.0 / denominator,
    );
    let y_direction = scale3(
        subtract3(scale3(radial1, u0.cos()), scale3(radial0, u1.cos())),
        1.0 / denominator,
    );
    let radial_delta = subtract3(radial1, radial0);
    let radial_delta_norm2 = dot3(radial_delta, radial_delta);
    if norm3(axis) <= 1.0e-12
        || norm3(radial0) <= 1.0e-12
        || norm3(radial1) <= 1.0e-12
        || norm3(x_direction) <= 1.0e-12
        || norm3(y_direction) <= 1.0e-12
        || radial_delta_norm2 <= 1.0e-12
    {
        return Ok(None);
    }

    let radius_at_v_min = dot3(
        subtract3(probe_sample.position, base_sample.position),
        radial_delta,
    ) / radial_delta_norm2;
    let reference_radius = radius_at_v_min - geometry.v_min * sin_angle;
    let payload = ConePayload {
        origin: subtract3(
            base_sample.position,
            add3(
                scale3(axis, geometry.v_min * cos_angle),
                scale3(radial0, radius_at_v_min),
            ),
        ),
        axis,
        x_direction,
        y_direction,
        reference_radius,
        semi_angle,
    };

    if !approx_points_eq(
        sample_cone(payload, [u0, geometry.v_min]).position,
        base_sample.position,
        1.0e-7,
    ) || !approx_points_eq(
        sample_cone(payload, [u1, geometry.v_min]).position,
        probe_sample.position,
        1.0e-7,
    ) || !approx_points_eq(
        sample_cone(payload, [u0, geometry.v_max]).position,
        base_top_sample.position,
        1.0e-7,
    ) || !approx_points_eq(
        sample_cone(payload, [u1, geometry.v_max]).position,
        probe_top_sample.position,
        1.0e-7,
    ) || !approx_points_eq(
        sample_cone(payload, [u0, geometry.v_min]).normal,
        normal0,
        1.0e-7,
    ) || !approx_points_eq(
        sample_cone(payload, [u1, geometry.v_min]).normal,
        normal1,
        1.0e-7,
    ) {
        return Ok(None);
    }

    Ok(Some(payload))
}

pub(super) fn ported_sphere_payload(
    context: &Context,
    shape: &Shape,
    geometry: FaceGeometry,
) -> Result<Option<SpherePayload>, Error> {
    let orientation = context.shape_orientation(shape)?;
    ported_sphere_payload_from_samples(geometry, orientation, |uv| {
        context.face_sample_occt(shape, uv)
    })
}

fn ported_sphere_payload_from_samples<F>(
    geometry: FaceGeometry,
    orientation: Orientation,
    mut sample_face: F,
) -> Result<Option<SpherePayload>, Error>
where
    F: FnMut([f64; 2]) -> Result<FaceSample, Error>,
{
    if geometry.kind != SurfaceKind::Sphere {
        return Ok(None);
    }

    let u0 = geometry.u_min;
    let u1 = match select_periodic_probe_parameter(geometry.u_min, geometry.u_max) {
        Some(parameter) => parameter,
        None => return Ok(None),
    };
    let denominator_u = (u1 - u0).sin();
    if denominator_u.abs() <= 1.0e-6 {
        return Ok(None);
    }

    let (v0, v1) = match select_sphere_latitude_pair(geometry.v_min, geometry.v_max) {
        Some(pair) => pair,
        None => return Ok(None),
    };
    let denominator_v = (v1 - v0).sin();
    if denominator_v.abs() <= 1.0e-6 || v0.cos().abs() <= 1.0e-6 {
        return Ok(None);
    }

    let normal_sign = if matches!(orientation, Orientation::Reversed) {
        -1.0
    } else {
        1.0
    };

    let base_sample = sample_face([u0, v0])?;
    let longitude_sample = sample_face([u1, v0])?;
    let latitude_sample = sample_face([u0, v1])?;
    let latitude_longitude_sample = sample_face([u1, v1])?;
    let normal00 = scale3(base_sample.normal, normal_sign);
    let normal10 = scale3(longitude_sample.normal, normal_sign);
    let normal01 = scale3(latitude_sample.normal, normal_sign);

    let normal_delta = subtract3(normal01, normal00);
    let normal_delta_norm2 = dot3(normal_delta, normal_delta);
    if normal_delta_norm2 <= 1.0e-12 {
        return Ok(None);
    }

    let radius = dot3(
        subtract3(latitude_sample.position, base_sample.position),
        normal_delta,
    ) / normal_delta_norm2;
    if radius.abs() <= 1.0e-12 {
        return Ok(None);
    }

    let normal = normalize3(scale3(
        subtract3(scale3(normal01, v0.cos()), scale3(normal00, v1.cos())),
        1.0 / denominator_v,
    ));
    let radial0 = scale3(
        subtract3(normal00, scale3(normal, v0.sin())),
        1.0 / v0.cos(),
    );
    let radial1 = scale3(
        subtract3(normal10, scale3(normal, v0.sin())),
        1.0 / v0.cos(),
    );
    let x_direction = normalize3(scale3(
        subtract3(scale3(radial0, u1.sin()), scale3(radial1, u0.sin())),
        1.0 / denominator_u,
    ));
    let y_direction = normalize3(scale3(
        subtract3(scale3(radial1, u0.cos()), scale3(radial0, u1.cos())),
        1.0 / denominator_u,
    ));
    if norm3(normal) <= 1.0e-12 || norm3(x_direction) <= 1.0e-12 || norm3(y_direction) <= 1.0e-12 {
        return Ok(None);
    }

    let payload = SpherePayload {
        center: subtract3(base_sample.position, scale3(normal00, radius)),
        normal,
        x_direction,
        y_direction,
        radius,
    };

    if !approx_points_eq(
        sample_sphere(payload, [u0, v0]).position,
        base_sample.position,
        1.0e-7,
    ) || !approx_points_eq(
        sample_sphere(payload, [u1, v0]).position,
        longitude_sample.position,
        1.0e-7,
    ) || !approx_points_eq(
        sample_sphere(payload, [u0, v1]).position,
        latitude_sample.position,
        1.0e-7,
    ) || !approx_points_eq(
        sample_sphere(payload, [u1, v1]).position,
        latitude_longitude_sample.position,
        1.0e-7,
    ) || !approx_points_eq(sample_sphere(payload, [u0, v0]).normal, normal00, 1.0e-7)
        || !approx_points_eq(sample_sphere(payload, [u1, v0]).normal, normal10, 1.0e-7)
        || !approx_points_eq(sample_sphere(payload, [u0, v1]).normal, normal01, 1.0e-7)
    {
        return Ok(None);
    }

    Ok(Some(payload))
}

pub(super) fn ported_torus_payload(
    context: &Context,
    shape: &Shape,
    geometry: FaceGeometry,
) -> Result<Option<TorusPayload>, Error> {
    let orientation = context.shape_orientation(shape)?;
    ported_torus_payload_from_samples(geometry, orientation, |uv| {
        context.face_sample_occt(shape, uv)
    })
}

fn ported_torus_payload_from_samples<F>(
    geometry: FaceGeometry,
    orientation: Orientation,
    mut sample_face: F,
) -> Result<Option<TorusPayload>, Error>
where
    F: FnMut([f64; 2]) -> Result<FaceSample, Error>,
{
    if geometry.kind != SurfaceKind::Torus {
        return Ok(None);
    }

    let u0 = geometry.u_min;
    let u1 = match select_periodic_probe_parameter(geometry.u_min, geometry.u_max) {
        Some(parameter) => parameter,
        None => return Ok(None),
    };
    let denominator_u = (u1 - u0).sin();
    if denominator_u.abs() <= 1.0e-6 {
        return Ok(None);
    }

    let v0 = geometry.v_min;
    let v1 = match select_periodic_probe_parameter(geometry.v_min, geometry.v_max) {
        Some(parameter) => parameter,
        None => return Ok(None),
    };
    let denominator_v = (v1 - v0).sin();
    if denominator_v.abs() <= 1.0e-6 {
        return Ok(None);
    }

    let normal_sign = if matches!(orientation, Orientation::Reversed) {
        -1.0
    } else {
        1.0
    };

    let sample00 = sample_face([u0, v0])?;
    let sample01 = sample_face([u0, v1])?;
    let sample10 = sample_face([u1, v0])?;
    let sample11 = sample_face([u1, v1])?;
    let normal00 = scale3(sample00.normal, normal_sign);
    let normal01 = scale3(sample01.normal, normal_sign);
    let normal10 = scale3(sample10.normal, normal_sign);
    let normal11 = scale3(sample11.normal, normal_sign);

    let radial0 = normalize3(scale3(
        subtract3(scale3(normal00, v1.sin()), scale3(normal01, v0.sin())),
        1.0 / denominator_v,
    ));
    let radial1 = normalize3(scale3(
        subtract3(scale3(normal10, v1.sin()), scale3(normal11, v0.sin())),
        1.0 / denominator_v,
    ));
    let axis0 = normalize3(scale3(
        subtract3(scale3(normal01, v0.cos()), scale3(normal00, v1.cos())),
        1.0 / denominator_v,
    ));
    let axis1 = normalize3(scale3(
        subtract3(scale3(normal11, v0.cos()), scale3(normal10, v1.cos())),
        1.0 / denominator_v,
    ));
    if norm3(radial0) <= 1.0e-12
        || norm3(radial1) <= 1.0e-12
        || norm3(axis0) <= 1.0e-12
        || norm3(axis1) <= 1.0e-12
        || !approx_points_eq(axis0, axis1, 1.0e-7)
    {
        return Ok(None);
    }

    let normal_delta0 = subtract3(normal01, normal00);
    let normal_delta1 = subtract3(normal11, normal10);
    let normal_delta0_norm2 = dot3(normal_delta0, normal_delta0);
    let normal_delta1_norm2 = dot3(normal_delta1, normal_delta1);
    if normal_delta0_norm2 <= 1.0e-12 || normal_delta1_norm2 <= 1.0e-12 {
        return Ok(None);
    }

    let minor_radius0 = dot3(
        subtract3(sample01.position, sample00.position),
        normal_delta0,
    ) / normal_delta0_norm2;
    let minor_radius1 = dot3(
        subtract3(sample11.position, sample10.position),
        normal_delta1,
    ) / normal_delta1_norm2;
    let minor_radius = 0.5 * (minor_radius0 + minor_radius1);
    if minor_radius.abs() <= 1.0e-12 || (minor_radius0 - minor_radius1).abs() > 1.0e-7 {
        return Ok(None);
    }

    let tube_center00 = subtract3(sample00.position, scale3(normal00, minor_radius));
    let tube_center01 = subtract3(sample01.position, scale3(normal01, minor_radius));
    let tube_center10 = subtract3(sample10.position, scale3(normal10, minor_radius));
    let tube_center11 = subtract3(sample11.position, scale3(normal11, minor_radius));
    if !approx_points_eq(tube_center00, tube_center01, 1.0e-7)
        || !approx_points_eq(tube_center10, tube_center11, 1.0e-7)
    {
        return Ok(None);
    }

    let radial_delta = subtract3(radial1, radial0);
    let radial_delta_norm2 = dot3(radial_delta, radial_delta);
    if radial_delta_norm2 <= 1.0e-12 {
        return Ok(None);
    }

    let major_radius0 =
        dot3(subtract3(tube_center10, tube_center00), radial_delta) / radial_delta_norm2;
    let major_radius1 =
        dot3(subtract3(tube_center11, tube_center01), radial_delta) / radial_delta_norm2;
    let major_radius = 0.5 * (major_radius0 + major_radius1);
    if major_radius.abs() <= 1.0e-12 || (major_radius0 - major_radius1).abs() > 1.0e-7 {
        return Ok(None);
    }

    let axis = axis0;
    let x_direction = normalize3(scale3(
        subtract3(scale3(radial0, u1.sin()), scale3(radial1, u0.sin())),
        1.0 / denominator_u,
    ));
    let y_direction = normalize3(scale3(
        subtract3(scale3(radial1, u0.cos()), scale3(radial0, u1.cos())),
        1.0 / denominator_u,
    ));
    if norm3(x_direction) <= 1.0e-12 || norm3(y_direction) <= 1.0e-12 {
        return Ok(None);
    }

    let payload = TorusPayload {
        center: subtract3(tube_center00, scale3(radial0, major_radius)),
        axis,
        x_direction,
        y_direction,
        major_radius,
        minor_radius,
    };

    if !approx_points_eq(
        sample_torus(payload, [u0, v0]).position,
        sample00.position,
        1.0e-7,
    ) || !approx_points_eq(
        sample_torus(payload, [u0, v1]).position,
        sample01.position,
        1.0e-7,
    ) || !approx_points_eq(
        sample_torus(payload, [u1, v0]).position,
        sample10.position,
        1.0e-7,
    ) || !approx_points_eq(
        sample_torus(payload, [u1, v1]).position,
        sample11.position,
        1.0e-7,
    ) || !approx_points_eq(sample_torus(payload, [u0, v0]).normal, normal00, 1.0e-7)
        || !approx_points_eq(sample_torus(payload, [u0, v1]).normal, normal01, 1.0e-7)
        || !approx_points_eq(sample_torus(payload, [u1, v0]).normal, normal10, 1.0e-7)
        || !approx_points_eq(sample_torus(payload, [u1, v1]).normal, normal11, 1.0e-7)
    {
        return Ok(None);
    }

    Ok(Some(payload))
}

pub(super) fn ported_offset_basis_surface_payload(
    context: &Context,
    shape: &Shape,
    offset: f64,
    basis_geometry: FaceGeometry,
) -> Result<Option<PortedSurface>, Error> {
    let orientation = context.shape_orientation(shape)?;
    let natural_normal_sign = if matches!(orientation, Orientation::Reversed) {
        -1.0
    } else {
        1.0
    };
    let mut sample_basis = |uv| {
        let offset_sample = context.face_sample_occt(shape, uv)?;
        let natural_normal = scale3(offset_sample.normal, natural_normal_sign);
        Ok(FaceSample {
            position: subtract3(offset_sample.position, scale3(natural_normal, offset)),
            normal: offset_sample.normal,
        })
    };

    match basis_geometry.kind {
        SurfaceKind::Plane => Ok(ported_plane_payload_from_samples(
            basis_geometry,
            &mut sample_basis,
        )?
        .map(PortedSurface::Plane)),
        SurfaceKind::Cylinder => Ok(ported_cylinder_payload_from_samples(
            basis_geometry,
            orientation,
            &mut sample_basis,
        )?
        .map(PortedSurface::Cylinder)),
        SurfaceKind::Cone => {
            Ok(
                ported_cone_payload_from_samples(basis_geometry, orientation, &mut sample_basis)?
                    .map(PortedSurface::Cone),
            )
        }
        SurfaceKind::Sphere => {
            Ok(
                ported_sphere_payload_from_samples(basis_geometry, orientation, &mut sample_basis)?
                    .map(PortedSurface::Sphere),
            )
        }
        SurfaceKind::Torus => {
            Ok(
                ported_torus_payload_from_samples(basis_geometry, orientation, &mut sample_basis)?
                    .map(PortedSurface::Torus),
            )
        }
        _ => Ok(None),
    }
}

pub(super) fn ported_offset_basis_swept_surface_payload(
    context: &Context,
    shape: &Shape,
    offset: f64,
    basis_geometry: FaceGeometry,
) -> Result<Option<PortedSweptSurface>, Error> {
    let orientation = context.shape_orientation(shape)?;
    let natural_normal_sign = if matches!(orientation, Orientation::Reversed) {
        -1.0
    } else {
        1.0
    };
    let mut sample_basis = |uv| {
        let offset_sample = context.face_sample_occt(shape, uv)?;
        let natural_normal = scale3(offset_sample.normal, natural_normal_sign);
        Ok(FaceSample {
            position: subtract3(offset_sample.position, scale3(natural_normal, offset)),
            normal: natural_normal,
        })
    };

    match basis_geometry.kind {
        SurfaceKind::Extrusion => {
            ported_offset_basis_extrusion_payload_from_samples(basis_geometry, &mut sample_basis)
        }
        SurfaceKind::Revolution => {
            ported_offset_basis_revolution_payload_from_samples(basis_geometry, &mut sample_basis)
        }
        _ => Ok(None),
    }
}

fn ported_offset_basis_extrusion_payload_from_samples<F>(
    basis_geometry: FaceGeometry,
    sample_basis: &mut F,
) -> Result<Option<PortedSweptSurface>, Error>
where
    F: FnMut([f64; 2]) -> Result<FaceSample, Error>,
{
    for curve_on_u in [true, false] {
        let direction =
            match extrusion_direction_from_samples(basis_geometry, curve_on_u, sample_basis)? {
                Some(direction) => direction,
                None => continue,
            };
        let (sweep_start, sweep_end) = sweep_parameter_range(basis_geometry, curve_on_u);
        let reference_sweep = reference_sweep_parameter(sweep_start, sweep_end);
        let mut sample_curve_position = |parameter| {
            let uv = swept_uv(curve_on_u, parameter, reference_sweep);
            let sample = sample_basis(uv)?;
            Ok(subtract3(
                sample.position,
                scale3(direction, reference_sweep),
            ))
        };
        let (basis_curve, basis_curve_geometry) =
            match ported_swept_basis_curve_from_position_samples(
                basis_geometry,
                curve_on_u,
                &mut sample_curve_position,
            )? {
                Some(value) => value,
                None => continue,
            };
        let payload = ExtrusionSurfacePayload {
            direction,
            basis_curve_kind: ported_curve_kind(basis_curve),
        };

        if validates_extrusion_basis_surface(
            basis_geometry,
            payload,
            basis_curve,
            basis_curve_geometry,
            sample_basis,
        )? {
            return Ok(Some(PortedSweptSurface::Extrusion {
                payload,
                basis_curve,
                basis_geometry: basis_curve_geometry,
            }));
        }
    }

    Ok(None)
}

fn ported_offset_basis_revolution_payload_from_samples<F>(
    basis_geometry: FaceGeometry,
    sample_basis: &mut F,
) -> Result<Option<PortedSweptSurface>, Error>
where
    F: FnMut([f64; 2]) -> Result<FaceSample, Error>,
{
    for curve_on_u in [true, false] {
        let (axis_origin, axis_direction) =
            match revolution_axis_from_samples(basis_geometry, curve_on_u, sample_basis)? {
                Some(axis) => axis,
                None => continue,
            };
        let (sweep_start, sweep_end) = sweep_parameter_range(basis_geometry, curve_on_u);
        let reference_sweep = reference_sweep_parameter(sweep_start, sweep_end);
        let mut sample_curve_position = |parameter| {
            let uv = swept_uv(curve_on_u, parameter, reference_sweep);
            let sample = sample_basis(uv)?;
            Ok(rotate_point_about_axis(
                sample.position,
                axis_origin,
                axis_direction,
                -reference_sweep,
            ))
        };
        let (basis_curve, basis_curve_geometry) =
            match ported_swept_basis_curve_from_position_samples(
                basis_geometry,
                curve_on_u,
                &mut sample_curve_position,
            )? {
                Some(value) => value,
                None => continue,
            };
        let payload = RevolutionSurfacePayload {
            axis_origin,
            axis_direction,
            basis_curve_kind: ported_curve_kind(basis_curve),
        };

        if validates_revolution_basis_surface(
            basis_geometry,
            payload,
            basis_curve,
            basis_curve_geometry,
            sample_basis,
        )? {
            return Ok(Some(PortedSweptSurface::Revolution {
                payload,
                basis_curve,
                basis_geometry: basis_curve_geometry,
            }));
        }
    }

    Ok(None)
}

fn extrusion_direction_from_samples<F>(
    geometry: FaceGeometry,
    curve_on_u: bool,
    sample_basis: &mut F,
) -> Result<Option<[f64; 3]>, Error>
where
    F: FnMut([f64; 2]) -> Result<FaceSample, Error>,
{
    let (curve_start, curve_end) = curve_parameter_range(geometry, curve_on_u);
    let curve_mid = 0.5 * (curve_start + curve_end);
    let (sweep_start, sweep_end) = sweep_parameter_range(geometry, curve_on_u);
    let sweep_span = sweep_end - sweep_start;
    if sweep_span.abs() <= 1.0e-12 {
        return Ok(None);
    }

    let derivative_at_curve = |sample_basis: &mut F, curve_parameter| -> Result<[f64; 3], Error> {
        let start = sample_basis(swept_uv(curve_on_u, curve_parameter, sweep_start))?;
        let end = sample_basis(swept_uv(curve_on_u, curve_parameter, sweep_end))?;
        Ok(scale3(
            subtract3(end.position, start.position),
            1.0 / sweep_span,
        ))
    };

    let derivative0 = derivative_at_curve(sample_basis, curve_start)?;
    let derivative1 = derivative_at_curve(sample_basis, curve_mid)?;
    let direction0 = normalize3(derivative0);
    let direction1 = normalize3(derivative1);
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

fn revolution_axis_from_samples<F>(
    geometry: FaceGeometry,
    curve_on_u: bool,
    sample_basis: &mut F,
) -> Result<Option<([f64; 3], [f64; 3])>, Error>
where
    F: FnMut([f64; 2]) -> Result<FaceSample, Error>,
{
    let (curve_start, curve_end) = curve_parameter_range(geometry, curve_on_u);
    let curve_probe = 0.5 * (curve_start + curve_end);
    let (sweep_start, sweep_end) = sweep_parameter_range(geometry, curve_on_u);
    let sweep_parameters = trigonometric_curve_probe_parameters(sweep_start, sweep_end);
    let sweep_parameters = match select_trigonometric_curve_parameters(sweep_parameters) {
        Some(parameters) => parameters,
        None => return Ok(None),
    };

    let (origin, axis_direction) = match revolution_axis_components_at_curve_parameter(
        curve_on_u,
        curve_start,
        sweep_parameters,
        sample_basis,
    )? {
        Some(axis) => axis,
        None => return Ok(None),
    };
    let (probe_origin, probe_axis_direction) = match revolution_axis_components_at_curve_parameter(
        curve_on_u,
        curve_probe,
        sweep_parameters,
        sample_basis,
    )? {
        Some(axis) => axis,
        None => return Ok(None),
    };

    if dot3(axis_direction, probe_axis_direction) < 1.0 - 1.0e-7 {
        return Ok(None);
    }
    let center_delta = subtract3(probe_origin, origin);
    if norm3(center_delta) > 1.0e-8
        && norm3(cross3(center_delta, axis_direction)) > 1.0e-6 * norm3(center_delta)
    {
        return Ok(None);
    }

    Ok(Some((origin, axis_direction)))
}

fn revolution_axis_components_at_curve_parameter<F>(
    curve_on_u: bool,
    curve_parameter: f64,
    sweep_parameters: [f64; 3],
    sample_basis: &mut F,
) -> Result<Option<([f64; 3], [f64; 3])>, Error>
where
    F: FnMut([f64; 2]) -> Result<FaceSample, Error>,
{
    let positions = [
        sample_basis(swept_uv(curve_on_u, curve_parameter, sweep_parameters[0]))?.position,
        sample_basis(swept_uv(curve_on_u, curve_parameter, sweep_parameters[1]))?.position,
        sample_basis(swept_uv(curve_on_u, curve_parameter, sweep_parameters[2]))?.position,
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

fn ported_swept_basis_curve_from_position_samples<F>(
    basis_geometry: FaceGeometry,
    curve_on_u: bool,
    sample_position: &mut F,
) -> Result<Option<(PortedCurve, EdgeGeometry)>, Error>
where
    F: FnMut(f64) -> Result<[f64; 3], Error>,
{
    let closed = {
        let (start, end) = curve_parameter_range(basis_geometry, curve_on_u);
        approx_points_eq(sample_position(start)?, sample_position(end)?, 1.0e-7)
    };

    let line_geometry =
        swept_basis_curve_geometry(basis_geometry, curve_on_u, CurveKind::Line, closed);
    if let Some(payload) =
        ported_line_payload_from_position_samples(line_geometry, &mut *sample_position)?
    {
        return Ok(Some((PortedCurve::Line(payload), line_geometry)));
    }

    let circle_geometry =
        swept_basis_curve_geometry(basis_geometry, curve_on_u, CurveKind::Circle, closed);
    if let Some(payload) =
        ported_circle_payload_from_position_samples(circle_geometry, &mut *sample_position)?
    {
        return Ok(Some((PortedCurve::Circle(payload), circle_geometry)));
    }

    let ellipse_geometry =
        swept_basis_curve_geometry(basis_geometry, curve_on_u, CurveKind::Ellipse, closed);
    if let Some(payload) =
        ported_ellipse_payload_from_position_samples(ellipse_geometry, &mut *sample_position)?
    {
        return Ok(Some((PortedCurve::Ellipse(payload), ellipse_geometry)));
    }

    Ok(None)
}

fn ported_line_payload_from_position_samples<F>(
    geometry: EdgeGeometry,
    mut sample_position: F,
) -> Result<Option<LinePayload>, Error>
where
    F: FnMut(f64) -> Result<[f64; 3], Error>,
{
    if geometry.kind != CurveKind::Line {
        return Ok(None);
    }

    let endpoints = EdgeEndpoints {
        start: sample_position(geometry.start_parameter)?,
        end: sample_position(geometry.end_parameter)?,
    };
    let payload = match ported_line_payload_from_endpoints(geometry, endpoints) {
        Some(payload) => payload,
        None => return Ok(None),
    };

    for fraction in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let parameter = geometry.start_parameter
            + (geometry.end_parameter - geometry.start_parameter) * fraction;
        if !approx_points_eq(
            sample_line(payload, parameter).position,
            sample_position(parameter)?,
            1.0e-7,
        ) {
            return Ok(None);
        }
    }

    Ok(Some(payload))
}

fn validates_extrusion_basis_surface<F>(
    face_geometry: FaceGeometry,
    payload: ExtrusionSurfacePayload,
    basis_curve: PortedCurve,
    basis_geometry: EdgeGeometry,
    sample_basis: &mut F,
) -> Result<bool, Error>
where
    F: FnMut([f64; 2]) -> Result<FaceSample, Error>,
{
    for uv_t in [
        [0.0, 0.0],
        [0.25, 0.75],
        [0.5, 0.5],
        [0.75, 0.25],
        [1.0, 1.0],
    ] {
        let uv = normalized_uv_to_uv(face_geometry, uv_t);
        let actual = sample_basis(uv)?;
        let expected = sample_extrusion_surface_normalized(
            basis_curve,
            face_geometry,
            basis_geometry,
            uv_t,
            payload.direction,
            Orientation::Forward,
        );
        if !approx_points_eq(expected.position, actual.position, 1.0e-6)
            || !approx_points_eq(expected.normal, actual.normal, 1.0e-6)
        {
            return Ok(false);
        }
    }
    Ok(true)
}

fn validates_revolution_basis_surface<F>(
    face_geometry: FaceGeometry,
    payload: RevolutionSurfacePayload,
    basis_curve: PortedCurve,
    basis_geometry: EdgeGeometry,
    sample_basis: &mut F,
) -> Result<bool, Error>
where
    F: FnMut([f64; 2]) -> Result<FaceSample, Error>,
{
    for uv_t in [
        [0.0, 0.0],
        [0.25, 0.75],
        [0.5, 0.5],
        [0.75, 0.25],
        [1.0, 1.0],
    ] {
        let uv = normalized_uv_to_uv(face_geometry, uv_t);
        let actual = sample_basis(uv)?;
        let expected = sample_revolution_surface_normalized(
            basis_curve,
            face_geometry,
            basis_geometry,
            uv_t,
            payload.axis_origin,
            payload.axis_direction,
            Orientation::Forward,
        );
        if !approx_points_eq(expected.position, actual.position, 1.0e-6)
            || !approx_points_eq(expected.normal, actual.normal, 1.0e-6)
        {
            return Ok(false);
        }
    }
    Ok(true)
}

fn swept_basis_curve_geometry(
    face_geometry: FaceGeometry,
    curve_on_u: bool,
    kind: CurveKind,
    is_closed: bool,
) -> EdgeGeometry {
    let (start_parameter, end_parameter) = curve_parameter_range(face_geometry, curve_on_u);
    EdgeGeometry {
        kind,
        start_parameter,
        end_parameter,
        is_closed,
        is_periodic: matches!(kind, CurveKind::Circle | CurveKind::Ellipse),
        period: if matches!(kind, CurveKind::Circle | CurveKind::Ellipse) {
            TAU
        } else {
            0.0
        },
    }
}

fn curve_parameter_range(face_geometry: FaceGeometry, curve_on_u: bool) -> (f64, f64) {
    if curve_on_u {
        (face_geometry.u_min, face_geometry.u_max)
    } else {
        (face_geometry.v_min, face_geometry.v_max)
    }
}

fn sweep_parameter_range(face_geometry: FaceGeometry, curve_on_u: bool) -> (f64, f64) {
    if curve_on_u {
        (face_geometry.v_min, face_geometry.v_max)
    } else {
        (face_geometry.u_min, face_geometry.u_max)
    }
}

fn swept_uv(curve_on_u: bool, curve_parameter: f64, sweep_parameter: f64) -> [f64; 2] {
    if curve_on_u {
        [curve_parameter, sweep_parameter]
    } else {
        [sweep_parameter, curve_parameter]
    }
}

fn reference_sweep_parameter(start: f64, end: f64) -> f64 {
    if start <= 0.0 && 0.0 <= end {
        0.0
    } else {
        start
    }
}

fn normalized_uv_to_uv(geometry: FaceGeometry, uv_t: [f64; 2]) -> [f64; 2] {
    [
        geometry.u_min + (geometry.u_max - geometry.u_min) * uv_t[0],
        geometry.v_min + (geometry.v_max - geometry.v_min) * uv_t[1],
    ]
}

fn ported_curve_kind(curve: PortedCurve) -> CurveKind {
    match curve {
        PortedCurve::Line(_) => CurveKind::Line,
        PortedCurve::Circle(_) => CurveKind::Circle,
        PortedCurve::Ellipse(_) => CurveKind::Ellipse,
    }
}

fn normalize_periodic_parameter(value: f64, period: f64) -> f64 {
    let period = period.abs();
    if period <= 1.0e-12 {
        return value;
    }

    let mut normalized = value % period;
    if normalized < 0.0 {
        normalized += period;
    }
    if normalized >= period - 1.0e-9 {
        0.0
    } else {
        normalized
    }
}

fn snap_periodic_parameter(value: f64, period: f64) -> f64 {
    if value.abs() <= 1.0e-9 {
        0.0
    } else if (value - period).abs() <= 1.0e-9 {
        period
    } else if (value + period).abs() <= 1.0e-9 {
        0.0
    } else {
        value
    }
}

fn select_periodic_probe_parameter(start: f64, end: f64) -> Option<f64> {
    [0.25, 0.5, 0.75, 1.0]
        .into_iter()
        .map(|fraction| start + (end - start) * fraction)
        .max_by(|lhs, rhs| {
            (lhs - start)
                .sin()
                .abs()
                .total_cmp(&(rhs - start).sin().abs())
        })
        .filter(|candidate| ((*candidate - start).sin()).abs() > 1.0e-6)
}

fn select_sphere_latitude_pair(start: f64, end: f64) -> Option<(f64, f64)> {
    let candidates = [0.0, 0.25, 0.5, 0.75, 1.0].map(|fraction| start + (end - start) * fraction);
    candidates
        .into_iter()
        .flat_map(|v0| {
            candidates
                .into_iter()
                .filter(move |&v1| (v1 - v0).abs() > 1.0e-12)
                .map(move |v1| (v0, v1))
        })
        .max_by(|(lhs0, lhs1), (rhs0, rhs1)| {
            (lhs0.cos().abs() * (lhs1 - lhs0).sin().abs())
                .total_cmp(&(rhs0.cos().abs() * (rhs1 - rhs0).sin().abs()))
        })
        .filter(|(v0, v1)| v0.cos().abs() * (v1 - v0).sin().abs() > 1.0e-6)
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

fn line_parameter(payload: LinePayload, point: [f64; 3]) -> Option<f64> {
    let direction_norm_sq = dot3(payload.direction, payload.direction);
    if direction_norm_sq <= 1.0e-24 {
        None
    } else {
        Some(dot3(subtract3(point, payload.origin), payload.direction) / direction_norm_sq)
    }
}

pub(super) fn circle_parameter(payload: CirclePayload, point: [f64; 3]) -> f64 {
    subtract3(point, payload.center).atan2_components(payload.x_direction, payload.y_direction)
}

pub(super) fn ellipse_parameter(payload: EllipsePayload, point: [f64; 3]) -> Option<f64> {
    if payload.major_radius.abs() <= 1.0e-12 || payload.minor_radius.abs() <= 1.0e-12 {
        return None;
    }

    let relative = subtract3(point, payload.center);
    Some(
        (dot3(relative, payload.y_direction) / payload.minor_radius)
            .atan2(dot3(relative, payload.x_direction) / payload.major_radius),
    )
}

fn approx_points_eq(lhs: [f64; 3], rhs: [f64; 3], tolerance: f64) -> bool {
    (lhs[0] - rhs[0]).abs() <= tolerance
        && (lhs[1] - rhs[1]).abs() <= tolerance
        && (lhs[2] - rhs[2]).abs() <= tolerance
}
