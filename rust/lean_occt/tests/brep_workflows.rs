mod support;

use std::f64::consts::PI;

use lean_occt::{
    BoxParams, ConeParams, CurveKind, CylinderParams, EllipseEdgeParams, HelixParams, LoopRole,
    ModelDocument, ModelKernel, OffsetParams, PrismParams, RevolutionParams, Shape, ShapeKind,
    SphereParams, SurfaceKind, ThroughHoleCut, TorusParams,
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

fn find_first_face_by_kind(
    kernel: &ModelKernel,
    shape: &Shape,
    kind: SurfaceKind,
) -> Result<Shape, Box<dyn std::error::Error>> {
    for face in kernel.context().subshapes(shape, ShapeKind::Face)? {
        if kernel.context().face_geometry(&face)?.kind == kind {
            return Ok(face);
        }
    }
    Err(std::io::Error::other(format!("expected face with surface kind {:?}", kind)).into())
}

fn assert_bbox_close(
    label: &str,
    lhs_min: [f64; 3],
    lhs_max: [f64; 3],
    rhs_min: [f64; 3],
    rhs_max: [f64; 3],
    tolerance: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    for axis in 0..3 {
        if (lhs_min[axis] - rhs_min[axis]).abs() > tolerance
            || (lhs_max[axis] - rhs_max[axis]).abs() > tolerance
        {
            return Err(std::io::Error::other(format!(
                "{label} bbox mismatch on axis {axis}: rust=({:?},{:?}) occt=({:?},{:?}) tol={tolerance}",
                lhs_min,
                lhs_max,
                rhs_min,
                rhs_max
            ))
            .into());
        }
    }
    Ok(())
}

#[test]
fn ported_brep_snapshot_captures_topology_and_analytic_entities(
) -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let mut document = ModelDocument::new()?;
    document.box_with_through_hole("cut", default_cut())?;

    let cut_step =
        support::export_document_shape(&mut document, "cut", "brep_workflows", "ported_brep_cut")?;
    let brep = document.brep("cut")?;
    let kernel_summary = document.summary("cut")?;
    let occt_summary = document
        .kernel()
        .context()
        .describe_shape(document.shape("cut")?)?;

    assert_eq!(brep.summary.primary_kind, ShapeKind::Solid);
    assert_eq!(brep.faces.len(), 7);
    assert_eq!(brep.summary.face_count, brep.faces.len());
    assert_eq!(brep.topology.vertex_positions.len(), brep.vertices.len());
    assert!(!brep.wires.is_empty());
    assert!(brep.wires.iter().all(|wire| !wire.edge_indices.is_empty()));
    assert_eq!(kernel_summary.face_count, brep.faces.len());
    assert_eq!(kernel_summary.edge_count, brep.edges.len());
    let wire_occurrence_length = brep
        .topology
        .wire_edge_indices
        .iter()
        .map(|&edge_index| brep.edges[edge_index].length)
        .sum::<f64>();
    assert!((kernel_summary.linear_length - wire_occurrence_length).abs() <= 1.0e-9);
    assert!(
        (kernel_summary.surface_area - brep.faces.iter().map(|face| face.area).sum::<f64>()).abs()
            <= 1.0e-9
    );
    assert!(
        (kernel_summary.linear_length - occt_summary.linear_length).abs() <= 5.0e-2,
        "linear length mismatch: rust={} occt={}",
        kernel_summary.linear_length,
        occt_summary.linear_length
    );
    assert!(
        (kernel_summary.surface_area - occt_summary.surface_area).abs() <= 5.0e-2,
        "surface area mismatch: rust={} occt={}",
        kernel_summary.surface_area,
        occt_summary.surface_area
    );
    let expected_cut_volume = 60.0 * 60.0 * 60.0 - PI * 12.0 * 12.0 * 60.0;
    assert!(
        (kernel_summary.volume - expected_cut_volume).abs() <= 1.0e-6,
        "exact cut volume mismatch: rust={} expected={}",
        kernel_summary.volume,
        expected_cut_volume
    );
    assert!(
        (kernel_summary.volume - occt_summary.volume).abs() <= 7.5e1,
        "volume mismatch: rust={} occt={}",
        kernel_summary.volume,
        occt_summary.volume
    );
    assert!(
        (brep.summary.volume - kernel_summary.volume).abs() <= 1.0e-9,
        "brep summary volume drifted from kernel summary: brep={} kernel={}",
        brep.summary.volume,
        kernel_summary.volume
    );

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
                && face
                    .loops
                    .iter()
                    .any(|face_loop| face_loop.role == LoopRole::Inner)
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
    let expected_cone_area = std::f64::consts::PI
        * (15.0_f64 + 5.0_f64)
        * (cone_radius_delta * cone_radius_delta + 30.0_f64.powi(2)).sqrt();
    assert!(
        (cone_face.area - expected_cone_area).abs() <= 2.0e-2,
        "unexpected cone face area: {}",
        cone_face.area
    );

    assert!(cut_step.is_file());
    Ok(())
}

