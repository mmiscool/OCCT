use crate::{
    CirclePayload, ConePayload, Context, CurveKind, CylinderPayload, EdgeGeometry, EdgeSample,
    EllipsePayload, Error, FaceGeometry, FaceSample, LinePayload, Orientation, PlanePayload, Shape,
    SpherePayload, SurfaceKind, TorusPayload,
};

#[derive(Clone, Copy, Debug)]
struct CurveEvaluation {
    position: [f64; 3],
    derivative: [f64; 3],
}

#[derive(Clone, Copy, Debug)]
pub enum PortedCurve {
    Line(LinePayload),
    Circle(CirclePayload),
    Ellipse(EllipsePayload),
}

#[derive(Clone, Copy, Debug)]
pub enum PortedSurface {
    Plane(PlanePayload),
    Cylinder(CylinderPayload),
    Cone(ConePayload),
    Sphere(SpherePayload),
    Torus(TorusPayload),
}

impl PortedCurve {
    pub fn from_context(context: &Context, shape: &Shape) -> Result<Option<Self>, Error> {
        let geometry = context.edge_geometry(shape)?;
        Self::from_context_with_geometry(context, shape, geometry)
    }

    pub fn from_context_with_geometry(
        context: &Context,
        shape: &Shape,
        geometry: EdgeGeometry,
    ) -> Result<Option<Self>, Error> {
        match geometry.kind {
            CurveKind::Line => Ok(Some(Self::Line(context.edge_line_payload(shape)?))),
            CurveKind::Circle => Ok(Some(Self::Circle(context.edge_circle_payload(shape)?))),
            CurveKind::Ellipse => Ok(Some(Self::Ellipse(context.edge_ellipse_payload(shape)?))),
            _ => Ok(None),
        }
    }

    pub fn sample(self, parameter: f64) -> EdgeSample {
        self.sample_with_reversal(parameter, false)
    }

    pub fn sample_with_geometry(self, geometry: EdgeGeometry, parameter: f64) -> EdgeSample {
        self.sample_with_reversal(parameter, geometry.start_parameter > geometry.end_parameter)
    }

    pub fn length_with_geometry(self, geometry: EdgeGeometry) -> f64 {
        let start = geometry.start_parameter;
        let end = geometry.end_parameter;
        match self {
            Self::Line(payload) => length_integral(start, end, |parameter| {
                let _ = parameter;
                norm3(payload.direction)
            }),
            Self::Circle(payload) => payload.radius.abs() * (end - start).abs(),
            Self::Ellipse(payload) => length_integral(start, end, |parameter| {
                norm3(ellipse_derivative(payload, parameter))
            }),
        }
    }

    fn sample_with_reversal(self, parameter: f64, is_reversed: bool) -> EdgeSample {
        let sample = match self {
            Self::Line(payload) => sample_line(payload, parameter),
            Self::Circle(payload) => sample_circle(payload, parameter),
            Self::Ellipse(payload) => sample_ellipse(payload, parameter),
        };

        if is_reversed {
            EdgeSample {
                position: sample.position,
                tangent: scale3(sample.tangent, -1.0),
            }
        } else {
            sample
        }
    }

    fn evaluate(self, parameter: f64) -> CurveEvaluation {
        match self {
            Self::Line(payload) => CurveEvaluation {
                position: add3(payload.origin, scale3(payload.direction, parameter)),
                derivative: payload.direction,
            },
            Self::Circle(payload) => CurveEvaluation {
                position: sample_circle(payload, parameter).position,
                derivative: circle_derivative(payload, parameter),
            },
            Self::Ellipse(payload) => CurveEvaluation {
                position: sample_ellipse(payload, parameter).position,
                derivative: ellipse_derivative(payload, parameter),
            },
        }
    }
}

impl PortedSurface {
    pub fn from_context(context: &Context, shape: &Shape) -> Result<Option<Self>, Error> {
        let geometry = context.face_geometry(shape)?;
        Self::from_context_with_geometry(context, shape, geometry)
    }

    pub fn from_context_with_geometry(
        context: &Context,
        shape: &Shape,
        geometry: FaceGeometry,
    ) -> Result<Option<Self>, Error> {
        match geometry.kind {
            SurfaceKind::Plane => Ok(Some(Self::Plane(context.face_plane_payload(shape)?))),
            SurfaceKind::Cylinder => {
                Ok(Some(Self::Cylinder(context.face_cylinder_payload(shape)?)))
            }
            SurfaceKind::Cone => Ok(Some(Self::Cone(context.face_cone_payload(shape)?))),
            SurfaceKind::Sphere => Ok(Some(Self::Sphere(context.face_sphere_payload(shape)?))),
            SurfaceKind::Torus => Ok(Some(Self::Torus(context.face_torus_payload(shape)?))),
            _ => Ok(None),
        }
    }

