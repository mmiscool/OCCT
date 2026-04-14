mod support;

use lean_occt::{
    feature_definitions, CurveKind, EdgeSelector, FaceSelector, FeaturePipeline, FeatureSpec,
    FeatureType, ShapeKind,
};
use serde_json::json;

#[test]
fn feature_specs_build_pipeline_from_defaults_and_overrides(
) -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let definitions = feature_definitions();
    assert!(definitions
        .iter()
        .any(|definition| definition.feature_type == FeatureType::Fillet));

    let mut pipeline = FeaturePipeline::new();

    let mut base = FeatureSpec::new(FeatureType::AddBox).with_name("Schema base");
    base.set_param("origin", [-20.0, -20.0, -20.0])?;
    base.set_param("size", [40.0, 40.0, 40.0])?;
    let base_id = pipeline.add_feature_spec(base)?;

    let mut drilled = FeatureSpec::new(FeatureType::CylindricalHole).with_name("Schema hole");
    drilled.push_input(&base_id);
    drilled.set_param(
        "selector",
        FaceSelector::BestAlignedPlane {
            normal_hint: [0.0, 0.0, 1.0],
        },
    )?;
    drilled.set_param("radius", 6.0)?;
    let drilled_id = pipeline.add_feature_spec(drilled)?;

    let mut rounded = FeatureSpec::new(FeatureType::Fillet).with_name("Schema round");
    rounded.push_input(&drilled_id);
    rounded.set_param(
        "selector",
        EdgeSelector::LongestByCurveKind(CurveKind::Line),
    )?;
    rounded.set_param("radius", 2.5)?;
    let rounded_id = pipeline.add_feature_spec(rounded)?;

    let mut build = pipeline.rebuild()?;
    let rounded_step = support::step_artifact_path("schema_workflows", "schema_driven_rounded")?;
    build.export_step(&rounded_id, &rounded_step)?;
    let rounded_summary = build.summary(&rounded_id)?;

    assert_eq!(rounded_summary.primary_kind, ShapeKind::Solid);
    assert!(rounded_summary.face_count >= 7);
    assert!(rounded_step.is_file());

    Ok(())
}

#[test]
fn feature_specs_validate_inputs_and_params() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let cut_error = FeatureSpec::new(FeatureType::Cut)
        .to_operation()
        .expect_err("cut spec without inputs should fail");
    assert!(cut_error.to_string().contains("expects 2 input(s)"));

    let mut invalid_box = FeatureSpec::new(FeatureType::AddBox);
    invalid_box.replace_params(json!({
        "origin": [0.0, 0.0, 0.0],
        "size": [10.0, 10.0, 10.0],
        "bogus": 1.0,
    }))?;
    let invalid_box_error = invalid_box
        .to_operation()
        .expect_err("unknown box parameter should fail");
    assert!(invalid_box_error
        .to_string()
        .contains("no parameter named 'bogus'"));

    let import_error = FeatureSpec::new(FeatureType::ImportStep)
        .to_operation()
        .expect_err("empty STEP import path should fail");
    assert!(import_error.to_string().contains("non-empty path"));

    Ok(())
}

#[test]
fn feature_specs_round_trip_through_json() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let mut pipeline = FeaturePipeline::new();

    let mut base = FeatureSpec::new(FeatureType::AddBox).with_name("JSON base");
    base.set_param("origin", [-18.0, -18.0, -18.0])?;
    base.set_param("size", [36.0, 36.0, 36.0])?;
    let base_json = serde_json::to_string_pretty(&base)?;
    let base: FeatureSpec = serde_json::from_str(&base_json)?;
    let base_id = pipeline.add_feature_spec(base)?;

    let mut hole = FeatureSpec::new(FeatureType::CylindricalHole).with_name("JSON hole");
    hole.push_input(&base_id);
    hole.set_param(
        "selector",
        FaceSelector::BestAlignedPlane {
            normal_hint: [0.0, 0.0, 1.0],
        },
    )?;
    hole.set_param("radius", 5.0)?;
    let hole_json = serde_json::to_string_pretty(&hole)?;
    let hole: FeatureSpec = serde_json::from_str(&hole_json)?;
    let hole_id = pipeline.add_feature_spec(hole)?;

    let mut fillet = FeatureSpec::new(FeatureType::Fillet).with_name("JSON round");
    fillet.push_input(&hole_id);
    fillet.set_param(
        "selector",
        EdgeSelector::LongestByCurveKind(CurveKind::Line),
    )?;
    fillet.set_param("radius", 2.0)?;
    let fillet_json = serde_json::to_string_pretty(&fillet)?;
    let fillet: FeatureSpec = serde_json::from_str(&fillet_json)?;
    let fillet_id = pipeline.add_feature_spec(fillet)?;

    let mut build = pipeline.rebuild()?;
    let rounded_step = support::step_artifact_path("schema_workflows", "schema_json_rounded")?;
    build.export_step(&fillet_id, &rounded_step)?;
    let rounded_summary = build.summary(&fillet_id)?;

    assert!(base_json.contains("\"feature_type\": \"add_box\""));
    assert!(hole_json.contains("\"feature_0001\""));
    assert!(fillet_json.contains("\"feature_type\": \"fillet\""));
    assert_eq!(rounded_summary.primary_kind, ShapeKind::Solid);
    assert!(rounded_step.is_file());

    Ok(())
}
