use lean_occt::{
    BoxParams, CurveKind, DrilledBlockRecipe, ModelDocument, RoundedDrilledBlockRecipe,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut document = ModelDocument::new()?;

    let drilled = DrilledBlockRecipe {
        box_params: BoxParams {
            origin: [-25.0, -25.0, -20.0],
            size: [50.0, 50.0, 40.0],
        },
        hole_normal_hint: [0.0, 0.0, 1.0],
        hole_radius: 7.5,
    }
    .build(&mut document, "mount")?;

    let rounded = RoundedDrilledBlockRecipe {
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
    }
    .build(&mut document, "fixture")?;

    let mount_summary = document.summary(drilled.final_shape())?;
    let fixture_report = document.report(rounded.final_shape())?;

    println!(
        "mount final={} faces={} volume={:.3}",
        drilled.final_shape(),
        mount_summary.face_count,
        mount_summary.volume,
    );
    println!(
        "fixture final={} faces={} triangles={} stages={}",
        rounded.final_shape(),
        fixture_report.summary.face_count,
        fixture_report.triangle_count(),
        rounded.stage_names().join(","),
    );

    Ok(())
}
