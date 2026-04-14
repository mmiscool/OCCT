mod support;

use lean_occt::{
    BoxParams, ConeParams, CurveKind, CylinderParams, LoopRole, ModelDocument, ShapeKind,
    SurfaceKind, ThroughHoleCut,
};

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

#[test]
fn ported_brep_snapshot_captures_topology_and_analytic_entities(
) -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let mut document = ModelDocument::new()?;
    document.box_with_through_hole("cut", default_cut())?;

    let cut_step = support::export_document_shape(
        &mut document,
        "cut",
        "brep_workflows",
        "ported_brep_cut",
    )?;
    let brep = document.brep("cut")?;

    assert_eq!(brep.summary.primary_kind, ShapeKind::Solid);
    assert_eq!(brep.faces.len(), 7);
    assert_eq!(brep.summary.face_count, brep.faces.len());
    assert_eq!(brep.topology.vertex_positions.len(), brep.vertices.len());
    assert!(!brep.wires.is_empty());
    assert!(brep.wires.iter().all(|wire| !wire.edge_indices.is_empty()));

    let circular_edge = brep
        .edges
        .iter()
        .find(|edge| edge.geometry.kind == CurveKind::Circle)
        .ok_or_else(|| std::io::Error::other("expected a circular edge in the cut solid"))?;
    assert!(circular_edge.ported_curve.is_some());
    assert_eq!(circular_edge.adjacent_face_indices.len(), 2);
    assert!(circular_edge.start_point.is_some());
    assert!(circular_edge.end_point.is_some());

    let holed_planar_face = brep
        .faces
        .iter()
        .find(|face| {
            face.geometry.kind == SurfaceKind::Plane
                && face.loops.iter().any(|face_loop| face_loop.role == LoopRole::Inner)
        })
        .ok_or_else(|| std::io::Error::other("expected a planar face with an inner loop"))?;
    assert!(holed_planar_face.ported_surface.is_some());
    assert_eq!(
        holed_planar_face
            .loops
            .iter()
            .filter(|face_loop| face_loop.role == LoopRole::Outer)
            .count(),
        1
    );
    assert_eq!(
        holed_planar_face
            .loops
            .iter()
            .filter(|face_loop| face_loop.role == LoopRole::Inner)
            .count(),
        1
    );
    let expected_holed_area = 60.0 * 60.0 - std::f64::consts::PI * 12.0 * 12.0;
    assert!(
        (holed_planar_face.area - expected_holed_area).abs() <= 1.0e-6,
        "unexpected holed planar area: {}",
        holed_planar_face.area
    );
    assert!(holed_planar_face.adjacent_face_indices.len() >= 2);

    let cylindrical_face = brep
        .faces
        .iter()
        .find(|face| face.geometry.kind == SurfaceKind::Cylinder)
        .ok_or_else(|| std::io::Error::other("expected a cylindrical face in the cut solid"))?;
    assert!(cylindrical_face.ported_surface.is_some());
    let expected_cylindrical_area = 2.0 * std::f64::consts::PI * 12.0 * 60.0;
    assert!(
        (cylindrical_face.area - expected_cylindrical_area).abs() <= 1.0e-2,
        "unexpected cylindrical face area: {}",
        cylindrical_face.area
    );
    assert!(cylindrical_face.adjacent_face_indices.len() >= 2);

    document.insert_cone(
        "cone",
        ConeParams {
            origin: [0.0, 0.0, 0.0],
            axis: [0.0, 0.0, 1.0],
            x_direction: [1.0, 0.0, 0.0],
            base_radius: 15.0,
            top_radius: 5.0,
            height: 30.0,
        },
    )?;
    let cone_brep = document.brep("cone")?;
    let cone_face = cone_brep
        .faces
        .iter()
        .find(|face| face.geometry.kind == SurfaceKind::Cone)
        .ok_or_else(|| std::io::Error::other("expected a cone face"))?;
    let cone_radius_delta = 15.0_f64 - 5.0_f64;
    let expected_cone_area =
        std::f64::consts::PI * (15.0_f64 + 5.0_f64) * (cone_radius_delta * cone_radius_delta + 30.0_f64.powi(2)).sqrt();
    assert!(
        (cone_face.area - expected_cone_area).abs() <= 2.0e-2,
        "unexpected cone face area: {}",
        cone_face.area
    );

    assert!(cut_step.is_file());
    Ok(())
}