    pub fn sample(self, uv: [f64; 2]) -> FaceSample {
        self.sample_with_orientation(uv, Orientation::Forward)
    }

    pub fn sample_normalized(self, geometry: FaceGeometry, uv_t: [f64; 2]) -> FaceSample {
        self.sample_with_orientation(normalized_uv_to_uv(geometry, uv_t), Orientation::Forward)
    }

    pub fn sample_normalized_with_orientation(
        self,
        geometry: FaceGeometry,
        uv_t: [f64; 2],
        orientation: Orientation,
    ) -> FaceSample {
        self.sample_with_orientation(normalized_uv_to_uv(geometry, uv_t), orientation)
    }

    pub fn sample_with_orientation(self, uv: [f64; 2], orientation: Orientation) -> FaceSample {
        let mut sample = match self {
            Self::Plane(payload) => sample_plane(payload, uv),
            Self::Cylinder(payload) => sample_cylinder(payload, uv),
            Self::Cone(payload) => sample_cone(payload, uv),
            Self::Sphere(payload) => sample_sphere(payload, uv),
            Self::Torus(payload) => sample_torus(payload, uv),
        };

        if matches!(orientation, Orientation::Reversed) {
            sample.normal = scale3(sample.normal, -1.0);
        }
        sample
    }

    fn point_to_uv(self, point: [f64; 3]) -> Option<[f64; 2]> {
        match self {
            Self::Plane(payload) => Some([
                dot3(subtract3(point, payload.origin), payload.x_direction),
                dot3(subtract3(point, payload.origin), payload.y_direction),
            ]),
            Self::Cylinder(payload) => {
                let relative = subtract3(point, payload.origin);
                let v = dot3(relative, payload.axis);
                let radial = subtract3(relative, scale3(payload.axis, v));
                Some([
                    radial.atan2_components(payload.x_direction, payload.y_direction),
                    v,
                ])
            }
            Self::Cone(payload) => {
                let cos_angle = payload.semi_angle.cos();
                if cos_angle.abs() <= 1.0e-12 {
                    return None;
                }
                let relative = subtract3(point, payload.origin);
                let axial = dot3(relative, payload.axis);
                let v = axial / cos_angle;
                let radial = subtract3(relative, scale3(payload.axis, axial));
                Some([
                    radial.atan2_components(payload.x_direction, payload.y_direction),
                    v,
                ])
            }
            Self::Sphere(payload) => {
                if payload.radius.abs() <= 1.0e-12 {
                    return None;
                }
                let relative = scale3(subtract3(point, payload.center), payload.radius.recip());
                Some([
                    dot3(relative, payload.y_direction)
                        .atan2(dot3(relative, payload.x_direction)),
                    clamp(dot3(relative, payload.normal), -1.0, 1.0).asin(),
                ])
            }
            Self::Torus(payload) => {
                let relative = subtract3(point, payload.center);
                let u = dot3(relative, payload.y_direction)
                    .atan2(dot3(relative, payload.x_direction));
                let radial_direction = add3(
                    scale3(payload.x_direction, u.cos()),
                    scale3(payload.y_direction, u.sin()),
                );
                let tube = subtract3(point, add3(payload.center, scale3(radial_direction, payload.major_radius)));
                Some([
                    u,
                    dot3(tube, payload.axis).atan2(dot3(tube, radial_direction)),
                ])
            }
        }
    }

    fn area_potential(self, v: f64) -> f64 {
        match self {
            Self::Plane(_) => v,
            Self::Cylinder(payload) => payload.radius * v,
            Self::Cone(payload) => {
                payload.reference_radius * v + 0.5 * payload.semi_angle.sin() * v * v
            }
            Self::Sphere(payload) => payload.radius * payload.radius * v.sin(),
            Self::Torus(payload) => {
                payload.minor_radius * payload.major_radius * v
                    + payload.minor_radius * payload.minor_radius * v.sin()
            }
        }
    }
}

impl Context {
    pub fn ported_edge_curve(&self, shape: &Shape) -> Result<Option<PortedCurve>, Error> {
        PortedCurve::from_context(self, shape)
    }

    pub fn ported_edge_sample_at_parameter(
        &self,
        shape: &Shape,
        parameter: f64,
    ) -> Result<Option<EdgeSample>, Error> {
        let geometry = self.edge_geometry(shape)?;
        Ok(
            PortedCurve::from_context_with_geometry(self, shape, geometry)?
                .map(|curve| curve.sample_with_geometry(geometry, parameter)),
        )
    }

    pub fn ported_edge_length(&self, shape: &Shape) -> Result<Option<f64>, Error> {
        let geometry = self.edge_geometry(shape)?;
        Ok(
            PortedCurve::from_context_with_geometry(self, shape, geometry)?
                .map(|curve| curve.length_with_geometry(geometry)),
        )
    }

