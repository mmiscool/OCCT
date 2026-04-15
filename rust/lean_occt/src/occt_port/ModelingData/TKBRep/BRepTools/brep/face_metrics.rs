use super::swept_face::{
    append_edge_sample_points, basis_parameter_on_u, oriented_wire_edges, positive_scalar_integral,
    revolution_surface_dv, rotate_point_about_axis, rotate_vector_about_axis,
    signed_scalar_integral,
};
use super::topology::oriented_edge_geometry;
use super::*;

#[derive(Clone, Copy, Debug)]
struct CurveDifferential {
    position: [f64; 3],
    first_derivative: [f64; 3],
    second_derivative: [f64; 3],
}

#[derive(Clone, Copy, Debug)]
struct OffsetCurveDifferential {
    position: [f64; 3],
    derivative: [f64; 3],
}

pub(super) fn analytic_face_area(
    context: &Context,
    surface: PortedSurface,
    face_geometry: FaceGeometry,
    loops: &[BrepFaceLoop],
    wires: &[BrepWire],
    edges: &[BrepEdge],
    edge_shapes: &[Shape],
) -> Option<f64> {
    if let Some(area) = exact_closed_face_area(surface, face_geometry) {
        return Some(area);
    }

    if loops.is_empty() {
        return match surface {
            PortedSurface::Sphere(payload) => Some(4.0 * PI * payload.radius.abs().powi(2)),
            PortedSurface::Torus(payload) => {
                Some(4.0 * PI * PI * payload.major_radius.abs() * payload.minor_radius.abs())
            }
            _ => Some(0.0),
        };
    }

    let plane = match surface {
        PortedSurface::Plane(plane) => Some(plane),
        _ => None,
    };

    let mut area = 0.0;
    for face_loop in loops {
        let wire = wires.get(face_loop.wire_index)?;
        let mut curve_segments = Vec::with_capacity(wire.edge_indices.len());
        let mut sampled_points = Vec::new();
        for (edge_index, edge_orientation) in oriented_wire_edges(wire, face_loop.orientation) {
            let edge = edges.get(edge_index)?;
            let geometry = oriented_edge_geometry(edge.geometry, edge_orientation);
            if let Some(curve) = edge.ported_curve {
                curve_segments.push((curve, geometry));
            }
            append_edge_sample_points(
                context,
                edge_shapes.get(edge_index)?,
                edge,
                geometry,
                &mut sampled_points,
            )
            .ok()?;
        }

        let wire_area = match plane {
            Some(plane) if curve_segments.len() == wire.edge_indices.len() => {
                planar_wire_signed_area(plane, &curve_segments).abs()
            }
            Some(_) => {
                analytic_sampled_wire_signed_area(surface, face_geometry, &sampled_points)?.abs()
            }
            None => {
                analytic_sampled_wire_signed_area(surface, face_geometry, &sampled_points)?.abs()
            }
        };
        match face_loop.role {
            LoopRole::Inner => area -= wire_area,
            LoopRole::Outer | LoopRole::Unknown => area += wire_area,
        }
    }
    Some(area.abs())
}

fn exact_closed_face_area(surface: PortedSurface, face_geometry: FaceGeometry) -> Option<f64> {
    match surface {
        PortedSurface::Sphere(payload)
            if periodic_span_matches(
                face_geometry.is_u_periodic,
                face_geometry.u_period,
                face_geometry.u_max - face_geometry.u_min,
            ) && approx_eq(
                face_geometry.v_max - face_geometry.v_min,
                PI,
                1.0e-6,
                1.0e-6,
            ) =>
        {
            Some(4.0 * PI * payload.radius.abs().powi(2))
        }
        PortedSurface::Torus(payload)
            if periodic_span_matches(
                face_geometry.is_u_periodic,
                face_geometry.u_period,
                face_geometry.u_max - face_geometry.u_min,
            ) && periodic_span_matches(
                face_geometry.is_v_periodic,
                face_geometry.v_period,
                face_geometry.v_max - face_geometry.v_min,
            ) =>
        {
            Some(4.0 * PI * PI * payload.major_radius.abs() * payload.minor_radius.abs())
        }
        _ => None,
    }
}

