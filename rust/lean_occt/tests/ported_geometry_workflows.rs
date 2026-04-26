mod support;

use std::f64::consts::PI;

use lean_occt::{
    BoxParams, ConeParams, CurveKind, CylinderParams, EllipseEdgeParams, HelixParams, ModelKernel,
    OffsetParams, PortedCurve, PortedFaceSurface, PortedOffsetBasisSurface, PortedOffsetSurface,
    PortedSurface, PortedSweptSurface, PrismParams, RevolutionParams, Shape, ShapeKind,
    SphereParams, SurfaceKind, ThroughHoleCut, TorusParams,
};

fn default_cut() -> ThroughHoleCut {
    ThroughHoleCut {
        box_params: BoxParams {
            origin: [-30.0, -30.0, -30.0],
            size: [60.0, 60.0, 60.0],
        },
        tool_params: CylinderParams {
            origin: [0.0, 0.0, -36.0],
            axis: [0.0, 0.0, 1.0],
            radius: 12.0,
            height: 72.0,
        },
    }
}

fn find_first_edge_by_kind(
    kernel: &ModelKernel,
    shape: &Shape,
    kind: CurveKind,
) -> Result<Shape, Box<dyn std::error::Error>> {
    for edge in kernel.context().subshapes(shape, ShapeKind::Edge)? {
        if kernel.context().edge_geometry(&edge)?.kind == kind {
            return Ok(edge);
        }
    }
    Err(std::io::Error::other(format!("expected edge with curve kind {:?}", kind)).into())
}

fn find_first_face_by_kind(
    kernel: &ModelKernel,
    shape: &Shape,
    kind: SurfaceKind,
) -> Result<Shape, Box<dyn std::error::Error>> {
    for face in kernel.context().subshapes(shape, ShapeKind::Face)? {
        if kernel.context().face_geometry(&face)?.kind == kind {
            return Ok(face);
        }
    }
    Err(std::io::Error::other(format!("expected face with surface kind {:?}", kind)).into())
}

fn find_first_swept_metadata_face_by_kind(
    kernel: &ModelKernel,
    shape: &Shape,
    kind: SurfaceKind,
) -> Result<Shape, Box<dyn std::error::Error>> {
    for face in kernel.context().subshapes(shape, ShapeKind::Face)? {
        if face.has_rust_swept_surface_face_metadata()
            && kernel.context().face_geometry(&face)?.kind == kind
        {
            return Ok(face);
        }
    }
    Err(std::io::Error::other(format!(
        "expected Rust-seeded swept face with surface kind {:?}",
        kind
    ))
    .into())
}

fn ported_curve_kind(curve: PortedCurve) -> CurveKind {
    match curve {
        PortedCurve::Line(_) => CurveKind::Line,
        PortedCurve::Circle(_) => CurveKind::Circle,
        PortedCurve::Ellipse(_) => CurveKind::Ellipse,
    }
}

fn require_ported_edge_curve(
    curve: Option<PortedCurve>,
    expected: CurveKind,
    label: &str,
) -> Result<PortedCurve, Box<dyn std::error::Error>> {
    let curve = curve
        .ok_or_else(|| std::io::Error::other(format!("{label} missing Rust curve descriptor")))?;
    let actual = ported_curve_kind(curve);
    if actual == expected {
        Ok(curve)
    } else {
        Err(std::io::Error::other(format!(
            "{label} expected Rust {expected:?} descriptor, got {actual:?}"
        ))
        .into())
    }
}

fn ported_surface_kind(surface: PortedSurface) -> SurfaceKind {
    match surface {
        PortedSurface::Plane(_) => SurfaceKind::Plane,
        PortedSurface::Cylinder(_) => SurfaceKind::Cylinder,
        PortedSurface::Cone(_) => SurfaceKind::Cone,
        PortedSurface::Sphere(_) => SurfaceKind::Sphere,
        PortedSurface::Torus(_) => SurfaceKind::Torus,
    }
}

fn require_ported_analytic_face_surface(
    surface: Option<PortedSurface>,
    expected: SurfaceKind,
    label: &str,
) -> Result<PortedSurface, Box<dyn std::error::Error>> {
    let surface = surface.ok_or_else(|| {
        std::io::Error::other(format!("{label} missing Rust analytic surface descriptor"))
    })?;
    let actual = ported_surface_kind(surface);
    if actual == expected {
        Ok(surface)
    } else {
        Err(std::io::Error::other(format!(
            "{label} expected Rust {expected:?} descriptor, got {actual:?}"
        ))
        .into())
    }
}

fn require_ported_swept_face_surface(
    surface: Option<PortedFaceSurface>,
    expected: SurfaceKind,
    label: &str,
) -> Result<PortedSweptSurface, Box<dyn std::error::Error>> {
    let surface = surface.ok_or_else(|| {
        std::io::Error::other(format!("{label} missing Rust swept surface descriptor"))
    })?;
    match surface {
        PortedFaceSurface::Swept(surface) => {
            let actual = match &surface {
                PortedSweptSurface::Revolution { .. } => SurfaceKind::Revolution,
                PortedSweptSurface::Extrusion { .. } => SurfaceKind::Extrusion,
            };
            if actual == expected {
                Ok(surface)
            } else {
                Err(std::io::Error::other(format!(
                    "{label} expected Rust {expected:?} swept descriptor, got {actual:?}"
                ))
                .into())
            }
        }
        descriptor => Err(std::io::Error::other(format!(
            "{label} expected Rust {expected:?} swept descriptor, got {descriptor:?}"
        ))
        .into()),
    }
}

fn require_ported_offset_face_surface(
    surface: Option<PortedFaceSurface>,
    label: &str,
) -> Result<PortedOffsetSurface, Box<dyn std::error::Error>> {
    let surface = surface.ok_or_else(|| {
        std::io::Error::other(format!("{label} missing Rust offset surface descriptor"))
    })?;
    match surface {
        PortedFaceSurface::Offset(surface) => Ok(surface),
        descriptor => Err(std::io::Error::other(format!(
            "{label} expected Rust Offset descriptor, got {descriptor:?}"
        ))
        .into()),
    }
}

fn assert_vec3_close(
    lhs: [f64; 3],
    rhs: [f64; 3],
    tolerance: f64,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let delta = [
        (lhs[0] - rhs[0]).abs(),
        (lhs[1] - rhs[1]).abs(),
        (lhs[2] - rhs[2]).abs(),
    ];
    if delta.iter().all(|value| *value <= tolerance) {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "{label} mismatch: lhs={lhs:?} rhs={rhs:?} delta={delta:?}"
        ))
        .into())
    }
}

fn assert_scalar_close(
    lhs: f64,
    rhs: f64,
    tolerance: f64,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let delta = (lhs - rhs).abs();
    if delta <= tolerance {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "{label} mismatch: lhs={lhs:?} rhs={rhs:?} delta={delta:?}"
        ))
        .into())
    }
}

fn assert_face_geometry_close(
    lhs: lean_occt::FaceGeometry,
    rhs: lean_occt::FaceGeometry,
    tolerance: f64,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if lhs.kind != rhs.kind
        || lhs.is_u_closed != rhs.is_u_closed
        || lhs.is_v_closed != rhs.is_v_closed
        || lhs.is_u_periodic != rhs.is_u_periodic
        || lhs.is_v_periodic != rhs.is_v_periodic
    {
        return Err(
            std::io::Error::other(format!("{label} mismatch: lhs={lhs:?} rhs={rhs:?}")).into(),
        );
    }

    assert_scalar_close(lhs.u_min, rhs.u_min, tolerance, &format!("{label} u_min"))?;
    assert_scalar_close(lhs.u_max, rhs.u_max, tolerance, &format!("{label} u_max"))?;
    assert_scalar_close(lhs.v_min, rhs.v_min, tolerance, &format!("{label} v_min"))?;
    assert_scalar_close(lhs.v_max, rhs.v_max, tolerance, &format!("{label} v_max"))?;
    assert_scalar_close(
        lhs.u_period,
        rhs.u_period,
        tolerance,
        &format!("{label} u_period"),
    )?;
    assert_scalar_close(
        lhs.v_period,
        rhs.v_period,
        tolerance,
        &format!("{label} v_period"),
    )?;
    Ok(())
}

fn assert_edge_geometry_close(
    lhs: lean_occt::EdgeGeometry,
    rhs: lean_occt::EdgeGeometry,
    tolerance: f64,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if lhs.kind != rhs.kind || lhs.is_closed != rhs.is_closed || lhs.is_periodic != rhs.is_periodic
    {
        return Err(
            std::io::Error::other(format!("{label} mismatch: lhs={lhs:?} rhs={rhs:?}")).into(),
        );
    }

    assert_scalar_close(
        lhs.start_parameter,
        rhs.start_parameter,
        tolerance,
        &format!("{label} start_parameter"),
    )?;
    assert_scalar_close(
        lhs.end_parameter,
        rhs.end_parameter,
        tolerance,
        &format!("{label} end_parameter"),
    )?;
    assert_scalar_close(
        lhs.period,
        rhs.period,
        tolerance,
        &format!("{label} period"),
    )?;
    Ok(())
}

fn assert_edge_geometry_span_close(
    lhs: lean_occt::EdgeGeometry,
    rhs: lean_occt::EdgeGeometry,
    tolerance: f64,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if lhs.kind != rhs.kind || lhs.is_closed != rhs.is_closed || lhs.is_periodic != rhs.is_periodic
    {
        return Err(
            std::io::Error::other(format!("{label} mismatch: lhs={lhs:?} rhs={rhs:?}")).into(),
        );
    }

    assert_scalar_close(
        (lhs.end_parameter - lhs.start_parameter).abs(),
        (rhs.end_parameter - rhs.start_parameter).abs(),
        tolerance,
        &format!("{label} parameter span"),
    )?;
    assert_scalar_close(
        lhs.period,
        rhs.period,
        tolerance,
        &format!("{label} period"),
    )?;
    Ok(())
}

fn assert_line_payload_close(
    lhs: lean_occt::LinePayload,
    rhs: lean_occt::LinePayload,
    tolerance: f64,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_vec3_close(
        lhs.origin,
        rhs.origin,
        tolerance,
        &format!("{label} origin"),
    )?;
    assert_vec3_close(
        lhs.direction,
        rhs.direction,
        tolerance,
        &format!("{label} direction"),
    )?;
    Ok(())
}

fn assert_circle_payload_close(
    lhs: lean_occt::CirclePayload,
    rhs: lean_occt::CirclePayload,
    tolerance: f64,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_vec3_close(
        lhs.center,
        rhs.center,
        tolerance,
        &format!("{label} center"),
    )?;
    assert_vec3_close(
        lhs.normal,
        rhs.normal,
        tolerance,
        &format!("{label} normal"),
    )?;
    assert_vec3_close(
        lhs.x_direction,
        rhs.x_direction,
        tolerance,
        &format!("{label} x_direction"),
    )?;
    assert_vec3_close(
        lhs.y_direction,
        rhs.y_direction,
        tolerance,
        &format!("{label} y_direction"),
    )?;
    assert_scalar_close(
        lhs.radius,
        rhs.radius,
        tolerance,
        &format!("{label} radius"),
    )?;
    Ok(())
}

fn assert_ellipse_payload_close(
    lhs: lean_occt::EllipsePayload,
    rhs: lean_occt::EllipsePayload,
    tolerance: f64,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_vec3_close(
        lhs.center,
        rhs.center,
        tolerance,
        &format!("{label} center"),
    )?;
    assert_vec3_close(
        lhs.normal,
        rhs.normal,
        tolerance,
        &format!("{label} normal"),
    )?;
    assert_vec3_close(
        lhs.x_direction,
        rhs.x_direction,
        tolerance,
        &format!("{label} x_direction"),
    )?;
    assert_vec3_close(
        lhs.y_direction,
        rhs.y_direction,
        tolerance,
        &format!("{label} y_direction"),
    )?;
    assert_scalar_close(
        lhs.major_radius,
        rhs.major_radius,
        tolerance,
        &format!("{label} major_radius"),
    )?;
    assert_scalar_close(
        lhs.minor_radius,
        rhs.minor_radius,
        tolerance,
        &format!("{label} minor_radius"),
    )?;
    Ok(())
}

fn assert_plane_payload_close(
    lhs: lean_occt::PlanePayload,
    rhs: lean_occt::PlanePayload,
    tolerance: f64,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_vec3_close(
        lhs.origin,
        rhs.origin,
        tolerance,
        &format!("{label} origin"),
    )?;
    assert_vec3_close(
        lhs.normal,
        rhs.normal,
        tolerance,
        &format!("{label} normal"),
    )?;
    assert_vec3_close(
        lhs.x_direction,
        rhs.x_direction,
        tolerance,
        &format!("{label} x_direction"),
    )?;
    assert_vec3_close(
        lhs.y_direction,
        rhs.y_direction,
        tolerance,
        &format!("{label} y_direction"),
    )?;
    Ok(())
}

fn assert_cylinder_payload_close(
    lhs: lean_occt::CylinderPayload,
    rhs: lean_occt::CylinderPayload,
    tolerance: f64,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_vec3_close(
        lhs.origin,
        rhs.origin,
        tolerance,
        &format!("{label} origin"),
    )?;
    assert_vec3_close(lhs.axis, rhs.axis, tolerance, &format!("{label} axis"))?;
    assert_vec3_close(
        lhs.x_direction,
        rhs.x_direction,
        tolerance,
        &format!("{label} x_direction"),
    )?;
    assert_vec3_close(
        lhs.y_direction,
        rhs.y_direction,
        tolerance,
        &format!("{label} y_direction"),
    )?;
    assert_scalar_close(
        lhs.radius,
        rhs.radius,
        tolerance,
        &format!("{label} radius"),
    )?;
    Ok(())
}

fn assert_cone_payload_close(
    lhs: lean_occt::ConePayload,
    rhs: lean_occt::ConePayload,
    tolerance: f64,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_vec3_close(
        lhs.origin,
        rhs.origin,
        tolerance,
        &format!("{label} origin"),
    )?;
    assert_vec3_close(lhs.axis, rhs.axis, tolerance, &format!("{label} axis"))?;
    assert_vec3_close(
        lhs.x_direction,
        rhs.x_direction,
        tolerance,
        &format!("{label} x_direction"),
    )?;
    assert_vec3_close(
        lhs.y_direction,
        rhs.y_direction,
        tolerance,
        &format!("{label} y_direction"),
    )?;
    assert_scalar_close(
        lhs.reference_radius,
        rhs.reference_radius,
        tolerance,
        &format!("{label} reference_radius"),
    )?;
    assert_scalar_close(
        lhs.semi_angle,
        rhs.semi_angle,
        tolerance,
        &format!("{label} semi_angle"),
    )?;
    Ok(())
}

fn assert_sphere_payload_close(
    lhs: lean_occt::SpherePayload,
    rhs: lean_occt::SpherePayload,
    tolerance: f64,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_vec3_close(
        lhs.center,
        rhs.center,
        tolerance,
        &format!("{label} center"),
    )?;
    assert_vec3_close(
        lhs.normal,
        rhs.normal,
        tolerance,
        &format!("{label} normal"),
    )?;
    assert_vec3_close(
        lhs.x_direction,
        rhs.x_direction,
        tolerance,
        &format!("{label} x_direction"),
    )?;
    assert_vec3_close(
        lhs.y_direction,
        rhs.y_direction,
        tolerance,
        &format!("{label} y_direction"),
    )?;
    assert_scalar_close(
        lhs.radius,
        rhs.radius,
        tolerance,
        &format!("{label} radius"),
    )?;
    Ok(())
}

fn assert_torus_payload_close(
    lhs: lean_occt::TorusPayload,
    rhs: lean_occt::TorusPayload,
    tolerance: f64,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_vec3_close(
        lhs.center,
        rhs.center,
        tolerance,
        &format!("{label} center"),
    )?;
    assert_vec3_close(lhs.axis, rhs.axis, tolerance, &format!("{label} axis"))?;
    assert_vec3_close(
        lhs.x_direction,
        rhs.x_direction,
        tolerance,
        &format!("{label} x_direction"),
    )?;
    assert_vec3_close(
        lhs.y_direction,
        rhs.y_direction,
        tolerance,
        &format!("{label} y_direction"),
    )?;
    assert_scalar_close(
        lhs.major_radius,
        rhs.major_radius,
        tolerance,
        &format!("{label} major_radius"),
    )?;
    assert_scalar_close(
        lhs.minor_radius,
        rhs.minor_radius,
        tolerance,
        &format!("{label} minor_radius"),
    )?;
    Ok(())
}

fn assert_revolution_payload_close(
    lhs: lean_occt::RevolutionSurfacePayload,
    rhs: lean_occt::RevolutionSurfacePayload,
    tolerance: f64,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_vec3_close(
        lhs.axis_origin,
        rhs.axis_origin,
        tolerance,
        &format!("{label} axis_origin"),
    )?;
    assert_vec3_close(
        lhs.axis_direction,
        rhs.axis_direction,
        tolerance,
        &format!("{label} axis_direction"),
    )?;
    assert_eq!(
        lhs.basis_curve_kind, rhs.basis_curve_kind,
        "{label} basis_curve_kind mismatch"
    );
    Ok(())
}

fn assert_extrusion_payload_close(
    lhs: lean_occt::ExtrusionSurfacePayload,
    rhs: lean_occt::ExtrusionSurfacePayload,
    tolerance: f64,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_vec3_close(
        lhs.direction,
        rhs.direction,
        tolerance,
        &format!("{label} direction"),
    )?;
    assert_eq!(
        lhs.basis_curve_kind, rhs.basis_curve_kind,
        "{label} basis_curve_kind mismatch"
    );
    Ok(())
}

fn assert_offset_payload_close(
    lhs: lean_occt::OffsetSurfacePayload,
    rhs: lean_occt::OffsetSurfacePayload,
    tolerance: f64,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_scalar_close(
        lhs.offset_value,
        rhs.offset_value,
        tolerance,
        &format!("{label} offset_value"),
    )?;
    assert_eq!(
        lhs.basis_surface_kind, rhs.basis_surface_kind,
        "{label} basis_surface_kind mismatch"
    );
    Ok(())
}

fn assert_ported_curve_close(
    lhs: PortedCurve,
    rhs: PortedCurve,
    tolerance: f64,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    match (lhs, rhs) {
        (PortedCurve::Line(lhs), PortedCurve::Line(rhs)) => {
            assert_line_payload_close(lhs, rhs, tolerance, label)
        }
        (PortedCurve::Circle(lhs), PortedCurve::Circle(rhs)) => {
            assert_circle_payload_close(lhs, rhs, tolerance, label)
        }
        (PortedCurve::Ellipse(lhs), PortedCurve::Ellipse(rhs)) => {
            assert_ellipse_payload_close(lhs, rhs, tolerance, label)
        }
        (lhs, rhs) => Err(std::io::Error::other(format!(
            "{label} curve kind mismatch: {lhs:?} vs {rhs:?}"
        ))
        .into()),
    }
}

