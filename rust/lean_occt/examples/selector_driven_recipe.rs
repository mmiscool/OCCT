use lean_occt::{
    BoxParams, CurveKind, EdgeSelector, FaceSelector, ModelDocument,
    SelectorDrivenRoundedBlockRecipe,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut document = ModelDocument::new()?;
    let build = SelectorDrivenRoundedBlockRecipe {
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
    }
    .build(&mut document, "selector_recipe")?;

    let report = document.report(build.final_shape())?;

    println!(
        "final={} faces={} triangles={} stages={}",
        build.final_shape(),
        report.summary.face_count,
        report.triangle_count(),
        build.stage_names().join(","),
    );

    Ok(())
}
