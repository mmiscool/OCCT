use std::f64::consts::TAU;

mod support;

use lean_occt::{
    BoxParams, ConeParams, CurveKind, CylinderParams, EllipseEdgeParams, ModelDocument,
    OperationRecord, PrismParams, RevolutionParams, ShapeKind, SphereParams, SurfaceKind,
    ThroughHoleCut, TorusParams,
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

fn find_first_face_by_kind(
    document: &ModelDocument,
    name: &str,
    kind: SurfaceKind,
) -> Result<lean_occt::Shape, Box<dyn std::error::Error>> {
    for face in document
        .kernel()
        .context()
        .subshapes(document.shape(name)?, ShapeKind::Face)?
    {
        if document.kernel().context().face_geometry(&face)?.kind == kind {
            return Ok(face);
        }
    }
    Err(std::io::Error::other(format!(
        "expected shape '{name}' to contain a {:?} face",
        kind
    ))
    .into())
}

#[test]
fn document_tracks_named_boolean_and_round_trip_workflow() -> Result<(), Box<dyn std::error::Error>>
{
    let _guard = support::test_guard();
    let mut document = ModelDocument::new()?;
    document.insert_box("base", default_cut().box_params)?;
    document.insert_cylinder("tool", default_cut().tool_params)?;
    document.cut("cut", "base", "tool")?;
    document.step_round_trip("cut_step", "cut")?;

    let cut_report = document.report("cut")?;
    let step_report = document.report("cut_step")?;

    assert!(document.contains_shape("base"));
    assert!(document.contains_shape("tool"));
    assert!(document.contains_shape("cut"));
    assert!(document.contains_shape("cut_step"));
    assert_eq!(cut_report.summary.primary_kind, ShapeKind::Solid);
    assert_eq!(step_report.summary.primary_kind, ShapeKind::Solid);
    assert_eq!(cut_report.summary.face_count, 7);
    assert_eq!(
        step_report.summary.face_count,
        cut_report.summary.face_count
    );
    assert_eq!(cut_report.triangle_count(), 160);
    assert_eq!(step_report.triangle_count(), cut_report.triangle_count());

    let names = document.shape_names().collect::<Vec<_>>();
    assert_eq!(names, vec!["base", "tool", "cut", "cut_step"]);
    assert!(matches!(
        &document.history()[0],
        OperationRecord::AddBox { output, .. } if output == "base"
    ));
    assert!(matches!(
        &document.history()[2],
        OperationRecord::Cut { output, lhs, rhs }
            if output == "cut" && lhs == "base" && rhs == "tool"
    ));
    assert!(matches!(
        &document.history()[3],
        OperationRecord::StepRoundTrip { output, input }
            if output == "cut_step" && input == "cut"
    ));
    let cut_step = support::export_document_shape(
        &mut document,
        "cut_step",
        "document_workflows",
        "cut_step",
    )?;
    assert!(cut_step.is_file());

    Ok(())
}

#[test]
fn document_runs_analytic_shape_pipeline() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let mut document = ModelDocument::new()?;
    document.insert_ellipse_edge(
        "ellipse",
        EllipseEdgeParams {
            origin: [30.0, 0.0, 0.0],
            axis: [0.0, 1.0, 0.0],
            x_direction: [1.0, 0.0, 0.0],
            major_radius: 10.0,
            minor_radius: 6.0,
        },
    )?;
    document.prism(
        "prism",
        "ellipse",
        PrismParams {
            direction: [0.0, 24.0, 0.0],
        },
    )?;
    document.revolution(
        "revolved",
        "ellipse",
        RevolutionParams {
            origin: [0.0, 0.0, 0.0],
            axis: [0.0, 0.0, 1.0],
            angle_radians: TAU,
        },
    )?;
    document.insert_cone(
        "cone",
        ConeParams {
            origin: [0.0, 0.0, 0.0],
            axis: [0.0, 0.0, 1.0],
            x_direction: [1.0, 0.0, 0.0],
            base_radius: 15.0,
            top_radius: 5.0,
            height: 30.0,
        },
    )?;
    document.insert_sphere(
        "sphere",
        SphereParams {
            origin: [0.0, 0.0, 0.0],
            axis: [0.0, 0.0, 1.0],
            x_direction: [1.0, 0.0, 0.0],
            radius: 14.0,
        },
    )?;
    document.insert_torus(
        "torus",
        TorusParams {
            origin: [0.0, 0.0, 0.0],
            axis: [0.0, 0.0, 1.0],
            x_direction: [1.0, 0.0, 0.0],
            major_radius: 25.0,
            minor_radius: 6.0,
        },
    )?;
    let prism_summary = document.summary("prism")?;
    let revolved_summary = document.summary("revolved")?;
    let cone_report = document.report("cone")?;
    let sphere_report = document.report("sphere")?;
    let torus_report = document.report("torus")?;

    assert!(prism_summary.face_count > 0);
    assert!(revolved_summary.face_count > 0);
    assert_eq!(cone_report.summary.primary_kind, ShapeKind::Solid);
    assert_eq!(sphere_report.summary.primary_kind, ShapeKind::Solid);
    assert_eq!(torus_report.summary.primary_kind, ShapeKind::Solid);

    let ellipse_geometry = document
        .kernel()
        .context()
        .edge_geometry(document.shape("ellipse")?)?;
    assert_eq!(ellipse_geometry.kind, CurveKind::Ellipse);

    let extrusion_face = find_first_face_by_kind(&document, "prism", SurfaceKind::Extrusion)?;
    let extrusion_payload = document
        .kernel()
        .context()
        .face_extrusion_payload(&extrusion_face)?;
    assert_eq!(extrusion_payload.basis_curve_kind, CurveKind::Ellipse);

    let revolution_face = find_first_face_by_kind(&document, "revolved", SurfaceKind::Revolution)?;
    let revolution_payload = document
        .kernel()
        .context()
        .face_revolution_payload(&revolution_face)?;
    assert_eq!(revolution_payload.basis_curve_kind, CurveKind::Ellipse);

    let cone_face = find_first_face_by_kind(&document, "cone", SurfaceKind::Cone)?;
    let cone_payload = document.kernel().context().face_cone_payload(&cone_face)?;
    assert!((cone_payload.reference_radius - 15.0).abs() <= 1.0e-9);

    let sphere_face = find_first_face_by_kind(&document, "sphere", SurfaceKind::Sphere)?;
    let sphere_payload = document
        .kernel()
        .context()
        .face_sphere_payload(&sphere_face)?;
    assert!((sphere_payload.radius - 14.0).abs() <= 1.0e-9);

    let torus_face = find_first_face_by_kind(&document, "torus", SurfaceKind::Torus)?;
    let torus_payload = document
        .kernel()
        .context()
        .face_torus_payload(&torus_face)?;
    assert!((torus_payload.major_radius - 25.0).abs() <= 1.0e-9);
    assert!((torus_payload.minor_radius - 6.0).abs() <= 1.0e-9);

    assert_eq!(document.history().len(), 6);
    assert!(document.shape_names().any(|name| name == "revolved"));
    let cone_step = support::export_document_shape(
        &mut document,
        "cone",
        "document_workflows",
        "analytic_cone",
    )?;
    let sphere_step = support::export_document_shape(
        &mut document,
        "sphere",
        "document_workflows",
        "analytic_sphere",
    )?;
    let torus_step = support::export_document_shape(
        &mut document,
        "torus",
        "document_workflows",
        "analytic_torus",
    )?;
    assert!(cone_step.is_file());
    assert!(sphere_step.is_file());
    assert!(torus_step.is_file());

    Ok(())
}