fn periodic_span_matches(is_periodic: bool, period: f64, span: f64) -> bool {
    is_periodic && approx_eq(span.abs(), period.abs(), 1.0e-6, 1.0e-6)
}

pub(super) fn analytic_ported_swept_face_area(
    surface: PortedSweptSurface,
    face_geometry: FaceGeometry,
) -> Option<f64> {
    match surface {
        PortedSweptSurface::Extrusion {
            payload,
            basis_curve,
            basis_geometry,
        } => {
            let span = swept_surface_span(face_geometry, basis_geometry)?;
            Some(extrusion_swept_area(
                basis_curve,
                basis_geometry,
                payload.direction,
                span,
            ))
        }
        PortedSweptSurface::Revolution {
            payload,
            basis_curve,
            basis_geometry,
        } => {
            let sweep_angle = swept_surface_span(face_geometry, basis_geometry)?;
            Some(revolution_swept_area(
                basis_curve,
                basis_geometry,
                payload.axis_origin,
                payload.axis_direction,
                sweep_angle,
            ))
        }
    }
}

pub(super) fn analytic_offset_face_area(
    context: &Context,
    surface: PortedOffsetSurface,
    face_geometry: FaceGeometry,
    loops: &[BrepFaceLoop],
    wires: &[BrepWire],
    edges: &[BrepEdge],
    edge_shapes: &[Shape],
) -> Option<f64> {
    if let Some(equivalent_surface) = surface.equivalent_analytic_surface() {
        return analytic_face_area(
            context,
            equivalent_surface,
            face_geometry,
            loops,
            wires,
            edges,
            edge_shapes,
        );
    }

    match surface.basis {
        PortedOffsetBasisSurface::Analytic(_) => None,
        PortedOffsetBasisSurface::Swept(PortedSweptSurface::Extrusion {
            payload,
            basis_curve,
            basis_geometry,
        }) => offset_extrusion_swept_area(
            surface.payload.offset_value,
            surface.basis_geometry,
            basis_curve,
            basis_geometry,
            payload.direction,
        ),
        PortedOffsetBasisSurface::Swept(PortedSweptSurface::Revolution {
            payload,
            basis_curve,
            basis_geometry,
        }) => offset_revolution_swept_area(
            surface.payload.offset_value,
            surface.basis_geometry,
            basis_curve,
            basis_geometry,
            payload.axis_origin,
            payload.axis_direction,
        ),
    }
}

pub(super) fn ported_face_area_from_surface(
    context: &Context,
    ported_face_surface: Option<PortedFaceSurface>,
    face_geometry: FaceGeometry,
    loops: &[BrepFaceLoop],
    wires: &[BrepWire],
    edges: &[BrepEdge],
    edge_shapes: &[Shape],
) -> Option<f64> {
    match ported_face_surface {
        Some(PortedFaceSurface::Analytic(surface)) => analytic_face_area(
            context,
            surface,
            face_geometry,
            loops,
            wires,
            edges,
            edge_shapes,
        ),
        Some(PortedFaceSurface::Offset(surface)) => analytic_offset_face_area(
            context,
            surface,
            face_geometry,
            loops,
            wires,
            edges,
            edge_shapes,
        ),
        Some(PortedFaceSurface::Swept(surface)) => {
            analytic_ported_swept_face_area(surface, face_geometry)
        }
        None => None,
    }
}

fn offset_extrusion_swept_area(
    offset: f64,
    surface_geometry: FaceGeometry,
    curve: PortedCurve,
    curve_geometry: EdgeGeometry,
    direction: [f64; 3],
) -> Option<f64> {
    let direction = normalize3(direction);
    if norm3(direction) <= 1.0e-12 {
        return None;
    }

    let sweep_span = swept_surface_span(surface_geometry, curve_geometry)?;
    let basis_on_u = basis_parameter_on_u(surface_geometry, curve_geometry);
    Some(
        sweep_span.abs()
            * positive_scalar_integral(
                curve_geometry.start_parameter,
                curve_geometry.end_parameter,
                |parameter| {
                    let differential = offset_extrusion_curve_differential(
                        curve, direction, offset, basis_on_u, parameter,
                    );
                    norm3(cross3(differential.derivative, direction))
                },
            ),
    )
}

