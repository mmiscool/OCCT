mod support;

use std::f64::consts::PI;

use lean_occt::{
    BoxParams, ConeParams, CurveKind, CylinderParams, EllipseEdgeParams, ModelKernel, PrismParams,
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
        let rust_length = kernel
            .context()
            .ported_edge_length(&edge)?
            .ok_or_else(|| std::io::Error::other(format!("expected ported {label} edge length")))?;
        let occt_sample = kernel
            .context()
            .edge_sample_at_parameter(&edge, parameter)?;
        let occt_length = kernel.context().describe_shape(&edge)?.linear_length;

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
        let rust_sample = kernel
            .context()
            .ported_face_sample_normalized(&face, [0.5, 0.5])?
            .ok_or_else(|| std::io::Error::other(format!("expected ported {label} surface")))?;
        let occt_sample = kernel.context().face_sample_normalized(&face, [0.5, 0.5])?;

        assert_vec3_close(rust_sample.position, occt_sample.position, 1.0e-7, label)?;
        assert_vec3_close(rust_sample.normal, occt_sample.normal, 1.0e-7, label)?;
    }

    assert!(cut_step.is_file());
    assert!(cone_step.is_file());
    assert!(sphere_step.is_file());
    assert!(torus_step.is_file());
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
        let brep = kernel.brep(shape)?;
        let rust_face = brep
            .faces
            .iter()
            .find(|face| face.geometry.kind == kind)
            .ok_or_else(|| std::io::Error::other(format!("expected a {:?} face in brep", kind)))?;
        let occt_face = find_first_face_by_kind(&kernel, shape, kind)?;
        let occt_sample = kernel
            .context()
            .face_sample_normalized(&occt_face, [0.5, 0.5])?;

        assert_vec3_close(
            rust_face.sample.position,
            occt_sample.position,
            1.0e-6,
            &format!("{kind:?} sample position"),
        )?;
        assert_vec3_close(
            rust_face.sample.normal,
            occt_sample.normal,
            1.0e-6,
            &format!("{kind:?} sample normal"),
        )?;
    }

    assert!(prism_step.is_file());
    assert!(revolution_step.is_file());
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
