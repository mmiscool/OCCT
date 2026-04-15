use crate::{EdgeGeometry, FaceGeometry, PlanePayload};

use super::{dot3, subtract3, PortedCurve, PortedSurface};

pub(crate) fn planar_wire_signed_area(
    plane: PlanePayload,
    curve_segments: &[(PortedCurve, EdgeGeometry)],
) -> f64 {
    0.5 * curve_segments
        .iter()
        .map(|(curve, geometry)| {
            signed_scalar_integral(
                geometry.start_parameter,
                geometry.end_parameter,
                |parameter| {
                    let evaluation = curve.evaluate(parameter);
                    let relative = subtract3(evaluation.position, plane.origin);
                    let x = dot3(relative, plane.x_direction);
                    let y = dot3(relative, plane.y_direction);
                    let dx = dot3(evaluation.derivative, plane.x_direction);
                    let dy = dot3(evaluation.derivative, plane.y_direction);
                    x * dy - y * dx
                },
            )
        })
        .sum::<f64>()
}

pub(crate) fn analytic_sampled_wire_signed_area(
    surface: PortedSurface,
    face_geometry: FaceGeometry,
    points: &[[f64; 3]],
) -> Option<f64> {
    let mut uv_points = Vec::with_capacity(points.len());
    for &point in points {
        let mut uv = surface.point_to_uv(point)?;
        if let Some(previous) = uv_points.last().copied() {
            uv = unwrap_uv(previous, uv, face_geometry);
        }
        uv_points.push(uv);
    }

    analytic_wire_signed_area_from_uv_points(surface, face_geometry, &uv_points)
}

pub(crate) fn analytic_sampled_wire_signed_volume(
    surface: PortedSurface,
    face_geometry: FaceGeometry,
    points: &[[f64; 3]],
) -> Option<f64> {
    let mut uv_points = Vec::with_capacity(points.len());
    for &point in points {
        let mut uv = surface.point_to_uv(point)?;
        if let Some(previous) = uv_points.last().copied() {
            uv = unwrap_uv(previous, uv, face_geometry);
        }
        uv_points.push(uv);
    }

    if uv_points.len() < 2 {
        return Some(0.0);
    }

    let first = uv_points[0];
    let closing = unwrap_uv(*uv_points.last()?, first, face_geometry);
    let mut volume = 0.0;
    for window in uv_points.windows(2) {
        let start = window[0];
        let end = window[1];
        volume += segment_volume_integral(surface, start, end);
    }
    volume += segment_volume_integral(surface, *uv_points.last()?, closing);
    Some(volume)
}

pub(super) fn length_integral<F>(start: f64, end: f64, integrand: F) -> f64
where
    F: Fn(f64) -> f64,
{
    signed_scalar_integral(start, end, |parameter| integrand(parameter).abs()).abs()
}

fn analytic_wire_signed_area_from_uv_points(
    surface: PortedSurface,
    face_geometry: FaceGeometry,
    uv_points: &[[f64; 2]],
) -> Option<f64> {
    if uv_points.len() < 2 {
        return Some(0.0);
    }

    let first = uv_points[0];
    let closing = unwrap_uv(*uv_points.last()?, first, face_geometry);
    let mut area = 0.0;
    for window in uv_points.windows(2) {
        let start = window[0];
        let end = window[1];
        area += segment_area_integral(surface, start, end);
    }
    area += segment_area_integral(surface, *uv_points.last()?, closing);
    Some(area)
}

fn signed_scalar_integral<F>(start: f64, end: f64, integrand: F) -> f64
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
    sign * adaptive_simpson(&integrand, a, b, fa, fm, fb, 1.0e-9, 12)
}

fn segment_area_integral(surface: PortedSurface, start: [f64; 2], end: [f64; 2]) -> f64 {
    let start_potential = surface.area_potential(start[1]);
    let end_potential = surface.area_potential(end[1]);
    -0.5 * (start_potential + end_potential) * (end[0] - start[0])
}

fn segment_volume_integral(surface: PortedSurface, start: [f64; 2], end: [f64; 2]) -> f64 {
    let start_potential = surface.volume_potential(start);
    let end_potential = surface.volume_potential(end);
    0.5 * (start_potential + end_potential) * (end[1] - start[1])
}

fn unwrap_uv(previous: [f64; 2], mut current: [f64; 2], geometry: FaceGeometry) -> [f64; 2] {
    if geometry.is_u_periodic && geometry.u_period.abs() > 1.0e-12 {
        current[0] = unwrap_periodic_component(previous[0], current[0], geometry.u_period);
    }
    if geometry.is_v_periodic && geometry.v_period.abs() > 1.0e-12 {
        current[1] = unwrap_periodic_component(previous[1], current[1], geometry.v_period);
    }
    current
}

fn unwrap_periodic_component(previous: f64, current: f64, period: f64) -> f64 {
    let mut adjusted = current;
    let half_period = 0.5 * period.abs();
    while adjusted - previous > half_period {
        adjusted -= period.abs();
    }
    while adjusted - previous < -half_period {
        adjusted += period.abs();
    }
    adjusted
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
