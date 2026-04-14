mod support;

use lean_occt::{
    BoxParams, CurveKind, EdgeSelector, FaceSelector, FeatureBuildSource, FeatureOperation,
    FeaturePipeline, OffsetParams, ShapeKind,
};

#[test]
fn feature_pipeline_uses_stable_ids_instead_of_names() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let mut pipeline = FeaturePipeline::new();
    let base = pipeline.add_box(
        "Base stock",
        BoxParams {
            origin: [-20.0, -20.0, -20.0],
            size: [40.0, 40.0, 40.0],
        },
    );
    let drilled = pipeline.add_cylindrical_hole(
        "Top hole",
        &base,
        FaceSelector::BestAlignedPlane {
            normal_hint: [0.0, 0.0, 1.0],
        },
        6.0,
    );
    let rounded = pipeline.add_fillet(
        "Long-edge round",
        &drilled,
        EdgeSelector::LongestByCurveKind(CurveKind::Line),
        2.5,
    );

    pipeline.rename_feature(&base, "Renamed stock")?;

    let mut build = pipeline.rebuild()?;
    let rounded_step =
        support::step_artifact_path("pipeline_workflows", "renamed_pipeline_rounded")?;
    build.export_step(&rounded, &rounded_step)?;
    let rounded_report = build.report(&rounded)?;

    assert_eq!(pipeline.feature(&base)?.name, "Renamed stock");
    assert_ne!(base.as_str(), "Renamed stock");
    assert_eq!(rounded_report.summary.primary_kind, ShapeKind::Solid);
    assert!(rounded_report.summary.face_count >= 7);
    assert!(rounded_step.is_file());

    Ok(())
}

#[test]
fn feature_pipeline_imports_step_and_continues_history() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let mut source_pipeline = FeaturePipeline::new();
    let source = source_pipeline.add_box(
        "Import source",
        BoxParams {
            origin: [-15.0, -15.0, -15.0],
            size: [30.0, 30.0, 30.0],
        },
    );

    let mut source_build = source_pipeline.rebuild()?;
    let source_step = support::step_artifact_path("pipeline_workflows", "import_source")?;
    source_build.export_step(&source, &source_step)?;

    let mut pipeline = FeaturePipeline::new();
    let imported = pipeline.import_step("Imported source", &source_step);
    let offset = pipeline.add_offset(
        "Imported offset",
        &imported,
        OffsetParams {
            offset: 1.5,
            tolerance: 1.0e-4,
        },
    );

    let mut build = pipeline.rebuild()?;
    let imported_summary = build.summary(&imported)?;
    let offset_summary = build.summary(&offset)?;
    let offset_step = support::step_artifact_path("pipeline_workflows", "import_offset")?;
    build.export_step(&offset, &offset_step)?;

    assert_eq!(imported_summary.primary_kind, ShapeKind::Solid);
    assert_eq!(offset_summary.primary_kind, ShapeKind::Solid);
    assert!(source_step.is_file());
    assert!(offset_step.is_file());

    Ok(())
}

#[test]
fn feature_pipeline_json_round_trips_stable_ids() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let mut pipeline = FeaturePipeline::new();
    let base = pipeline.add_box(
        "Base stock",
        BoxParams {
            origin: [-18.0, -18.0, -18.0],
            size: [36.0, 36.0, 36.0],
        },
    );
    let drilled = pipeline.add_cylindrical_hole(
        "Top hole",
        &base,
        FaceSelector::BestAlignedPlane {
            normal_hint: [0.0, 0.0, 1.0],
        },
        5.0,
    );
    let rounded = pipeline.add_fillet(
        "Rounded edge",
        &drilled,
        EdgeSelector::LongestByCurveKind(CurveKind::Line),
        2.0,
    );

    let json = pipeline.to_json_string_pretty()?;
    let mut reloaded = FeaturePipeline::from_json_str(&json)?;
    let mut build = reloaded.rebuild()?;
    let rounded_step = support::step_artifact_path("pipeline_workflows", "json_roundtrip_rounded")?;
    build.export_step(&rounded, &rounded_step)?;
    let rounded_summary = build.summary(&rounded)?;

    assert!(json.contains("\"feature_0001\""));
    assert!(json.contains("\"Rounded edge\""));
    assert_eq!(reloaded.feature(&base)?.name, "Base stock");
    assert_eq!(reloaded.feature(&drilled)?.name, "Top hole");
    assert_eq!(rounded_summary.primary_kind, ShapeKind::Solid);
    assert!(rounded_step.is_file());

    Ok(())
}

#[test]
fn feature_pipeline_rebuilds_only_dirty_suffix() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let mut pipeline = FeaturePipeline::new();
    let base = pipeline.add_box(
        "Base stock",
        BoxParams {
            origin: [-20.0, -20.0, -20.0],
            size: [40.0, 40.0, 40.0],
        },
    );
    let drilled = pipeline.add_cylindrical_hole(
        "Top hole",
        &base,
        FaceSelector::BestAlignedPlane {
            normal_hint: [0.0, 0.0, 1.0],
        },
        6.0,
    );
    let rounded = pipeline.add_fillet(
        "Long-edge round",
        &drilled,
        EdgeSelector::LongestByCurveKind(CurveKind::Line),
        2.5,
    );

    pipeline.rebuild()?;
    assert_eq!(
        pipeline.feature(&base)?.runtime.last_build_source,
        Some(FeatureBuildSource::Rebuilt)
    );
    assert_eq!(
        pipeline.feature(&drilled)?.runtime.last_build_source,
        Some(FeatureBuildSource::Rebuilt)
    );
    assert_eq!(
        pipeline.feature(&rounded)?.runtime.last_build_source,
        Some(FeatureBuildSource::Rebuilt)
    );

    pipeline.replace_feature_operation(
        &drilled,
        FeatureOperation::CylindricalHole {
            input: base.clone(),
            selector: FaceSelector::BestAlignedPlane {
                normal_hint: [0.0, 0.0, 1.0],
            },
            radius: 8.0,
        },
    )?;
    assert_eq!(pipeline.dirty_start_index(), Some(1));

    let mut build = pipeline.rebuild()?;
    let rounded_step = support::step_artifact_path("pipeline_workflows", "dirty_suffix_rounded")?;
    build.export_step(&rounded, &rounded_step)?;
    let rounded_summary = build.summary(&rounded)?;

    assert_eq!(
        pipeline.feature(&base)?.runtime.last_build_source,
        Some(FeatureBuildSource::CachedImport)
    );
    assert_eq!(
        pipeline.feature(&drilled)?.runtime.last_build_source,
        Some(FeatureBuildSource::Rebuilt)
    );
    assert_eq!(
        pipeline.feature(&rounded)?.runtime.last_build_source,
        Some(FeatureBuildSource::Rebuilt)
    );
    assert_eq!(rounded_summary.primary_kind, ShapeKind::Solid);
    assert!(rounded_step.is_file());

    Ok(())
}