fn assert_offset_swept_basis_curve_close(
    context: &lean_occt::Context,
    offset_face: &Shape,
    basis_curve: PortedCurve,
    basis_geometry: lean_occt::EdgeGeometry,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let basis_curve_geometry = context.face_offset_basis_curve_geometry(offset_face)?;
    let basis_curve_geometry_occt = context.face_offset_basis_curve_geometry_occt(offset_face)?;
    assert_edge_geometry_close(
        basis_curve_geometry,
        basis_curve_geometry_occt,
        1.0e-12,
        &format!("{label} basis curve occt geometry"),
    )?;
    assert_edge_geometry_close(
        basis_curve_geometry,
        basis_geometry,
        1.0e-12,
        &format!("{label} descriptor basis curve geometry"),
    )?;
    assert_eq!(basis_curve_geometry.kind, CurveKind::Ellipse);

    match basis_curve {
        PortedCurve::Ellipse(payload) => {
            let public_payload = context.face_offset_basis_curve_ellipse_payload(offset_face)?;
            let public_payload_occt =
                context.face_offset_basis_curve_ellipse_payload_occt(offset_face)?;
            assert_ellipse_payload_close(
                public_payload,
                payload,
                1.0e-12,
                &format!("{label} basis ellipse descriptor payload"),
            )?;
            assert_ellipse_payload_close(
                public_payload,
                public_payload_occt,
                1.0e-12,
                &format!("{label} basis ellipse occt payload"),
            )?;
            let error = context
                .face_offset_basis_curve_line_payload(offset_face)
                .expect_err(
                    "ellipse offset basis curve should reject line payload requests in Rust",
                );
            assert!(error.to_string().contains(
                "requested Line offset-basis curve payload for ported Ellipse offset basis curve"
            ));
            let error = context
                .face_offset_basis_curve_circle_payload(offset_face)
                .expect_err(
                    "ellipse offset basis curve should reject circle payload requests in Rust",
                );
            assert!(error.to_string().contains(
                "requested Circle offset-basis curve payload for ported Ellipse offset basis curve"
            ));
        }
        curve => {
            return Err(std::io::Error::other(format!(
                "unexpected {label} offset swept basis curve: {curve:?}"
            ))
            .into())
        }
    }

    Ok(())
}

fn assert_swept_offset_basis_mirrors_source(
    context: &lean_occt::Context,
    label: &str,
    basis_kind: SurfaceKind,
    source_face: &Shape,
    offset_face: &Shape,
    expected_offset: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    let offset_payload = context.face_offset_payload(offset_face)?;
    assert_eq!(offset_payload.basis_surface_kind, basis_kind);
    assert_scalar_close(
        offset_payload.offset_value,
        expected_offset,
        1.0e-12,
        &format!("{label} swept offset value"),
    )?;

    let source_geometry = context.face_geometry(source_face)?;
    let source_ported_geometry = context
        .ported_face_geometry(source_face)?
        .ok_or_else(|| std::io::Error::other(format!("{label} missing Rust basis geometry")))?;
    assert_eq!(source_ported_geometry.kind, basis_kind);
    assert_face_geometry_close(
        source_geometry,
        source_ported_geometry,
        1.0e-12,
        &format!("{label} swept source basis geometry"),
    )?;
    assert_face_geometry_close(
        context.face_offset_basis_geometry(offset_face)?,
        source_geometry,
        1.0e-12,
        &format!("{label} swept offset basis geometry"),
    )?;

    let source_surface = require_ported_swept_face_surface(
        context.ported_face_surface_descriptor(source_face)?,
        basis_kind,
        &format!("{label} source basis"),
    )?;
    let offset_surface = require_ported_offset_face_surface(
        context.ported_face_surface_descriptor(offset_face)?,
        &format!("{label} offset basis"),
    )?;
    assert_offset_payload_close(
        offset_surface.payload,
        offset_payload,
        1.0e-12,
        &format!("{label} offset descriptor payload"),
    )?;
    assert_face_geometry_close(
        offset_surface.basis_geometry,
        source_geometry,
        1.0e-12,
        &format!("{label} offset descriptor basis geometry"),
    )?;

    match (basis_kind, source_surface, offset_surface.basis) {
        (
            SurfaceKind::Extrusion,
            PortedSweptSurface::Extrusion {
                payload: source_payload,
                basis_curve: source_basis_curve,
                basis_geometry: source_basis_geometry,
            },
            PortedOffsetBasisSurface::Swept(PortedSweptSurface::Extrusion {
                payload,
                basis_curve,
                basis_geometry,
            }),
        ) => {
            assert_extrusion_payload_close(
                context.face_offset_basis_extrusion_payload(offset_face)?,
                context.face_extrusion_payload(source_face)?,
                1.0e-12,
                &format!("{label} public offset basis mirrors source payload"),
            )?;
            assert_extrusion_payload_close(
                payload,
                source_payload,
                1.0e-12,
                &format!("{label} descriptor swept payload mirrors source"),
            )?;
            let public_basis_geometry = context.face_offset_basis_curve_geometry(offset_face)?;
            assert_edge_geometry_close(
                public_basis_geometry,
                basis_geometry,
                1.0e-12,
                &format!("{label} public offset basis curve geometry matches descriptor"),
            )?;
            assert_edge_geometry_span_close(
                public_basis_geometry,
                source_basis_geometry,
                1.0e-12,
                &format!("{label} offset basis curve span mirrors source"),
            )?;
            assert_edge_geometry_close(
                basis_geometry,
                public_basis_geometry,
                1.0e-12,
                &format!("{label} descriptor basis curve geometry matches public query"),
            )?;
            assert_ported_curve_close(
                basis_curve,
                source_basis_curve,
                1.0e-12,
                &format!("{label} descriptor basis curve mirrors source"),
            )?;
        }
        (
            SurfaceKind::Revolution,
            PortedSweptSurface::Revolution {
                payload: source_payload,
                basis_curve: source_basis_curve,
                basis_geometry: source_basis_geometry,
            },
            PortedOffsetBasisSurface::Swept(PortedSweptSurface::Revolution {
                payload,
                basis_curve,
                basis_geometry,
            }),
        ) => {
            assert_revolution_payload_close(
                context.face_offset_basis_revolution_payload(offset_face)?,
                context.face_revolution_payload(source_face)?,
                1.0e-12,
                &format!("{label} public offset basis mirrors source payload"),
            )?;
            assert_revolution_payload_close(
                payload,
                source_payload,
                1.0e-12,
                &format!("{label} descriptor swept payload mirrors source"),
            )?;
            let public_basis_geometry = context.face_offset_basis_curve_geometry(offset_face)?;
            assert_edge_geometry_close(
                public_basis_geometry,
                basis_geometry,
                1.0e-12,
                &format!("{label} public offset basis curve geometry matches descriptor"),
            )?;
            assert_edge_geometry_span_close(
                public_basis_geometry,
                source_basis_geometry,
                1.0e-12,
                &format!("{label} offset basis curve span mirrors source"),
            )?;
            assert_edge_geometry_close(
                basis_geometry,
                public_basis_geometry,
                1.0e-12,
                &format!("{label} descriptor basis curve geometry matches public query"),
            )?;
            assert_ported_curve_close(
                basis_curve,
                source_basis_curve,
                1.0e-12,
                &format!("{label} descriptor basis curve mirrors source"),
            )?;
        }
        (expected, source, basis) => {
            return Err(std::io::Error::other(format!(
                "unexpected {label} swept offset metadata: expected {expected:?}, source {source:?}, basis {basis:?}"
            ))
            .into())
        }
    }

    Ok(())
}

fn assert_analytic_offset_basis_rejects_curve_queries(
    context: &lean_occt::Context,
    offset_face: &Shape,
    basis_kind: SurfaceKind,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let error = context
        .face_offset_basis_curve_geometry(offset_face)
        .expect_err("analytic offset basis should reject curve geometry requests in Rust");
    assert!(
        error.to_string().contains(&format!(
            "requested offset-basis curve geometry for ported {basis_kind:?} offset basis"
        )),
        "{label} unexpected curve geometry error: {error}"
    );

    let error = context
        .face_offset_basis_curve_ellipse_payload(offset_face)
        .expect_err("analytic offset basis should reject curve payload requests in Rust");
    assert!(
        error.to_string().contains(&format!(
            "requested Ellipse offset-basis curve payload for ported {basis_kind:?} offset basis"
        )),
        "{label} unexpected curve payload error: {error}"
    );

    Ok(())
}

fn normalized_uv_to_uv(geometry: lean_occt::FaceGeometry, uv_t: [f64; 2]) -> [f64; 2] {
    [
        geometry.u_min + (geometry.u_max - geometry.u_min) * uv_t[0],
        geometry.v_min + (geometry.v_max - geometry.v_min) * uv_t[1],
    ]
}

fn simpson_integral(start: f64, end: f64, steps: usize, f: impl Fn(f64) -> f64) -> f64 {
    let steps = if steps % 2 == 0 { steps } else { steps + 1 };
    let h = (end - start) / steps as f64;
    let mut sum = f(start) + f(end);
    for step in 1..steps {
        let x = start + step as f64 * h;
        let weight = if step % 2 == 0 { 2.0 } else { 4.0 };
        sum += weight * f(x);
    }
    sum * h / 3.0
}

fn ellipse_perimeter(major_radius: f64, minor_radius: f64) -> f64 {
    simpson_integral(0.0, 2.0 * PI, 4096, |parameter| {
        ((major_radius * parameter.sin()).powi(2) + (minor_radius * parameter.cos()).powi(2)).sqrt()
    })
}

fn revolved_ellipse_area(
    center_x: f64,
    major_radius: f64,
    minor_radius: f64,
    sweep_angle: f64,
) -> f64 {
    sweep_angle.abs()
        * simpson_integral(0.0, 2.0 * PI, 4096, |parameter| {
            let radius_to_axis = center_x + major_radius * parameter.cos();
            let speed = ((major_radius * parameter.sin()).powi(2)
                + (minor_radius * parameter.cos()).powi(2))
            .sqrt();
            radius_to_axis.abs() * speed
        })
}

#[test]
fn public_root_edge_endpoints_are_topology_backed() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let kernel = ModelKernel::new()?;
    let cut = kernel.box_with_through_hole(default_cut())?;
    let ellipse_edge = kernel.make_ellipse_edge(EllipseEdgeParams {
        origin: [30.0, 0.0, 0.0],
        axis: [0.0, 1.0, 0.0],
        x_direction: [1.0, 0.0, 0.0],
        major_radius: 10.0,
        minor_radius: 6.0,
    })?;

    for (label, expected_kind, edge) in [
        (
            "line",
            CurveKind::Line,
            find_first_edge_by_kind(&kernel, &cut, CurveKind::Line)?,
        ),
        (
            "circle",
            CurveKind::Circle,
            find_first_edge_by_kind(&kernel, &cut, CurveKind::Circle)?,
        ),
        ("ellipse", CurveKind::Ellipse, ellipse_edge),
    ] {
        let summary = kernel.context().describe_shape_occt(&edge)?;
        assert_eq!(
            summary.root_kind,
            ShapeKind::Edge,
            "{label} fixture should be a root edge"
        );
        assert_eq!(
            kernel.context().edge_geometry(&edge)?.kind,
            expected_kind,
            "{label} fixture geometry kind changed"
        );

        let topology = kernel
            .context()
            .ported_topology(&edge)?
            .ok_or_else(|| std::io::Error::other(format!("{label} missing ported topology")))?;
        let [topology_edge] = topology.edges.as_slice() else {
            return Err(std::io::Error::other(format!(
                "{label} expected exactly one topology edge, found {}",
                topology.edges.len()
            ))
            .into());
        };
        let start_index = topology_edge.start_vertex.ok_or_else(|| {
            std::io::Error::other(format!("{label} missing topology start vertex"))
        })?;
        let end_index = topology_edge
            .end_vertex
            .ok_or_else(|| std::io::Error::other(format!("{label} missing topology end vertex")))?;
        let topology_start = topology
            .vertex_positions
            .get(start_index)
            .copied()
            .ok_or_else(|| std::io::Error::other(format!("{label} bad start vertex index")))?;
        let topology_end = topology
            .vertex_positions
            .get(end_index)
            .copied()
            .ok_or_else(|| std::io::Error::other(format!("{label} bad end vertex index")))?;
        let vertex_shapes = kernel.context().subshapes(&edge, ShapeKind::Vertex)?;
        assert_eq!(
            vertex_shapes.len(),
            topology.vertex_positions.len(),
            "{label} public vertex inventory should be topology-backed"
        );
        for (vertex_index, (vertex_shape, topology_position)) in vertex_shapes
            .iter()
            .zip(topology.vertex_positions.iter())
            .enumerate()
        {
            let vertex_point = kernel.context().vertex_point(vertex_shape)?;
            assert_vec3_close(
                vertex_point,
                *topology_position,
                1.0e-12,
                &format!("{label} vertex {vertex_index} point/topology"),
            )?;
        }

        let ported_endpoints = kernel
            .context()
            .ported_edge_endpoints(&edge)?
            .ok_or_else(|| std::io::Error::other(format!("{label} missing ported endpoints")))?;
        let public_endpoints = kernel.context().edge_endpoints(&edge)?;
        let occt_endpoints = kernel.context().edge_endpoints_occt(&edge)?;
        for (actual, expected, description) in [
            (
                ported_endpoints.start,
                topology_start,
                "ported/topology start",
            ),
            (ported_endpoints.end, topology_end, "ported/topology end"),
            (
                public_endpoints.start,
                topology_start,
                "public/topology start",
            ),
            (public_endpoints.end, topology_end, "public/topology end"),
            (occt_endpoints.start, topology_start, "occt/topology start"),
            (occt_endpoints.end, topology_end, "occt/topology end"),
        ] {
            assert_vec3_close(actual, expected, 1.0e-12, &format!("{label} {description}"))?;
        }
    }

    Ok(())
}

#[test]
fn root_edge_endpoints_and_topology_use_ported_seed() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let kernel = ModelKernel::new()?;
    let cut = kernel.box_with_through_hole(default_cut())?;
    let ellipse_edge = kernel.make_ellipse_edge(EllipseEdgeParams {
        origin: [30.0, 0.0, 0.0],
        axis: [0.0, 1.0, 0.0],
        x_direction: [1.0, 0.0, 0.0],
        major_radius: 10.0,
        minor_radius: 6.0,
    })?;

    for (label, expected_kind, edge) in [
        (
            "line",
            CurveKind::Line,
            find_first_edge_by_kind(&kernel, &cut, CurveKind::Line)?,
        ),
        (
            "circle",
            CurveKind::Circle,
            find_first_edge_by_kind(&kernel, &cut, CurveKind::Circle)?,
        ),
        ("ellipse", CurveKind::Ellipse, ellipse_edge),
    ] {
        let summary = kernel.context().describe_shape_occt(&edge)?;
        assert_eq!(
            summary.root_kind,
            ShapeKind::Edge,
            "{label} fixture should enter the root edge endpoint seed"
        );

        let geometry = kernel.context().edge_geometry(&edge)?;
        assert_eq!(
            geometry.kind, expected_kind,
            "{label} fixture geometry kind changed"
        );
        let ported_geometry = kernel
            .context()
            .ported_edge_geometry(&edge)?
            .ok_or_else(|| std::io::Error::other(format!("{label} missing ported geometry")))?;
        assert_edge_geometry_close(
            geometry,
            ported_geometry,
            1.0e-12,
            &format!("{label} public/ported root geometry"),
        )?;
        assert_eq!(
            kernel.context().edge_geometry_occt(&edge)?.kind,
            expected_kind,
            "{label} raw oracle geometry kind changed"
        );
        let public_endpoints = kernel.context().edge_endpoints(&edge)?;
        let ported_endpoints = kernel
            .context()
            .ported_edge_endpoints(&edge)?
            .ok_or_else(|| std::io::Error::other(format!("{label} missing ported endpoints")))?;
        let occt_endpoints = kernel.context().edge_endpoints_occt(&edge)?;
        assert_vec3_close(
            public_endpoints.start,
            ported_endpoints.start,
            1.0e-12,
            &format!("{label} public/ported start"),
        )?;
        assert_vec3_close(
            public_endpoints.end,
            ported_endpoints.end,
            1.0e-12,
            &format!("{label} public/ported end"),
        )?;
        assert_vec3_close(
            public_endpoints.start,
            occt_endpoints.start,
            1.0e-12,
            &format!("{label} public/occt start"),
        )?;
        assert_vec3_close(
            public_endpoints.end,
            occt_endpoints.end,
            1.0e-12,
            &format!("{label} public/occt end"),
        )?;

        let topology = kernel.context().topology(&edge)?;
        assert_eq!(topology.edges.len(), 1, "{label} root edge topology");
        let topology_edge = topology
            .edges
            .first()
            .ok_or_else(|| std::io::Error::other(format!("{label} missing topology edge")))?;
        let ported_length = kernel
            .context()
            .ported_edge_length(&edge)?
            .ok_or_else(|| std::io::Error::other(format!("{label} missing ported length")))?;
        assert_scalar_close(
            topology_edge.length,
            ported_length,
            1.0e-10,
            &format!("{label} topology length"),
        )?;
        if expected_kind == CurveKind::Line {
            let delta = [
                public_endpoints.end[0] - public_endpoints.start[0],
                public_endpoints.end[1] - public_endpoints.start[1],
                public_endpoints.end[2] - public_endpoints.start[2],
            ];
            let endpoint_length =
                (delta[0] * delta[0] + delta[1] * delta[1] + delta[2] * delta[2]).sqrt();
            assert_scalar_close(
                topology_edge.length,
                endpoint_length,
                1.0e-12,
                &format!("{label} topology endpoint length"),
            )?;
        }
        if label == "ellipse" {
            assert_scalar_close(
                topology_edge.length,
                ellipse_perimeter(10.0, 6.0),
                1.0e-3,
                "ellipse topology analytic length",
            )?;
        }
        let start_index = topology_edge.start_vertex.ok_or_else(|| {
            std::io::Error::other(format!("{label} missing topology start vertex"))
        })?;
        let end_index = topology_edge
            .end_vertex
            .ok_or_else(|| std::io::Error::other(format!("{label} missing topology end vertex")))?;
        let topology_start = topology
            .vertex_positions
            .get(start_index)
            .copied()
            .ok_or_else(|| std::io::Error::other(format!("{label} bad start vertex index")))?;
        let topology_end = topology
            .vertex_positions
            .get(end_index)
            .copied()
            .ok_or_else(|| std::io::Error::other(format!("{label} bad end vertex index")))?;
        assert_vec3_close(
            topology_start,
            public_endpoints.start,
            1.0e-12,
            &format!("{label} topology start"),
        )?;
        assert_vec3_close(
            topology_end,
            public_endpoints.end,
            1.0e-12,
            &format!("{label} topology end"),
        )?;

        assert_eq!(
            kernel.context().subshape_count(&edge, ShapeKind::Edge)?,
            topology.edges.len(),
            "{label} public edge count should come from ported root topology"
        );
        assert_eq!(
            kernel.context().subshape_count(&edge, ShapeKind::Vertex)?,
            topology.vertex_positions.len(),
            "{label} public vertex count should come from ported root topology"
        );
        let edge_shapes = kernel.context().subshapes(&edge, ShapeKind::Edge)?;
        assert_eq!(
            edge_shapes.len(),
            1,
            "{label} public edge handles should come from ported root topology"
        );
        assert_edge_geometry_close(
            kernel.context().edge_geometry(&edge_shapes[0])?,
            geometry,
            1.0e-12,
            &format!("{label} topology edge handle/public geometry"),
        )?;
        assert_eq!(
            kernel.context().edge_geometry(&edge_shapes[0])?.kind,
            expected_kind,
            "{label} ported root edge handle geometry"
        );
        let vertex_shapes = kernel.context().subshapes(&edge, ShapeKind::Vertex)?;
        assert_eq!(
            vertex_shapes.len(),
            topology.vertex_positions.len(),
            "{label} public vertex handles should come from ported root topology"
        );
        for (vertex_index, vertex_shape) in vertex_shapes.iter().enumerate() {
            assert_vec3_close(
                kernel.context().vertex_point(vertex_shape)?,
                topology.vertex_positions[vertex_index],
                1.0e-12,
                &format!("{label} ported root vertex handle {vertex_index}"),
            )?;
        }
        for empty_kind in [ShapeKind::Wire, ShapeKind::Face, ShapeKind::Shell] {
            assert_eq!(
                kernel.context().subshapes(&edge, empty_kind)?.len(),
                0,
                "{label} public {empty_kind:?} handles should be empty from ported root topology"
            );
        }
        assert_eq!(
            kernel
                .context()
                .subshape_count_occt(&edge, ShapeKind::Vertex)?,
            topology.vertex_positions.len(),
            "{label} ported root topology vertex count should match OCCT"
        );
    }

    Ok(())
}