    pub fn ported_face_surface(&self, shape: &Shape) -> Result<Option<PortedSurface>, Error> {
        PortedSurface::from_context(self, shape)
    }

    pub fn ported_face_sample_normalized(
        &self,
        shape: &Shape,
        uv_t: [f64; 2],
    ) -> Result<Option<FaceSample>, Error> {
        let geometry = self.face_geometry(shape)?;
        let orientation = self.shape_orientation(shape)?;
        Ok(
            PortedSurface::from_context_with_geometry(self, shape, geometry)?.map(|surface| {
                surface.sample_normalized_with_orientation(geometry, uv_t, orientation)
            }),
        )
    }
}

fn sample_line(payload: LinePayload, parameter: f64) -> EdgeSample {
    EdgeSample {
        position: add3(payload.origin, scale3(payload.direction, parameter)),
        tangent: normalize3(payload.direction),
    }
}

fn sample_circle(payload: CirclePayload, parameter: f64) -> EdgeSample {
    let cos_u = parameter.cos();
    let sin_u = parameter.sin();
    let radial = add3(
        scale3(payload.x_direction, cos_u),
        scale3(payload.y_direction, sin_u),
    );
    let tangent = add3(
        scale3(payload.x_direction, -sin_u),
        scale3(payload.y_direction, cos_u),
    );
    EdgeSample {
        position: add3(payload.center, scale3(radial, payload.radius)),
        tangent: normalize3(tangent),
    }
}

fn circle_derivative(payload: CirclePayload, parameter: f64) -> [f64; 3] {
    add3(
        scale3(payload.x_direction, -payload.radius * parameter.sin()),
        scale3(payload.y_direction, payload.radius * parameter.cos()),
    )
}

fn sample_ellipse(payload: EllipsePayload, parameter: f64) -> EdgeSample {
    let cos_u = parameter.cos();
    let sin_u = parameter.sin();
    let x_component = scale3(payload.x_direction, payload.major_radius * cos_u);
    let y_component = scale3(payload.y_direction, payload.minor_radius * sin_u);
    let tangent = add3(
        scale3(payload.x_direction, -payload.major_radius * sin_u),
        scale3(payload.y_direction, payload.minor_radius * cos_u),
    );
    EdgeSample {
        position: add3(payload.center, add3(x_component, y_component)),
        tangent: normalize3(tangent),
    }
}

fn ellipse_derivative(payload: EllipsePayload, parameter: f64) -> [f64; 3] {
    add3(
        scale3(payload.x_direction, -payload.major_radius * parameter.sin()),
        scale3(payload.y_direction, payload.minor_radius * parameter.cos()),
    )
}

fn sample_plane(payload: PlanePayload, uv: [f64; 2]) -> FaceSample {
    FaceSample {
        position: add3(
            payload.origin,
            add3(
                scale3(payload.x_direction, uv[0]),
                scale3(payload.y_direction, uv[1]),
            ),
        ),
        normal: normalize3(payload.normal),
    }
}

fn sample_cylinder(payload: CylinderPayload, uv: [f64; 2]) -> FaceSample {
    let cos_u = uv[0].cos();
    let sin_u = uv[0].sin();
    let radial = add3(
        scale3(payload.x_direction, cos_u),
        scale3(payload.y_direction, sin_u),
    );
    FaceSample {
        position: add3(
            payload.origin,
            add3(scale3(payload.axis, uv[1]), scale3(radial, payload.radius)),
        ),
        normal: normalize3(radial),
    }
}

fn sample_cone(payload: ConePayload, uv: [f64; 2]) -> FaceSample {
    let cos_u = uv[0].cos();
    let sin_u = uv[0].sin();
    let sin_angle = payload.semi_angle.sin();
    let cos_angle = payload.semi_angle.cos();
    let radial = add3(
        scale3(payload.x_direction, cos_u),
        scale3(payload.y_direction, sin_u),
    );
    let tangential = add3(
        scale3(payload.x_direction, -sin_u),
        scale3(payload.y_direction, cos_u),
    );
    let radius = payload.reference_radius + uv[1] * sin_angle;
    let position = add3(
        payload.origin,
        add3(
            scale3(payload.axis, uv[1] * cos_angle),
            scale3(radial, radius),
        ),
    );
    let du = scale3(tangential, radius);
    let dv = add3(scale3(payload.axis, cos_angle), scale3(radial, sin_angle));

    FaceSample {
        position,
        normal: normalize3(cross3(du, dv)),
    }
}