fn offset_revolution_swept_area(
    offset: f64,
    surface_geometry: FaceGeometry,
    curve: PortedCurve,
    curve_geometry: EdgeGeometry,
    axis_origin: [f64; 3],
    axis_direction: [f64; 3],
) -> Option<f64> {
    let axis_direction = normalize3(axis_direction);
    if norm3(axis_direction) <= 1.0e-12 {
        return None;
    }

    let sweep_angle = swept_surface_span(surface_geometry, curve_geometry)?;
    let basis_on_u = basis_parameter_on_u(surface_geometry, curve_geometry);
    Some(
        sweep_angle.abs()
            * positive_scalar_integral(
                curve_geometry.start_parameter,
                curve_geometry.end_parameter,
                |parameter| {
                    let differential = offset_revolution_curve_differential(
                        curve,
                        axis_origin,
                        axis_direction,
                        offset,
                        basis_on_u,
                        parameter,
                    );
                    let sweep_derivative = cross3(
                        axis_direction,
                        subtract3(differential.position, axis_origin),
                    );
                    norm3(cross3(differential.derivative, sweep_derivative))
                },
            ),
    )
}

fn swept_surface_span(surface_geometry: FaceGeometry, curve_geometry: EdgeGeometry) -> Option<f64> {
    let span = if basis_parameter_on_u(surface_geometry, curve_geometry) {
        surface_geometry.v_max - surface_geometry.v_min
    } else {
        surface_geometry.u_max - surface_geometry.u_min
    };
    (span.abs() > 1.0e-12).then_some(span)
}

fn offset_extrusion_curve_differential(
    curve: PortedCurve,
    direction: [f64; 3],
    offset: f64,
    basis_on_u: bool,
    parameter: f64,
) -> OffsetCurveDifferential {
    let differential = curve_differential(curve, parameter);
    let (normal_source, normal_source_derivative) = if basis_on_u {
        (
            cross3(differential.first_derivative, direction),
            cross3(differential.second_derivative, direction),
        )
    } else {
        (
            cross3(direction, differential.first_derivative),
            cross3(direction, differential.second_derivative),
        )
    };
    let normal = normalize3(normal_source);
    let normal_derivative =
        normalized_direction_derivative(normal_source, normal_source_derivative);
    OffsetCurveDifferential {
        position: add3(differential.position, scale3(normal, offset)),
        derivative: add3(
            differential.first_derivative,
            scale3(normal_derivative, offset),
        ),
    }
}

fn offset_revolution_curve_differential(
    curve: PortedCurve,
    axis_origin: [f64; 3],
    axis_direction: [f64; 3],
    offset: f64,
    basis_on_u: bool,
    parameter: f64,
) -> OffsetCurveDifferential {
    let differential = curve_differential(curve, parameter);
    let sweep_derivative = cross3(
        axis_direction,
        subtract3(differential.position, axis_origin),
    );
    let sweep_second_derivative = cross3(axis_direction, differential.first_derivative);
    let (normal_source, normal_source_derivative) = if basis_on_u {
        (
            cross3(differential.first_derivative, sweep_derivative),
            add3(
                cross3(differential.second_derivative, sweep_derivative),
                cross3(differential.first_derivative, sweep_second_derivative),
            ),
        )
    } else {
        (
            cross3(sweep_derivative, differential.first_derivative),
            add3(
                cross3(sweep_second_derivative, differential.first_derivative),
                cross3(sweep_derivative, differential.second_derivative),
            ),
        )
    };
    let normal = normalize3(normal_source);
    let normal_derivative =
        normalized_direction_derivative(normal_source, normal_source_derivative);
    OffsetCurveDifferential {
        position: add3(differential.position, scale3(normal, offset)),
        derivative: add3(
            differential.first_derivative,
            scale3(normal_derivative, offset),
        ),
    }
}