#[test]
fn unsupported_root_edge_does_not_use_generic_raw_topology_inventory(
) -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let kernel = ModelKernel::new()?;
    let helix = kernel.make_helix(HelixParams {
        origin: [0.0, 0.0, 0.0],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        radius: 8.0,
        height: 24.0,
        pitch: 6.0,
    })?;
    let edge = kernel.context().subshape_occt(&helix, ShapeKind::Edge, 0)?;
    let summary = kernel.context().describe_shape_occt(&edge)?;
    assert_eq!(
        summary.root_kind,
        ShapeKind::Edge,
        "helix child should be tested as a root edge"
    );

    let geometry = kernel.context().edge_geometry_occt(&edge)?;
    assert!(
        !matches!(
            geometry.kind,
            CurveKind::Line | CurveKind::Circle | CurveKind::Ellipse
        ),
        "helix edge fixture should exercise unsupported root-edge classification, got {:?}",
        geometry.kind
    );
    assert!(
        kernel.context().ported_topology(&edge)?.is_none(),
        "unsupported root edge must not fall through to the generic raw topology inventory"
    );
    assert!(
        kernel.context().ported_edge_geometry(&edge)?.is_none(),
        "unsupported root edge geometry must stay outside the ported geometry path"
    );
    assert!(
        kernel.context().ported_edge_endpoints(&edge)?.is_none(),
        "unsupported root edge endpoints must stay outside the ported endpoint path"
    );
    assert_edge_geometry_close(
        kernel.context().edge_geometry(&edge)?,
        geometry,
        1.0e-12,
        "unsupported root edge public/raw geometry",
    )?;

    let public_endpoints = kernel.context().edge_endpoints(&edge)?;
    let occt_endpoints = kernel.context().edge_endpoints_occt(&edge)?;
    assert_vec3_close(
        public_endpoints.start,
        occt_endpoints.start,
        1.0e-12,
        "unsupported root edge public/occt start",
    )?;
    assert_vec3_close(
        public_endpoints.end,
        occt_endpoints.end,
        1.0e-12,
        "unsupported root edge public/occt end",
    )?;

    let public_sample = kernel.context().edge_sample(&edge, 0.5)?;
    let occt_sample = kernel.context().edge_sample_occt(&edge, 0.5)?;
    assert_vec3_close(
        public_sample.position,
        occt_sample.position,
        1.0e-12,
        "unsupported root edge public/occt sampled position",
    )?;
    assert_vec3_close(
        public_sample.tangent,
        occt_sample.tangent,
        1.0e-12,
        "unsupported root edge public/occt sampled tangent",
    )?;

    let parameter = 0.5 * (geometry.start_parameter + geometry.end_parameter);
    let public_parameter_sample = kernel
        .context()
        .edge_sample_at_parameter(&edge, parameter)?;
    let occt_parameter_sample = kernel
        .context()
        .edge_sample_at_parameter_occt(&edge, parameter)?;
    assert_vec3_close(
        public_parameter_sample.position,
        occt_parameter_sample.position,
        1.0e-12,
        "unsupported root edge public/occt parameter-sampled position",
    )?;
    assert_vec3_close(
        public_parameter_sample.tangent,
        occt_parameter_sample.tangent,
        1.0e-12,
        "unsupported root edge public/occt parameter-sampled tangent",
    )?;

    Ok(())
}

#[test]
fn ported_edge_geometry_returns_none_for_non_edge_shapes() -> Result<(), Box<dyn std::error::Error>>
{
    let _guard = support::test_guard();
    let kernel = ModelKernel::new()?;
    let cut = kernel.box_with_through_hole(default_cut())?;
    let face = find_first_face_by_kind(&kernel, &cut, SurfaceKind::Plane)?;
    let summary = kernel.context().describe_shape_occt(&face)?;
    assert_ne!(
        summary.root_kind,
        ShapeKind::Edge,
        "fixture should exercise a non-edge ported geometry query"
    );

    assert!(
        kernel.context().ported_edge_geometry(&face)?.is_none(),
        "ported edge geometry must return None instead of entering a raw edge classifier"
    );
    assert!(
        kernel.context().edge_geometry(&face).is_err(),
        "public edge geometry should still reject non-edge shapes through the explicit raw API"
    );

    Ok(())
}

#[test]
fn ported_vertex_points_match_occt() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let kernel = ModelKernel::new()?;
    let cut = kernel.box_with_through_hole(default_cut())?;

    for (index, vertex) in kernel
        .context()
        .subshapes(&cut, ShapeKind::Vertex)?
        .into_iter()
        .enumerate()
    {
        let context_point = kernel.context().vertex_point(&vertex)?;
        let ported_point = kernel
            .context()
            .ported_vertex_point(&vertex)?
            .ok_or_else(|| std::io::Error::other("expected Rust-owned vertex point"))?;
        let occt_point = kernel.context().vertex_point_occt(&vertex)?;
        assert_vec3_close(
            context_point,
            ported_point,
            1.0e-12,
            &format!("vertex {index} ported"),
        )?;
        assert_vec3_close(
            context_point,
            occt_point,
            1.0e-12,
            &format!("vertex {index}"),
        )?;
        let topology = kernel.context().topology(&vertex)?;
        assert_eq!(
            topology.vertex_positions.len(),
            1,
            "vertex {index} public topology should contain the root vertex"
        );
        assert_vec3_close(
            topology.vertex_positions[0],
            context_point,
            1.0e-12,
            &format!("vertex {index} topology point"),
        )?;
        assert!(
            topology.edges.is_empty()
                && topology.edge_faces.is_empty()
                && topology.edge_face_indices.is_empty()
                && topology.wires.is_empty()
                && topology.wire_edge_indices.is_empty()
                && topology.wire_edge_orientations.is_empty()
                && topology.wire_vertices.is_empty()
                && topology.wire_vertex_indices.is_empty()
                && topology.faces.is_empty()
                && topology.face_wire_indices.is_empty()
                && topology.face_wire_orientations.is_empty()
                && topology.face_wire_roles.is_empty(),
            "vertex {index} public topology should have no edge/wire/face inventory"
        );
        assert_eq!(
            kernel
                .context()
                .subshape_count(&vertex, ShapeKind::Vertex)?,
            1,
            "vertex {index} public vertex count should come from root topology"
        );
        let public_vertices = kernel.context().subshapes(&vertex, ShapeKind::Vertex)?;
        assert_eq!(
            public_vertices.len(),
            1,
            "vertex {index} public vertex handle should come from root topology"
        );
        let indexed_vertex = kernel.context().subshape(&vertex, ShapeKind::Vertex, 0)?;
        for (label, public_vertex) in [
            ("subshapes", &public_vertices[0]),
            ("subshape", &indexed_vertex),
        ] {
            assert_vec3_close(
                kernel.context().vertex_point(public_vertex)?,
                context_point,
                1.0e-12,
                &format!("vertex {index} {label} handle point"),
            )?;
        }
        for empty_kind in [
            ShapeKind::Edge,
            ShapeKind::Wire,
            ShapeKind::Face,
            ShapeKind::Shell,
        ] {
            assert_eq!(
                kernel.context().subshape_count(&vertex, empty_kind)?,
                0,
                "vertex {index} public {empty_kind:?} count should be empty from root topology"
            );
            assert_eq!(
                kernel.context().subshapes(&vertex, empty_kind)?.len(),
                0,
                "vertex {index} public {empty_kind:?} handles should be empty from root topology"
            );
        }
    }

    Ok(())
}

#[test]
fn ported_curve_sampling_matches_occt() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let kernel = ModelKernel::new()?;
    let cut = kernel.box_with_through_hole(default_cut())?;
    let cut_step =
        support::export_kernel_shape(&kernel, &cut, "ported_geometry_workflows", "ported_cut")?;

    let ellipse_edge = kernel.make_ellipse_edge(EllipseEdgeParams {
        origin: [30.0, 0.0, 0.0],
        axis: [0.0, 1.0, 0.0],
        x_direction: [1.0, 0.0, 0.0],
        major_radius: 10.0,
        minor_radius: 6.0,
    })?;
    let ellipse_step = support::export_kernel_shape(
        &kernel,
        &ellipse_edge,
        "ported_geometry_workflows",
        "ported_ellipse",
    )?;

    for (label, edge) in [
        (
            "line",
            find_first_edge_by_kind(&kernel, &cut, CurveKind::Line)?,
        ),
        (
            "circle",
            find_first_edge_by_kind(&kernel, &cut, CurveKind::Circle)?,
        ),
        ("ellipse", ellipse_edge),
    ] {
        let geometry = kernel.context().edge_geometry(&edge)?;
        let summary = kernel.context().describe_shape_occt(&edge)?;
        let root_edge = summary.root_kind == ShapeKind::Edge;
        let geometry_occt = kernel.context().edge_geometry_occt(&edge)?;
        let context_endpoints = kernel.context().edge_endpoints(&edge)?;
        let occt_endpoints = kernel.context().edge_endpoints_occt(&edge)?;
        let ported_geometry = kernel
            .context()
            .ported_edge_geometry(&edge)?
            .ok_or_else(|| std::io::Error::other(format!("expected Rust {label} geometry")))?;
        let ported_endpoints = kernel
            .context()
            .ported_edge_endpoints(&edge)?
            .ok_or_else(|| std::io::Error::other(format!("expected Rust {label} endpoints")))?;
        let parameter = 0.5 * (geometry.start_parameter + geometry.end_parameter);
        let ported = kernel
            .context()
            .ported_edge_curve(&edge)?
            .ok_or_else(|| std::io::Error::other(format!("expected ported {label} curve")))?;
        let rust_sample = ported.sample_with_geometry(geometry, parameter);
        let rust_sample_via_context = kernel
            .context()
            .ported_edge_sample_at_parameter(&edge, parameter)?
            .ok_or_else(|| std::io::Error::other(format!("expected ported {label} edge sample")))?;
        let context_sample = kernel
            .context()
            .edge_sample_at_parameter(&edge, parameter)?;
        let rust_normalized_sample = kernel
            .context()
            .ported_edge_sample(&edge, 0.5)?
            .ok_or_else(|| {
                std::io::Error::other(format!("expected ported {label} normalized edge sample"))
            })?;
        let context_normalized_sample = kernel.context().edge_sample(&edge, 0.5)?;
        let rust_length = kernel
            .context()
            .ported_edge_length(&edge)?
            .ok_or_else(|| std::io::Error::other(format!("expected ported {label} edge length")))?;
        let occt_sample = if root_edge {
            kernel.context().edge_sample_occt(&edge, 0.5)?
        } else {
            kernel
                .context()
                .edge_sample_at_parameter_occt(&edge, parameter)?
        };
        let occt_normalized_sample = kernel.context().edge_sample_occt(&edge, 0.5)?;
        let occt_length = summary.linear_length;

        match ported {
            PortedCurve::Line(payload) => {
                let public_payload = kernel.context().edge_line_payload(&edge)?;
                assert_line_payload_close(
                    public_payload,
                    payload,
                    1.0e-12,
                    &format!("{label} public payload"),
                )?;
                let error = kernel
                    .context()
                    .edge_circle_payload(&edge)
                    .expect_err("line edge should reject circle payload requests in Rust");
                assert!(error
                    .to_string()
                    .contains("requested Circle payload for ported Line edge"));
            }
            PortedCurve::Circle(payload) => {
                let public_payload = kernel.context().edge_circle_payload(&edge)?;
                assert_circle_payload_close(
                    public_payload,
                    payload,
                    1.0e-12,
                    &format!("{label} public payload"),
                )?;
                let error = kernel
                    .context()
                    .edge_line_payload(&edge)
                    .expect_err("circle edge should reject line payload requests in Rust");
                assert!(error
                    .to_string()
                    .contains("requested Line payload for ported Circle edge"));
            }
            PortedCurve::Ellipse(payload) => {
                let public_payload = kernel.context().edge_ellipse_payload(&edge)?;
                assert_ellipse_payload_close(
                    public_payload,
                    payload,
                    1.0e-12,
                    &format!("{label} public payload"),
                )?;
            }
        }

        assert_edge_geometry_close(geometry, ported_geometry, 1.0e-12, label)?;
        if root_edge {
            assert_eq!(
                geometry.kind, geometry_occt.kind,
                "{label} raw geometry kind"
            );
        } else {
            assert_edge_geometry_close(geometry, geometry_occt, 1.0e-8, label)?;
        }
        assert_vec3_close(
            context_endpoints.start,
            ported_endpoints.start,
            1.0e-12,
            &format!("{label} ported start endpoint"),
        )?;
        assert_vec3_close(
            context_endpoints.end,
            ported_endpoints.end,
            1.0e-12,
            &format!("{label} ported end endpoint"),
        )?;
        assert_vec3_close(
            context_endpoints.start,
            occt_endpoints.start,
            1.0e-12,
            &format!("{label} start endpoint"),
        )?;
        assert_vec3_close(
            context_endpoints.end,
            occt_endpoints.end,
            1.0e-12,
            &format!("{label} end endpoint"),
        )?;
        assert_vec3_close(rust_sample.position, occt_sample.position, 1.0e-8, label)?;
        assert_vec3_close(rust_sample.tangent, occt_sample.tangent, 1.0e-8, label)?;
        assert_vec3_close(
            rust_sample_via_context.position,
            occt_sample.position,
            1.0e-8,
            label,
        )?;
        assert_vec3_close(
            rust_sample_via_context.tangent,
            occt_sample.tangent,
            1.0e-8,
            label,
        )?;
        assert_vec3_close(
            context_sample.position,
            rust_sample_via_context.position,
            1.0e-12,
            label,
        )?;
        assert_vec3_close(
            context_sample.tangent,
            rust_sample_via_context.tangent,
            1.0e-12,
            label,
        )?;
        assert_vec3_close(
            rust_normalized_sample.position,
            occt_normalized_sample.position,
            1.0e-8,
            label,
        )?;
        assert_vec3_close(
            rust_normalized_sample.tangent,
            occt_normalized_sample.tangent,
            1.0e-8,
            label,
        )?;
        assert_vec3_close(
            context_normalized_sample.position,
            rust_normalized_sample.position,
            1.0e-12,
            label,
        )?;
        assert_vec3_close(
            context_normalized_sample.tangent,
            rust_normalized_sample.tangent,
            1.0e-12,
            label,
        )?;
        for (parameter_label, parameter) in [
            ("start", geometry.start_parameter),
            ("end", geometry.end_parameter),
        ] {
            let rust_parameter_sample = ported.sample_with_geometry(geometry, parameter);
            let occt_parameter_sample = if root_edge {
                kernel
                    .context()
                    .edge_sample_occt(&edge, if parameter_label == "start" { 0.0 } else { 1.0 })?
            } else {
                kernel
                    .context()
                    .edge_sample_at_parameter_occt(&edge, parameter)?
            };
            assert_vec3_close(
                rust_parameter_sample.position,
                occt_parameter_sample.position,
                1.0e-8,
                &format!("{label} {parameter_label} parameter position"),
            )?;
            assert_vec3_close(
                rust_parameter_sample.tangent,
                occt_parameter_sample.tangent,
                1.0e-8,
                &format!("{label} {parameter_label} parameter tangent"),
            )?;
        }
        let length_tolerance = if label == "ellipse" { 5.0e-2 } else { 1.0e-7 };
        assert!(
            (rust_length - occt_length).abs() <= length_tolerance,
            "{label} length mismatch: rust={rust_length} occt={occt_length}"
        );
    }

    assert!(cut_step.is_file());
    assert!(ellipse_step.is_file());
    Ok(())
}