#[test]
fn ported_brep_uses_exact_primitive_surface_and_volume_formulas(
) -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let mut document = ModelDocument::new()?;
    document.insert_box(
        "box",
        BoxParams {
            origin: [-10.0, -20.0, -30.0],
            size: [20.0, 30.0, 40.0],
        },
    )?;
    document.insert_cylinder(
        "cylinder",
        CylinderParams {
            origin: [5.0, -7.0, 2.0],
            axis: [0.0, 0.0, 1.0],
            radius: 6.0,
            height: 18.0,
        },
    )?;
    document.insert_cone(
        "cone",
        ConeParams {
            origin: [0.0, 0.0, 0.0],
            axis: [0.0, 0.0, 1.0],
            x_direction: [1.0, 0.0, 0.0],
            base_radius: 9.0,
            top_radius: 3.0,
            height: 15.0,
        },
    )?;
    document.insert_sphere(
        "sphere",
        SphereParams {
            origin: [0.0, 0.0, 0.0],
            axis: [0.0, 0.0, 1.0],
            x_direction: [1.0, 0.0, 0.0],
            radius: 7.0,
        },
    )?;
    document.insert_torus(
        "torus",
        TorusParams {
            origin: [0.0, 0.0, 0.0],
            axis: [0.0, 0.0, 1.0],
            x_direction: [1.0, 0.0, 0.0],
            major_radius: 15.0,
            minor_radius: 4.0,
        },
    )?;

    let cone_slant = (15.0_f64.powi(2) + (9.0_f64 - 3.0_f64).powi(2)).sqrt();
    let expected = [
        (
            "box",
            2.0 * (20.0 * 30.0 + 20.0 * 40.0 + 30.0 * 40.0),
            20.0 * 30.0 * 40.0,
            "primitive_box",
        ),
        (
            "cylinder",
            2.0 * PI * 6.0 * (18.0 + 6.0),
            PI * 6.0 * 6.0 * 18.0,
            "primitive_cylinder",
        ),
        (
            "cone",
            PI * (9.0 + 3.0) * cone_slant + PI * (9.0 * 9.0 + 3.0 * 3.0),
            PI * 15.0 * (9.0 * 9.0 + 9.0 * 3.0 + 3.0 * 3.0) / 3.0,
            "primitive_cone",
        ),
        (
            "sphere",
            4.0 * PI * 7.0 * 7.0,
            4.0 * PI * 7.0 * 7.0 * 7.0 / 3.0,
            "primitive_sphere",
        ),
        (
            "torus",
            4.0 * PI * PI * 15.0 * 4.0,
            2.0 * PI * PI * 15.0 * 4.0 * 4.0,
            "primitive_torus",
        ),
    ];

    for (name, expected_area, expected_volume, artifact_name) in expected {
        let artifact =
            support::export_document_shape(&mut document, name, "brep_workflows", artifact_name)?;
        let brep = document.brep(name)?;
        let summary = document.summary(name)?;
        let occt_summary = document
            .kernel()
            .context()
            .describe_shape(document.shape(name)?)?;
        let brep_face_area_sum = brep.faces.iter().map(|face| face.area).sum::<f64>();

        assert!(
            (summary.surface_area - expected_area).abs() <= 1.0e-6,
            "{name} surface area mismatch: rust={} expected={}",
            summary.surface_area,
            expected_area
        );
        assert!(
            (summary.volume - expected_volume).abs() <= 1.0e-6,
            "{name} volume mismatch: rust={} expected={}",
            summary.volume,
            expected_volume
        );
        assert!(
            (brep.summary.surface_area - expected_area).abs() <= 1.0e-6,
            "{name} brep surface area mismatch: brep={} expected={}",
            brep.summary.surface_area,
            expected_area
        );
        assert!(
            (brep.summary.volume - expected_volume).abs() <= 1.0e-6,
            "{name} brep volume mismatch: brep={} expected={}",
            brep.summary.volume,
            expected_volume
        );
        assert!(
            (brep_face_area_sum - expected_area).abs() <= 5.0e-2,
            "{name} summed face area mismatch: faces={} expected={}",
            brep_face_area_sum,
            expected_area
        );
        assert!(
            (summary.surface_area - occt_summary.surface_area).abs() <= 5.0e-2,
            "{name} surface area drifted from OCCT: rust={} occt={}",
            summary.surface_area,
            occt_summary.surface_area
        );
        assert!(
            (summary.volume - occt_summary.volume).abs() <= 5.0e-2,
            "{name} volume drifted from OCCT: rust={} occt={}",
            summary.volume,
            occt_summary.volume
        );
        assert!(artifact.is_file(), "{name} artifact should exist");
    }

    Ok(())
}

