use lean_occt::{
    feature_definitions, CurveKind, EdgeSelector, FaceSelector, FeaturePipeline, FeatureSpec,
    FeatureType,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut pipeline = FeaturePipeline::new();

    let mut base = FeatureSpec::new(FeatureType::AddBox).with_name("Schema base");
    base.set_param("origin", [-20.0, -20.0, -20.0])?;
    base.set_param("size", [40.0, 40.0, 40.0])?;
    let base_id = pipeline.add_feature_spec(base)?;

    let mut hole = FeatureSpec::new(FeatureType::CylindricalHole).with_name("Schema hole");
    hole.push_input(&base_id);
    hole.set_param(
        "selector",
        FaceSelector::BestAlignedPlane {
            normal_hint: [0.0, 0.0, 1.0],
        },
    )?;
    hole.set_param("radius", 6.0)?;
    let hole_id = pipeline.add_feature_spec(hole)?;

    let mut round = FeatureSpec::new(FeatureType::Fillet).with_name("Schema round");
    round.push_input(&hole_id);
    round.set_param(
        "selector",
        EdgeSelector::LongestByCurveKind(CurveKind::Line),
    )?;
    round.set_param("radius", 2.5)?;
    let round_id = pipeline.add_feature_spec(round)?;

    let build = pipeline.rebuild()?;
    let report = build.report(&round_id)?;
    let feature_count = feature_definitions().len();
    let hole_definition = FeatureType::CylindricalHole.definition();

    println!(
        "defs={} hole_params={} final={} faces={} triangles={}",
        feature_count,
        hole_definition.params.len(),
        round_id,
        report.summary.face_count,
        report.triangle_count(),
    );

    Ok(())
}
