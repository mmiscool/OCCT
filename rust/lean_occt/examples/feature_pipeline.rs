use lean_occt::{BoxParams, CurveKind, EdgeSelector, FaceSelector, FeaturePipeline};

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let build = pipeline.rebuild()?;
    let report = build.report(&rounded)?;

    println!(
        "final={} faces={} triangles={} renamed_base_id={} base_name={}",
        rounded,
        report.summary.face_count,
        report.triangle_count(),
        base,
        pipeline.feature(&base)?.name,
    );

    Ok(())
}