#[test]
fn ported_brep_summarizes_swept_revolution_solids_in_rust() -> Result<(), Box<dyn std::error::Error>>
{
    let _guard = support::test_guard();
    let kernel = ModelKernel::new()?;
    let ellipse = kernel.make_ellipse_edge(EllipseEdgeParams {
        origin: [30.0, 0.0, 0.0],
        axis: [0.0, 1.0, 0.0],
        x_direction: [1.0, 0.0, 0.0],
        major_radius: 10.0,
        minor_radius: 6.0,
    })?;
    let prism = kernel.make_prism(
        &ellipse,
        PrismParams {
            direction: [0.0, 24.0, 0.0],
        },
    )?;
    let extrusion_face = find_first_face_by_kind(&kernel, &prism, SurfaceKind::Extrusion)?;
    let revolved = kernel.make_revolution(
        &extrusion_face,
        RevolutionParams {
            origin: [0.0, 0.0, 0.0],
            axis: [0.0, 0.0, 1.0],
            angle_radians: PI,
        },
    )?;

    let artifact = support::export_kernel_shape(
        &kernel,
        &revolved,
        "brep_workflows",
        "swept_revolution_solid",
    )?;
    let summary = kernel.summarize(&revolved)?;
    let occt_summary = kernel.context().describe_shape(&revolved)?;
    let brep = kernel.brep(&revolved)?;

    assert_eq!(summary.primary_kind, ShapeKind::Solid);
    assert_eq!(summary.solid_count, 1);
    assert!(brep
        .faces
        .iter()
        .any(|face| face.geometry.kind == SurfaceKind::Extrusion));
    assert!(brep
        .faces
        .iter()
        .any(|face| face.geometry.kind == SurfaceKind::Revolution));
    assert!(
        (summary.surface_area - occt_summary.surface_area).abs() <= 2.0e-1,
        "swept revolution surface area drifted from OCCT: rust={} occt={}",
        summary.surface_area,
        occt_summary.surface_area
    );
    assert!(
        (summary.volume - occt_summary.volume).abs() <= 2.0e-1,
        "swept revolution volume drifted from OCCT: rust={} occt={}",
        summary.volume,
        occt_summary.volume
    );
    assert!(
        (brep.summary.volume - summary.volume).abs() <= 1.0e-9,
        "brep summary volume drifted from kernel summary: brep={} kernel={}",
        brep.summary.volume,
        summary.volume
    );
    assert!(artifact.is_file());

    Ok(())
}

#[test]
fn ported_brep_uses_rust_mesh_area_for_offset_faces() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let kernel = ModelKernel::new()?;
    let ellipse = kernel.make_ellipse_edge(EllipseEdgeParams {
        origin: [30.0, 0.0, 0.0],
        axis: [0.0, 1.0, 0.0],
        x_direction: [1.0, 0.0, 0.0],
        major_radius: 10.0,
        minor_radius: 6.0,
    })?;
    let revolution = kernel.make_revolution(
        &ellipse,
        RevolutionParams {
            origin: [0.0, 0.0, 0.0],
            axis: [0.0, 0.0, 1.0],
            angle_radians: PI,
        },
    )?;
    let revolution_face = find_first_face_by_kind(&kernel, &revolution, SurfaceKind::Revolution)?;
    let offset_surface = kernel.make_offset(
        &revolution_face,
        OffsetParams {
            offset: 2.5,
            tolerance: 1.0e-4,
        },
    )?;

    let artifact = support::export_kernel_shape(
        &kernel,
        &offset_surface,
        "brep_workflows",
        "offset_surface_mesh_area",
    )?;
    let brep = kernel.brep(&offset_surface)?;
    let offset_face = brep
        .faces
        .iter()
        .find(|face| face.geometry.kind == SurfaceKind::Offset)
        .ok_or_else(|| std::io::Error::other("expected an offset face"))?;
    let occt_face = find_first_face_by_kind(&kernel, &offset_surface, SurfaceKind::Offset)?;
    let occt_area = kernel.context().describe_shape(&occt_face)?.surface_area;

    assert_eq!(brep.summary.primary_kind, ShapeKind::Shell);
    assert_eq!(brep.summary.face_count, 1);
    assert!(
        offset_face.ported_surface.is_none(),
        "offset face should still be on the non-ported surface path"
    );
    assert!(
        (offset_face.area - occt_area).abs() <= 5.0e-2,
        "offset face area drifted from OCCT: rust={} occt={}",
        offset_face.area,
        occt_area
    );
    assert!(
        (brep.summary.surface_area - offset_face.area).abs() <= 1.0e-9,
        "offset brep summary surface area drifted from face area: summary={} face={}",
        brep.summary.surface_area,
        offset_face.area
    );
    assert!(artifact.is_file());

    Ok(())
}

