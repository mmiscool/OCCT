use std::f64::consts::TAU;

mod support;

use lean_occt::{
    BoxParams, ConeParams, CurveKind, CylinderParams, CylindricalHoleParams, EllipseEdgeParams,
    FilletParams, HelixParams, ModelKernel, OffsetParams, PrismParams, RevolutionParams, ShapeKind,
    SphereParams, SurfaceKind, ThroughHoleCut, TorusParams,
};

fn default_box_with_hole() -> ThroughHoleCut {
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

fn vector_length(vector: [f64; 3]) -> f64 {
    vector
        .into_iter()
        .map(|value| value * value)
        .sum::<f64>()
        .sqrt()
}

fn find_first_face_by_kind(
    kernel: &ModelKernel,
    shape: &lean_occt::Shape,
    kind: SurfaceKind,
) -> Result<lean_occt::Shape, Box<dyn std::error::Error>> {
    for face in kernel.context().subshapes(shape, ShapeKind::Face)? {
        if kernel.context().face_geometry(&face)?.kind == kind {
            return Ok(face);
        }
    }
    Err(std::io::Error::other(format!("expected face with surface kind {:?}", kind)).into())
}

#[test]
fn box_with_hole_round_trip_matches_source() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let kernel = ModelKernel::new()?;
    let cut = kernel.box_with_through_hole(default_box_with_hole())?;
    let cut_step = support::export_kernel_shape(&kernel, &cut, "workflows", "box_with_hole_cut")?;
    let source_report = kernel.inspect(&cut)?;

    assert_eq!(source_report.summary.primary_kind, ShapeKind::Solid);
    assert_eq!(source_report.summary.solid_count, 1);
    assert_eq!(source_report.summary.face_count, 7);
    assert_eq!(source_report.triangle_count(), 160);
    assert_eq!(source_report.edge_segment_count(), 84);
    assert_eq!(
        source_report.topology.faces.len(),
        source_report.summary.face_count
    );

    let imported = kernel.step_round_trip_temp(&cut)?;
    let imported_report = kernel.inspect(&imported)?;

    assert_eq!(imported_report.summary.primary_kind, ShapeKind::Solid);
    assert_eq!(
        imported_report.summary.solid_count,
        source_report.summary.solid_count
    );
    assert_eq!(
        imported_report.summary.face_count,
        source_report.summary.face_count
    );
    assert_eq!(
        imported_report.triangle_count(),
        source_report.triangle_count()
    );
    assert_eq!(
        imported_report.edge_segment_count(),
        source_report.edge_segment_count()
    );
    assert!(cut_step.is_file());

    Ok(())
}

#[test]
fn retained_authoring_operations_stay_available() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let kernel = ModelKernel::new()?;

    let fillet_source = kernel.make_box(BoxParams {
        origin: [0.0, 0.0, 0.0],
        size: [40.0, 30.0, 20.0],
    })?;
    let fillet = kernel.make_fillet(
        &fillet_source,
        FilletParams {
            radius: 3.0,
            edge_index: 0,
        },
    )?;
    let fillet_step =
        support::export_kernel_shape(&kernel, &fillet, "workflows", "retained_fillet")?;
    let fillet_report = kernel.inspect(&fillet)?;
    assert_eq!(fillet_report.summary.primary_kind, ShapeKind::Solid);
    assert!(fillet_report.triangle_count() > 0);

    let offset_source = kernel.make_box(BoxParams {
        origin: [0.0, 0.0, 0.0],
        size: [30.0, 30.0, 30.0],
    })?;
    let offset = kernel.make_offset(
        &offset_source,
        OffsetParams {
            offset: 2.0,
            tolerance: 1.0e-4,
        },
    )?;
    let offset_step =
        support::export_kernel_shape(&kernel, &offset, "workflows", "retained_offset")?;
    let offset_report = kernel.inspect(&offset)?;
    assert_eq!(offset_report.summary.primary_kind, ShapeKind::Solid);
    assert!(offset_report.summary.face_count >= 6);

    let feature_source = kernel.make_box(BoxParams {
        origin: [0.0, 0.0, 0.0],
        size: [40.0, 40.0, 30.0],
    })?;
    let feature = kernel.make_cylindrical_hole(
        &feature_source,
        CylindricalHoleParams {
            origin: [20.0, 20.0, -10.0],
            axis: [0.0, 0.0, 1.0],
            radius: 6.0,
        },
    )?;
    let feature_step =
        support::export_kernel_shape(&kernel, &feature, "workflows", "retained_feature")?;
    let feature_report = kernel.inspect(&feature)?;
    assert_eq!(feature_report.summary.primary_kind, ShapeKind::Solid);
    assert!(feature_report.triangle_count() > 0);

    let helix = kernel.make_helix(HelixParams {
        origin: [0.0, 0.0, 0.0],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        radius: 20.0,
        height: 30.0,
        pitch: 10.0,
    })?;
    let helix_summary = kernel.context().describe_shape(&helix)?;
    assert_eq!(helix_summary.primary_kind, ShapeKind::Wire);
    assert_eq!(helix_summary.wire_count, 1);
    assert_eq!(helix_summary.edge_count, 3);
    assert!(fillet_step.is_file());
    assert!(offset_step.is_file());
    assert!(feature_step.is_file());

    Ok(())
}