fn sample_sphere(payload: SpherePayload, uv: [f64; 2]) -> FaceSample {
    let cos_u = uv[0].cos();
    let sin_u = uv[0].sin();
    let cos_v = uv[1].cos();
    let sin_v = uv[1].sin();
    let radial = add3(
        scale3(payload.x_direction, cos_v * cos_u),
        add3(
            scale3(payload.y_direction, cos_v * sin_u),
            scale3(payload.normal, sin_v),
        ),
    );
    FaceSample {
        position: add3(payload.center, scale3(radial, payload.radius)),
        normal: normalize3(radial),
    }
}

fn sample_torus(payload: TorusPayload, uv: [f64; 2]) -> FaceSample {
    let cos_u = uv[0].cos();
    let sin_u = uv[0].sin();
    let cos_v = uv[1].cos();
    let sin_v = uv[1].sin();
    let radial = add3(
        scale3(payload.x_direction, cos_u),
        scale3(payload.y_direction, sin_u),
    );
    let tube_offset = add3(
        scale3(radial, payload.minor_radius * cos_v),
        scale3(payload.axis, payload.minor_radius * sin_v),
    );
    FaceSample {
        position: add3(
            payload.center,
            add3(scale3(radial, payload.major_radius), tube_offset),
        ),
        normal: normalize3(tube_offset),
    }
}

fn normalized_uv_to_uv(geometry: FaceGeometry, uv_t: [f64; 2]) -> [f64; 2] {
    [
        geometry.u_min + (geometry.u_max - geometry.u_min) * uv_t[0],
        geometry.v_min + (geometry.v_max - geometry.v_min) * uv_t[1],
    ]
}

pub(crate) fn planar_wire_signed_area(
    plane: PlanePayload,
    curve_segments: &[(PortedCurve, EdgeGeometry)],
) -> f64 {
    0.5 * curve_segments
        .iter()
        .map(|(curve, geometry)| {
            signed_scalar_integral(geometry.start_parameter, geometry.end_parameter, |parameter| {
                let evaluation = curve.evaluate(parameter);
                let relative = subtract3(evaluation.position, plane.origin);
                let x = dot3(relative, plane.x_direction);
                let y = dot3(relative, plane.y_direction);
                let dx = dot3(evaluation.derivative, plane.x_direction);
                let dy = dot3(evaluation.derivative, plane.y_direction);
                x * dy - y * dx
            })
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

fn length_integral<F>(start: f64, end: f64, integrand: F) -> f64
where
    F: Fn(f64) -> f64,
{
    signed_scalar_integral(start, end, |parameter| integrand(parameter).abs()).abs()
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

    adaptive_simpson(integrand, a, midpoint, fa, flm, fm, 0.5 * tolerance, depth - 1)
        + adaptive_simpson(integrand, midpoint, b, fm, frm, fb, 0.5 * tolerance, depth - 1)
}

fn simpson_step(a: f64, b: f64, fa: f64, fm: f64, fb: f64) -> f64 {
    (b - a) * (fa + 4.0 * fm + fb) / 6.0
}

fn add3(lhs: [f64; 3], rhs: [f64; 3]) -> [f64; 3] {
    [lhs[0] + rhs[0], lhs[1] + rhs[1], lhs[2] + rhs[2]]
}

fn subtract3(lhs: [f64; 3], rhs: [f64; 3]) -> [f64; 3] {
    [lhs[0] - rhs[0], lhs[1] - rhs[1], lhs[2] - rhs[2]]
}

fn scale3(value: [f64; 3], scale: f64) -> [f64; 3] {
    [value[0] * scale, value[1] * scale, value[2] * scale]
}

fn cross3(lhs: [f64; 3], rhs: [f64; 3]) -> [f64; 3] {
    [
        lhs[1] * rhs[2] - lhs[2] * rhs[1],
        lhs[2] * rhs[0] - lhs[0] * rhs[2],
        lhs[0] * rhs[1] - lhs[1] * rhs[0],
    ]
}

fn dot3(lhs: [f64; 3], rhs: [f64; 3]) -> f64 {
    lhs[0] * rhs[0] + lhs[1] * rhs[1] + lhs[2] * rhs[2]
}

fn normalize3(value: [f64; 3]) -> [f64; 3] {
    let length_sq = dot3(value, value);
    if length_sq <= 1.0e-24 {
        [0.0, 0.0, 0.0]
    } else {
        scale3(value, length_sq.sqrt().recip())
    }
}

fn norm3(value: [f64; 3]) -> f64 {
    dot3(value, value).sqrt()
}

trait Atan2Components {
    fn atan2_components(self, x_direction: [f64; 3], y_direction: [f64; 3]) -> f64;
}

impl Atan2Components for [f64; 3] {
    fn atan2_components(self, x_direction: [f64; 3], y_direction: [f64; 3]) -> f64 {
        dot3(self, y_direction).atan2(dot3(self, x_direction))
    }
}

fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.max(min).min(max)
}