#[test]
fn document_supports_query_driven_features() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let mut document = ModelDocument::new()?;
    document.insert_box(
        "base",
        BoxParams {
            origin: [-20.0, -20.0, -20.0],
            size: [40.0, 40.0, 40.0],
        },
    )?;

    let selected_face = document.cylindrical_hole_from_best_aligned_planar_face(
        "drilled",
        "base",
        [0.0, 0.0, 1.0],
        6.0,
    )?;
    let selected_edge =
        document.fillet_first_edge_by_curve_kind("rounded", "drilled", CurveKind::Line, 2.5)?;
    let rounded_step = support::export_document_shape(
        &mut document,
        "rounded",
        "document_workflows",
        "query_driven_rounded",
    )?;

    let planar_faces = document.face_indices_by_surface_kind("drilled", SurfaceKind::Plane)?;
    let cylindrical_faces =
        document.face_indices_by_surface_kind("drilled", SurfaceKind::Cylinder)?;
    let line_edges = document.edge_indices_by_curve_kind("drilled", CurveKind::Line)?;
    let circle_edges = document.edge_indices_by_curve_kind("drilled", CurveKind::Circle)?;

    let rounded_report = document.report("rounded")?;

    assert_eq!(selected_face.geometry.kind, SurfaceKind::Plane);
    assert!(selected_face.sample.normal[2] > 0.9);
    assert_eq!(selected_edge.geometry.kind, CurveKind::Line);
    assert!(!planar_faces.is_empty());
    assert_eq!(cylindrical_faces.len(), 1);
    assert!(!line_edges.is_empty());
    assert!(!circle_edges.is_empty());
    assert_eq!(rounded_report.summary.primary_kind, ShapeKind::Solid);
    assert!(rounded_report.triangle_count() > 0);
    assert!(rounded_step.is_file());

    Ok(())
}