#[test]
fn ported_surface_sampling_matches_occt() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let kernel = ModelKernel::new()?;

    let cut = kernel.box_with_through_hole(default_cut())?;
    let cut_step = support::export_kernel_shape(
        &kernel,
        &cut,
        "ported_geometry_workflows",
        "ported_surface_cut",
    )?;
    let cone = kernel.make_cone(ConeParams {
        origin: [0.0, 0.0, 0.0],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        base_radius: 15.0,
        top_radius: 5.0,
        height: 30.0,
    })?;
    let sphere = kernel.make_sphere(SphereParams {
        origin: [0.0, 0.0, 0.0],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        radius: 14.0,
    })?;
    let torus = kernel.make_torus(TorusParams {
        origin: [0.0, 0.0, 0.0],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        major_radius: 25.0,
        minor_radius: 6.0,
    })?;

    let cone_step =
        support::export_kernel_shape(&kernel, &cone, "ported_geometry_workflows", "ported_cone")?;
    let sphere_step = support::export_kernel_shape(
        &kernel,
        &sphere,
        "ported_geometry_workflows",
        "ported_sphere",
    )?;
    let torus_step =
        support::export_kernel_shape(&kernel, &torus, "ported_geometry_workflows", "ported_torus")?;

    for (label, face) in [
        (
            "plane",
            find_first_face_by_kind(&kernel, &cut, SurfaceKind::Plane)?,
        ),
        (
            "cylinder",
            find_first_face_by_kind(&kernel, &cut, SurfaceKind::Cylinder)?,
        ),
        (
            "cone",
            find_first_face_by_kind(&kernel, &cone, SurfaceKind::Cone)?,
        ),
        (
            "sphere",
            find_first_face_by_kind(&kernel, &sphere, SurfaceKind::Sphere)?,
        ),
        (
            "torus",
            find_first_face_by_kind(&kernel, &torus, SurfaceKind::Torus)?,
        ),
    ] {
        let geometry = kernel.context().face_geometry(&face)?;
        let geometry_occt = kernel.context().face_geometry_occt(&face)?;
        let ported_geometry = kernel
            .context()
            .ported_face_geometry(&face)?
            .ok_or_else(|| std::io::Error::other(format!("expected Rust {label} geometry")))?;
        let context_bounds = kernel.context().face_uv_bounds(&face)?;
        let occt_bounds = kernel.context().face_uv_bounds_occt(&face)?;
        let uv = geometry.center_uv();
        let rust_sample = kernel
            .context()
            .ported_face_sample_normalized(&face, [0.5, 0.5])?
            .ok_or_else(|| std::io::Error::other(format!("expected ported {label} surface")))?;
        let context_sample = kernel.context().face_sample_normalized(&face, [0.5, 0.5])?;
        let rust_uv_sample = kernel
            .context()
            .ported_face_sample(&face, uv)?
            .ok_or_else(|| std::io::Error::other(format!("expected ported {label} UV sample")))?;
        let context_uv_sample = kernel.context().face_sample(&face, uv)?;
        let occt_sample = kernel
            .context()
            .face_sample_normalized_occt(&face, [0.5, 0.5])?;
        let occt_uv_sample = kernel.context().face_sample_occt(&face, uv)?;
        let ported_surface = kernel
            .context()
            .ported_face_surface(&face)?
            .ok_or_else(|| std::io::Error::other(format!("expected ported {label} payload")))?;

        match ported_surface {
            PortedSurface::Plane(payload) => {
                let public_payload = kernel.context().face_plane_payload(&face)?;
                assert_plane_payload_close(
                    public_payload,
                    payload,
                    1.0e-12,
                    &format!("{label} public payload"),
                )?;
                let error = kernel
                    .context()
                    .face_cylinder_payload(&face)
                    .expect_err("plane face should reject cylinder payload requests in Rust");
                assert!(error
                    .to_string()
                    .contains("requested Cylinder payload for ported Plane face"));
            }
            PortedSurface::Cylinder(payload) => {
                let public_payload = kernel.context().face_cylinder_payload(&face)?;
                assert_cylinder_payload_close(
                    public_payload,
                    payload,
                    1.0e-12,
                    &format!("{label} public payload"),
                )?;
                let error = kernel
                    .context()
                    .face_plane_payload(&face)
                    .expect_err("cylinder face should reject plane payload requests in Rust");
                assert!(error
                    .to_string()
                    .contains("requested Plane payload for ported Cylinder face"));
            }
            PortedSurface::Cone(payload) => {
                let public_payload = kernel.context().face_cone_payload(&face)?;
                assert_cone_payload_close(
                    public_payload,
                    payload,
                    1.0e-12,
                    &format!("{label} public payload"),
                )?;
            }
            PortedSurface::Sphere(payload) => {
                let public_payload = kernel.context().face_sphere_payload(&face)?;
                assert_sphere_payload_close(
                    public_payload,
                    payload,
                    1.0e-12,
                    &format!("{label} public payload"),
                )?;
            }
            PortedSurface::Torus(payload) => {
                let public_payload = kernel.context().face_torus_payload(&face)?;
                assert_torus_payload_close(
                    public_payload,
                    payload,
                    1.0e-12,
                    &format!("{label} public payload"),
                )?;
            }
        }

        assert!(
            (context_bounds.u_min - geometry.u_min).abs() <= 1.0e-12
                && (context_bounds.u_max - geometry.u_max).abs() <= 1.0e-12
                && (context_bounds.v_min - geometry.v_min).abs() <= 1.0e-12
                && (context_bounds.v_max - geometry.v_max).abs() <= 1.0e-12,
            "{label} bounds mismatch: context={context_bounds:?} geometry={geometry:?}"
        );
        assert!(
            (context_bounds.u_min - occt_bounds.u_min).abs() <= 1.0e-12
                && (context_bounds.u_max - occt_bounds.u_max).abs() <= 1.0e-12
                && (context_bounds.v_min - occt_bounds.v_min).abs() <= 1.0e-12
                && (context_bounds.v_max - occt_bounds.v_max).abs() <= 1.0e-12,
            "{label} bounds mismatch: context={context_bounds:?} occt={occt_bounds:?}"
        );
        assert_face_geometry_close(
            geometry,
            ported_geometry,
            1.0e-12,
            &format!("{label} ported geometry"),
        )?;
        assert_face_geometry_close(geometry, geometry_occt, 1.0e-12, label)?;
        assert_vec3_close(rust_sample.position, occt_sample.position, 1.0e-7, label)?;
        assert_vec3_close(rust_sample.normal, occt_sample.normal, 1.0e-7, label)?;
        assert_vec3_close(
            context_sample.position,
            rust_sample.position,
            1.0e-12,
            label,
        )?;
        assert_vec3_close(context_sample.normal, rust_sample.normal, 1.0e-12, label)?;
        assert_vec3_close(
            rust_uv_sample.position,
            occt_uv_sample.position,
            1.0e-7,
            label,
        )?;
        assert_vec3_close(rust_uv_sample.normal, occt_uv_sample.normal, 1.0e-7, label)?;
        assert_vec3_close(
            context_uv_sample.position,
            rust_uv_sample.position,
            1.0e-12,
            label,
        )?;
        assert_vec3_close(
            context_uv_sample.normal,
            rust_uv_sample.normal,
            1.0e-12,
            label,
        )?;
    }

    assert!(cut_step.is_file());
    assert!(cone_step.is_file());
    assert!(sphere_step.is_file());
    assert!(torus_step.is_file());
    Ok(())
}

#[test]
fn ported_face_surface_descriptors_cover_supported_faces() -> Result<(), Box<dyn std::error::Error>>
{
    let _guard = support::test_guard();
    let kernel = ModelKernel::new()?;

    let cut = kernel.box_with_through_hole(default_cut())?;
    let plane_source = kernel.make_box(BoxParams {
        origin: [-8.0, -6.0, -4.0],
        size: [16.0, 12.0, 8.0],
    })?;
    let cylinder = kernel.make_cylinder(CylinderParams {
        origin: [4.0, -3.0, 1.5],
        axis: [0.0, 0.0, 1.0],
        radius: 6.0,
        height: 18.0,
    })?;
    let cone = kernel.make_cone(ConeParams {
        origin: [-6.0, 5.0, 2.0],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        base_radius: 9.0,
        top_radius: 3.0,
        height: 15.0,
    })?;
    let sphere = kernel.make_sphere(SphereParams {
        origin: [5.0, -4.0, 3.0],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        radius: 7.0,
    })?;
    let torus = kernel.make_torus(TorusParams {
        origin: [-8.0, 6.0, -1.5],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        major_radius: 15.0,
        minor_radius: 4.0,
    })?;
    let ellipse_edge = kernel.make_ellipse_edge(EllipseEdgeParams {
        origin: [30.0, 0.0, 0.0],
        axis: [0.0, 1.0, 0.0],
        x_direction: [1.0, 0.0, 0.0],
        major_radius: 10.0,
        minor_radius: 6.0,
    })?;
    let prism = kernel.make_prism(
        &ellipse_edge,
        PrismParams {
            direction: [0.0, 24.0, 0.0],
        },
    )?;
    let revolution = kernel.make_revolution(
        &ellipse_edge,
        RevolutionParams {
            origin: [0.0, 0.0, 0.0],
            axis: [0.0, 0.0, 1.0],
            angle_radians: PI,
        },
    )?;
    let extrusion_face = find_first_face_by_kind(&kernel, &prism, SurfaceKind::Extrusion)?;
    let revolution_face = find_first_face_by_kind(&kernel, &revolution, SurfaceKind::Revolution)?;
    let offset_face_shape = kernel.make_offset(
        &revolution_face,
        OffsetParams {
            offset: 2.5,
            tolerance: 1.0e-4,
        },
    )?;
    let extrusion_offset_face = kernel.context().make_offset_surface_face(
        &extrusion_face,
        OffsetParams {
            offset: 1.25,
            tolerance: 1.0e-4,
        },
    )?;
    let revolution_offset_face = kernel.context().make_offset_surface_face(
        &revolution_face,
        OffsetParams {
            offset: 1.25,
            tolerance: 1.0e-4,
        },
    )?;
    let plane_face = find_first_face_by_kind(&kernel, &plane_source, SurfaceKind::Plane)?;
    let cylinder_face = find_first_face_by_kind(&kernel, &cylinder, SurfaceKind::Cylinder)?;
    let cone_face = find_first_face_by_kind(&kernel, &cone, SurfaceKind::Cone)?;
    let sphere_face = find_first_face_by_kind(&kernel, &sphere, SurfaceKind::Sphere)?;
    let torus_face = find_first_face_by_kind(&kernel, &torus, SurfaceKind::Torus)?;
    let plane_offset_face = kernel.context().make_offset_surface_face(
        &plane_face,
        OffsetParams {
            offset: 1.25,
            tolerance: 1.0e-4,
        },
    )?;
    let cylinder_offset_face = kernel.context().make_offset_surface_face(
        &cylinder_face,
        OffsetParams {
            offset: 1.25,
            tolerance: 1.0e-4,
        },
    )?;
    let cone_offset_face = kernel.context().make_offset_surface_face(
        &cone_face,
        OffsetParams {
            offset: 1.25,
            tolerance: 1.0e-4,
        },
    )?;
    let sphere_offset_face = kernel.context().make_offset_surface_face(
        &sphere_face,
        OffsetParams {
            offset: 1.25,
            tolerance: 1.0e-4,
        },
    )?;
    let torus_offset_face = kernel.context().make_offset_surface_face(
        &torus_face,
        OffsetParams {
            offset: 1.25,
            tolerance: 1.0e-4,
        },
    )?;

    for (label, face, uv_t) in [
        (
            "plane",
            find_first_face_by_kind(&kernel, &cut, SurfaceKind::Plane)?,
            [0.5, 0.5],
        ),
        ("extrusion", extrusion_face, [0.2, 0.7]),
        ("revolution", revolution_face, [0.2, 0.7]),
        ("offset-extrusion", extrusion_offset_face, [0.2, 0.7]),
        (
            "offset-revolution-direct",
            revolution_offset_face,
            [0.2, 0.7],
        ),
        (
            "offset-revolution",
            find_first_face_by_kind(&kernel, &offset_face_shape, SurfaceKind::Offset)?,
            [0.5, 0.5],
        ),
        ("offset-plane", plane_offset_face, [0.5, 0.5]),
        ("offset-cylinder", cylinder_offset_face, [0.2, 0.7]),
        ("offset-cone", cone_offset_face, [0.2, 0.7]),
        ("offset-sphere", sphere_offset_face, [0.2, 0.7]),
        ("offset-torus", torus_offset_face, [0.2, 0.7]),
    ] {
        let geometry = kernel.context().face_geometry(&face)?;
        let ported_geometry = kernel
            .context()
            .ported_face_geometry(&face)?
            .ok_or_else(|| std::io::Error::other(format!("expected Rust {label} geometry")))?;
        assert_face_geometry_close(
            geometry,
            ported_geometry,
            1.0e-12,
            &format!("{label} public geometry"),
        )?;
        let orientation = kernel.context().shape_orientation(&face)?;
        let uv = normalized_uv_to_uv(geometry, uv_t);
        let descriptor = kernel
            .context()
            .ported_face_surface_descriptor(&face)?
            .ok_or_else(|| std::io::Error::other(format!("expected ported {label} descriptor")))?;

        match (label, descriptor) {
            ("plane", PortedFaceSurface::Analytic(_)) => {}
            (
                "extrusion",
                PortedFaceSurface::Swept(PortedSweptSurface::Extrusion {
                    payload,
                    basis_curve,
                    ..
                }),
            ) => {
                assert_eq!(payload.basis_curve_kind, CurveKind::Ellipse);
                assert!(matches!(basis_curve, PortedCurve::Ellipse(_)));
            }
            (
                "revolution",
                PortedFaceSurface::Swept(PortedSweptSurface::Revolution {
                    payload,
                    basis_curve,
                    ..
                }),
            ) => {
                assert_eq!(payload.basis_curve_kind, CurveKind::Ellipse);
                assert!(matches!(basis_curve, PortedCurve::Ellipse(_)));
            }
            ("offset-extrusion", PortedFaceSurface::Offset(surface)) => {
                assert_eq!(surface.payload.basis_surface_kind, SurfaceKind::Extrusion);
                assert!(matches!(
                    surface.basis,
                    PortedOffsetBasisSurface::Swept(PortedSweptSurface::Extrusion { .. })
                ));
            }
            (
                "offset-revolution" | "offset-revolution-direct",
                PortedFaceSurface::Offset(surface),
            ) => {
                assert_eq!(surface.payload.basis_surface_kind, SurfaceKind::Revolution);
                assert!(matches!(
                    surface.basis,
                    PortedOffsetBasisSurface::Swept(PortedSweptSurface::Revolution { .. })
                ));
            }
            ("offset-plane", PortedFaceSurface::Offset(surface)) => {
                assert_eq!(surface.payload.basis_surface_kind, SurfaceKind::Plane);
                assert!(matches!(
                    surface.basis,
                    PortedOffsetBasisSurface::Analytic(PortedSurface::Plane(_))
                ));
            }
            ("offset-cylinder", PortedFaceSurface::Offset(surface)) => {
                assert_eq!(surface.payload.basis_surface_kind, SurfaceKind::Cylinder);
                assert!(matches!(
                    surface.basis,
                    PortedOffsetBasisSurface::Analytic(PortedSurface::Cylinder(_))
                ));
            }
            ("offset-cone", PortedFaceSurface::Offset(surface)) => {
                assert_eq!(surface.payload.basis_surface_kind, SurfaceKind::Cone);
                assert!(matches!(
                    surface.basis,
                    PortedOffsetBasisSurface::Analytic(PortedSurface::Cone(_))
                ));
            }
            ("offset-sphere", PortedFaceSurface::Offset(surface)) => {
                assert_eq!(surface.payload.basis_surface_kind, SurfaceKind::Sphere);
                assert!(matches!(
                    surface.basis,
                    PortedOffsetBasisSurface::Analytic(PortedSurface::Sphere(_))
                ));
            }
            ("offset-torus", PortedFaceSurface::Offset(surface)) => {
                assert_eq!(surface.payload.basis_surface_kind, SurfaceKind::Torus);
                assert!(matches!(
                    surface.basis,
                    PortedOffsetBasisSurface::Analytic(PortedSurface::Torus(_))
                ));
            }
            _ => {
                return Err(std::io::Error::other(format!(
                    "unexpected ported face descriptor for {label}: {descriptor:?}"
                ))
                .into())
            }
        }

        let rust_sample =
            descriptor.sample_normalized_with_orientation(geometry, uv_t, orientation);
        let occt_sample = kernel.context().face_sample_normalized_occt(&face, uv_t)?;
        assert_vec3_close(
            rust_sample.position,
            occt_sample.position,
            1.0e-6,
            &format!("{label} descriptor sample position"),
        )?;
        assert_vec3_close(
            rust_sample.normal,
            occt_sample.normal,
            1.0e-6,
            &format!("{label} descriptor sample normal"),
        )?;
        let rust_uv_sample = kernel
            .context()
            .ported_face_sample(&face, uv)?
            .ok_or_else(|| std::io::Error::other(format!("expected ported {label} UV sample")))?;
        let context_uv_sample = kernel.context().face_sample(&face, uv)?;
        let occt_uv_sample = kernel.context().face_sample_occt(&face, uv)?;
        assert_vec3_close(
            rust_uv_sample.position,
            occt_uv_sample.position,
            1.0e-6,
            &format!("{label} UV sample position"),
        )?;
        assert_vec3_close(
            rust_uv_sample.normal,
            occt_uv_sample.normal,
            1.0e-6,
            &format!("{label} UV sample normal"),
        )?;
        assert_vec3_close(
            context_uv_sample.position,
            rust_uv_sample.position,
            1.0e-12,
            &format!("{label} context UV sample position"),
        )?;
        assert_vec3_close(
            context_uv_sample.normal,
            rust_uv_sample.normal,
            1.0e-12,
            &format!("{label} context UV sample normal"),
        )?;
        if label.starts_with("offset") {
            let topology = kernel.context().ported_topology(&face)?.ok_or_else(|| {
                std::io::Error::other(format!("expected ported {label} topology"))
            })?;
            assert_eq!(topology.faces.len(), 1);
            if label == "offset-extrusion" {
                assert!(
                    topology.wires.is_empty() && topology.edges.is_empty(),
                    "offset-extrusion should exercise the Rust zero-wire single-face topology path"
                );
            }

            let rust_area = kernel
                .context()
                .ported_face_area(&face)?
                .ok_or_else(|| std::io::Error::other(format!("expected ported {label} area")))?;
            let occt_area = kernel.context().describe_shape_occt(&face)?.surface_area;
            assert_scalar_close(
                rust_area,
                occt_area,
                5.0e-1,
                &format!("{label} ported area"),
            )?;
        }
    }

    Ok(())
}

#[test]
fn ported_box_plane_faces_use_rust_analytic_seed_metadata() -> Result<(), Box<dyn std::error::Error>>
{
    let _guard = support::test_guard();
    let kernel = ModelKernel::new()?;
    let context = kernel.context();

    let box_shape = kernel.make_box(BoxParams {
        origin: [-11.0, -7.0, 3.0],
        size: [23.0, 31.0, 43.0],
    })?;
    assert_eq!(box_shape.rust_multi_face_analytic_source_count(), Some(3));

    let faces = context.subshapes(&box_shape, ShapeKind::Face)?;
    assert_eq!(faces.len(), 6);

    for (face_index, face) in faces.iter().enumerate() {
        assert!(
            face.has_rust_analytic_surface_face_metadata(),
            "box face {face_index} should carry Rust analytic seed metadata"
        );

        let geometry = context.face_geometry(face)?;
        let ported_geometry = context
            .ported_face_geometry(face)?
            .ok_or_else(|| std::io::Error::other("expected Rust box plane geometry"))?;
        let occt_geometry = context.face_geometry_occt(face)?;
        assert_eq!(geometry.kind, SurfaceKind::Plane);
        assert_face_geometry_close(
            geometry,
            ported_geometry,
            1.0e-12,
            &format!("box face {face_index} ported geometry"),
        )?;
        assert_face_geometry_close(
            ported_geometry,
            occt_geometry,
            1.0e-12,
            &format!("box face {face_index} OCCT geometry"),
        )?;

        let descriptor = context
            .ported_face_surface_descriptor(face)?
            .ok_or_else(|| std::io::Error::other("expected Rust box plane descriptor"))?;
        assert!(
            matches!(
                descriptor,
                PortedFaceSurface::Analytic(PortedSurface::Plane(_))
            ),
            "box face {face_index} should classify as a Rust plane descriptor"
        );

        let orientation = context.shape_orientation(face)?;
        for uv_t in [[0.23, 0.31], [0.37, 0.61], [0.58, 0.47], [0.79, 0.73]] {
            let rust_sample =
                descriptor.sample_normalized_with_orientation(geometry, uv_t, orientation);
            let context_sample = context
                .ported_face_sample_normalized(face, uv_t)?
                .ok_or_else(|| std::io::Error::other("expected Rust box plane sample"))?;
            let occt_sample = context.face_sample_normalized_occt(face, uv_t)?;

            assert_vec3_close(
                rust_sample.position,
                occt_sample.position,
                1.0e-6,
                &format!("box face {face_index} descriptor sample position"),
            )?;
            assert_vec3_close(
                rust_sample.normal,
                occt_sample.normal,
                1.0e-6,
                &format!("box face {face_index} descriptor sample normal"),
            )?;
            assert_vec3_close(
                context_sample.position,
                rust_sample.position,
                1.0e-12,
                &format!("box face {face_index} context sample position"),
            )?;
            assert_vec3_close(
                context_sample.normal,
                rust_sample.normal,
                1.0e-12,
                &format!("box face {face_index} context sample normal"),
            )?;
        }
    }

    Ok(())
}

