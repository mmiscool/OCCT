mod support;

use lean_occt::{
    BoxParams, CurveKind, DrilledBlockRecipe, ModelDocument, RoundedDrilledBlockRecipe, ShapeKind,
    SurfaceKind,
};

#[test]
fn drilled_block_recipe_builds_named_stages() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let mut document = ModelDocument::new()?;
    let recipe = DrilledBlockRecipe {
        box_params: BoxParams {
            origin: [-25.0, -25.0, -20.0],
            size: [50.0, 50.0, 40.0],
        },
        hole_normal_hint: [0.0, 0.0, 1.0],
        hole_radius: 7.5,
    };

    let build = recipe.build(&mut document, "mount")?;
    let mount_step = support::export_document_shape(
        &mut document,
        build.final_shape(),
        "recipe_workflows",
        "mount_drilled",
    )?;
    let summary = document.summary(build.final_shape())?;
    let cylindrical_faces =
        document.face_indices_by_surface_kind(build.final_shape(), SurfaceKind::Cylinder)?;
    let circle_edges =
        document.edge_indices_by_curve_kind(build.final_shape(), CurveKind::Circle)?;

    assert_eq!(
        build.stage_names(),
        &["mount_base".to_owned(), "mount_drilled".to_owned()]
    );
    assert!(document.contains_shape("mount_base"));
    assert!(document.contains_shape("mount_drilled"));
    assert_eq!(summary.primary_kind, ShapeKind::Solid);
    assert_eq!(cylindrical_faces.len(), 1);
    assert!(!circle_edges.is_empty());
    assert!(mount_step.is_file());

    Ok(())
}

#[test]
fn rounded_drilled_block_recipe_extends_drilled_recipe() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let mut document = ModelDocument::new()?;
    let recipe = RoundedDrilledBlockRecipe {
        drilled_block: DrilledBlockRecipe {
            box_params: BoxParams {
                origin: [-20.0, -20.0, -20.0],
                size: [40.0, 40.0, 40.0],
            },
            hole_normal_hint: [0.0, 0.0, 1.0],
            hole_radius: 6.0,
        },
        fillet_curve_kind: CurveKind::Line,
        fillet_radius: 2.5,
    };

    let build = recipe.build(&mut document, "fixture")?;
    document.step_round_trip("fixture_step", build.final_shape())?;
    let fixture_step = support::export_document_shape(
        &mut document,
        build.final_shape(),
        "recipe_workflows",
        "fixture_rounded",
    )?;
    let fixture_roundtrip_step = support::export_document_shape(
        &mut document,
        "fixture_step",
        "recipe_workflows",
        "fixture_roundtrip",
    )?;

    let rounded = document.report(build.final_shape())?;
    let stepped = document.summary("fixture_step")?;

    assert_eq!(
        build.stage_names(),
        &[
            "fixture_base".to_owned(),
            "fixture_drilled".to_owned(),
            "fixture_rounded".to_owned(),
        ]
    );
    assert_eq!(rounded.summary.primary_kind, ShapeKind::Solid);
    assert_eq!(stepped.primary_kind, ShapeKind::Solid);
    assert!(rounded.triangle_count() > 0);
    assert!(rounded.summary.face_count >= 7);
    assert!(fixture_step.is_file());
    assert!(fixture_roundtrip_step.is_file());

    Ok(())
}
