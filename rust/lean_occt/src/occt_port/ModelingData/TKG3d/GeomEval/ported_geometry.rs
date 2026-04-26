use std::f64::consts::TAU;

mod payloads;
mod wire_metrics;

use self::wire_metrics::length_integral;
pub(crate) use self::wire_metrics::{
    analytic_sampled_wire_signed_area, analytic_sampled_wire_signed_volume, planar_wire_signed_area,
};

use self::payloads::*;
use crate::brep::{
    ported_face_area as ported_face_area_value,
    ported_face_surface_descriptor as ported_face_surface_descriptor_value,
};
use crate::{
    CirclePayload, ConePayload, Context, CurveKind, CylinderPayload, EdgeGeometry, EdgeSample,
    EllipsePayload, Error, ExtrusionSurfacePayload, FaceGeometry, FaceSample, FaceUvBounds,
    LinePayload, OffsetSurfaceFaceMetadata, OffsetSurfacePayload, Orientation, PlanePayload,
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

const ANALYTIC_SURFACE_KINDS: [SurfaceKind; 5] = [
    SurfaceKind::Plane,
    SurfaceKind::Cylinder,
    SurfaceKind::Cone,
    SurfaceKind::Sphere,
    SurfaceKind::Torus,
];

fn is_analytic_surface_kind(kind: SurfaceKind) -> bool {
    matches!(
        kind,
        SurfaceKind::Plane
            | SurfaceKind::Cylinder
            | SurfaceKind::Cone
            | SurfaceKind::Sphere
            | SurfaceKind::Torus
    )
}

fn ported_analytic_surface_kind(surface: PortedSurface) -> SurfaceKind {
    match surface {
        PortedSurface::Plane(_) => SurfaceKind::Plane,
        PortedSurface::Cylinder(_) => SurfaceKind::Cylinder,
        PortedSurface::Cone(_) => SurfaceKind::Cone,
        PortedSurface::Sphere(_) => SurfaceKind::Sphere,
        PortedSurface::Torus(_) => SurfaceKind::Torus,
    }
}

impl PortedCurve {
    pub fn from_context(context: &Context, shape: &Shape) -> Result<Option<Self>, Error> {
        let geometry = context.edge_geometry(shape)?;
        Self::from_context_with_ported_payloads(context, shape, geometry)
    }

    pub fn from_context_with_geometry(
        context: &Context,
        shape: &Shape,
        geometry: EdgeGeometry,
    ) -> Result<Option<Self>, Error> {
        Self::from_context_with_ported_payloads(context, shape, geometry)
    }

    pub(crate) fn from_context_with_ported_payloads(
        context: &Context,
        shape: &Shape,
        geometry: EdgeGeometry,
    ) -> Result<Option<Self>, Error> {
        match geometry.kind {
            CurveKind::Line => Ok(ported_line_payload(context, shape, geometry)?.map(Self::Line)),
            CurveKind::Circle => {
                Ok(ported_circle_payload(context, shape, geometry)?.map(Self::Circle))
            }
            CurveKind::Ellipse => {
                Ok(ported_ellipse_payload(context, shape, geometry)?.map(Self::Ellipse))
            }
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
    if (u_delta - v_delta).abs() <= 1.0e-9 {
        return match face_geometry.kind {
            SurfaceKind::Revolution => false,
            SurfaceKind::Extrusion => true,
            _ => true,
        };
    }
    u_delta < v_delta
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
        Self::from_context_with_ported_payloads(context, shape, geometry)
    }

    pub(crate) fn from_context_with_ported_payloads(
        context: &Context,
        shape: &Shape,
        geometry: FaceGeometry,
    ) -> Result<Option<Self>, Error> {
        match geometry.kind {
            SurfaceKind::Plane => {
                Ok(ported_plane_payload(context, shape, geometry)?.map(Self::Plane))
            }
            SurfaceKind::Cylinder => {
                Ok(ported_cylinder_payload(context, shape, geometry)?.map(Self::Cylinder))
            }
            SurfaceKind::Cone => Ok(ported_cone_payload(context, shape, geometry)?.map(Self::Cone)),
            SurfaceKind::Sphere => {
                Ok(ported_sphere_payload(context, shape, geometry)?.map(Self::Sphere))
            }
            SurfaceKind::Torus => {
                Ok(ported_torus_payload(context, shape, geometry)?.map(Self::Torus))
            }
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

fn periodic_edge_direction_preference(geometry: EdgeGeometry) -> Option<f64> {
    let span = geometry.end_parameter - geometry.start_parameter;
    if span.abs() > 1.0e-9 {
        Some(span.signum())
    } else {
        None
    }
}

impl Context {
    pub fn ported_edge_geometry(&self, shape: &Shape) -> Result<Option<EdgeGeometry>, Error> {
        let geometry = self.edge_geometry_occt(shape)?;
        let endpoints = self.edge_endpoints(shape)?;

        if geometry.kind == CurveKind::Line {
            if let Some(payload) = ported_line_payload_from_endpoints(geometry, endpoints) {
                return Ok(ported_line_geometry(payload, endpoints));
            }
            return Ok(None);
        }

        if geometry.kind == CurveKind::Circle {
            let Some(payload) = ported_circle_payload(self, shape, geometry)? else {
                return Ok(None);
            };
            let edge_length = shape.linear_length();
            let direction_preference = periodic_edge_direction_preference(geometry);
            return Ok(ported_periodic_curve_geometry(
                CurveKind::Circle,
                endpoints,
                edge_length,
                TAU,
                direction_preference,
                |point| Some(circle_parameter(payload, point)),
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

        if geometry.kind == CurveKind::Ellipse {
            let Some(payload) = ported_ellipse_payload(self, shape, geometry)? else {
                return Ok(None);
            };
            let edge_length = shape.linear_length();
            let direction_preference = periodic_edge_direction_preference(geometry);
            return Ok(ported_periodic_curve_geometry(
                CurveKind::Ellipse,
                endpoints,
                edge_length,
                TAU,
                direction_preference,
                |point| ellipse_parameter(payload, point),
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
        let raw_geometry = self.face_geometry_occt(shape)?;

        if matches!(
            raw_geometry.kind,
            SurfaceKind::Revolution | SurfaceKind::Extrusion | SurfaceKind::Offset
        ) {
            let descriptor = ported_face_surface_descriptor_value(self, shape, raw_geometry)?;
            return match (raw_geometry.kind, descriptor) {
                (
                    SurfaceKind::Revolution,
                    Some(PortedFaceSurface::Swept(PortedSweptSurface::Revolution { .. })),
                )
                | (
                    SurfaceKind::Extrusion,
                    Some(PortedFaceSurface::Swept(PortedSweptSurface::Extrusion { .. })),
                )
                | (SurfaceKind::Offset, Some(PortedFaceSurface::Offset(_))) => {
                    Ok(Some(raw_geometry))
                }
                _ => Ok(None),
            };
        }

        let bounds = self.face_uv_bounds_occt(shape)?;
        if is_analytic_surface_kind(raw_geometry.kind) {
            return ported_analytic_face_geometry_candidate(self, shape, raw_geometry.kind, bounds);
        }

        for candidate_kind in ANALYTIC_SURFACE_KINDS {
            if let Some(geometry) =
                ported_analytic_face_geometry_candidate(self, shape, candidate_kind, bounds)?
            {
                return Ok(Some(geometry));
            }
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
        let geometry = self.face_geometry(shape)?;
        self.ported_offset_surface_with_geometry(shape, geometry)
    }

    pub(crate) fn ported_offset_surface_with_geometry(
        &self,
        shape: &Shape,
        geometry: FaceGeometry,
    ) -> Result<Option<PortedOffsetSurface>, Error> {
        if geometry.kind != SurfaceKind::Offset {
            return Ok(None);
        }

        if let Some(metadata) = shape.offset_surface_face_metadata() {
            return ported_offset_surface_from_metadata(self, shape, metadata);
        }

        let payload = self.face_offset_payload_occt(shape)?;
        let basis_geometry = self.face_offset_basis_geometry_occt(shape)?;
        let basis = match payload.basis_surface_kind {
            SurfaceKind::Plane
            | SurfaceKind::Cylinder
            | SurfaceKind::Cone
            | SurfaceKind::Sphere
            | SurfaceKind::Torus => match ported_offset_basis_surface_payload(
                self,
                shape,
                payload.offset_value,
                basis_geometry,
            )? {
                Some(surface) => PortedOffsetBasisSurface::Analytic(surface),
                None => return Ok(None),
            },
            SurfaceKind::Revolution => {
                match ported_offset_basis_swept_surface_payload(
                    self,
                    shape,
                    payload.offset_value,
                    basis_geometry,
                )? {
                    Some(surface @ PortedSweptSurface::Revolution { .. }) => {
                        PortedOffsetBasisSurface::Swept(surface)
                    }
                    _ => return Ok(None),
                }
            }
            SurfaceKind::Extrusion => {
                match ported_offset_basis_swept_surface_payload(
                    self,
                    shape,
                    payload.offset_value,
                    basis_geometry,
                )? {
                    Some(surface @ PortedSweptSurface::Extrusion { .. }) => {
                        PortedOffsetBasisSurface::Swept(surface)
                    }
                    _ => return Ok(None),
                }
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

fn ported_offset_surface_from_metadata(
    context: &Context,
    shape: &Shape,
    metadata: OffsetSurfaceFaceMetadata,
) -> Result<Option<PortedOffsetSurface>, Error> {
    let surface = PortedOffsetSurface {
        payload: OffsetSurfacePayload {
            offset_value: metadata.offset_value,
            basis_surface_kind: ported_analytic_surface_kind(metadata.basis_surface),
        },
        basis_geometry: metadata.basis_geometry,
        basis: PortedOffsetBasisSurface::Analytic(metadata.basis_surface),
    };

    if ported_offset_surface_matches_occt_samples(context, shape, surface)? {
        Ok(Some(surface))
    } else {
        Ok(None)
    }
}

fn ported_offset_surface_matches_occt_samples(
    context: &Context,
    shape: &Shape,
    surface: PortedOffsetSurface,
) -> Result<bool, Error> {
    let orientation = context.shape_orientation(shape)?;
    for uv_t in [[0.23, 0.31], [0.37, 0.61], [0.58, 0.47], [0.79, 0.73]] {
        let expected = context.face_sample_normalized_occt(shape, uv_t)?;
        let actual = surface.sample_normalized_with_orientation(uv_t, orientation);
        if !approx_vec3_eq(actual.position, expected.position, 1.0e-6)
            || !approx_vec3_eq(actual.normal, expected.normal, 1.0e-6)
        {
            return Ok(false);
        }
    }

    Ok(true)
}

fn approx_vec3_eq(lhs: [f64; 3], rhs: [f64; 3], tolerance: f64) -> bool {
    (lhs[0] - rhs[0]).abs() <= tolerance
        && (lhs[1] - rhs[1]).abs() <= tolerance
        && (lhs[2] - rhs[2]).abs() <= tolerance
}

fn ported_analytic_face_geometry_candidate(
    context: &Context,
    shape: &Shape,
    kind: SurfaceKind,
    bounds: FaceUvBounds,
) -> Result<Option<FaceGeometry>, Error> {
    let geometry = ported_analytic_face_geometry(kind, bounds);
    let has_payload = match kind {
        SurfaceKind::Plane => ported_plane_payload(context, shape, geometry)?.is_some(),
        SurfaceKind::Cylinder => ported_cylinder_payload(context, shape, geometry)?.is_some(),
        SurfaceKind::Cone => ported_cone_payload(context, shape, geometry)?.is_some(),
        SurfaceKind::Sphere => ported_sphere_payload(context, shape, geometry)?.is_some(),
        SurfaceKind::Torus => ported_torus_payload(context, shape, geometry)?.is_some(),
        _ => false,
    };

    Ok(has_payload.then_some(geometry))
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