#[test]
fn ported_cylinder_faces_use_rust_analytic_seed_metadata() -> Result<(), Box<dyn std::error::Error>>
{
    let _guard = support::test_guard();
    let kernel = ModelKernel::new()?;
    let context = kernel.context();

    let cylinder = kernel.make_cylinder(CylinderParams {
        origin: [3.0, -5.0, 7.0],
        axis: [0.0, 0.0, 2.0],
        radius: 4.5,
        height: 17.25,
    })?;
    assert_eq!(cylinder.rust_multi_face_analytic_source_count(), Some(2));

    let faces = context.subshapes(&cylinder, ShapeKind::Face)?;
    assert_eq!(faces.len(), 3);

    let mut cylinder_face_count = 0;
    let mut cap_face_count = 0;
    for (face_index, face) in faces.iter().enumerate() {
        assert!(
            face.has_rust_analytic_surface_face_metadata(),
            "cylinder face {face_index} should carry Rust analytic seed metadata"
        );

        let geometry = context.face_geometry(face)?;
        let ported_geometry = context
            .ported_face_geometry(face)?
            .ok_or_else(|| std::io::Error::other("expected Rust cylinder analytic geometry"))?;
        let occt_geometry = context.face_geometry_occt(face)?;
        match geometry.kind {
            SurfaceKind::Cylinder => cylinder_face_count += 1,
            SurfaceKind::Plane => cap_face_count += 1,
            kind => {
                return Err(std::io::Error::other(format!(
                    "unexpected cylinder constructor face kind {kind:?}"
                ))
                .into())
            }
        }
        assert_face_geometry_close(
            geometry,
            ported_geometry,
            1.0e-12,
            &format!("cylinder face {face_index} ported geometry"),
        )?;
        assert_face_geometry_close(
            ported_geometry,
            occt_geometry,
            1.0e-12,
            &format!("cylinder face {face_index} OCCT geometry"),
        )?;

        let descriptor = context
            .ported_face_surface_descriptor(face)?
            .ok_or_else(|| std::io::Error::other("expected Rust cylinder analytic descriptor"))?;
        match descriptor {
            PortedFaceSurface::Analytic(surface) => {
                assert_eq!(ported_surface_kind(surface), geometry.kind);
            }
            descriptor => {
                return Err(std::io::Error::other(format!(
                    "cylinder face {face_index} should classify as analytic, got {descriptor:?}"
                ))
                .into())
            }
        }

        let orientation = context.shape_orientation(face)?;
        for uv_t in [[0.23, 0.31], [0.37, 0.61], [0.58, 0.47], [0.79, 0.73]] {
            let rust_sample =
                descriptor.sample_normalized_with_orientation(geometry, uv_t, orientation);
            let context_sample = context
                .ported_face_sample_normalized(face, uv_t)?
                .ok_or_else(|| std::io::Error::other("expected Rust cylinder analytic sample"))?;
            let occt_sample = context.face_sample_normalized_occt(face, uv_t)?;

            assert_vec3_close(
                rust_sample.position,
                occt_sample.position,
                1.0e-6,
                &format!("cylinder face {face_index} descriptor sample position"),
            )?;
            assert_vec3_close(
                rust_sample.normal,
                occt_sample.normal,
                1.0e-6,
                &format!("cylinder face {face_index} descriptor sample normal"),
            )?;
            assert_vec3_close(
                context_sample.position,
                rust_sample.position,
                1.0e-12,
                &format!("cylinder face {face_index} context sample position"),
            )?;
            assert_vec3_close(
                context_sample.normal,
                rust_sample.normal,
                1.0e-12,
                &format!("cylinder face {face_index} context sample normal"),
            )?;
        }
    }

    assert_eq!(cylinder_face_count, 1);
    assert_eq!(cap_face_count, 2);

    Ok(())
}

#[test]
fn ported_cone_faces_use_rust_analytic_seed_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let kernel = ModelKernel::new()?;
    let context = kernel.context();

    let cone = kernel.make_cone(ConeParams {
        origin: [-6.0, 5.0, 2.0],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        base_radius: 9.0,
        top_radius: 3.0,
        height: 15.0,
    })?;
    assert_eq!(cone.rust_multi_face_analytic_source_count(), Some(3));

    let faces = context.subshapes(&cone, ShapeKind::Face)?;
    assert_eq!(faces.len(), 3);

    let mut cone_face_count = 0;
    let mut cap_face_count = 0;
    for (face_index, face) in faces.iter().enumerate() {
        assert!(
            face.has_rust_analytic_surface_face_metadata(),
            "cone face {face_index} should carry Rust analytic seed metadata"
        );

        let geometry = context.face_geometry(face)?;
        let ported_geometry = context
            .ported_face_geometry(face)?
            .ok_or_else(|| std::io::Error::other("expected Rust cone analytic geometry"))?;
        let occt_geometry = context.face_geometry_occt(face)?;
        match geometry.kind {
            SurfaceKind::Cone => cone_face_count += 1,
            SurfaceKind::Plane => cap_face_count += 1,
            kind => {
                return Err(std::io::Error::other(format!(
                    "unexpected cone constructor face kind {kind:?}"
                ))
                .into())
            }
        }
        assert_face_geometry_close(
            geometry,
            ported_geometry,
            1.0e-12,
            &format!("cone face {face_index} ported geometry"),
        )?;
        assert_face_geometry_close(
            ported_geometry,
            occt_geometry,
            1.0e-12,
            &format!("cone face {face_index} OCCT geometry"),
        )?;

        let descriptor = context
            .ported_face_surface_descriptor(face)?
            .ok_or_else(|| std::io::Error::other("expected Rust cone analytic descriptor"))?;
        match descriptor {
            PortedFaceSurface::Analytic(surface) => {
                assert_eq!(ported_surface_kind(surface), geometry.kind);
            }
            descriptor => {
                return Err(std::io::Error::other(format!(
                    "cone face {face_index} should classify as analytic, got {descriptor:?}"
                ))
                .into())
            }
        }

        let orientation = context.shape_orientation(face)?;
        for uv_t in [[0.23, 0.31], [0.37, 0.61], [0.58, 0.47], [0.79, 0.73]] {
            let rust_sample =
                descriptor.sample_normalized_with_orientation(geometry, uv_t, orientation);
            let context_sample = context
                .ported_face_sample_normalized(face, uv_t)?
                .ok_or_else(|| std::io::Error::other("expected Rust cone analytic sample"))?;
            let occt_sample = context.face_sample_normalized_occt(face, uv_t)?;

            assert_vec3_close(
                rust_sample.position,
                occt_sample.position,
                1.0e-6,
                &format!("cone face {face_index} descriptor sample position"),
            )?;
            assert_vec3_close(
                rust_sample.normal,
                occt_sample.normal,
                1.0e-6,
                &format!("cone face {face_index} descriptor sample normal"),
            )?;
            assert_vec3_close(
                context_sample.position,
                rust_sample.position,
                1.0e-12,
                &format!("cone face {face_index} context sample position"),
            )?;
            assert_vec3_close(
                context_sample.normal,
                rust_sample.normal,
                1.0e-12,
                &format!("cone face {face_index} context sample normal"),
            )?;
        }
    }

    assert_eq!(cone_face_count, 1);
    assert_eq!(cap_face_count, 2);

    Ok(())
}

#[test]
fn ported_sphere_faces_use_rust_analytic_seed_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let kernel = ModelKernel::new()?;
    let context = kernel.context();

    let sphere = kernel.make_sphere(SphereParams {
        origin: [4.0, -8.0, 6.0],
        axis: [0.0, 0.0, 2.0],
        x_direction: [2.0, 0.0, 0.0],
        radius: 13.75,
    })?;
    assert_eq!(sphere.rust_multi_face_analytic_source_count(), Some(1));

    let faces = context.subshapes(&sphere, ShapeKind::Face)?;
    assert_eq!(faces.len(), 1);

    for (face_index, face) in faces.iter().enumerate() {
        assert!(
            face.has_rust_analytic_surface_face_metadata(),
            "sphere face {face_index} should carry Rust analytic seed metadata"
        );

        let geometry = context.face_geometry(face)?;
        let ported_geometry = context
            .ported_face_geometry(face)?
            .ok_or_else(|| std::io::Error::other("expected Rust sphere analytic geometry"))?;
        let occt_geometry = context.face_geometry_occt(face)?;
        assert_eq!(geometry.kind, SurfaceKind::Sphere);
        assert_face_geometry_close(
            geometry,
            ported_geometry,
            1.0e-12,
            &format!("sphere face {face_index} ported geometry"),
        )?;
        assert_face_geometry_close(
            ported_geometry,
            occt_geometry,
            1.0e-12,
            &format!("sphere face {face_index} OCCT geometry"),
        )?;

        let descriptor = context
            .ported_face_surface_descriptor(face)?
            .ok_or_else(|| std::io::Error::other("expected Rust sphere analytic descriptor"))?;
        match descriptor {
            PortedFaceSurface::Analytic(surface) => {
                assert_eq!(ported_surface_kind(surface), SurfaceKind::Sphere);
            }
            descriptor => {
                return Err(std::io::Error::other(format!(
                    "sphere face {face_index} should classify as analytic, got {descriptor:?}"
                ))
                .into())
            }
        }

        let orientation = context.shape_orientation(face)?;
        for uv_t in [[0.23, 0.31], [0.37, 0.61], [0.58, 0.47], [0.79, 0.73]] {
            let rust_sample =
                descriptor.sample_normalized_with_orientation(geometry, uv_t, orientation);
            let context_sample = context
                .ported_face_sample_normalized(face, uv_t)?
                .ok_or_else(|| std::io::Error::other("expected Rust sphere analytic sample"))?;
            let occt_sample = context.face_sample_normalized_occt(face, uv_t)?;

            assert_vec3_close(
                rust_sample.position,
                occt_sample.position,
                1.0e-6,
                &format!("sphere face {face_index} descriptor sample position"),
            )?;
            assert_vec3_close(
                rust_sample.normal,
                occt_sample.normal,
                1.0e-6,
                &format!("sphere face {face_index} descriptor sample normal"),
            )?;
            assert_vec3_close(
                context_sample.position,
                rust_sample.position,
                1.0e-12,
                &format!("sphere face {face_index} context sample position"),
            )?;
            assert_vec3_close(
                context_sample.normal,
                rust_sample.normal,
                1.0e-12,
                &format!("sphere face {face_index} context sample normal"),
            )?;
        }
    }

    Ok(())
}

#[test]
fn ported_torus_faces_use_rust_analytic_seed_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let kernel = ModelKernel::new()?;
    let context = kernel.context();

    let torus = kernel.make_torus(TorusParams {
        origin: [-9.0, 7.0, 5.0],
        axis: [0.0, 0.0, 3.0],
        x_direction: [2.0, 0.0, 0.0],
        major_radius: 18.5,
        minor_radius: 4.25,
    })?;
    assert_eq!(torus.rust_multi_face_analytic_source_count(), Some(1));

    let faces = context.subshapes(&torus, ShapeKind::Face)?;
    assert_eq!(faces.len(), 1);

    for (face_index, face) in faces.iter().enumerate() {
        assert!(
            face.has_rust_analytic_surface_face_metadata(),
            "torus face {face_index} should carry Rust analytic seed metadata"
        );

        let geometry = context.face_geometry(face)?;
        let ported_geometry = context
            .ported_face_geometry(face)?
            .ok_or_else(|| std::io::Error::other("expected Rust torus analytic geometry"))?;
        let occt_geometry = context.face_geometry_occt(face)?;
        assert_eq!(geometry.kind, SurfaceKind::Torus);
        assert_face_geometry_close(
            geometry,
            ported_geometry,
            1.0e-12,
            &format!("torus face {face_index} ported geometry"),
        )?;
        assert_face_geometry_close(
            ported_geometry,
            occt_geometry,
            1.0e-12,
            &format!("torus face {face_index} OCCT geometry"),
        )?;

        let descriptor = context
            .ported_face_surface_descriptor(face)?
            .ok_or_else(|| std::io::Error::other("expected Rust torus analytic descriptor"))?;
        match descriptor {
            PortedFaceSurface::Analytic(surface) => {
                assert_eq!(ported_surface_kind(surface), SurfaceKind::Torus);
            }
            descriptor => {
                return Err(std::io::Error::other(format!(
                    "torus face {face_index} should classify as analytic, got {descriptor:?}"
                ))
                .into())
            }
        }

        let orientation = context.shape_orientation(face)?;
        for uv_t in [[0.23, 0.31], [0.37, 0.61], [0.58, 0.47], [0.79, 0.73]] {
            let rust_sample =
                descriptor.sample_normalized_with_orientation(geometry, uv_t, orientation);
            let context_sample = context
                .ported_face_sample_normalized(face, uv_t)?
                .ok_or_else(|| std::io::Error::other("expected Rust torus analytic sample"))?;
            let occt_sample = context.face_sample_normalized_occt(face, uv_t)?;

            assert_vec3_close(
                rust_sample.position,
                occt_sample.position,
                1.0e-6,
                &format!("torus face {face_index} descriptor sample position"),
            )?;
            assert_vec3_close(
                rust_sample.normal,
                occt_sample.normal,
                1.0e-6,
                &format!("torus face {face_index} descriptor sample normal"),
            )?;
            assert_vec3_close(
                context_sample.position,
                rust_sample.position,
                1.0e-12,
                &format!("torus face {face_index} context sample position"),
            )?;
            assert_vec3_close(
                context_sample.normal,
                rust_sample.normal,
                1.0e-12,
                &format!("torus face {face_index} context sample normal"),
            )?;
        }
    }

    Ok(())
}

#[test]
fn public_swept_and_offset_payload_queries_match_occt() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let kernel = ModelKernel::new()?;

    let ellipse_edge = kernel.make_ellipse_edge(EllipseEdgeParams {
        origin: [30.0, 0.0, 0.0],
        axis: [0.0, 1.0, 0.0],
        x_direction: [1.0, 0.0, 0.0],
        major_radius: 10.0,
        minor_radius: 6.0,
    })?;
    let prism = kernel.make_prism(
        &ellipse_edge,
        PrismParams {
            direction: [0.0, 24.0, 0.0],
        },
    )?;
    let extrusion_face = find_first_face_by_kind(&kernel, &prism, SurfaceKind::Extrusion)?;
    let context = kernel.context();
    let extrusion_descriptor = require_ported_swept_face_surface(
        context.ported_face_surface_descriptor(&extrusion_face)?,
        SurfaceKind::Extrusion,
        "extrusion public payload",
    )?;
    let extrusion_descriptor_payload = match extrusion_descriptor {
        PortedSweptSurface::Extrusion { payload, .. } => payload,
        descriptor => unreachable!("validated extrusion descriptor was {descriptor:?}"),
    };
    let extrusion_payload = context.face_extrusion_payload(&extrusion_face)?;
    let extrusion_payload_occt = context.face_extrusion_payload_occt(&extrusion_face)?;
    assert_extrusion_payload_close(
        extrusion_payload,
        extrusion_descriptor_payload,
        1.0e-12,
        "extrusion public descriptor payload",
    )?;
    assert_extrusion_payload_close(
        extrusion_payload,
        extrusion_payload_occt,
        1.0e-12,
        "extrusion public occt payload",
    )?;
    let error = context
        .face_revolution_payload(&extrusion_face)
        .expect_err("extrusion face should reject revolution payload requests in Rust");
    assert!(error
        .to_string()
        .contains("requested Revolution payload for ported Extrusion face"));
    let error = context
        .face_plane_payload(&extrusion_face)
        .expect_err("extrusion face should reject analytic plane payload requests in Rust");
    assert!(error
        .to_string()
        .contains("requested Plane payload for ported Extrusion face"));

    let revolution = kernel.make_revolution(
        &ellipse_edge,
        RevolutionParams {
            origin: [0.0, 0.0, 0.0],
            axis: [0.0, 0.0, 1.0],
            angle_radians: PI,
        },
    )?;
    let revolution_face = find_first_face_by_kind(&kernel, &revolution, SurfaceKind::Revolution)?;
    let revolution_descriptor = require_ported_swept_face_surface(
        context.ported_face_surface_descriptor(&revolution_face)?,
        SurfaceKind::Revolution,
        "revolution public payload",
    )?;
    let revolution_descriptor_payload = match revolution_descriptor {
        PortedSweptSurface::Revolution { payload, .. } => payload,
        descriptor => unreachable!("validated revolution descriptor was {descriptor:?}"),
    };
    let revolution_payload = context.face_revolution_payload(&revolution_face)?;
    let revolution_payload_occt = context.face_revolution_payload_occt(&revolution_face)?;
    assert_revolution_payload_close(
        revolution_payload,
        revolution_descriptor_payload,
        1.0e-12,
        "revolution public descriptor payload",
    )?;
    assert_revolution_payload_close(
        revolution_payload,
        revolution_payload_occt,
        1.0e-12,
        "revolution public occt payload",
    )?;
    let error = context
        .face_extrusion_payload(&revolution_face)
        .expect_err("revolution face should reject extrusion payload requests in Rust");
    assert!(error
        .to_string()
        .contains("requested Extrusion payload for ported Revolution face"));

    let offset_shape = kernel.make_offset(
        &revolution_face,
        OffsetParams {
            offset: 2.5,
            tolerance: 1.0e-4,
        },
    )?;
    let offset_face = find_first_face_by_kind(&kernel, &offset_shape, SurfaceKind::Offset)?;
    let offset_descriptor = require_ported_offset_face_surface(
        context.ported_face_surface_descriptor(&offset_face)?,
        "offset public payload",
    )?;
    let offset_payload = context.face_offset_payload(&offset_face)?;
    let offset_payload_occt = context.face_offset_payload_occt(&offset_face)?;
    assert_offset_payload_close(
        offset_payload,
        offset_descriptor.payload,
        1.0e-12,
        "offset public descriptor payload",
    )?;
    assert_offset_payload_close(
        offset_payload,
        offset_payload_occt,
        1.0e-12,
        "offset public occt payload",
    )?;
    let error = context
        .face_revolution_payload(&offset_face)
        .expect_err("offset face should reject top-level revolution payload requests in Rust");
    assert!(error
        .to_string()
        .contains("requested Revolution payload for ported Offset face"));

    Ok(())
}