#[test]
fn ported_brep_uses_rust_owned_bounding_boxes() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let kernel = ModelKernel::new()?;

    let cut = kernel.box_with_through_hole(default_cut())?;
    let sphere = kernel.make_sphere(SphereParams {
        origin: [5.0, -4.0, 3.0],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        radius: 7.0,
    })?;
    let helix = kernel.make_helix(HelixParams {
        origin: [0.0, 0.0, 0.0],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        radius: 20.0,
        height: 30.0,
        pitch: 10.0,
    })?;

    let cut_step = support::export_kernel_shape(&kernel, &cut, "brep_workflows", "bbox_cut_shape")?;
    let sphere_step =
        support::export_kernel_shape(&kernel, &sphere, "brep_workflows", "bbox_sphere_shape")?;
    let helix_step =
        support::export_kernel_shape(&kernel, &helix, "brep_workflows", "bbox_helix_shape")?;

    let cut_summary = kernel.summarize(&cut)?;
    let cut_occt = kernel.context().describe_shape(&cut)?;
    assert_bbox_close(
        "cut",
        cut_summary.bbox_min,
        cut_summary.bbox_max,
        cut_occt.bbox_min,
        cut_occt.bbox_max,
        5.0e-7,
    )?;
    assert_eq!(cut_summary.bbox_min, [-30.0, -30.0, -30.0]);
    assert_eq!(cut_summary.bbox_max, [30.0, 30.0, 30.0]);

    let sphere_summary = kernel.summarize(&sphere)?;
    let sphere_occt = kernel.context().describe_shape(&sphere)?;
    assert_bbox_close(
        "sphere",
        sphere_summary.bbox_min,
        sphere_summary.bbox_max,
        sphere_occt.bbox_min,
        sphere_occt.bbox_max,
        5.0e-2,
    )?;

    let helix_summary = kernel.summarize(&helix)?;
    let helix_occt = kernel.context().describe_shape(&helix)?;
    assert_bbox_close(
        "helix",
        helix_summary.bbox_min,
        helix_summary.bbox_max,
        helix_occt.bbox_min,
        helix_occt.bbox_max,
        5.0e-2,
    )?;

    assert!(cut_step.is_file());
    assert!(sphere_step.is_file());
    assert!(helix_step.is_file());

    Ok(())
}

#[test]
fn ported_brep_uses_rust_kind_classification() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let kernel = ModelKernel::new()?;

    let cut = kernel.box_with_through_hole(default_cut())?;
    let helix = kernel.make_helix(HelixParams {
        origin: [0.0, 0.0, 0.0],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        radius: 20.0,
        height: 30.0,
        pitch: 10.0,
    })?;
    let ellipse = kernel.make_ellipse_edge(EllipseEdgeParams {
        origin: [30.0, 0.0, 0.0],
        axis: [0.0, 1.0, 0.0],
        x_direction: [1.0, 0.0, 0.0],
        major_radius: 10.0,
        minor_radius: 6.0,
    })?;
    let revolution = kernel.make_revolution(
        &ellipse,
        RevolutionParams {
            origin: [0.0, 0.0, 0.0],
            axis: [0.0, 0.0, 1.0],
            angle_radians: PI,
        },
    )?;
    let revolution_face = find_first_face_by_kind(&kernel, &revolution, SurfaceKind::Revolution)?;
    let offset_surface = kernel.make_offset(
        &revolution_face,
        OffsetParams {
            offset: 2.5,
            tolerance: 1.0e-4,
        },
    )?;

    for (label, shape) in [
        ("cut", &cut),
        ("helix", &helix),
        ("offset", &offset_surface),
    ] {
        let rust_summary = kernel.summarize(shape)?;
        let occt_summary = kernel.context().describe_shape(shape)?;
        assert_eq!(
            rust_summary.root_kind, occt_summary.root_kind,
            "{label} root_kind drifted from OCCT"
        );
        assert_eq!(
            rust_summary.primary_kind, occt_summary.primary_kind,
            "{label} primary_kind drifted from OCCT"
        );
    }

    Ok(())
}