#[test]
fn analytic_geometry_queries_cover_retained_shapes() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let kernel = ModelKernel::new()?;

    let ellipse_edge = kernel.make_ellipse_edge(EllipseEdgeParams {
        origin: [30.0, 0.0, 0.0],
        axis: [0.0, 1.0, 0.0],
        x_direction: [1.0, 0.0, 0.0],
        major_radius: 10.0,
        minor_radius: 6.0,
    })?;
    let ellipse_geometry = kernel.context().edge_geometry(&ellipse_edge)?;
    let ellipse_payload = kernel.context().edge_ellipse_payload(&ellipse_edge)?;
    assert_eq!(ellipse_geometry.kind, CurveKind::Ellipse);
    assert!((ellipse_payload.major_radius - 10.0).abs() <= 1.0e-9);
    assert!((ellipse_payload.minor_radius - 6.0).abs() <= 1.0e-9);

    let prism = kernel.make_prism(
        &ellipse_edge,
        PrismParams {
            direction: [0.0, 24.0, 0.0],
        },
    )?;
    let extrusion_face = find_first_face_by_kind(&kernel, &prism, SurfaceKind::Extrusion)?;
    let extrusion_payload = kernel.context().face_extrusion_payload(&extrusion_face)?;
    assert_eq!(extrusion_payload.basis_curve_kind, CurveKind::Ellipse);
    assert!((vector_length(extrusion_payload.direction) - 1.0).abs() <= 1.0e-9);

    let revolution = kernel.make_revolution(
        &ellipse_edge,
        RevolutionParams {
            origin: [0.0, 0.0, 0.0],
            axis: [0.0, 0.0, 1.0],
            angle_radians: TAU,
        },
    )?;
    let revolution_face = find_first_face_by_kind(&kernel, &revolution, SurfaceKind::Revolution)?;
    let revolution_payload = kernel.context().face_revolution_payload(&revolution_face)?;
    assert_eq!(revolution_payload.basis_curve_kind, CurveKind::Ellipse);
    assert!((vector_length(revolution_payload.axis_direction) - 1.0).abs() <= 1.0e-9);

    let offset_surface = kernel.make_offset(
        &revolution_face,
        OffsetParams {
            offset: 2.5,
            tolerance: 1.0e-4,
        },
    )?;
    let offset_face = find_first_face_by_kind(&kernel, &offset_surface, SurfaceKind::Offset)?;
    let offset_payload = kernel.context().face_offset_payload(&offset_face)?;
    assert_eq!(offset_payload.basis_surface_kind, SurfaceKind::Revolution);
    assert!((offset_payload.offset_value.abs() - 2.5).abs() <= 1.0e-9);

    let cone = kernel.make_cone(ConeParams {
        origin: [0.0, 0.0, 0.0],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        base_radius: 15.0,
        top_radius: 5.0,
        height: 30.0,
    })?;
    let cone_step = support::export_kernel_shape(&kernel, &cone, "workflows", "analytic_cone")?;
    let cone_face = find_first_face_by_kind(&kernel, &cone, SurfaceKind::Cone)?;
    let cone_payload = kernel.context().face_cone_payload(&cone_face)?;
    assert!((cone_payload.reference_radius - 15.0).abs() <= 1.0e-9);

    let sphere = kernel.make_sphere(SphereParams {
        origin: [0.0, 0.0, 0.0],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        radius: 14.0,
    })?;
    let sphere_step =
        support::export_kernel_shape(&kernel, &sphere, "workflows", "analytic_sphere")?;
    let sphere_face = find_first_face_by_kind(&kernel, &sphere, SurfaceKind::Sphere)?;
    let sphere_payload = kernel.context().face_sphere_payload(&sphere_face)?;
    assert!((sphere_payload.radius - 14.0).abs() <= 1.0e-9);

    let torus = kernel.make_torus(TorusParams {
        origin: [0.0, 0.0, 0.0],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        major_radius: 25.0,
        minor_radius: 6.0,
    })?;
    let torus_step = support::export_kernel_shape(&kernel, &torus, "workflows", "analytic_torus")?;
    let torus_face = find_first_face_by_kind(&kernel, &torus, SurfaceKind::Torus)?;
    let torus_payload = kernel.context().face_torus_payload(&torus_face)?;
    assert!((torus_payload.major_radius - 25.0).abs() <= 1.0e-9);
    assert!((torus_payload.minor_radius - 6.0).abs() <= 1.0e-9);
    assert!(cone_step.is_file());
    assert!(sphere_step.is_file());
    assert!(torus_step.is_file());

    Ok(())
}