#[test]
fn public_analytic_curve_and_surface_payload_queries_match_occt(
) -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let kernel = ModelKernel::new()?;

    let cut = kernel.box_with_through_hole(default_cut())?;
    let ellipse_edge = kernel.make_ellipse_edge(EllipseEdgeParams {
        origin: [30.0, 0.0, 0.0],
        axis: [0.0, 1.0, 0.0],
        x_direction: [1.0, 0.0, 0.0],
        major_radius: 10.0,
        minor_radius: 6.0,
    })?;
    let cone = kernel.make_cone(ConeParams {
        origin: [0.0, 0.0, 0.0],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        base_radius: 15.0,
        top_radius: 5.0,
        height: 30.0,
    })?;
    let sphere = kernel.make_sphere(SphereParams {
        origin: [0.0, 0.0, 0.0],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        radius: 14.0,
    })?;
    let torus = kernel.make_torus(TorusParams {
        origin: [0.0, 0.0, 0.0],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        major_radius: 25.0,
        minor_radius: 6.0,
    })?;

    let context = kernel.context();

    let line_edge = find_first_edge_by_kind(&kernel, &cut, CurveKind::Line)?;
    let line_descriptor = require_ported_edge_curve(
        context.ported_edge_curve(&line_edge)?,
        CurveKind::Line,
        "line public payload",
    )?;
    let line_payload = context.edge_line_payload(&line_edge)?;
    let PortedCurve::Line(line_descriptor_payload) = line_descriptor else {
        unreachable!("descriptor kind was checked above");
    };
    assert_line_payload_close(
        line_payload,
        line_descriptor_payload,
        1.0e-12,
        "line public descriptor payload",
    )?;
    let line_payload_occt = context.edge_line_payload_occt(&line_edge)?;
    assert_line_payload_close(
        line_payload,
        line_payload_occt,
        1.0e-12,
        "line public occt payload",
    )?;
    let error = context
        .edge_circle_payload(&line_edge)
        .expect_err("line edge should reject circle payload requests in Rust");
    assert!(error
        .to_string()
        .contains("requested Circle payload for ported Line edge"));

    let circle_edge = find_first_edge_by_kind(&kernel, &cut, CurveKind::Circle)?;
    let circle_descriptor = require_ported_edge_curve(
        context.ported_edge_curve(&circle_edge)?,
        CurveKind::Circle,
        "circle public payload",
    )?;
    let circle_payload = context.edge_circle_payload(&circle_edge)?;
    let PortedCurve::Circle(circle_descriptor_payload) = circle_descriptor else {
        unreachable!("descriptor kind was checked above");
    };
    assert_circle_payload_close(
        circle_payload,
        circle_descriptor_payload,
        1.0e-12,
        "circle public descriptor payload",
    )?;
    let circle_payload_occt = context.edge_circle_payload_occt(&circle_edge)?;
    assert_circle_payload_close(
        circle_payload,
        circle_payload_occt,
        1.0e-12,
        "circle public occt payload",
    )?;
    let error = context
        .edge_line_payload(&circle_edge)
        .expect_err("circle edge should reject line payload requests in Rust");
    assert!(error
        .to_string()
        .contains("requested Line payload for ported Circle edge"));

    let ellipse_descriptor = require_ported_edge_curve(
        context.ported_edge_curve(&ellipse_edge)?,
        CurveKind::Ellipse,
        "ellipse public payload",
    )?;
    let ellipse_payload = context.edge_ellipse_payload(&ellipse_edge)?;
    let PortedCurve::Ellipse(ellipse_descriptor_payload) = ellipse_descriptor else {
        unreachable!("descriptor kind was checked above");
    };
    assert_ellipse_payload_close(
        ellipse_payload,
        ellipse_descriptor_payload,
        1.0e-12,
        "ellipse public descriptor payload",
    )?;
    let ellipse_payload_occt = context.edge_ellipse_payload_occt(&ellipse_edge)?;
    assert_ellipse_payload_close(
        ellipse_payload,
        ellipse_payload_occt,
        1.0e-12,
        "ellipse public occt payload",
    )?;
    let error = context
        .edge_line_payload(&ellipse_edge)
        .expect_err("ellipse edge should reject line payload requests in Rust");
    assert!(error
        .to_string()
        .contains("requested Line payload for ported Ellipse edge"));

    let plane_face = find_first_face_by_kind(&kernel, &cut, SurfaceKind::Plane)?;
    let plane_descriptor = require_ported_analytic_face_surface(
        context.ported_face_surface(&plane_face)?,
        SurfaceKind::Plane,
        "plane public payload",
    )?;
    let plane_payload = context.face_plane_payload(&plane_face)?;
    let PortedSurface::Plane(plane_descriptor_payload) = plane_descriptor else {
        unreachable!("descriptor kind was checked above");
    };
    assert_vec3_close(
        plane_payload.origin,
        plane_descriptor_payload.origin,
        1.0e-12,
        "plane public descriptor origin",
    )?;
    assert_vec3_close(
        plane_payload.normal,
        plane_descriptor_payload.normal,
        1.0e-12,
        "plane public descriptor normal",
    )?;
    assert_vec3_close(
        plane_payload.x_direction,
        plane_descriptor_payload.x_direction,
        1.0e-12,
        "plane public descriptor x direction",
    )?;
    assert_vec3_close(
        plane_payload.y_direction,
        plane_descriptor_payload.y_direction,
        1.0e-12,
        "plane public descriptor y direction",
    )?;
    let plane_payload_occt = context.face_plane_payload_occt(&plane_face)?;
    assert_vec3_close(
        plane_payload.origin,
        plane_payload_occt.origin,
        1.0e-12,
        "plane payload origin",
    )?;
    assert_vec3_close(
        plane_payload.normal,
        plane_payload_occt.normal,
        1.0e-12,
        "plane payload normal",
    )?;
    assert_vec3_close(
        plane_payload.x_direction,
        plane_payload_occt.x_direction,
        1.0e-12,
        "plane payload x direction",
    )?;
    assert_vec3_close(
        plane_payload.y_direction,
        plane_payload_occt.y_direction,
        1.0e-12,
        "plane payload y direction",
    )?;

    let cylinder_face = find_first_face_by_kind(&kernel, &cut, SurfaceKind::Cylinder)?;
    let cylinder_descriptor = require_ported_analytic_face_surface(
        context.ported_face_surface(&cylinder_face)?,
        SurfaceKind::Cylinder,
        "cylinder public payload",
    )?;
    let cylinder_payload = context.face_cylinder_payload(&cylinder_face)?;
    let PortedSurface::Cylinder(cylinder_descriptor_payload) = cylinder_descriptor else {
        unreachable!("descriptor kind was checked above");
    };
    assert_vec3_close(
        cylinder_payload.origin,
        cylinder_descriptor_payload.origin,
        1.0e-12,
        "cylinder public descriptor origin",
    )?;
    assert_vec3_close(
        cylinder_payload.axis,
        cylinder_descriptor_payload.axis,
        1.0e-12,
        "cylinder public descriptor axis",
    )?;
    assert_vec3_close(
        cylinder_payload.x_direction,
        cylinder_descriptor_payload.x_direction,
        1.0e-12,
        "cylinder public descriptor x direction",
    )?;
    assert_vec3_close(
        cylinder_payload.y_direction,
        cylinder_descriptor_payload.y_direction,
        1.0e-12,
        "cylinder public descriptor y direction",
    )?;
    assert_scalar_close(
        cylinder_payload.radius,
        cylinder_descriptor_payload.radius,
        1.0e-12,
        "cylinder public descriptor radius",
    )?;
    let cylinder_payload_occt = context.face_cylinder_payload_occt(&cylinder_face)?;
    assert_vec3_close(
        cylinder_payload.origin,
        cylinder_payload_occt.origin,
        1.0e-12,
        "cylinder payload origin",
    )?;
    assert_vec3_close(
        cylinder_payload.axis,
        cylinder_payload_occt.axis,
        1.0e-12,
        "cylinder payload axis",
    )?;
    assert_vec3_close(
        cylinder_payload.x_direction,
        cylinder_payload_occt.x_direction,
        1.0e-12,
        "cylinder payload x direction",
    )?;
    assert_vec3_close(
        cylinder_payload.y_direction,
        cylinder_payload_occt.y_direction,
        1.0e-12,
        "cylinder payload y direction",
    )?;
    assert_scalar_close(
        cylinder_payload.radius,
        cylinder_payload_occt.radius,
        1.0e-12,
        "cylinder payload radius",
    )?;

    let cone_face = find_first_face_by_kind(&kernel, &cone, SurfaceKind::Cone)?;
    let cone_descriptor = require_ported_analytic_face_surface(
        context.ported_face_surface(&cone_face)?,
        SurfaceKind::Cone,
        "cone public payload",
    )?;
    let cone_payload = context.face_cone_payload(&cone_face)?;
    let PortedSurface::Cone(cone_descriptor_payload) = cone_descriptor else {
        unreachable!("descriptor kind was checked above");
    };
    assert_vec3_close(
        cone_payload.origin,
        cone_descriptor_payload.origin,
        1.0e-12,
        "cone public descriptor origin",
    )?;
    assert_vec3_close(
        cone_payload.axis,
        cone_descriptor_payload.axis,
        1.0e-12,
        "cone public descriptor axis",
    )?;
    assert_vec3_close(
        cone_payload.x_direction,
        cone_descriptor_payload.x_direction,
        1.0e-12,
        "cone public descriptor x direction",
    )?;
    assert_vec3_close(
        cone_payload.y_direction,
        cone_descriptor_payload.y_direction,
        1.0e-12,
        "cone public descriptor y direction",
    )?;
    assert_scalar_close(
        cone_payload.reference_radius,
        cone_descriptor_payload.reference_radius,
        1.0e-12,
        "cone public descriptor reference radius",
    )?;
    assert_scalar_close(
        cone_payload.semi_angle,
        cone_descriptor_payload.semi_angle,
        1.0e-12,
        "cone public descriptor semi angle",
    )?;
    let cone_payload_occt = context.face_cone_payload_occt(&cone_face)?;
    assert_vec3_close(
        cone_payload.origin,
        cone_payload_occt.origin,
        1.0e-12,
        "cone payload origin",
    )?;
    assert_vec3_close(
        cone_payload.axis,
        cone_payload_occt.axis,
        1.0e-12,
        "cone payload axis",
    )?;
    assert_vec3_close(
        cone_payload.x_direction,
        cone_payload_occt.x_direction,
        1.0e-12,
        "cone payload x direction",
    )?;
    assert_vec3_close(
        cone_payload.y_direction,
        cone_payload_occt.y_direction,
        1.0e-12,
        "cone payload y direction",
    )?;
    assert_scalar_close(
        cone_payload.reference_radius,
        cone_payload_occt.reference_radius,
        1.0e-12,
        "cone payload reference radius",
    )?;
    assert_scalar_close(
        cone_payload.semi_angle,
        cone_payload_occt.semi_angle,
        1.0e-12,
        "cone payload semi angle",
    )?;

    let sphere_face = find_first_face_by_kind(&kernel, &sphere, SurfaceKind::Sphere)?;
    let sphere_descriptor = require_ported_analytic_face_surface(
        context.ported_face_surface(&sphere_face)?,
        SurfaceKind::Sphere,
        "sphere public payload",
    )?;
    let sphere_payload = context.face_sphere_payload(&sphere_face)?;
    let PortedSurface::Sphere(sphere_descriptor_payload) = sphere_descriptor else {
        unreachable!("descriptor kind was checked above");
    };
    assert_vec3_close(
        sphere_payload.center,
        sphere_descriptor_payload.center,
        1.0e-12,
        "sphere public descriptor center",
    )?;
    assert_vec3_close(
        sphere_payload.normal,
        sphere_descriptor_payload.normal,
        1.0e-12,
        "sphere public descriptor normal",
    )?;
    assert_vec3_close(
        sphere_payload.x_direction,
        sphere_descriptor_payload.x_direction,
        1.0e-12,
        "sphere public descriptor x direction",
    )?;
    assert_vec3_close(
        sphere_payload.y_direction,
        sphere_descriptor_payload.y_direction,
        1.0e-12,
        "sphere public descriptor y direction",
    )?;
    assert_scalar_close(
        sphere_payload.radius,
        sphere_descriptor_payload.radius,
        1.0e-12,
        "sphere public descriptor radius",
    )?;
    let sphere_payload_occt = context.face_sphere_payload_occt(&sphere_face)?;
    assert_vec3_close(
        sphere_payload.center,
        sphere_payload_occt.center,
        1.0e-12,
        "sphere payload center",
    )?;
    assert_vec3_close(
        sphere_payload.normal,
        sphere_payload_occt.normal,
        1.0e-12,
        "sphere payload normal",
    )?;
    assert_vec3_close(
        sphere_payload.x_direction,
        sphere_payload_occt.x_direction,
        1.0e-12,
        "sphere payload x direction",
    )?;
    assert_vec3_close(
        sphere_payload.y_direction,
        sphere_payload_occt.y_direction,
        1.0e-12,
        "sphere payload y direction",
    )?;
    assert_scalar_close(
        sphere_payload.radius,
        sphere_payload_occt.radius,
        1.0e-12,
        "sphere payload radius",
    )?;

    let torus_face = find_first_face_by_kind(&kernel, &torus, SurfaceKind::Torus)?;
    let torus_descriptor = require_ported_analytic_face_surface(
        context.ported_face_surface(&torus_face)?,
        SurfaceKind::Torus,
        "torus public payload",
    )?;
    let torus_payload = context.face_torus_payload(&torus_face)?;
    let PortedSurface::Torus(torus_descriptor_payload) = torus_descriptor else {
        unreachable!("descriptor kind was checked above");
    };
    assert_vec3_close(
        torus_payload.center,
        torus_descriptor_payload.center,
        1.0e-12,
        "torus public descriptor center",
    )?;
    assert_vec3_close(
        torus_payload.axis,
        torus_descriptor_payload.axis,
        1.0e-12,
        "torus public descriptor axis",
    )?;
    assert_vec3_close(
        torus_payload.x_direction,
        torus_descriptor_payload.x_direction,
        1.0e-12,
        "torus public descriptor x direction",
    )?;
    assert_vec3_close(
        torus_payload.y_direction,
        torus_descriptor_payload.y_direction,
        1.0e-12,
        "torus public descriptor y direction",
    )?;
    assert_scalar_close(
        torus_payload.major_radius,
        torus_descriptor_payload.major_radius,
        1.0e-12,
        "torus public descriptor major radius",
    )?;
    assert_scalar_close(
        torus_payload.minor_radius,
        torus_descriptor_payload.minor_radius,
        1.0e-12,
        "torus public descriptor minor radius",
    )?;
    let torus_payload_occt = context.face_torus_payload_occt(&torus_face)?;
    assert_vec3_close(
        torus_payload.center,
        torus_payload_occt.center,
        1.0e-12,
        "torus payload center",
    )?;
    assert_vec3_close(
        torus_payload.axis,
        torus_payload_occt.axis,
        1.0e-12,
        "torus payload axis",
    )?;
    assert_vec3_close(
        torus_payload.x_direction,
        torus_payload_occt.x_direction,
        1.0e-12,
        "torus payload x direction",
    )?;
    assert_vec3_close(
        torus_payload.y_direction,
        torus_payload_occt.y_direction,
        1.0e-12,
        "torus payload y direction",
    )?;
    assert_scalar_close(
        torus_payload.major_radius,
        torus_payload_occt.major_radius,
        1.0e-12,
        "torus payload major radius",
    )?;
    assert_scalar_close(
        torus_payload.minor_radius,
        torus_payload_occt.minor_radius,
        1.0e-12,
        "torus payload minor radius",
    )?;

    Ok(())
}

