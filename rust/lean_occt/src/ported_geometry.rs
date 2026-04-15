use std::f64::consts::TAU;

use crate::brep::{
    ported_face_area as ported_face_area_value,
    ported_face_surface_descriptor as ported_face_surface_descriptor_value,
};
use crate::{
    CirclePayload, ConePayload, Context, CurveKind, CylinderPayload, EdgeEndpoints, EdgeGeometry,
    EdgeSample, EllipsePayload, Error, ExtrusionSurfacePayload, FaceGeometry, FaceSample,
    FaceUvBounds, LinePayload, OffsetSurfacePayload, Orientation, PlanePayload,
    RevolutionSurfacePayload, Shape, SpherePayload, SurfaceKind, TorusPayload,
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct CurveEvaluation {
    pub(crate) position: [f64; 3],
    pub(crate) derivative: [f64; 3],
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

#[derive(Clone, Copy, Debug)]
pub enum PortedSweptSurface {
    Revolution {
        payload: RevolutionSurfacePayload,
        basis_curve: PortedCurve,
        basis_geometry: EdgeGeometry,
    },
    Extrusion {
        payload: ExtrusionSurfacePayload,
        basis_curve: PortedCurve,
        basis_geometry: EdgeGeometry,
    },
}

#[derive(Clone, Copy, Debug)]
pub enum PortedOffsetBasisSurface {
    Analytic(PortedSurface),
    Swept(PortedSweptSurface),
}

#[derive(Clone, Copy, Debug)]
pub struct PortedOffsetSurface {
    pub payload: OffsetSurfacePayload,
    pub basis_geometry: FaceGeometry,
    pub basis: PortedOffsetBasisSurface,
}

#[derive(Clone, Copy, Debug)]
pub enum PortedFaceSurface {
    Analytic(PortedSurface),
    Swept(PortedSweptSurface),
    Offset(PortedOffsetSurface),
}

impl PortedCurve {
    pub fn from_context(context: &Context, shape: &Shape) -> Result<Option<Self>, Error> {
        let geometry = context.edge_geometry_occt(shape)?;
        Self::from_context_with_ported_payloads(context, shape, geometry)
    }

    pub fn from_context_with_geometry(
        context: &Context,
        shape: &Shape,
        geometry: EdgeGeometry,
    ) -> Result<Option<Self>, Error> {
        match geometry.kind {
            CurveKind::Line => Ok(Some(Self::Line(
                ported_line_payload(context, shape, geometry)?
                    .unwrap_or(context.edge_line_payload_occt(shape)?),
            ))),
            CurveKind::Circle => Ok(Some(Self::Circle(context.edge_circle_payload_occt(shape)?))),
            CurveKind::Ellipse => Ok(Some(Self::Ellipse(
                context.edge_ellipse_payload_occt(shape)?,
            ))),
            _ => Ok(None),
        }
    }

    fn from_context_with_ported_payloads(
        context: &Context,
        shape: &Shape,
        geometry: EdgeGeometry,
    ) -> Result<Option<Self>, Error> {
        match geometry.kind {
            CurveKind::Line => Ok(Some(Self::Line(
                ported_line_payload(context, shape, geometry)?
                    .unwrap_or(context.edge_line_payload_occt(shape)?),
            ))),
            CurveKind::Circle => Ok(Some(Self::Circle(
                ported_circle_payload(context, shape, geometry)?
                    .unwrap_or(context.edge_circle_payload_occt(shape)?),
            ))),
            CurveKind::Ellipse => Ok(Some(Self::Ellipse(
                ported_ellipse_payload(context, shape, geometry)?
                    .unwrap_or(context.edge_ellipse_payload_occt(shape)?),
            ))),
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

    pub(crate) fn evaluate(self, parameter: f64) -> CurveEvaluation {
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

pub(crate) fn extrusion_swept_area(
    curve: PortedCurve,
    geometry: EdgeGeometry,
    direction: [f64; 3],
    span: f64,
) -> f64 {
    let direction = normalize3(direction);
    span.abs()
        * length_integral(
            geometry.start_parameter,
            geometry.end_parameter,
            |parameter| {
                let evaluation = curve.evaluate(parameter);
                norm3(cross3(evaluation.derivative, direction))
            },
        )
}

pub(crate) fn revolution_swept_area(
    curve: PortedCurve,
    geometry: EdgeGeometry,
    axis_origin: [f64; 3],
    axis_direction: [f64; 3],
    sweep_angle: f64,
) -> f64 {
    let axis_direction = normalize3(axis_direction);
    sweep_angle.abs()
        * length_integral(
            geometry.start_parameter,
            geometry.end_parameter,
            |parameter| {
                let evaluation = curve.evaluate(parameter);
                let radius_velocity =
                    cross3(axis_direction, subtract3(evaluation.position, axis_origin));
                norm3(cross3(evaluation.derivative, radius_velocity))
            },
        )
}

pub(crate) fn sample_extrusion_surface_normalized(
    curve: PortedCurve,
    face_geometry: FaceGeometry,
    basis_geometry: EdgeGeometry,
    uv_t: [f64; 2],
    direction: [f64; 3],
    orientation: Orientation,
) -> FaceSample {
    let uv = normalized_uv_to_uv(face_geometry, uv_t);
    let basis_on_u = basis_parameter_on_u(face_geometry, basis_geometry);
    let (curve_parameter, sweep_parameter) =
        swept_surface_parameters(face_geometry, basis_geometry, uv);
    let direction = normalize3(direction);
    let evaluation = curve.evaluate(curve_parameter);
    let curve_derivative = evaluation.derivative;
    let sweep_derivative = direction;
    let mut sample = FaceSample {
        position: add3(evaluation.position, scale3(direction, sweep_parameter)),
        normal: normalize3(if basis_on_u {
            cross3(curve_derivative, sweep_derivative)
        } else {
            cross3(sweep_derivative, curve_derivative)
        }),
    };
    if matches!(orientation, Orientation::Reversed) {
        sample.normal = scale3(sample.normal, -1.0);
    }
    sample
}

pub(crate) fn sample_revolution_surface_normalized(
    curve: PortedCurve,
    face_geometry: FaceGeometry,
    basis_geometry: EdgeGeometry,
    uv_t: [f64; 2],
    axis_origin: [f64; 3],
    axis_direction: [f64; 3],
    orientation: Orientation,
) -> FaceSample {
    let uv = normalized_uv_to_uv(face_geometry, uv_t);
    let basis_on_u = basis_parameter_on_u(face_geometry, basis_geometry);
    let (curve_parameter, sweep_parameter) =
        swept_surface_parameters(face_geometry, basis_geometry, uv);
    let axis_direction = normalize3(axis_direction);
    let evaluation = curve.evaluate(curve_parameter);
    let position = rotate_point_about_axis(
        evaluation.position,
        axis_origin,
        axis_direction,
        sweep_parameter,
    );
    let curve_derivative =
        rotate_vector_about_axis(evaluation.derivative, axis_direction, sweep_parameter);
    let sweep_derivative = cross3(axis_direction, subtract3(position, axis_origin));
    let mut sample = FaceSample {
        position,
        normal: normalize3(if basis_on_u {
            cross3(curve_derivative, sweep_derivative)
        } else {
            cross3(sweep_derivative, curve_derivative)
        }),
    };
    if matches!(orientation, Orientation::Reversed) {
        sample.normal = scale3(sample.normal, -1.0);
    }
    sample
}

fn swept_surface_parameters(
    face_geometry: FaceGeometry,
    basis_geometry: EdgeGeometry,
    uv: [f64; 2],
) -> (f64, f64) {
    if basis_parameter_on_u(face_geometry, basis_geometry) {
        (uv[0], uv[1])
    } else {
        (uv[1], uv[0])
    }
}

fn basis_parameter_on_u(face_geometry: FaceGeometry, basis_geometry: EdgeGeometry) -> bool {
    let basis_span = (basis_geometry.end_parameter - basis_geometry.start_parameter).abs();
    let u_span = (face_geometry.u_max - face_geometry.u_min).abs();
    let v_span = (face_geometry.v_max - face_geometry.v_min).abs();
    let u_delta = (u_span - basis_span).abs();
    let v_delta = (v_span - basis_span).abs();
    u_delta <= v_delta
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
            SurfaceKind::Plane => Ok(Some(Self::Plane(
                ported_plane_payload(context, shape, geometry)?
                    .unwrap_or(context.face_plane_payload_occt(shape)?),
            ))),
            SurfaceKind::Cylinder => Ok(Some(Self::Cylinder(
                ported_cylinder_payload(context, shape, geometry)?
                    .unwrap_or(context.face_cylinder_payload_occt(shape)?),
            ))),
            SurfaceKind::Cone => Ok(Some(Self::Cone(
                ported_cone_payload(context, shape, geometry)?
                    .unwrap_or(context.face_cone_payload_occt(shape)?),
            ))),
            SurfaceKind::Sphere => Ok(Some(Self::Sphere(
                ported_sphere_payload(context, shape, geometry)?
                    .unwrap_or(context.face_sphere_payload_occt(shape)?),
            ))),
            SurfaceKind::Torus => Ok(Some(Self::Torus(
                ported_torus_payload(context, shape, geometry)?
                    .unwrap_or(context.face_torus_payload_occt(shape)?),
            ))),
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
                    dot3(relative, payload.y_direction).atan2(dot3(relative, payload.x_direction)),
                    clamp(dot3(relative, payload.normal), -1.0, 1.0).asin(),
                ])
            }
            Self::Torus(payload) => {
                let relative = subtract3(point, payload.center);
                let u =
                    dot3(relative, payload.y_direction).atan2(dot3(relative, payload.x_direction));
                let radial_direction = add3(
                    scale3(payload.x_direction, u.cos()),
                    scale3(payload.y_direction, u.sin()),
                );
                let tube = subtract3(
                    point,
                    add3(
                        payload.center,
                        scale3(radial_direction, payload.major_radius),
                    ),
                );
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

    fn volume_potential(self, uv: [f64; 2]) -> f64 {
        match self {
            Self::Plane(payload) => dot3(payload.origin, payload.normal) * uv[0] / 3.0,
            Self::Cylinder(payload) => {
                let ox = dot3(payload.origin, payload.x_direction);
                let oy = dot3(payload.origin, payload.y_direction);
                (payload.radius * (ox * uv[0].sin() - oy * uv[0].cos())
                    + payload.radius * payload.radius * uv[0])
                    / 3.0
            }
            Self::Cone(payload) => {
                let sin_angle = payload.semi_angle.sin();
                let cos_angle = payload.semi_angle.cos();
                let radius = payload.reference_radius + sin_angle * uv[1];
                let ox = dot3(payload.origin, payload.x_direction);
                let oy = dot3(payload.origin, payload.y_direction);
                let oz = dot3(payload.origin, payload.axis);
                radius
                    * (cos_angle
                        * (ox * uv[0].sin() - oy * uv[0].cos() + payload.reference_radius * uv[0])
                        - sin_angle * oz * uv[0])
                    / 3.0
            }
            Self::Sphere(payload) => {
                let cx = dot3(payload.center, payload.x_direction);
                let cy = dot3(payload.center, payload.y_direction);
                let cz = dot3(payload.center, payload.normal);
                let cos_v = uv[1].cos();
                let sin_v = uv[1].sin();
                payload.radius
                    * payload.radius
                    * (cx * cos_v * cos_v * uv[0].sin() - cy * cos_v * cos_v * uv[0].cos()
                        + (cz * sin_v * cos_v + payload.radius * cos_v) * uv[0])
                    / 3.0
            }
            Self::Torus(payload) => {
                let cx = dot3(payload.center, payload.x_direction);
                let cy = dot3(payload.center, payload.y_direction);
                let cz = dot3(payload.center, payload.axis);
                let cos_v = uv[1].cos();
                let sin_v = uv[1].sin();
                payload.minor_radius
                    * (payload.major_radius + payload.minor_radius * cos_v)
                    * (cos_v * (cx * uv[0].sin() - cy * uv[0].cos() + payload.major_radius * uv[0])
                        + (cz * sin_v + payload.minor_radius) * uv[0])
                    / 3.0
            }
        }
    }
}

impl PortedSweptSurface {
    pub fn sample_normalized(self, geometry: FaceGeometry, uv_t: [f64; 2]) -> FaceSample {
        self.sample_normalized_with_orientation(geometry, uv_t, Orientation::Forward)
    }

    pub fn sample_normalized_with_orientation(
        self,
        geometry: FaceGeometry,
        uv_t: [f64; 2],
        orientation: Orientation,
    ) -> FaceSample {
        match self {
            Self::Revolution {
                payload,
                basis_curve,
                basis_geometry,
            } => sample_revolution_surface_normalized(
                basis_curve,
                geometry,
                basis_geometry,
                uv_t,
                payload.axis_origin,
                payload.axis_direction,
                orientation,
            ),
            Self::Extrusion {
                payload,
                basis_curve,
                basis_geometry,
            } => sample_extrusion_surface_normalized(
                basis_curve,
                geometry,
                basis_geometry,
                uv_t,
                payload.direction,
                orientation,
            ),
        }
    }
}

impl PortedOffsetSurface {
    pub fn sample_normalized(self, uv_t: [f64; 2]) -> FaceSample {
        self.sample_normalized_with_orientation(uv_t, Orientation::Forward)
    }

    pub fn sample_normalized_with_orientation(
        self,
        uv_t: [f64; 2],
        orientation: Orientation,
    ) -> FaceSample {
        let mut basis_sample = match self.basis {
            PortedOffsetBasisSurface::Analytic(surface) => {
                surface.sample_normalized(self.basis_geometry, uv_t)
            }
            PortedOffsetBasisSurface::Swept(PortedSweptSurface::Revolution {
                payload,
                basis_curve,
                basis_geometry,
            }) => sample_revolution_surface_normalized(
                basis_curve,
                self.basis_geometry,
                basis_geometry,
                uv_t,
                payload.axis_origin,
                payload.axis_direction,
                Orientation::Forward,
            ),
            PortedOffsetBasisSurface::Swept(PortedSweptSurface::Extrusion {
                payload,
                basis_curve,
                basis_geometry,
            }) => sample_extrusion_surface_normalized(
                basis_curve,
                self.basis_geometry,
                basis_geometry,
                uv_t,
                payload.direction,
                Orientation::Forward,
            ),
        };
        basis_sample.position = add3(
            basis_sample.position,
            scale3(basis_sample.normal, self.payload.offset_value),
        );
        if matches!(orientation, Orientation::Reversed) {
            basis_sample.normal = scale3(basis_sample.normal, -1.0);
        }
        basis_sample
    }

    pub(crate) fn equivalent_analytic_surface(self) -> Option<PortedSurface> {
        let offset = self.payload.offset_value;
        match self.basis {
            PortedOffsetBasisSurface::Analytic(PortedSurface::Plane(payload)) => {
                Some(PortedSurface::Plane(PlanePayload {
                    origin: add3(payload.origin, scale3(payload.normal, offset)),
                    ..payload
                }))
            }
            PortedOffsetBasisSurface::Analytic(PortedSurface::Cylinder(payload)) => {
                let radius = payload.radius + offset;
                (radius.abs() > 1.0e-9).then_some(PortedSurface::Cylinder(CylinderPayload {
                    radius,
                    ..payload
                }))
            }
            PortedOffsetBasisSurface::Analytic(PortedSurface::Cone(payload)) => {
                Some(PortedSurface::Cone(ConePayload {
                    origin: add3(
                        payload.origin,
                        scale3(payload.axis, -offset * payload.semi_angle.sin()),
                    ),
                    reference_radius: payload.reference_radius + offset * payload.semi_angle.cos(),
                    ..payload
                }))
            }
            PortedOffsetBasisSurface::Analytic(PortedSurface::Sphere(payload)) => {
                let radius = payload.radius + offset;
                (radius.abs() > 1.0e-9)
                    .then_some(PortedSurface::Sphere(SpherePayload { radius, ..payload }))
            }
            PortedOffsetBasisSurface::Analytic(PortedSurface::Torus(payload)) => {
                let minor_radius = payload.minor_radius + offset;
                (minor_radius.abs() > 1.0e-9).then_some(PortedSurface::Torus(TorusPayload {
                    minor_radius,
                    ..payload
                }))
            }
            PortedOffsetBasisSurface::Swept(_) => None,
        }
    }
}

impl PortedFaceSurface {
    pub fn sample_normalized(self, geometry: FaceGeometry, uv_t: [f64; 2]) -> FaceSample {
        self.sample_normalized_with_orientation(geometry, uv_t, Orientation::Forward)
    }

    pub fn sample_normalized_with_orientation(
        self,
        geometry: FaceGeometry,
        uv_t: [f64; 2],
        orientation: Orientation,
    ) -> FaceSample {
        match self {
            Self::Analytic(surface) => {
                surface.sample_normalized_with_orientation(geometry, uv_t, orientation)
            }
            Self::Swept(surface) => {
                surface.sample_normalized_with_orientation(geometry, uv_t, orientation)
            }
            Self::Offset(surface) => surface.sample_normalized_with_orientation(uv_t, orientation),
        }
    }
}

impl Context {
    pub fn ported_edge_geometry(&self, shape: &Shape) -> Result<Option<EdgeGeometry>, Error> {
        let geometry = self.edge_geometry_occt(shape)?;
        let endpoints = self.edge_endpoints(shape)?;

        if geometry.kind == CurveKind::Line {
            let line_payload = ported_line_payload_from_endpoints(geometry, endpoints)
                .or_else(|| self.edge_line_payload_occt(shape).ok());
            if let Some(payload) = line_payload {
                return Ok(ported_line_geometry(payload, endpoints));
            }
        }

        let edge_length = shape.linear_length();
        let start_tangent = self.edge_sample_occt(shape, 0.0)?.tangent;

        let circle_payload = match ported_circle_payload(self, shape, geometry)? {
            Some(payload) => Some(payload),
            None => self.edge_circle_payload_occt(shape).ok(),
        };
        if let Some(payload) = circle_payload {
            return Ok(ported_periodic_curve_geometry(
                CurveKind::Circle,
                endpoints,
                start_tangent,
                edge_length,
                TAU,
                |point| Some(circle_parameter(payload, point)),
                circle_derivative_from_parameter(payload),
                |start_parameter, end_parameter| {
                    PortedCurve::Circle(payload).length_with_geometry(EdgeGeometry {
                        kind: CurveKind::Circle,
                        start_parameter,
                        end_parameter,
                        is_closed: false,
                        is_periodic: true,
                        period: TAU,
                    })
                },
            ));
        }

        let ellipse_payload = match ported_ellipse_payload(self, shape, geometry)? {
            Some(payload) => Some(payload),
            None => self.edge_ellipse_payload_occt(shape).ok(),
        };
        if let Some(payload) = ellipse_payload {
            return Ok(ported_periodic_curve_geometry(
                CurveKind::Ellipse,
                endpoints,
                start_tangent,
                edge_length,
                TAU,
                |point| ellipse_parameter(payload, point),
                ellipse_derivative_from_parameter(payload),
                |start_parameter, end_parameter| {
                    PortedCurve::Ellipse(payload).length_with_geometry(EdgeGeometry {
                        kind: CurveKind::Ellipse,
                        start_parameter,
                        end_parameter,
                        is_closed: false,
                        is_periodic: true,
                        period: TAU,
                    })
                },
            ));
        }

        Ok(None)
    }

    pub fn ported_face_geometry(&self, shape: &Shape) -> Result<Option<FaceGeometry>, Error> {
        let bounds = self.face_uv_bounds_occt(shape)?;
        let plane_geometry = ported_analytic_face_geometry(SurfaceKind::Plane, bounds);
        let cylinder_geometry = ported_analytic_face_geometry(SurfaceKind::Cylinder, bounds);
        let cone_geometry = ported_analytic_face_geometry(SurfaceKind::Cone, bounds);
        let sphere_geometry = ported_analytic_face_geometry(SurfaceKind::Sphere, bounds);
        let torus_geometry = ported_analytic_face_geometry(SurfaceKind::Torus, bounds);

        if ported_plane_payload(self, shape, plane_geometry)?.is_some()
            || self.face_plane_payload_occt(shape).is_ok()
        {
            return Ok(Some(plane_geometry));
        }

        if ported_cylinder_payload(self, shape, cylinder_geometry)?.is_some()
            || self.face_cylinder_payload_occt(shape).is_ok()
        {
            return Ok(Some(cylinder_geometry));
        }

        if ported_cone_payload(self, shape, cone_geometry)?.is_some()
            || self.face_cone_payload_occt(shape).is_ok()
        {
            return Ok(Some(cone_geometry));
        }

        if ported_sphere_payload(self, shape, sphere_geometry)?.is_some()
            || self.face_sphere_payload_occt(shape).is_ok()
        {
            return Ok(Some(sphere_geometry));
        }

        if ported_torus_payload(self, shape, torus_geometry)?.is_some()
            || self.face_torus_payload_occt(shape).is_ok()
        {
            return Ok(Some(torus_geometry));
        }

        Ok(None)
    }

    pub fn ported_edge_curve(&self, shape: &Shape) -> Result<Option<PortedCurve>, Error> {
        PortedCurve::from_context(self, shape)
    }

    pub fn ported_edge_sample(&self, shape: &Shape, t: f64) -> Result<Option<EdgeSample>, Error> {
        if !(t >= 0.0 && t <= 1.0) {
            return Err(Error::new("Edge sample parameter must be within [0, 1]."));
        }

        let geometry = self.edge_geometry(shape)?;
        let parameter = interpolate_range(geometry.start_parameter, geometry.end_parameter, t);
        Ok(
            PortedCurve::from_context_with_ported_payloads(self, shape, geometry)?
                .map(|curve| curve.sample_with_geometry(geometry, parameter)),
        )
    }

    pub fn ported_edge_sample_at_parameter(
        &self,
        shape: &Shape,
        parameter: f64,
    ) -> Result<Option<EdgeSample>, Error> {
        let geometry = self.edge_geometry(shape)?;
        Ok(
            PortedCurve::from_context_with_ported_payloads(self, shape, geometry)?
                .map(|curve| curve.sample_with_geometry(geometry, parameter)),
        )
    }

    pub fn ported_edge_length(&self, shape: &Shape) -> Result<Option<f64>, Error> {
        let geometry = self.edge_geometry(shape)?;
        Ok(
            PortedCurve::from_context_with_ported_payloads(self, shape, geometry)?
                .map(|curve| curve.length_with_geometry(geometry)),
        )
    }

    pub fn ported_face_surface(&self, shape: &Shape) -> Result<Option<PortedSurface>, Error> {
        PortedSurface::from_context(self, shape)
    }

    pub fn ported_face_surface_descriptor(
        &self,
        shape: &Shape,
    ) -> Result<Option<PortedFaceSurface>, Error> {
        let geometry = self.face_geometry(shape)?;
        ported_face_surface_descriptor_value(self, shape, geometry)
    }

    pub fn ported_face_sample_normalized(
        &self,
        shape: &Shape,
        uv_t: [f64; 2],
    ) -> Result<Option<FaceSample>, Error> {
        let geometry = self.face_geometry(shape)?;
        let orientation = self.shape_orientation(shape)?;
        Ok(ported_face_surface_descriptor_value(self, shape, geometry)?
            .map(|surface| surface.sample_normalized_with_orientation(geometry, uv_t, orientation)))
    }

    pub fn ported_face_sample(
        &self,
        shape: &Shape,
        uv: [f64; 2],
    ) -> Result<Option<FaceSample>, Error> {
        let geometry = self.face_geometry(shape)?;
        if !(uv[0] >= geometry.u_min && uv[0] <= geometry.u_max)
            || !(uv[1] >= geometry.v_min && uv[1] <= geometry.v_max)
        {
            return Err(Error::new(
                "Requested UV sample was outside the face bounds.",
            ));
        }

        let orientation = self.shape_orientation(shape)?;
        let uv_t = uv_to_normalized_uv(geometry, uv);
        Ok(ported_face_surface_descriptor_value(self, shape, geometry)?
            .map(|surface| surface.sample_normalized_with_orientation(geometry, uv_t, orientation)))
    }

    pub fn ported_face_area(&self, shape: &Shape) -> Result<Option<f64>, Error> {
        ported_face_area_value(self, shape)
    }

    pub(crate) fn ported_offset_surface(
        &self,
        shape: &Shape,
    ) -> Result<Option<PortedOffsetSurface>, Error> {
        if self.face_geometry_occt(shape)?.kind != SurfaceKind::Offset {
            return Ok(None);
        }

        let payload = self.face_offset_payload_occt(shape)?;
        let basis_geometry = self.face_offset_basis_geometry_occt(shape)?;
        let basis = match payload.basis_surface_kind {
            SurfaceKind::Plane => PortedOffsetBasisSurface::Analytic(PortedSurface::Plane(
                self.face_offset_basis_plane_payload_occt(shape)?,
            )),
            SurfaceKind::Cylinder => PortedOffsetBasisSurface::Analytic(PortedSurface::Cylinder(
                self.face_offset_basis_cylinder_payload_occt(shape)?,
            )),
            SurfaceKind::Cone => PortedOffsetBasisSurface::Analytic(PortedSurface::Cone(
                self.face_offset_basis_cone_payload_occt(shape)?,
            )),
            SurfaceKind::Sphere => PortedOffsetBasisSurface::Analytic(PortedSurface::Sphere(
                self.face_offset_basis_sphere_payload_occt(shape)?,
            )),
            SurfaceKind::Torus => PortedOffsetBasisSurface::Analytic(PortedSurface::Torus(
                self.face_offset_basis_torus_payload_occt(shape)?,
            )),
            SurfaceKind::Revolution => {
                let payload = self.face_offset_basis_revolution_payload_occt(shape)?;
                let basis_geometry = self.face_offset_basis_curve_geometry_occt(shape)?;
                let basis_curve = match payload.basis_curve_kind {
                    CurveKind::Line => {
                        PortedCurve::Line(self.face_offset_basis_curve_line_payload_occt(shape)?)
                    }
                    CurveKind::Circle => PortedCurve::Circle(
                        self.face_offset_basis_curve_circle_payload_occt(shape)?,
                    ),
                    CurveKind::Ellipse => PortedCurve::Ellipse(
                        self.face_offset_basis_curve_ellipse_payload_occt(shape)?,
                    ),
                    _ => return Ok(None),
                };
                PortedOffsetBasisSurface::Swept(PortedSweptSurface::Revolution {
                    payload,
                    basis_curve,
                    basis_geometry,
                })
            }
            SurfaceKind::Extrusion => {
                let payload = self.face_offset_basis_extrusion_payload_occt(shape)?;
                let basis_geometry = self.face_offset_basis_curve_geometry_occt(shape)?;
                let basis_curve = match payload.basis_curve_kind {
                    CurveKind::Line => {
                        PortedCurve::Line(self.face_offset_basis_curve_line_payload_occt(shape)?)
                    }
                    CurveKind::Circle => PortedCurve::Circle(
                        self.face_offset_basis_curve_circle_payload_occt(shape)?,
                    ),
                    CurveKind::Ellipse => PortedCurve::Ellipse(
                        self.face_offset_basis_curve_ellipse_payload_occt(shape)?,
                    ),
                    _ => return Ok(None),
                };
                PortedOffsetBasisSurface::Swept(PortedSweptSurface::Extrusion {
                    payload,
                    basis_curve,
                    basis_geometry,
                })
            }
            _ => return Ok(None),
        };

        Ok(Some(PortedOffsetSurface {
            payload,
            basis_geometry,
            basis,
        }))
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

fn uv_to_normalized_uv(geometry: FaceGeometry, uv: [f64; 2]) -> [f64; 2] {
    [
        normalize_range(geometry.u_min, geometry.u_max, uv[0]),
        normalize_range(geometry.v_min, geometry.v_max, uv[1]),
    ]
}

fn normalize_range(start: f64, end: f64, value: f64) -> f64 {
    let span = end - start;
    if span.abs() <= 1.0e-12 {
        0.0
    } else {
        (value - start) / span
    }
}

fn interpolate_range(start: f64, end: f64, t: f64) -> f64 {
    start + (end - start) * t
}

fn rotate_point_about_axis(
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

fn rotate_vector_about_axis(vector: [f64; 3], axis_direction: [f64; 3], angle: f64) -> [f64; 3] {
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

fn ported_line_geometry(payload: LinePayload, endpoints: EdgeEndpoints) -> Option<EdgeGeometry> {
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

fn ported_periodic_curve_geometry<F, G, H>(
    kind: CurveKind,
    endpoints: EdgeEndpoints,
    start_tangent: [f64; 3],
    edge_length: f64,
    period: f64,
    parameter_at_point: F,
    derivative_at_parameter: G,
    length_with_parameters: H,
) -> Option<EdgeGeometry>
where
    F: Fn([f64; 3]) -> Option<f64>,
    G: Fn(f64) -> [f64; 3],
    H: Fn(f64, f64) -> f64,
{
    let start_parameter = parameter_at_point(endpoints.start)?;
    let end_parameter_base = parameter_at_point(endpoints.end)?;
    let direction_sign = if dot3(derivative_at_parameter(start_parameter), start_tangent) >= 0.0 {
        1.0
    } else {
        -1.0
    };
    let closed = approx_points_eq(endpoints.start, endpoints.end, 1.0e-7);
    let (start_parameter, end_parameter) = if closed && edge_length > 1.0e-9 {
        if direction_sign >= 0.0 {
            let start_parameter = normalize_periodic_parameter(start_parameter, period);
            (start_parameter, start_parameter + period)
        } else {
            let end_parameter = normalize_periodic_parameter(end_parameter_base, period);
            (end_parameter + period, end_parameter)
        }
    } else {
        let end_parameter = select_periodic_end_parameter(
            direction_sign,
            edge_length,
            period,
            start_parameter,
            end_parameter_base,
            length_with_parameters,
        )?;
        (start_parameter, end_parameter)
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
    direction_sign: f64,
    edge_length: f64,
    period: f64,
    start_parameter: f64,
    end_parameter_base: f64,
    length_with_parameters: H,
) -> Option<f64>
where
    H: Fn(f64, f64) -> f64,
{
    let candidates = [
        end_parameter_base - period,
        end_parameter_base,
        end_parameter_base + period,
    ];

    candidates
        .into_iter()
        .filter(|candidate| {
            let delta = *candidate - start_parameter;
            if direction_sign >= 0.0 {
                delta >= -1.0e-9
            } else {
                delta <= 1.0e-9
            }
        })
        .min_by(|lhs, rhs| {
            let lhs_error = (length_with_parameters(start_parameter, *lhs) - edge_length).abs();
            let rhs_error = (length_with_parameters(start_parameter, *rhs) - edge_length).abs();
            lhs_error.total_cmp(&rhs_error)
        })
}

fn canonicalize_periodic_parameters(
    mut start_parameter: f64,
    mut end_parameter: f64,
    period: f64,
    direction_sign: f64,
) -> (f64, f64) {
    let period = period.abs();
    start_parameter = snap_periodic_parameter(start_parameter, period);
    end_parameter = snap_periodic_parameter(end_parameter, period);

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

fn ported_analytic_face_geometry(kind: SurfaceKind, bounds: FaceUvBounds) -> FaceGeometry {
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

fn ported_line_payload(
    context: &Context,
    shape: &Shape,
    geometry: EdgeGeometry,
) -> Result<Option<LinePayload>, Error> {
    let endpoints = context.edge_endpoints(shape)?;
    Ok(ported_line_payload_from_endpoints(geometry, endpoints))
}

fn ported_line_payload_from_endpoints(
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

fn ported_circle_payload(
    context: &Context,
    shape: &Shape,
    geometry: EdgeGeometry,
) -> Result<Option<CirclePayload>, Error> {
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

    let sample0 = context.edge_sample_at_parameter_occt(shape, parameter0)?;
    let sample1 = context.edge_sample_at_parameter_occt(shape, parameter1)?;
    let sample2 = context.edge_sample_at_parameter_occt(shape, parameter2)?;
    let (center, x_component, y_component) = match solve_trigonometric_curve_components(
        [parameter0, parameter1, parameter2],
        [sample0.position, sample1.position, sample2.position],
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
        let sample = context.edge_sample_at_parameter_occt(shape, parameter)?;
        if !approx_points_eq(
            sample_circle(payload, parameter).position,
            sample.position,
            1.0e-7,
        ) {
            return Ok(None);
        }
    }

    Ok(Some(payload))
}

fn ported_ellipse_payload(
    context: &Context,
    shape: &Shape,
    geometry: EdgeGeometry,
) -> Result<Option<EllipsePayload>, Error> {
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

    let sample0 = context.edge_sample_at_parameter_occt(shape, parameter0)?;
    let sample1 = context.edge_sample_at_parameter_occt(shape, parameter1)?;
    let sample2 = context.edge_sample_at_parameter_occt(shape, parameter2)?;
    let (center, x_component, y_component) = match solve_trigonometric_curve_components(
        [parameter0, parameter1, parameter2],
        [sample0.position, sample1.position, sample2.position],
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
        let sample = context.edge_sample_at_parameter_occt(shape, parameter)?;
        if !approx_points_eq(
            sample_ellipse(payload, parameter).position,
            sample.position,
            1.0e-7,
        ) {
            return Ok(None);
        }
    }

    Ok(Some(payload))
}

fn ported_plane_payload(
    context: &Context,
    shape: &Shape,
    geometry: FaceGeometry,
) -> Result<Option<PlanePayload>, Error> {
    if geometry.kind != SurfaceKind::Plane {
        return Ok(None);
    }

    let u_span = geometry.u_max - geometry.u_min;
    let v_span = geometry.v_max - geometry.v_min;
    if u_span.abs() <= 1.0e-12 || v_span.abs() <= 1.0e-12 {
        return Ok(None);
    }

    let origin_sample = context.face_sample_occt(shape, [geometry.u_min, geometry.v_min])?;
    let u_sample = context.face_sample_occt(shape, [geometry.u_max, geometry.v_min])?;
    let v_sample = context.face_sample_occt(shape, [geometry.u_min, geometry.v_max])?;

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

fn ported_cylinder_payload(
    context: &Context,
    shape: &Shape,
    geometry: FaceGeometry,
) -> Result<Option<CylinderPayload>, Error> {
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

    let orientation = context.shape_orientation(shape)?;
    let base_sample = context.face_sample_occt(shape, [u0, geometry.v_min])?;
    let axis_sample = context.face_sample_occt(shape, [u0, geometry.v_max])?;
    let probe_sample = context.face_sample_occt(shape, [u1, geometry.v_min])?;
    let probe_top_sample = context.face_sample_occt(shape, [u1, geometry.v_max])?;
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

fn ported_cone_payload(
    context: &Context,
    shape: &Shape,
    geometry: FaceGeometry,
) -> Result<Option<ConePayload>, Error> {
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

    let orientation = context.shape_orientation(shape)?;
    let normal_sign = if matches!(orientation, Orientation::Reversed) {
        -1.0
    } else {
        1.0
    };

    let base_sample = context.face_sample_occt(shape, [u0, geometry.v_min])?;
    let base_top_sample = context.face_sample_occt(shape, [u0, geometry.v_max])?;
    let probe_sample = context.face_sample_occt(shape, [u1, geometry.v_min])?;
    let probe_top_sample = context.face_sample_occt(shape, [u1, geometry.v_max])?;
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

fn ported_sphere_payload(
    context: &Context,
    shape: &Shape,
    geometry: FaceGeometry,
) -> Result<Option<SpherePayload>, Error> {
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

    let orientation = context.shape_orientation(shape)?;
    let normal_sign = if matches!(orientation, Orientation::Reversed) {
        -1.0
    } else {
        1.0
    };

    let base_sample = context.face_sample_occt(shape, [u0, v0])?;
    let longitude_sample = context.face_sample_occt(shape, [u1, v0])?;
    let latitude_sample = context.face_sample_occt(shape, [u0, v1])?;
    let latitude_longitude_sample = context.face_sample_occt(shape, [u1, v1])?;
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

fn ported_torus_payload(
    context: &Context,
    shape: &Shape,
    geometry: FaceGeometry,
) -> Result<Option<TorusPayload>, Error> {
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

    let orientation = context.shape_orientation(shape)?;
    let normal_sign = if matches!(orientation, Orientation::Reversed) {
        -1.0
    } else {
        1.0
    };

    let sample00 = context.face_sample_occt(shape, [u0, v0])?;
    let sample01 = context.face_sample_occt(shape, [u0, v1])?;
    let sample10 = context.face_sample_occt(shape, [u1, v0])?;
    let sample11 = context.face_sample_occt(shape, [u1, v1])?;
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

fn circle_parameter(payload: CirclePayload, point: [f64; 3]) -> f64 {
    subtract3(point, payload.center).atan2_components(payload.x_direction, payload.y_direction)
}

fn ellipse_parameter(payload: EllipsePayload, point: [f64; 3]) -> Option<f64> {
    if payload.major_radius.abs() <= 1.0e-12 || payload.minor_radius.abs() <= 1.0e-12 {
        return None;
    }

    let relative = subtract3(point, payload.center);
    Some(
        (dot3(relative, payload.y_direction) / payload.minor_radius)
            .atan2(dot3(relative, payload.x_direction) / payload.major_radius),
    )
}

fn circle_derivative_from_parameter(payload: CirclePayload) -> impl Fn(f64) -> [f64; 3] {
    move |parameter| circle_derivative(payload, parameter)
}

fn ellipse_derivative_from_parameter(payload: EllipsePayload) -> impl Fn(f64) -> [f64; 3] {
    move |parameter| ellipse_derivative(payload, parameter)
}

fn approx_points_eq(lhs: [f64; 3], rhs: [f64; 3], tolerance: f64) -> bool {
    (lhs[0] - rhs[0]).abs() <= tolerance
        && (lhs[1] - rhs[1]).abs() <= tolerance
        && (lhs[2] - rhs[2]).abs() <= tolerance
}
