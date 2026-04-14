use lean_occt::{BoxParams, CylinderParams, ModelDocument, ShapeKind, ThroughHoleCut};

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut document = ModelDocument::new()?;
    let cut = default_cut();

    document.insert_box("base", cut.box_params)?;
    document.insert_cylinder("tool", cut.tool_params)?;
    document.cut("cut", "base", "tool")?;
    document.step_round_trip("cut_step", "cut")?;

    let cut_report = document.report("cut")?;
    let step_summary = document.summary("cut_step")?;

    println!(
        "cut primary={:?} solids={} faces={} triangles={} edge_segments={}",
        cut_report.summary.primary_kind,
        cut_report.summary.solid_count,
        cut_report.summary.face_count,
        cut_report.triangle_count(),
        cut_report.edge_segment_count(),
    );
    println!(
        "step primary={:?} solids={} faces={} volume={:.3}",
        step_summary.primary_kind,
        step_summary.solid_count,
        step_summary.face_count,
        step_summary.volume,
    );

    assert_eq!(cut_report.summary.primary_kind, ShapeKind::Solid);
    assert_eq!(step_summary.primary_kind, ShapeKind::Solid);

    Ok(())
}