#[test]
fn public_offset_basis_queries_match_occt() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let kernel = ModelKernel::new()?;

    let plane_source = kernel.make_box(BoxParams {
        origin: [-8.0, -6.0, -4.0],
        size: [16.0, 12.0, 8.0],
    })?;
    let cylinder = kernel.make_cylinder(CylinderParams {
        origin: [4.0, -3.0, 1.5],
        axis: [0.0, 0.0, 1.0],
        radius: 6.0,
        height: 18.0,
    })?;
    let cone = kernel.make_cone(ConeParams {
        origin: [-6.0, 5.0, 2.0],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        base_radius: 9.0,
        top_radius: 3.0,
        height: 15.0,
    })?;
    let sphere = kernel.make_sphere(SphereParams {
        origin: [5.0, -4.0, 3.0],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        radius: 7.0,
    })?;
    let torus = kernel.make_torus(TorusParams {
        origin: [-8.0, 6.0, -1.5],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        major_radius: 15.0,
        minor_radius: 4.0,
    })?;
    let ellipse_edge = kernel.make_ellipse_edge(EllipseEdgeParams {
        origin: [30.0, 0.0, 0.0],
        axis: [0.0, 1.0, 0.0],
        x_direction: [1.0, 0.0, 0.0],
        major_radius: 10.0,
        minor_radius: 6.0,
    })?;
    let prism = kernel.make_prism(
        &ellipse_edge,
        PrismParams {
            direction: [0.0, 24.0, 0.0],
        },
    )?;
    let revolution = kernel.make_revolution(
        &ellipse_edge,
        RevolutionParams {
            origin: [0.0, 0.0, 0.0],
            axis: [0.0, 0.0, 1.0],
            angle_radians: PI,
        },
    )?;

    let context = kernel.context();
    let plane_face = find_first_face_by_kind(&kernel, &plane_source, SurfaceKind::Plane)?;
    let plane_offset_face = context.make_offset_surface_face(
        &plane_face,
        OffsetParams {
            offset: 1.25,
            tolerance: 1.0e-4,
        },
    )?;
    let cylinder_face = find_first_face_by_kind(&kernel, &cylinder, SurfaceKind::Cylinder)?;
    let cylinder_offset_face = context.make_offset_surface_face(
        &cylinder_face,
        OffsetParams {
            offset: 1.25,
            tolerance: 1.0e-4,
        },
    )?;
    let cone_face = find_first_face_by_kind(&kernel, &cone, SurfaceKind::Cone)?;
    let cone_offset_face = context.make_offset_surface_face(
        &cone_face,
        OffsetParams {
            offset: 1.25,
            tolerance: 1.0e-4,
        },
    )?;
    let sphere_face = find_first_face_by_kind(&kernel, &sphere, SurfaceKind::Sphere)?;
    let sphere_offset_face = context.make_offset_surface_face(
        &sphere_face,
        OffsetParams {
            offset: 1.25,
            tolerance: 1.0e-4,
        },
    )?;
    let torus_face = find_first_face_by_kind(&kernel, &torus, SurfaceKind::Torus)?;
    let torus_offset_face = context.make_offset_surface_face(
        &torus_face,
        OffsetParams {
            offset: 1.25,
            tolerance: 1.0e-4,
        },
    )?;

    for (label, basis_kind, basis_face, offset_face) in [
        ("plane", SurfaceKind::Plane, &plane_face, &plane_offset_face),
        (
            "cylinder",
            SurfaceKind::Cylinder,
            &cylinder_face,
            &cylinder_offset_face,
        ),
        ("cone", SurfaceKind::Cone, &cone_face, &cone_offset_face),
        (
            "sphere",
            SurfaceKind::Sphere,
            &sphere_face,
            &sphere_offset_face,
        ),
        ("torus", SurfaceKind::Torus, &torus_face, &torus_offset_face),
    ] {
        let offset_payload = context.face_offset_payload(offset_face)?;
        assert_eq!(offset_payload.basis_surface_kind, basis_kind);
        assert!(
            (offset_payload.offset_value - 1.25).abs() <= 1.0e-12,
            "{label} direct analytic offset value mismatch"
        );

        let basis_geometry = context.face_geometry(basis_face)?;
        let ported_basis_geometry = context
            .ported_face_geometry(basis_face)?
            .ok_or_else(|| std::io::Error::other(format!("{label} missing Rust basis geometry")))?;
        assert_eq!(ported_basis_geometry.kind, basis_kind);
        assert_face_geometry_close(
            basis_geometry,
            ported_basis_geometry,
            1.0e-12,
            &format!("{label} direct analytic Rust basis geometry"),
        )?;

        assert_face_geometry_close(
            context.face_offset_basis_geometry(offset_face)?,
            basis_geometry,
            1.0e-12,
            &format!("{label} direct analytic offset basis geometry"),
        )?;

        match basis_kind {
            SurfaceKind::Plane => assert_plane_payload_close(
                context.face_offset_basis_plane_payload(offset_face)?,
                context.face_plane_payload(basis_face)?,
                1.0e-12,
                "direct plane offset basis payload",
            )?,
            SurfaceKind::Cylinder => assert_cylinder_payload_close(
                context.face_offset_basis_cylinder_payload(offset_face)?,
                context.face_cylinder_payload(basis_face)?,
                1.0e-12,
                "direct cylinder offset basis payload",
            )?,
            SurfaceKind::Cone => assert_cone_payload_close(
                context.face_offset_basis_cone_payload(offset_face)?,
                context.face_cone_payload(basis_face)?,
                1.0e-12,
                "direct cone offset basis payload",
            )?,
            SurfaceKind::Sphere => assert_sphere_payload_close(
                context.face_offset_basis_sphere_payload(offset_face)?,
                context.face_sphere_payload(basis_face)?,
                1.0e-12,
                "direct sphere offset basis payload",
            )?,
            SurfaceKind::Torus => assert_torus_payload_close(
                context.face_offset_basis_torus_payload(offset_face)?,
                context.face_torus_payload(basis_face)?,
                1.0e-12,
                "direct torus offset basis payload",
            )?,
            _ => unreachable!("direct analytic offset test only covers analytic bases"),
        }
    }

    let extrusion_source_face = find_first_face_by_kind(&kernel, &prism, SurfaceKind::Extrusion)?;
    let extrusion_direct_offset_face = context.make_offset_surface_face(
        &extrusion_source_face,
        OffsetParams {
            offset: 1.25,
            tolerance: 1.0e-4,
        },
    )?;
    let extrusion_offset_shape = kernel.make_offset(
        &extrusion_source_face,
        OffsetParams {
            offset: 2.5,
            tolerance: 1.0e-4,
        },
    )?;
    let revolution_source_face =
        find_first_face_by_kind(&kernel, &revolution, SurfaceKind::Revolution)?;
    let revolution_direct_offset_face = context.make_offset_surface_face(
        &revolution_source_face,
        OffsetParams {
            offset: 1.25,
            tolerance: 1.0e-4,
        },
    )?;
    let revolution_offset_shape = kernel.make_offset(
        &revolution_source_face,
        OffsetParams {
            offset: 2.5,
            tolerance: 1.0e-4,
        },
    )?;
    let extrusion_offset_face =
        find_first_face_by_kind(&kernel, &extrusion_offset_shape, SurfaceKind::Offset)?;
    let revolution_offset_face =
        find_first_face_by_kind(&kernel, &revolution_offset_shape, SurfaceKind::Offset)?;

    for (label, basis_kind, source_face, offset_face) in [
        (
            "extrusion-direct",
            SurfaceKind::Extrusion,
            &extrusion_source_face,
            &extrusion_direct_offset_face,
        ),
        (
            "revolution-direct",
            SurfaceKind::Revolution,
            &revolution_source_face,
            &revolution_direct_offset_face,
        ),
    ] {
        let offset_payload = context.face_offset_payload(offset_face)?;
        assert_eq!(offset_payload.basis_surface_kind, basis_kind);
        assert!(
            (offset_payload.offset_value - 1.25).abs() <= 1.0e-12,
            "{label} direct swept offset value mismatch"
        );

        let source_geometry = context.face_geometry(source_face)?;
        let source_ported_geometry = context
            .ported_face_geometry(source_face)?
            .ok_or_else(|| std::io::Error::other(format!("{label} missing Rust basis geometry")))?;
        assert_eq!(source_ported_geometry.kind, basis_kind);
        assert_face_geometry_close(
            source_geometry,
            source_ported_geometry,
            1.0e-12,
            &format!("{label} direct swept Rust basis geometry"),
        )?;
        assert_face_geometry_close(
            context.face_offset_basis_geometry(offset_face)?,
            source_geometry,
            1.0e-12,
            &format!("{label} direct swept offset basis geometry"),
        )?;

        let source_surface = require_ported_swept_face_surface(
            context.ported_face_surface_descriptor(source_face)?,
            basis_kind,
            &format!("{label} source basis"),
        )?;
        let offset_surface = require_ported_offset_face_surface(
            context.ported_face_surface_descriptor(offset_face)?,
            &format!("{label} direct offset basis"),
        )?;
        assert_offset_payload_close(
            offset_surface.payload,
            offset_payload,
            1.0e-12,
            &format!("{label} direct offset descriptor payload"),
        )?;
        assert_face_geometry_close(
            offset_surface.basis_geometry,
            source_geometry,
            1.0e-12,
            &format!("{label} direct offset descriptor basis geometry"),
        )?;

        match (basis_kind, source_surface, offset_surface.basis) {
            (
                SurfaceKind::Extrusion,
                PortedSweptSurface::Extrusion {
                    payload: source_payload,
                    basis_curve: source_basis_curve,
                    basis_geometry: source_basis_geometry,
                },
                PortedOffsetBasisSurface::Swept(PortedSweptSurface::Extrusion {
                    payload,
                    basis_curve,
                    basis_geometry,
                }),
            ) => {
                assert_extrusion_payload_close(
                    context.face_offset_basis_extrusion_payload(offset_face)?,
                    context.face_extrusion_payload(source_face)?,
                    1.0e-12,
                    &format!("{label} public offset basis mirrors source payload"),
                )?;
                assert_extrusion_payload_close(
                    payload,
                    source_payload,
                    1.0e-12,
                    &format!("{label} descriptor swept payload mirrors source"),
                )?;
                let public_basis_geometry =
                    context.face_offset_basis_curve_geometry(offset_face)?;
                assert_edge_geometry_close(
                    public_basis_geometry,
                    basis_geometry,
                    1.0e-12,
                    &format!("{label} public offset basis curve geometry matches descriptor"),
                )?;
                assert_edge_geometry_span_close(
                    public_basis_geometry,
                    source_basis_geometry,
                    1.0e-12,
                    &format!("{label} offset basis curve span mirrors source"),
                )?;
                assert_edge_geometry_close(
                    basis_geometry,
                    public_basis_geometry,
                    1.0e-12,
                    &format!("{label} descriptor basis curve geometry matches public query"),
                )?;
                assert_ported_curve_close(
                    basis_curve,
                    source_basis_curve,
                    1.0e-12,
                    &format!("{label} descriptor basis curve mirrors source"),
                )?;
            }
            (
                SurfaceKind::Revolution,
                PortedSweptSurface::Revolution {
                    payload: source_payload,
                    basis_curve: source_basis_curve,
                    basis_geometry: source_basis_geometry,
                },
                PortedOffsetBasisSurface::Swept(PortedSweptSurface::Revolution {
                    payload,
                    basis_curve,
                    basis_geometry,
                }),
            ) => {
                assert_revolution_payload_close(
                    context.face_offset_basis_revolution_payload(offset_face)?,
                    context.face_revolution_payload(source_face)?,
                    1.0e-12,
                    &format!("{label} public offset basis mirrors source payload"),
                )?;
                assert_revolution_payload_close(
                    payload,
                    source_payload,
                    1.0e-12,
                    &format!("{label} descriptor swept payload mirrors source"),
                )?;
                let public_basis_geometry =
                    context.face_offset_basis_curve_geometry(offset_face)?;
                assert_edge_geometry_close(
                    public_basis_geometry,
                    basis_geometry,
                    1.0e-12,
                    &format!("{label} public offset basis curve geometry matches descriptor"),
                )?;
                assert_edge_geometry_span_close(
                    public_basis_geometry,
                    source_basis_geometry,
                    1.0e-12,
                    &format!("{label} offset basis curve span mirrors source"),
                )?;
                assert_edge_geometry_close(
                    basis_geometry,
                    public_basis_geometry,
                    1.0e-12,
                    &format!("{label} descriptor basis curve geometry matches public query"),
                )?;
                assert_ported_curve_close(
                    basis_curve,
                    source_basis_curve,
                    1.0e-12,
                    &format!("{label} descriptor basis curve mirrors source"),
                )?;
            }
            (expected, source, basis) => {
                return Err(std::io::Error::other(format!(
                    "unexpected {label} direct swept metadata: expected {expected:?}, source {source:?}, basis {basis:?}"
                ))
                .into())
            }
        }
    }

    for (label, basis_kind, source_face, offset_face) in [
        (
            "extrusion",
            SurfaceKind::Extrusion,
            &extrusion_source_face,
            &extrusion_offset_face,
        ),
        (
            "revolution",
            SurfaceKind::Revolution,
            &revolution_source_face,
            &revolution_offset_face,
        ),
    ] {
        assert_swept_offset_basis_mirrors_source(
            context,
            label,
            basis_kind,
            source_face,
            offset_face,
            2.5,
        )?;
    }

    for (label, basis_kind, offset_face) in [
        ("plane", SurfaceKind::Plane, plane_offset_face),
        ("cylinder", SurfaceKind::Cylinder, cylinder_offset_face),
        ("cone", SurfaceKind::Cone, cone_offset_face),
        ("sphere", SurfaceKind::Sphere, sphere_offset_face),
        ("torus", SurfaceKind::Torus, torus_offset_face),
        ("extrusion", SurfaceKind::Extrusion, extrusion_offset_face),
        (
            "extrusion-direct",
            SurfaceKind::Extrusion,
            extrusion_direct_offset_face,
        ),
        (
            "revolution",
            SurfaceKind::Revolution,
            revolution_offset_face,
        ),
        (
            "revolution-direct",
            SurfaceKind::Revolution,
            revolution_direct_offset_face,
        ),
    ] {
        assert!(
            offset_face.has_rust_offset_surface_face_metadata(),
            "{label} offset face geometry should be backed by retained Rust metadata before ported geometry is queried"
        );
        let public_geometry = context.face_geometry(&offset_face)?;
        let ported_geometry = context
            .ported_face_geometry(&offset_face)?
            .ok_or_else(|| std::io::Error::other(format!("{label} missing Rust face geometry")))?;
        let occt_geometry = context.face_geometry_occt(&offset_face)?;
        assert_eq!(public_geometry.kind, SurfaceKind::Offset);
        assert_face_geometry_close(
            public_geometry,
            ported_geometry,
            1.0e-12,
            &format!("{label} public offset geometry"),
        )?;
        assert_face_geometry_close(
            public_geometry,
            occt_geometry,
            1.0e-12,
            &format!("{label} offset occt geometry"),
        )?;
        if basis_kind == SurfaceKind::Revolution {
            assert!(
                !ported_geometry.is_v_closed,
                "{label} revolution offset geometry should preserve OCCT's non-closed offset V axis"
            );
        }
        if label == "revolution-direct" {
            assert!(
                !ported_geometry.is_u_periodic && ported_geometry.u_period.abs() <= 1.0e-12,
                "{label} direct rectangular-trimmed offset geometry should not expose a partial revolution as U-periodic"
            );
        }

        let offset_surface = require_ported_offset_face_surface(
            context.ported_face_surface_descriptor(&offset_face)?,
            &format!("{label} offset public payload"),
        )?;
        let offset_payload = context.face_offset_payload(&offset_face)?;
        let offset_payload_occt = context.face_offset_payload_occt(&offset_face)?;

        assert_eq!(offset_payload.basis_surface_kind, basis_kind);
        assert_offset_payload_close(
            offset_payload,
            offset_surface.payload,
            1.0e-12,
            &format!("{label} offset descriptor payload"),
        )?;
        assert_offset_payload_close(
            offset_payload,
            offset_payload_occt,
            1.0e-12,
            &format!("{label} offset occt payload"),
        )?;

        let basis_geometry = context.face_offset_basis_geometry(&offset_face)?;
        let basis_geometry_occt = context.face_offset_basis_geometry_occt(&offset_face)?;
        assert_face_geometry_close(
            basis_geometry,
            basis_geometry_occt,
            1.0e-12,
            &format!("{label} basis geometry"),
        )?;
        assert_face_geometry_close(
            basis_geometry,
            offset_surface.basis_geometry,
            1.0e-12,
            &format!("{label} descriptor basis geometry"),
        )?;

        match (basis_kind, offset_surface.basis) {
            (
                SurfaceKind::Plane,
                PortedOffsetBasisSurface::Analytic(PortedSurface::Plane(payload)),
            ) => {
                let public_payload = context.face_offset_basis_plane_payload(&offset_face)?;
                let public_payload_occt =
                    context.face_offset_basis_plane_payload_occt(&offset_face)?;
                assert_plane_payload_close(
                    public_payload,
                    payload,
                    1.0e-12,
                    "plane offset-basis descriptor payload",
                )?;
                assert_plane_payload_close(
                    public_payload,
                    public_payload_occt,
                    1.0e-12,
                    "plane offset-basis occt payload",
                )?;
                let error = context
                    .face_offset_basis_cylinder_payload(&offset_face)
                    .expect_err(
                        "plane offset should reject cylinder basis payload requests in Rust",
                    );
                assert!(error.to_string().contains(
                    "requested Cylinder offset-basis payload for ported Plane offset basis"
                ));
                assert_analytic_offset_basis_rejects_curve_queries(
                    context,
                    &offset_face,
                    SurfaceKind::Plane,
                    label,
                )?;
            }
            (
                SurfaceKind::Cylinder,
                PortedOffsetBasisSurface::Analytic(PortedSurface::Cylinder(payload)),
            ) => {
                let public_payload = context.face_offset_basis_cylinder_payload(&offset_face)?;
                let public_payload_occt =
                    context.face_offset_basis_cylinder_payload_occt(&offset_face)?;
                assert_cylinder_payload_close(
                    public_payload,
                    payload,
                    1.0e-12,
                    "cylinder offset-basis descriptor payload",
                )?;
                assert_cylinder_payload_close(
                    public_payload,
                    public_payload_occt,
                    1.0e-12,
                    "cylinder offset-basis occt payload",
                )?;
                let error = context
                    .face_offset_basis_plane_payload(&offset_face)
                    .expect_err(
                        "cylinder offset should reject plane basis payload requests in Rust",
                    );
                assert!(error.to_string().contains(
                    "requested Plane offset-basis payload for ported Cylinder offset basis"
                ));
                assert_analytic_offset_basis_rejects_curve_queries(
                    context,
                    &offset_face,
                    SurfaceKind::Cylinder,
                    label,
                )?;
            }
            (
                SurfaceKind::Cone,
                PortedOffsetBasisSurface::Analytic(PortedSurface::Cone(payload)),
            ) => {
                let public_payload = context.face_offset_basis_cone_payload(&offset_face)?;
                let public_payload_occt =
                    context.face_offset_basis_cone_payload_occt(&offset_face)?;
                assert_cone_payload_close(
                    public_payload,
                    payload,
                    1.0e-12,
                    "cone offset-basis descriptor payload",
                )?;
                assert_cone_payload_close(
                    public_payload,
                    public_payload_occt,
                    1.0e-12,
                    "cone offset-basis occt payload",
                )?;
                assert_analytic_offset_basis_rejects_curve_queries(
                    context,
                    &offset_face,
                    SurfaceKind::Cone,
                    label,
                )?;
            }
            (
                SurfaceKind::Sphere,
                PortedOffsetBasisSurface::Analytic(PortedSurface::Sphere(payload)),
            ) => {
                let public_payload = context.face_offset_basis_sphere_payload(&offset_face)?;
                let public_payload_occt =
                    context.face_offset_basis_sphere_payload_occt(&offset_face)?;
                assert_sphere_payload_close(
                    public_payload,
                    payload,
                    1.0e-12,
                    "sphere offset-basis descriptor payload",
                )?;
                assert_sphere_payload_close(
                    public_payload,
                    public_payload_occt,
                    1.0e-12,
                    "sphere offset-basis occt payload",
                )?;
                assert_analytic_offset_basis_rejects_curve_queries(
                    context,
                    &offset_face,
                    SurfaceKind::Sphere,
                    label,
                )?;
            }
            (
                SurfaceKind::Torus,
                PortedOffsetBasisSurface::Analytic(PortedSurface::Torus(payload)),
            ) => {
                let public_payload = context.face_offset_basis_torus_payload(&offset_face)?;
                let public_payload_occt =
                    context.face_offset_basis_torus_payload_occt(&offset_face)?;
                assert_torus_payload_close(
                    public_payload,
                    payload,
                    1.0e-12,
                    "torus offset-basis descriptor payload",
                )?;
                assert_torus_payload_close(
                    public_payload,
                    public_payload_occt,
                    1.0e-12,
                    "torus offset-basis occt payload",
                )?;
                assert_analytic_offset_basis_rejects_curve_queries(
                    context,
                    &offset_face,
                    SurfaceKind::Torus,
                    label,
                )?;
            }
            (
                SurfaceKind::Extrusion,
                PortedOffsetBasisSurface::Swept(PortedSweptSurface::Extrusion {
                    payload: descriptor_payload,
                    basis_curve,
                    basis_geometry,
                }),
            ) => {
                assert_offset_swept_basis_curve_close(
                    context,
                    &offset_face,
                    basis_curve,
                    basis_geometry,
                    label,
                )?;
                let payload = context.face_offset_basis_extrusion_payload(&offset_face)?;
                let payload_occt =
                    context.face_offset_basis_extrusion_payload_occt(&offset_face)?;
                assert_extrusion_payload_close(
                    payload,
                    payload_occt,
                    1.0e-12,
                    "extrusion offset-basis occt payload",
                )?;
                assert_extrusion_payload_close(
                    payload,
                    descriptor_payload,
                    1.0e-12,
                    "extrusion offset-basis descriptor payload",
                )?;
                let error = context
                    .face_offset_basis_revolution_payload(&offset_face)
                    .expect_err(
                        "extrusion offset should reject revolution basis payload requests in Rust",
                    );
                assert!(error.to_string().contains(
                    "requested Revolution offset-basis payload for ported Extrusion offset basis"
                ));
                let error = context
                    .face_offset_basis_plane_payload(&offset_face)
                    .expect_err(
                        "extrusion offset should reject plane basis payload requests in Rust",
                    );
                assert!(error.to_string().contains(
                    "requested Plane offset-basis payload for ported Extrusion offset basis"
                ));
            }
            (
                SurfaceKind::Revolution,
                PortedOffsetBasisSurface::Swept(PortedSweptSurface::Revolution {
                    payload: descriptor_payload,
                    basis_curve,
                    basis_geometry,
                }),
            ) => {
                assert_offset_swept_basis_curve_close(
                    context,
                    &offset_face,
                    basis_curve,
                    basis_geometry,
                    label,
                )?;
                let payload = context.face_offset_basis_revolution_payload(&offset_face)?;
                let payload_occt =
                    context.face_offset_basis_revolution_payload_occt(&offset_face)?;
                assert_revolution_payload_close(
                    payload,
                    payload_occt,
                    1.0e-12,
                    "revolution offset-basis occt payload",
                )?;
                assert_revolution_payload_close(
                    payload,
                    descriptor_payload,
                    1.0e-12,
                    "revolution offset-basis descriptor payload",
                )?;
                let error = context
                    .face_offset_basis_extrusion_payload(&offset_face)
                    .expect_err(
                        "revolution offset should reject extrusion basis payload requests in Rust",
                    );
                assert!(error.to_string().contains(
                    "requested Extrusion offset-basis payload for ported Revolution offset basis"
                ));
                let error = context
                    .face_offset_basis_cylinder_payload(&offset_face)
                    .expect_err(
                        "revolution offset should reject cylinder basis payload requests in Rust",
                    );
                assert!(error.to_string().contains(
                    "requested Cylinder offset-basis payload for ported Revolution offset basis"
                ));
            }
            (expected, basis) => {
                return Err(std::io::Error::other(format!(
                "unexpected {label} offset basis descriptor: expected {expected:?}, got {basis:?}"
            ))
                .into())
            }
        }
        let orientation = context.shape_orientation(&offset_face)?;
        let uv_t = [0.37, 0.61];
        let rust_sample = offset_surface.sample_normalized_with_orientation(uv_t, orientation);
        let occt_sample = context.face_sample_normalized_occt(&offset_face, uv_t)?;
        assert_vec3_close(
            rust_sample.position,
            occt_sample.position,
            1.0e-6,
            &format!("{label} offset descriptor sample position"),
        )?;
        assert_vec3_close(
            rust_sample.normal,
            occt_sample.normal,
            1.0e-6,
            &format!("{label} offset descriptor sample normal"),
        )?;
    }

    Ok(())
}