fn curve_differential(curve: PortedCurve, parameter: f64) -> CurveDifferential {
    match curve {
        PortedCurve::Line(payload) => CurveDifferential {
            position: add3(payload.origin, scale3(payload.direction, parameter)),
            first_derivative: payload.direction,
            second_derivative: [0.0; 3],
        },
        PortedCurve::Circle(payload) => {
            let cos_parameter = parameter.cos();
            let sin_parameter = parameter.sin();
            CurveDifferential {
                position: add3(
                    payload.center,
                    add3(
                        scale3(payload.x_direction, payload.radius * cos_parameter),
                        scale3(payload.y_direction, payload.radius * sin_parameter),
                    ),
                ),
                first_derivative: add3(
                    scale3(payload.x_direction, -payload.radius * sin_parameter),
                    scale3(payload.y_direction, payload.radius * cos_parameter),
                ),
                second_derivative: add3(
                    scale3(payload.x_direction, -payload.radius * cos_parameter),
                    scale3(payload.y_direction, -payload.radius * sin_parameter),
                ),
            }
        }
        PortedCurve::Ellipse(payload) => {
            let cos_parameter = parameter.cos();
            let sin_parameter = parameter.sin();
            CurveDifferential {
                position: add3(
                    payload.center,
                    add3(
                        scale3(payload.x_direction, payload.major_radius * cos_parameter),
                        scale3(payload.y_direction, payload.minor_radius * sin_parameter),
                    ),
                ),
                first_derivative: add3(
                    scale3(payload.x_direction, -payload.major_radius * sin_parameter),
                    scale3(payload.y_direction, payload.minor_radius * cos_parameter),
                ),
                second_derivative: add3(
                    scale3(payload.x_direction, -payload.major_radius * cos_parameter),
                    scale3(payload.y_direction, -payload.minor_radius * sin_parameter),
                ),
            }
        }
    }
}

fn normalized_direction_derivative(direction: [f64; 3], derivative: [f64; 3]) -> [f64; 3] {
    let direction_norm = norm3(direction);
    if direction_norm <= 1.0e-12 {
        return [0.0; 3];
    }

    let unit_direction = scale3(direction, direction_norm.recip());
    scale3(
        subtract3(
            derivative,
            scale3(unit_direction, dot3(unit_direction, derivative)),
        ),
        direction_norm.recip(),
    )
}

pub(super) fn analytic_ported_swept_face_volume(
    face: &BrepFace,
    face_geometry: FaceGeometry,
    surface: PortedSweptSurface,
) -> Option<f64> {
    match surface {
        PortedSweptSurface::Extrusion {
            payload,
            basis_curve,
            basis_geometry,
        } => {
            let direction = normalize3(payload.direction);
            if norm3(direction) <= 1.0e-12 {
                return None;
            }
            let sweep = scale3(
                direction,
                swept_surface_span(face_geometry, basis_geometry)?.abs(),
            );
            let midpoint_parameter =
                0.5 * (basis_geometry.start_parameter + basis_geometry.end_parameter);
            let midpoint = basis_curve.evaluate(midpoint_parameter);
            let midpoint_position = add3(midpoint.position, scale3(sweep, 0.5));
            let midpoint_du = midpoint.derivative;
            let midpoint_dv = sweep;
            let sign = oriented_surface_sign(face, midpoint_position, midpoint_du, midpoint_dv);
            Some(
                sign * signed_scalar_integral(
                    basis_geometry.start_parameter,
                    basis_geometry.end_parameter,
                    |parameter| {
                        let evaluation = basis_curve.evaluate(parameter);
                        dot3(evaluation.position, cross3(evaluation.derivative, sweep)) / 3.0
                    },
                ),
            )
        }
        PortedSweptSurface::Revolution {
            payload,
            basis_curve,
            basis_geometry,
        } => {
            let axis_direction = normalize3(payload.axis_direction);
            if norm3(axis_direction) <= 1.0e-12 {
                return None;
            }
            let sweep_angle = swept_surface_span(face_geometry, basis_geometry)?.abs();
            let midpoint_parameter =
                0.5 * (basis_geometry.start_parameter + basis_geometry.end_parameter);
            let midpoint_evaluation = basis_curve.evaluate(midpoint_parameter);
            let midpoint_position = rotate_point_about_axis(
                midpoint_evaluation.position,
                payload.axis_origin,
                axis_direction,
                0.5 * sweep_angle,
            );
            let midpoint_du = rotate_vector_about_axis(
                midpoint_evaluation.derivative,
                axis_direction,
                0.5 * sweep_angle,
            );
            let midpoint_dv =
                revolution_surface_dv(midpoint_position, payload.axis_origin, axis_direction);
            let sign = oriented_surface_sign(face, midpoint_position, midpoint_du, midpoint_dv);

            Some(
                sign * signed_scalar_integral(
                    basis_geometry.start_parameter,
                    basis_geometry.end_parameter,
                    |parameter| {
                        let evaluation = basis_curve.evaluate(parameter);
                        signed_scalar_integral(0.0, sweep_angle, |angle| {
                            let position = rotate_point_about_axis(
                                evaluation.position,
                                payload.axis_origin,
                                axis_direction,
                                angle,
                            );
                            let du = rotate_vector_about_axis(
                                evaluation.derivative,
                                axis_direction,
                                angle,
                            );
                            let dv = revolution_surface_dv(
                                position,
                                payload.axis_origin,
                                axis_direction,
                            );
                            dot3(position, cross3(du, dv)) / 3.0
                        })
                    },
                ),
            )
        }
    }
}

