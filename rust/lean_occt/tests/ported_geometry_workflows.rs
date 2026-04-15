mod support;

use std::f64::consts::PI;

use lean_occt::{
    BoxParams, ConeParams, CurveKind, CylinderParams, EllipseEdgeParams, ModelKernel, OffsetParams,
    PortedCurve, PortedFaceSurface, PortedOffsetBasisSurface, PortedSweptSurface, PrismParams,
    RevolutionParams, Shape, ShapeKind, SphereParams, SurfaceKind, ThroughHoleCut, TorusParams,
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
        let occt_point = kernel.context().vertex_point_occt(&vertex)?;
        assert_vec3_close(
            context_point,
            occt_point,
            1.0e-12,
            &format!("vertex {index}"),
        )?;
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
        let geometry_occt = kernel.context().edge_geometry_occt(&edge)?;
        let context_endpoints = kernel.context().edge_endpoints(&edge)?;
        let occt_endpoints = kernel.context().edge_endpoints_occt(&edge)?;
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
        let occt_sample = kernel
            .context()
            .edge_sample_at_parameter_occt(&edge, parameter)?;
        let occt_normalized_sample = kernel.context().edge_sample_occt(&edge, 0.5)?;
        let occt_length = kernel.context().describe_shape_occt(&edge)?.linear_length;

        assert_edge_geometry_close(geometry, geometry_occt, 1.0e-8, label)?;
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

    for (label, face, uv_t) in [
        (
            "plane",
            find_first_face_by_kind(&kernel, &cut, SurfaceKind::Plane)?,
            [0.5, 0.5],
        ),
        (
            "extrusion",
            find_first_face_by_kind(&kernel, &prism, SurfaceKind::Extrusion)?,
            [0.2, 0.7],
        ),
        ("revolution", revolution_face, [0.2, 0.7]),
        (
            "offset",
            find_first_face_by_kind(&kernel, &offset_face_shape, SurfaceKind::Offset)?,
            [0.5, 0.5],
        ),
    ] {
        let geometry = kernel.context().face_geometry(&face)?;
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
            ("offset", PortedFaceSurface::Offset(surface)) => {
                assert_eq!(surface.payload.basis_surface_kind, SurfaceKind::Revolution);
                assert!(matches!(
                    surface.basis,
                    PortedOffsetBasisSurface::Swept(PortedSweptSurface::Revolution { .. })
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
    let extrusion_payload = context.face_extrusion_payload(&extrusion_face)?;
    let extrusion_payload_occt = context.face_extrusion_payload_occt(&extrusion_face)?;

    assert_eq!(
        extrusion_payload.basis_curve_kind,
        extrusion_payload_occt.basis_curve_kind
    );
    assert_vec3_close(
        extrusion_payload.direction,
        extrusion_payload_occt.direction,
        1.0e-12,
        "extrusion payload direction",
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
    let revolution_payload = context.face_revolution_payload(&revolution_face)?;
    let revolution_payload_occt = context.face_revolution_payload_occt(&revolution_face)?;

    assert_eq!(
        revolution_payload.basis_curve_kind,
        revolution_payload_occt.basis_curve_kind
    );
    assert_vec3_close(
        revolution_payload.axis_origin,
        revolution_payload_occt.axis_origin,
        1.0e-12,
        "revolution payload axis origin",
    )?;
    assert_vec3_close(
        revolution_payload.axis_direction,
        revolution_payload_occt.axis_direction,
        1.0e-12,
        "revolution payload axis direction",
    )?;

    let offset_shape = kernel.make_offset(
        &revolution_face,
        OffsetParams {
            offset: 2.5,
            tolerance: 1.0e-4,
        },
    )?;
    let offset_face = find_first_face_by_kind(&kernel, &offset_shape, SurfaceKind::Offset)?;
    let offset_payload = context.face_offset_payload(&offset_face)?;
    let offset_payload_occt = context.face_offset_payload_occt(&offset_face)?;

    assert_eq!(
        offset_payload.basis_surface_kind,
        offset_payload_occt.basis_surface_kind
    );
    assert!(
        (offset_payload.offset_value - offset_payload_occt.offset_value).abs() <= 1.0e-12,
        "offset payload mismatch: rust={offset_payload:?} occt={offset_payload_occt:?}"
    );

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
    let line_payload = context.edge_line_payload(&line_edge)?;
    let line_payload_occt = context.edge_line_payload_occt(&line_edge)?;
    assert_vec3_close(
        line_payload.origin,
        line_payload_occt.origin,
        1.0e-12,
        "line payload origin",
    )?;
    assert_vec3_close(
        line_payload.direction,
        line_payload_occt.direction,
        1.0e-12,
        "line payload direction",
    )?;

    let circle_edge = find_first_edge_by_kind(&kernel, &cut, CurveKind::Circle)?;
    let circle_payload = context.edge_circle_payload(&circle_edge)?;
    let circle_payload_occt = context.edge_circle_payload_occt(&circle_edge)?;
    assert_vec3_close(
        circle_payload.center,
        circle_payload_occt.center,
        1.0e-12,
        "circle payload center",
    )?;
    assert_vec3_close(
        circle_payload.normal,
        circle_payload_occt.normal,
        1.0e-12,
        "circle payload normal",
    )?;
    assert_vec3_close(
        circle_payload.x_direction,
        circle_payload_occt.x_direction,
        1.0e-12,
        "circle payload x direction",
    )?;
    assert_vec3_close(
        circle_payload.y_direction,
        circle_payload_occt.y_direction,
        1.0e-12,
        "circle payload y direction",
    )?;
    assert_scalar_close(
        circle_payload.radius,
        circle_payload_occt.radius,
        1.0e-12,
        "circle payload radius",
    )?;

    let ellipse_payload = context.edge_ellipse_payload(&ellipse_edge)?;
    let ellipse_payload_occt = context.edge_ellipse_payload_occt(&ellipse_edge)?;
    assert_vec3_close(
        ellipse_payload.center,
        ellipse_payload_occt.center,
        1.0e-12,
        "ellipse payload center",
    )?;
    assert_vec3_close(
        ellipse_payload.normal,
        ellipse_payload_occt.normal,
        1.0e-12,
        "ellipse payload normal",
    )?;
    assert_vec3_close(
        ellipse_payload.x_direction,
        ellipse_payload_occt.x_direction,
        1.0e-12,
        "ellipse payload x direction",
    )?;
    assert_vec3_close(
        ellipse_payload.y_direction,
        ellipse_payload_occt.y_direction,
        1.0e-12,
        "ellipse payload y direction",
    )?;
    assert_scalar_close(
        ellipse_payload.major_radius,
        ellipse_payload_occt.major_radius,
        1.0e-12,
        "ellipse payload major radius",
    )?;
    assert_scalar_close(
        ellipse_payload.minor_radius,
        ellipse_payload_occt.minor_radius,
        1.0e-12,
        "ellipse payload minor radius",
    )?;

    let plane_face = find_first_face_by_kind(&kernel, &cut, SurfaceKind::Plane)?;
    let plane_payload = context.face_plane_payload(&plane_face)?;
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
    let cylinder_payload = context.face_cylinder_payload(&cylinder_face)?;
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
    let cone_payload = context.face_cone_payload(&cone_face)?;
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
    let sphere_payload = context.face_sphere_payload(&sphere_face)?;
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
    let torus_payload = context.face_torus_payload(&torus_face)?;
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
fn public_swept_offset_basis_queries_match_occt() -> Result<(), Box<dyn std::error::Error>> {
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

    let context = kernel.context();
    for (label, basis_kind, source_face) in [
        (
            "extrusion",
            SurfaceKind::Extrusion,
            find_first_face_by_kind(&kernel, &prism, SurfaceKind::Extrusion)?,
        ),
        (
            "revolution",
            SurfaceKind::Revolution,
            find_first_face_by_kind(&kernel, &revolution, SurfaceKind::Revolution)?,
        ),
    ] {
        let offset_shape = kernel.make_offset(
            &source_face,
            OffsetParams {
                offset: 2.5,
                tolerance: 1.0e-4,
            },
        )?;
        let offset_face = find_first_face_by_kind(&kernel, &offset_shape, SurfaceKind::Offset)?;
        let offset_payload = context.face_offset_payload(&offset_face)?;

        assert_eq!(offset_payload.basis_surface_kind, basis_kind);

        let basis_geometry = context.face_offset_basis_geometry(&offset_face)?;
        let basis_geometry_occt = context.face_offset_basis_geometry_occt(&offset_face)?;
        assert_face_geometry_close(
            basis_geometry,
            basis_geometry_occt,
            1.0e-12,
            &format!("{label} basis geometry"),
        )?;

        let basis_curve_geometry = context.face_offset_basis_curve_geometry(&offset_face)?;
        let basis_curve_geometry_occt =
            context.face_offset_basis_curve_geometry_occt(&offset_face)?;
        assert_edge_geometry_close(
            basis_curve_geometry,
            basis_curve_geometry_occt,
            1.0e-12,
            &format!("{label} basis curve geometry"),
        )?;
        assert_eq!(basis_curve_geometry.kind, CurveKind::Ellipse);

        let ellipse_payload = context.face_offset_basis_curve_ellipse_payload(&offset_face)?;
        let ellipse_payload_occt =
            context.face_offset_basis_curve_ellipse_payload_occt(&offset_face)?;
        assert_vec3_close(
            ellipse_payload.center,
            ellipse_payload_occt.center,
            1.0e-12,
            &format!("{label} basis ellipse center"),
        )?;
        assert_vec3_close(
            ellipse_payload.normal,
            ellipse_payload_occt.normal,
            1.0e-12,
            &format!("{label} basis ellipse normal"),
        )?;
        assert_vec3_close(
            ellipse_payload.x_direction,
            ellipse_payload_occt.x_direction,
            1.0e-12,
            &format!("{label} basis ellipse x direction"),
        )?;
        assert_vec3_close(
            ellipse_payload.y_direction,
            ellipse_payload_occt.y_direction,
            1.0e-12,
            &format!("{label} basis ellipse y direction"),
        )?;
        assert_scalar_close(
            ellipse_payload.major_radius,
            ellipse_payload_occt.major_radius,
            1.0e-12,
            &format!("{label} basis ellipse major radius"),
        )?;
        assert_scalar_close(
            ellipse_payload.minor_radius,
            ellipse_payload_occt.minor_radius,
            1.0e-12,
            &format!("{label} basis ellipse minor radius"),
        )?;

        match basis_kind {
            SurfaceKind::Extrusion => {
                let payload = context.face_offset_basis_extrusion_payload(&offset_face)?;
                let payload_occt =
                    context.face_offset_basis_extrusion_payload_occt(&offset_face)?;
                assert_eq!(payload.basis_curve_kind, CurveKind::Ellipse);
                assert_eq!(payload.basis_curve_kind, payload_occt.basis_curve_kind);
                assert_vec3_close(
                    payload.direction,
                    payload_occt.direction,
                    1.0e-12,
                    "extrusion basis direction",
                )?;
            }
            SurfaceKind::Revolution => {
                let payload = context.face_offset_basis_revolution_payload(&offset_face)?;
                let payload_occt =
                    context.face_offset_basis_revolution_payload_occt(&offset_face)?;
                assert_eq!(payload.basis_curve_kind, CurveKind::Ellipse);
                assert_eq!(payload.basis_curve_kind, payload_occt.basis_curve_kind);
                assert_vec3_close(
                    payload.axis_origin,
                    payload_occt.axis_origin,
                    1.0e-12,
                    "revolution basis axis origin",
                )?;
                assert_vec3_close(
                    payload.axis_direction,
                    payload_occt.axis_direction,
                    1.0e-12,
                    "revolution basis axis direction",
                )?;
            }
            _ => unreachable!("unexpected swept basis kind"),
        }
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
        for uv_t in [[0.5, 0.5], [0.2, 0.7]] {
            let rust_sample = kernel
                .context()
                .ported_face_sample_normalized(&occt_face, uv_t)?
                .ok_or_else(|| {
                    std::io::Error::other(format!("expected a ported {:?} face sample", kind))
                })?;
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
        }
    }

    assert!(prism_step.is_file());
    assert!(revolution_step.is_file());
    Ok(())
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
    let rust_sample = kernel
        .context()
        .ported_face_sample_normalized(&offset_face, [0.5, 0.5])?
        .ok_or_else(|| std::io::Error::other("expected ported offset surface sample"))?;
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