#[test]
fn ported_swept_surface_sampling_matches_occt() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let kernel = ModelKernel::new()?;

    let ellipse_edge = kernel.make_ellipse_edge(EllipseEdgeParams {
        origin: [30.0, 0.0, 0.0],
        axis: [0.0, 1.0, 0.0],
        x_direction: [1.0, 0.0, 0.0],
        major_radius: 10.0,
        minor_radius: 6.0,
    })?;
    let prism = kernel.make_prism(
        &ellipse_edge,
        PrismParams {
            direction: [0.0, 24.0, 0.0],
        },
    )?;
    let revolution = kernel.make_revolution(
        &ellipse_edge,
        RevolutionParams {
            origin: [0.0, 0.0, 0.0],
            axis: [0.0, 0.0, 1.0],
            angle_radians: PI,
        },
    )?;
    let source_extrusion_face = find_first_face_by_kind(&kernel, &prism, SurfaceKind::Extrusion)?;
    let face_source_prism = kernel.make_prism(
        &source_extrusion_face,
        PrismParams {
            direction: [7.0, 0.0, 11.0],
        },
    )?;
    let face_source_revolution = kernel.make_revolution(
        &source_extrusion_face,
        RevolutionParams {
            origin: [0.0, 0.0, 0.0],
            axis: [0.0, 0.0, 1.0],
            angle_radians: PI,
        },
    )?;
    assert!(
        face_source_revolution
            .rust_multi_face_swept_source_count()
            .unwrap_or(0)
            > 0,
        "face-source revolution should retain Rust swept side-face seeds"
    );
    let face_source_prism_faces = kernel
        .context()
        .subshapes(&face_source_prism, ShapeKind::Face)?;
    assert!(
        face_source_prism_faces
            .iter()
            .all(|face| !face.has_rust_swept_surface_face_metadata()),
        "face-source prism caps and analytic side faces should not receive swept metadata"
    );

    let prism_step = support::export_kernel_shape(
        &kernel,
        &prism,
        "ported_geometry_workflows",
        "ported_prism_sample_shell",
    )?;
    let revolution_step = support::export_kernel_shape(
        &kernel,
        &revolution,
        "ported_geometry_workflows",
        "ported_revolution_sample_shell",
    )?;

    for (shape, kind) in [
        (&prism, SurfaceKind::Extrusion),
        (&revolution, SurfaceKind::Revolution),
    ] {
        let occt_face = find_first_face_by_kind(&kernel, shape, kind)?;
        assert!(
            occt_face.has_rust_swept_surface_face_metadata(),
            "constructor-owned {kind:?} face should carry a Rust UV seed"
        );
        let geometry = kernel.context().face_geometry(&occt_face)?;
        let geometry_occt = kernel.context().face_geometry_occt(&occt_face)?;
        let ported_geometry = kernel
            .context()
            .ported_face_geometry(&occt_face)?
            .ok_or_else(|| {
                std::io::Error::other(format!("expected Rust {:?} face geometry", kind))
            })?;
        assert_face_geometry_close(
            geometry,
            ported_geometry,
            1.0e-12,
            &format!("{kind:?} ported geometry"),
        )?;
        assert_face_geometry_close(
            geometry,
            geometry_occt,
            1.0e-12,
            &format!("{kind:?} occt geometry"),
        )?;
        let descriptor = kernel
            .context()
            .ported_face_surface_descriptor(&occt_face)?
            .ok_or_else(|| {
                std::io::Error::other(format!("expected Rust {:?} face descriptor", kind))
            })?;
        match (kind, descriptor) {
            (
                SurfaceKind::Extrusion,
                PortedFaceSurface::Swept(PortedSweptSurface::Extrusion { .. }),
            )
            | (
                SurfaceKind::Revolution,
                PortedFaceSurface::Swept(PortedSweptSurface::Revolution { .. }),
            ) => {}
            (_, descriptor) => {
                return Err(std::io::Error::other(format!(
                    "unexpected Rust {:?} face descriptor: {descriptor:?}",
                    kind
                ))
                .into())
            }
        }

        for uv_t in [[0.5, 0.5], [0.2, 0.7]] {
            let rust_sample = kernel
                .context()
                .ported_face_sample_normalized(&occt_face, uv_t)?
                .ok_or_else(|| {
                    std::io::Error::other(format!("expected a ported {:?} face sample", kind))
                })?;
            let context_sample = kernel.context().face_sample_normalized(&occt_face, uv_t)?;
            let occt_sample = kernel
                .context()
                .face_sample_normalized_occt(&occt_face, uv_t)?;

            assert_vec3_close(
                rust_sample.position,
                occt_sample.position,
                1.0e-6,
                &format!("{kind:?} sample position at {:?}", uv_t),
            )?;
            assert_vec3_close(
                rust_sample.normal,
                occt_sample.normal,
                1.0e-6,
                &format!("{kind:?} sample normal at {:?}", uv_t),
            )?;
            assert_vec3_close(
                context_sample.position,
                rust_sample.position,
                1.0e-12,
                &format!("{kind:?} public sample position at {:?}", uv_t),
            )?;
            assert_vec3_close(
                context_sample.normal,
                rust_sample.normal,
                1.0e-12,
                &format!("{kind:?} public sample normal at {:?}", uv_t),
            )?;
        }
    }

    for (shape, kind, label) in [(
        &face_source_revolution,
        SurfaceKind::Revolution,
        "face-source revolution side face",
    )] {
        let occt_face = find_first_swept_metadata_face_by_kind(&kernel, shape, kind)?;
        let geometry = kernel.context().face_geometry(&occt_face)?;
        let geometry_occt = kernel.context().face_geometry_occt(&occt_face)?;
        let ported_geometry = kernel
            .context()
            .ported_face_geometry(&occt_face)?
            .ok_or_else(|| std::io::Error::other(format!("expected Rust {label} geometry")))?;
        assert_face_geometry_close(
            geometry,
            ported_geometry,
            1.0e-12,
            &format!("{label} ported geometry"),
        )?;
        assert_face_geometry_close(
            geometry,
            geometry_occt,
            1.0e-12,
            &format!("{label} occt geometry"),
        )?;

        let surface = kernel
            .context()
            .ported_face_surface_descriptor(&occt_face)?
            .ok_or_else(|| std::io::Error::other(format!("expected Rust {label} descriptor")))?;
        let orientation = kernel.context().shape_orientation(&occt_face)?;
        let sample =
            surface.sample_normalized_with_orientation(geometry, [0.37, 0.61], orientation);
        let public_sample = kernel
            .context()
            .ported_face_sample_normalized(&occt_face, [0.37, 0.61])?
            .ok_or_else(|| std::io::Error::other(format!("expected Rust {label} sample")))?;
        let occt_sample = kernel
            .context()
            .face_sample_normalized_occt(&occt_face, [0.37, 0.61])?;
        assert_vec3_close(
            sample.position,
            occt_sample.position,
            1.0e-6,
            &format!("{label} sample position"),
        )?;
        assert_vec3_close(
            sample.normal,
            occt_sample.normal,
            1.0e-6,
            &format!("{label} sample normal"),
        )?;
        assert_vec3_close(
            public_sample.position,
            occt_sample.position,
            1.0e-6,
            &format!("{label} public sample position"),
        )?;
        assert_vec3_close(
            public_sample.normal,
            occt_sample.normal,
            1.0e-6,
            &format!("{label} public sample normal"),
        )?;
    }

    assert!(prism_step.is_file());
    assert!(revolution_step.is_file());
    Ok(())
}

#[test]
fn ported_face_geometry_classifies_constructor_metadata_before_raw_geometry() {
    let source = include_str!("../src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs");
    let function = source
        .split("    pub fn ported_face_geometry")
        .nth(1)
        .expect("ported_face_geometry source should be present")
        .split("    pub fn ported_edge_curve")
        .next()
        .expect("ported_face_geometry source should end before ported_edge_curve");
    assert!(
        function.contains("ported_swept_surface_from_metadata_face_geometry(self, shape)?"),
        "constructor-owned swept faces should use Rust metadata before raw UV bounds"
    );
    assert!(
        function.contains("ported_analytic_surface_from_metadata_face_geometry(self, shape)?"),
        "constructor-owned analytic faces should use Rust metadata before raw UV bounds"
    );
    let swept_metadata_index = function
        .find("ported_swept_surface_from_metadata_face_geometry(self, shape)?")
        .expect("swept metadata classifier should be present");
    let analytic_metadata_index = function
        .find("ported_analytic_surface_from_metadata_face_geometry(self, shape)?")
        .expect("analytic metadata classifier should be present");
    let raw_bounds_index = function
        .find("let bounds = self.face_uv_bounds_occt(shape)?;")
        .expect("raw UV bounds fallback should be present");
    assert!(
        swept_metadata_index < raw_bounds_index,
        "swept metadata classifier should run before raw UV bounds fallback"
    );
    assert!(
        analytic_metadata_index < raw_bounds_index,
        "analytic metadata classifier should run before raw UV bounds fallback"
    );
    assert!(
        function.contains("ported_swept_face_geometry_candidate(self, shape, bounds)?"),
        "ported_face_geometry should try Rust swept geometry"
    );
    assert!(
        !function.contains("face_geometry_occt(shape)"),
        "ported_face_geometry must not call the raw face-geometry classifier"
    );
    assert!(
        !function.contains("SurfaceKind::Revolution | SurfaceKind::Extrusion"),
        "raw geometry must not classify supported swept faces"
    );
    assert!(
        !function.contains("PortedFaceSurface::Swept"),
        "raw geometry must not dispatch through the swept descriptor"
    );
    assert!(
        !function.contains("ported_face_surface_descriptor_value"),
        "raw geometry must not own swept descriptor validation"
    );

    let swept_metadata_helper = source
        .split("fn ported_swept_surface_from_metadata_face_geometry")
        .nth(1)
        .expect("swept metadata helper source should be present")
        .split("fn ported_swept_face_geometry_candidate")
        .next()
        .expect("swept metadata helper should end before generic swept candidate");
    assert!(
        !swept_metadata_helper.contains("face_uv_bounds_occt"),
        "swept metadata helper must not call the raw UV bounds seed"
    );

    let analytic_metadata_helper = source
        .split("fn ported_analytic_surface_from_metadata_face_geometry")
        .nth(1)
        .expect("analytic metadata helper source should be present")
        .split("fn ported_swept_face_geometry_candidate")
        .next()
        .expect("analytic metadata helper should end before generic swept candidate");
    assert!(
        !analytic_metadata_helper.contains("face_uv_bounds_occt"),
        "analytic metadata helper must not call the raw UV bounds seed"
    );
}

#[test]
fn ported_offset_surface_sampling_matches_occt() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let kernel = ModelKernel::new()?;

    let ellipse_edge = kernel.make_ellipse_edge(EllipseEdgeParams {
        origin: [30.0, 0.0, 0.0],
        axis: [0.0, 1.0, 0.0],
        x_direction: [1.0, 0.0, 0.0],
        major_radius: 10.0,
        minor_radius: 6.0,
    })?;
    let revolution = kernel.make_revolution(
        &ellipse_edge,
        RevolutionParams {
            origin: [0.0, 0.0, 0.0],
            axis: [0.0, 0.0, 1.0],
            angle_radians: PI,
        },
    )?;
    let revolution_face = find_first_face_by_kind(&kernel, &revolution, SurfaceKind::Revolution)?;
    let offset_face_shape = kernel.make_offset(
        &revolution_face,
        OffsetParams {
            offset: 2.5,
            tolerance: 1.0e-4,
        },
    )?;
    let offset_step = support::export_kernel_shape(
        &kernel,
        &offset_face_shape,
        "ported_geometry_workflows",
        "ported_offset_surface",
    )?;

    let offset_face = find_first_face_by_kind(&kernel, &offset_face_shape, SurfaceKind::Offset)?;
    let geometry = kernel.context().face_geometry(&offset_face)?;
    let geometry_occt = kernel.context().face_geometry_occt(&offset_face)?;
    let ported_geometry = kernel
        .context()
        .ported_face_geometry(&offset_face)?
        .ok_or_else(|| std::io::Error::other("expected Rust offset face geometry"))?;
    assert_face_geometry_close(geometry, ported_geometry, 1.0e-12, "offset ported geometry")?;
    assert_face_geometry_close(geometry, geometry_occt, 1.0e-12, "offset occt geometry")?;
    let descriptor = require_ported_offset_face_surface(
        kernel
            .context()
            .ported_face_surface_descriptor(&offset_face)?,
        "offset sample descriptor",
    )?;
    assert_eq!(
        descriptor.payload.basis_surface_kind,
        SurfaceKind::Revolution
    );
    let rust_sample = kernel
        .context()
        .ported_face_sample_normalized(&offset_face, [0.5, 0.5])?
        .ok_or_else(|| std::io::Error::other("expected ported offset surface sample"))?;
    let context_sample = kernel
        .context()
        .face_sample_normalized(&offset_face, [0.5, 0.5])?;
    let occt_sample = kernel
        .context()
        .face_sample_normalized_occt(&offset_face, [0.5, 0.5])?;

    assert_vec3_close(
        rust_sample.position,
        occt_sample.position,
        1.0e-6,
        "offset sample position",
    )?;
    assert_vec3_close(
        rust_sample.normal,
        occt_sample.normal,
        1.0e-6,
        "offset sample normal",
    )?;
    assert_vec3_close(
        context_sample.position,
        rust_sample.position,
        1.0e-12,
        "offset public sample position",
    )?;
    assert_vec3_close(
        context_sample.normal,
        rust_sample.normal,
        1.0e-12,
        "offset public sample normal",
    )?;
    assert!(offset_step.is_file());

    Ok(())
}

#[test]
fn ported_face_areas_match_occt() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let kernel = ModelKernel::new()?;

    let cut = kernel.box_with_through_hole(default_cut())?;
    let cone = kernel.make_cone(ConeParams {
        origin: [0.0, 0.0, 0.0],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        base_radius: 15.0,
        top_radius: 5.0,
        height: 30.0,
    })?;
    let sphere = kernel.make_sphere(SphereParams {
        origin: [0.0, 0.0, 0.0],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        radius: 14.0,
    })?;
    let torus = kernel.make_torus(TorusParams {
        origin: [0.0, 0.0, 0.0],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        major_radius: 25.0,
        minor_radius: 6.0,
    })?;
    let ellipse_edge = kernel.make_ellipse_edge(EllipseEdgeParams {
        origin: [30.0, 0.0, 0.0],
        axis: [0.0, 1.0, 0.0],
        x_direction: [1.0, 0.0, 0.0],
        major_radius: 10.0,
        minor_radius: 6.0,
    })?;
    let prism = kernel.make_prism(
        &ellipse_edge,
        PrismParams {
            direction: [0.0, 24.0, 0.0],
        },
    )?;
    let revolution = kernel.make_revolution(
        &ellipse_edge,
        RevolutionParams {
            origin: [0.0, 0.0, 0.0],
            axis: [0.0, 0.0, 1.0],
            angle_radians: PI,
        },
    )?;
    let revolution_face = find_first_face_by_kind(&kernel, &revolution, SurfaceKind::Revolution)?;
    let offset_face_shape = kernel.make_offset(
        &revolution_face,
        OffsetParams {
            offset: 2.5,
            tolerance: 1.0e-4,
        },
    )?;

    for (label, face, tolerance) in [
        (
            "plane",
            find_first_face_by_kind(&kernel, &cut, SurfaceKind::Plane)?,
            1.0e-6,
        ),
        (
            "cylinder",
            find_first_face_by_kind(&kernel, &cut, SurfaceKind::Cylinder)?,
            1.0e-6,
        ),
        (
            "cone",
            find_first_face_by_kind(&kernel, &cone, SurfaceKind::Cone)?,
            1.0e-6,
        ),
        (
            "sphere",
            find_first_face_by_kind(&kernel, &sphere, SurfaceKind::Sphere)?,
            1.0e-6,
        ),
        (
            "torus",
            find_first_face_by_kind(&kernel, &torus, SurfaceKind::Torus)?,
            1.0e-6,
        ),
        (
            "offset",
            find_first_face_by_kind(&kernel, &offset_face_shape, SurfaceKind::Offset)?,
            5.0e-1,
        ),
    ] {
        let rust_area = kernel
            .context()
            .ported_face_area(&face)?
            .ok_or_else(|| std::io::Error::other(format!("expected a ported {label} face area")))?;
        let occt_area = kernel.context().describe_shape_occt(&face)?.surface_area;

        assert!(
            (rust_area - occt_area).abs() <= tolerance,
            "{label} face area drifted from OCCT: rust={rust_area} occt={occt_area}"
        );
    }

    let extrusion_face = find_first_face_by_kind(&kernel, &prism, SurfaceKind::Extrusion)?;
    let rust_extrusion_area = kernel
        .context()
        .ported_face_area(&extrusion_face)?
        .ok_or_else(|| std::io::Error::other("expected a ported extrusion face area"))?;
    let expected_extrusion_area = ellipse_perimeter(10.0, 6.0) * 24.0;
    assert!(
        (rust_extrusion_area - expected_extrusion_area).abs() <= 1.0e-3,
        "extrusion face area drifted from expected integral: rust={rust_extrusion_area} expected={expected_extrusion_area}"
    );

    let rust_revolution_area = kernel
        .context()
        .ported_face_area(&revolution_face)?
        .ok_or_else(|| std::io::Error::other("expected a ported revolution face area"))?;
    let expected_revolution_area = revolved_ellipse_area(30.0, 10.0, 6.0, PI);
    assert!(
        (rust_revolution_area - expected_revolution_area).abs() <= 1.0e-2,
        "revolution face area drifted from expected integral: rust={rust_revolution_area} expected={expected_revolution_area}"
    );

    Ok(())
}

#[test]
fn ported_sweep_face_areas_match_numeric_integrals() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let kernel = ModelKernel::new()?;

    let ellipse_edge = kernel.make_ellipse_edge(EllipseEdgeParams {
        origin: [30.0, 0.0, 0.0],
        axis: [0.0, 1.0, 0.0],
        x_direction: [1.0, 0.0, 0.0],
        major_radius: 10.0,
        minor_radius: 6.0,
    })?;
    let prism = kernel.make_prism(
        &ellipse_edge,
        PrismParams {
            direction: [0.0, 24.0, 0.0],
        },
    )?;
    let revolution = kernel.make_revolution(
        &ellipse_edge,
        RevolutionParams {
            origin: [0.0, 0.0, 0.0],
            axis: [0.0, 0.0, 1.0],
            angle_radians: PI,
        },
    )?;

    let prism_step = support::export_kernel_shape(
        &kernel,
        &prism,
        "ported_geometry_workflows",
        "ported_prism_shell",
    )?;
    let revolution_step = support::export_kernel_shape(
        &kernel,
        &revolution,
        "ported_geometry_workflows",
        "ported_revolution_shell",
    )?;

    let prism_brep = kernel.brep(&prism)?;
    let revolution_brep = kernel.brep(&revolution)?;
    let prism_face = prism_brep
        .faces
        .iter()
        .find(|face| face.geometry.kind == SurfaceKind::Extrusion)
        .ok_or_else(|| std::io::Error::other("expected an extrusion face in the prism shell"))?;
    let revolution_face = revolution_brep
        .faces
        .iter()
        .find(|face| face.geometry.kind == SurfaceKind::Revolution)
        .ok_or_else(|| std::io::Error::other("expected a revolution face in the revolved shell"))?;

    let expected_prism_area = ellipse_perimeter(10.0, 6.0) * 24.0;
    let expected_revolution_area = revolved_ellipse_area(30.0, 10.0, 6.0, PI);

    assert!(
        (prism_face.area - expected_prism_area).abs() <= 1.0e-3,
        "unexpected extrusion face area: rust={} expected={}",
        prism_face.area,
        expected_prism_area
    );
    assert!(
        (prism_brep.summary.surface_area - expected_prism_area).abs() <= 1.0e-3,
        "unexpected prism shell area: rust={} expected={}",
        prism_brep.summary.surface_area,
        expected_prism_area
    );
    assert!(
        (revolution_face.area - expected_revolution_area).abs() <= 1.0e-2,
        "unexpected revolution face area: rust={} expected={}",
        revolution_face.area,
        expected_revolution_area
    );
    assert!(
        (revolution_brep.summary.surface_area - expected_revolution_area).abs() <= 1.0e-2,
        "unexpected revolution shell area: rust={} expected={}",
        revolution_brep.summary.surface_area,
        expected_revolution_area
    );

    assert!(prism_step.is_file());
    assert!(revolution_step.is_file());
    Ok(())
}