pub(super) fn analytic_offset_face_volume(
    context: &Context,
    face: &BrepFace,
    surface: PortedOffsetSurface,
    face_geometry: FaceGeometry,
    loops: &[BrepFaceLoop],
    wires: &[BrepWire],
    edges: &[BrepEdge],
    edge_shapes: &[Shape],
) -> Option<f64> {
    if let Some(equivalent_surface) = surface.equivalent_analytic_surface() {
        return analytic_face_volume(
            context,
            face,
            equivalent_surface,
            face_geometry,
            loops,
            wires,
            edges,
            edge_shapes,
        );
    }

    match surface.basis {
        PortedOffsetBasisSurface::Analytic(_) => None,
        PortedOffsetBasisSurface::Swept(PortedSweptSurface::Extrusion {
            payload,
            basis_curve,
            basis_geometry,
        }) => analytic_offset_extrusion_face_volume(
            face,
            surface.payload.offset_value,
            surface.basis_geometry,
            basis_curve,
            basis_geometry,
            payload.direction,
        ),
        PortedOffsetBasisSurface::Swept(PortedSweptSurface::Revolution {
            payload,
            basis_curve,
            basis_geometry,
        }) => analytic_offset_revolution_face_volume(
            face,
            surface.payload.offset_value,
            surface.basis_geometry,
            basis_curve,
            basis_geometry,
            payload.axis_origin,
            payload.axis_direction,
        ),
    }
}

fn analytic_offset_extrusion_face_volume(
    face: &BrepFace,
    offset: f64,
    surface_geometry: FaceGeometry,
    curve: PortedCurve,
    curve_geometry: EdgeGeometry,
    direction: [f64; 3],
) -> Option<f64> {
    let direction = normalize3(direction);
    if norm3(direction) <= 1.0e-12 {
        return None;
    }

    let sweep = scale3(
        direction,
        swept_surface_span(surface_geometry, curve_geometry)?.abs(),
    );
    let basis_on_u = basis_parameter_on_u(surface_geometry, curve_geometry);
    let midpoint_parameter = 0.5 * (curve_geometry.start_parameter + curve_geometry.end_parameter);
    let midpoint = offset_extrusion_curve_differential(
        curve,
        direction,
        offset,
        basis_on_u,
        midpoint_parameter,
    );
    let midpoint_position = add3(midpoint.position, scale3(sweep, 0.5));
    let midpoint_du = midpoint.derivative;
    let midpoint_dv = sweep;
    let sign = oriented_surface_sign(face, midpoint_position, midpoint_du, midpoint_dv);

    Some(
        sign * signed_scalar_integral(
            curve_geometry.start_parameter,
            curve_geometry.end_parameter,
            |parameter| {
                let differential = offset_extrusion_curve_differential(
                    curve, direction, offset, basis_on_u, parameter,
                );
                dot3(
                    differential.position,
                    cross3(differential.derivative, sweep),
                ) / 3.0
            },
        ),
    )
}

