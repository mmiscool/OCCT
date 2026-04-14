mod support;

use lean_occt::{
    BoxParams, CurveKind, EdgeSelector, FaceSelector, ModelDocument,
    SelectorDrivenRoundedBlockRecipe, ShapeKind, SurfaceKind,
};

#[test]
fn selectors_choose_expected_faces_and_edges() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let mut document = ModelDocument::new()?;
    document.insert_box(
        "base",
        BoxParams {
            origin: [-30.0, -15.0, -5.0],
            size: [60.0, 30.0, 10.0],
        },
    )?;

    let largest_plane = document.select_face(
        "base",
        FaceSelector::LargestBySurfaceKind(SurfaceKind::Plane),
    )?;
    let top_face = document.select_face(
        "base",
        FaceSelector::BestAlignedPlane {
            normal_hint: [0.0, 0.0, 1.0],
        },
    )?;
    let longest_edge =
        document.select_edge("base", EdgeSelector::LongestByCurveKind(CurveKind::Line))?;
    let shortest_edge =
        document.select_edge("base", EdgeSelector::ShortestByCurveKind(CurveKind::Line))?;

    let base_step = support::export_document_shape(
        &mut document,
        "base",
        "selector_workflows",
        "selection_base",
    )?;

    assert_eq!(largest_plane.geometry.kind, SurfaceKind::Plane);
    assert!((largest_plane.area - 1800.0).abs() <= 1.0e-9);
    assert_eq!(top_face.geometry.kind, SurfaceKind::Plane);
    assert!(top_face.sample.normal[2] > 0.9);
    assert_eq!(longest_edge.geometry.kind, CurveKind::Line);
    assert_eq!(shortest_edge.geometry.kind, CurveKind::Line);
    assert!((longest_edge.length - 60.0).abs() <= 1.0e-9);
    assert!((shortest_edge.length - 10.0).abs() <= 1.0e-9);
    assert!(base_step.is_file());

    Ok(())
}

#[test]
fn selector_driven_recipe_builds_with_declarative_targets() -> Result<(), Box<dyn std::error::Error>>
{
    let _guard = support::test_guard();
    let mut document = ModelDocument::new()?;
    let recipe = SelectorDrivenRoundedBlockRecipe {
        box_params: BoxParams {
            origin: [-24.0, -18.0, -12.0],
            size: [48.0, 36.0, 24.0],
        },
        hole_face_selector: FaceSelector::BestAlignedPlane {
            normal_hint: [0.0, 0.0, 1.0],
        },
        hole_radius: 5.0,
        fillet_edge_selector: EdgeSelector::LongestByCurveKind(CurveKind::Line),
        fillet_radius: 2.0,
    };

    let build = recipe.build(&mut document, "selector_recipe")?;
    let rounded_step = support::export_document_shape(
        &mut document,
        build.final_shape(),
        "selector_workflows",
        "selector_recipe_rounded",
    )?;
    let report = document.report(build.final_shape())?;
    let cylindrical_faces =
        document.face_indices_by_surface_kind(build.final_shape(), SurfaceKind::Cylinder)?;
    let line_edges = document.edge_indices_by_curve_kind(build.final_shape(), CurveKind::Line)?;

    assert_eq!(
        build.stage_names(),
        &[
            "selector_recipe_base".to_owned(),
            "selector_recipe_drilled".to_owned(),
            "selector_recipe_rounded".to_owned(),
        ]
    );
    assert_eq!(report.summary.primary_kind, ShapeKind::Solid);
    assert!(report.summary.face_count >= 7);
    assert!(!cylindrical_faces.is_empty());
    assert!(!line_edges.is_empty());
    assert!(rounded_step.is_file());

    Ok(())
}
