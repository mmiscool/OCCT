use lean_occt::{BoxParams, CurveKind, ModelDocument, ShapeKind, SurfaceKind};

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let planar_faces = document.face_indices_by_surface_kind("drilled", SurfaceKind::Plane)?;
    let cylindrical_faces =
        document.face_indices_by_surface_kind("drilled", SurfaceKind::Cylinder)?;
    let line_edges = document.edge_indices_by_curve_kind("drilled", CurveKind::Line)?;
    let circle_edges = document.edge_indices_by_curve_kind("drilled", CurveKind::Circle)?;
    let rounded = document.report("rounded")?;

    println!(
        "selected face={} kind={:?} center=({:.1},{:.1},{:.1}) normal=({:.1},{:.1},{:.1})",
        selected_face.index,
        selected_face.geometry.kind,
        selected_face.sample.position[0],
        selected_face.sample.position[1],
        selected_face.sample.position[2],
        selected_face.sample.normal[0],
        selected_face.sample.normal[1],
        selected_face.sample.normal[2],
    );
    println!(
        "selected edge={} kind={:?} length={:.3}",
        selected_edge.index, selected_edge.geometry.kind, selected_edge.length,
    );
    println!(
        "drilled planar_faces={} cylindrical_faces={} line_edges={} circle_edges={}",
        planar_faces.len(),
        cylindrical_faces.len(),
        line_edges.len(),
        circle_edges.len(),
    );
    println!(
        "rounded primary={:?} faces={} triangles={}",
        rounded.summary.primary_kind,
        rounded.summary.face_count,
        rounded.triangle_count(),
    );

    assert_eq!(rounded.summary.primary_kind, ShapeKind::Solid);

    Ok(())
}