fn analytic_offset_revolution_face_volume(
    face: &BrepFace,
    offset: f64,
    surface_geometry: FaceGeometry,
    curve: PortedCurve,
    curve_geometry: EdgeGeometry,
    axis_origin: [f64; 3],
    axis_direction: [f64; 3],
) -> Option<f64> {
    let axis_direction = normalize3(axis_direction);
    if norm3(axis_direction) <= 1.0e-12 {
        return None;
    }

    let sweep_angle = swept_surface_span(surface_geometry, curve_geometry)?.abs();
    let basis_on_u = basis_parameter_on_u(surface_geometry, curve_geometry);
    let midpoint_parameter = 0.5 * (curve_geometry.start_parameter + curve_geometry.end_parameter);
    let midpoint = offset_revolution_curve_differential(
        curve,
        axis_origin,
        axis_direction,
        offset,
        basis_on_u,
        midpoint_parameter,
    );
    let midpoint_position = rotate_point_about_axis(
        midpoint.position,
        axis_origin,
        axis_direction,
        0.5 * sweep_angle,
    );
    let midpoint_du =
        rotate_vector_about_axis(midpoint.derivative, axis_direction, 0.5 * sweep_angle);
    let midpoint_dv = revolution_surface_dv(midpoint_position, axis_origin, axis_direction);
    let sign = oriented_surface_sign(face, midpoint_position, midpoint_du, midpoint_dv);

    Some(
        sign * signed_scalar_integral(
            curve_geometry.start_parameter,
            curve_geometry.end_parameter,
            |parameter| {
                let differential = offset_revolution_curve_differential(
                    curve,
                    axis_origin,
                    axis_direction,
                    offset,
                    basis_on_u,
                    parameter,
                );
                signed_scalar_integral(0.0, sweep_angle, |angle| {
                    let position = rotate_point_about_axis(
                        differential.position,
                        axis_origin,
                        axis_direction,
                        angle,
                    );
                    let du =
                        rotate_vector_about_axis(differential.derivative, axis_direction, angle);
                    let dv = revolution_surface_dv(position, axis_origin, axis_direction);
                    dot3(position, cross3(du, dv)) / 3.0
                })
            },
        ),
    )
}

pub(super) fn analytic_face_volume(
    context: &Context,
    face: &BrepFace,
    surface: PortedSurface,
    face_geometry: FaceGeometry,
    loops: &[BrepFaceLoop],
    wires: &[BrepWire],
    edges: &[BrepEdge],
    edge_shapes: &[Shape],
) -> Option<f64> {
    if matches!(surface, PortedSurface::Plane(_)) {
        return Some(face.area * dot3(face.sample.position, face.sample.normal) / 3.0);
    }

    let mut volume = 0.0;
    for face_loop in loops {
        let wire = wires.get(face_loop.wire_index)?;
        let mut sampled_points = Vec::new();
        for (edge_index, edge_orientation) in oriented_wire_edges(wire, face_loop.orientation) {
            let edge = edges.get(edge_index)?;
            let geometry = oriented_edge_geometry(edge.geometry, edge_orientation);
            append_edge_sample_points(
                context,
                edge_shapes.get(edge_index)?,
                edge,
                geometry,
                &mut sampled_points,
            )
            .ok()?;
        }
        let loop_volume =
            analytic_sampled_wire_signed_volume(surface, face_geometry, &sampled_points)?;
        match face_loop.role {
            LoopRole::Inner => volume -= loop_volume,
            LoopRole::Outer | LoopRole::Unknown => volume += loop_volume,
        }
    }
    Some(volume)
}

fn oriented_surface_sign(face: &BrepFace, position: [f64; 3], du: [f64; 3], dv: [f64; 3]) -> f64 {
    let _ = position;
    let normal = normalize3(cross3(du, dv));
    if dot3(normal, face.sample.normal) >= 0.0 {
        1.0
    } else {
        -1.0
    }
}
