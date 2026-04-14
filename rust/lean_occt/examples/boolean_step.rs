use std::collections::BTreeSet;
use std::f64::consts::TAU;

use lean_occt::{
    BoxParams, ConeParams, Context, CurveKind, CylinderParams, CylindricalHoleParams,
    EllipseEdgeParams, Error, FilletParams, HelixParams, LoopRole, MeshParams, OffsetParams,
    Orientation, PrismParams, RevolutionParams, Shape, ShapeKind, SphereParams, SurfaceKind,
    TorusParams,
};

fn approx_eq3(lhs: [f64; 3], rhs: [f64; 3], tolerance: f64) -> bool {
    lhs.into_iter()
        .zip(rhs)
        .all(|(lhs_value, rhs_value)| (lhs_value - rhs_value).abs() <= tolerance)
}

fn vector_length(vector: [f64; 3]) -> f64 {
    vector
        .into_iter()
        .map(|value| value * value)
        .sum::<f64>()
        .sqrt()
}

fn dot(lhs: [f64; 3], rhs: [f64; 3]) -> f64 {
    lhs.into_iter()
        .zip(rhs)
        .map(|(lhs_value, rhs_value)| lhs_value * rhs_value)
        .sum()
}

fn find_first_face_by_kind(
    ctx: &Context,
    shape: &Shape,
    kind: SurfaceKind,
) -> Result<Option<Shape>, Error> {
    for face in ctx.subshapes(shape, ShapeKind::Face)? {
        if ctx.face_geometry(&face)?.kind == kind {
            return Ok(Some(face));
        }
    }
    Ok(None)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = Context::new()?;

    let box_shape = ctx.make_box(BoxParams {
        origin: [-30.0, -30.0, -30.0],
        size: [60.0, 60.0, 60.0],
    })?;

    let cylinder = ctx.make_cylinder(CylinderParams {
        origin: [0.0, 0.0, -36.0],
        axis: [0.0, 0.0, 1.0],
        radius: 12.0,
        height: 72.0,
    })?;

    let cut = ctx.cut(&box_shape, &cylinder)?;
    let _fuse = ctx.fuse(&box_shape, &cylinder)?;
    let _common = ctx.common(&box_shape, &cylinder)?;

    let mesh = ctx.mesh(&cut, MeshParams::default())?;
    let cut_summary = ctx.describe_shape(&cut)?;
    println!(
        "cut root={:?} primary={:?} solids={} faces={} triangles={} edge_segments={} volume={:.3}",
        cut_summary.root_kind,
        cut_summary.primary_kind,
        mesh.solid_count,
        mesh.face_count,
        mesh.triangle_indices.len() / 3,
        mesh.edge_segments.len(),
        cut_summary.volume
    );
    assert_eq!(cut_summary.primary_kind, ShapeKind::Solid);
    assert_eq!(cut_summary.solid_count, 1);
    assert_eq!(cut_summary.face_count, 7);
    let faces = ctx.subshapes(&cut, ShapeKind::Face)?;
    assert_eq!(faces.len(), cut_summary.face_count);
    let face_summary = ctx.describe_shape(&faces[0])?;
    let face_wire_count = ctx.subshape_count(&faces[0], ShapeKind::Wire)?;
    let first_wire = ctx.subshape(&faces[0], ShapeKind::Wire, 0)?;
    let wire_summary = ctx.describe_shape(&first_wire)?;
    let wire_edge_count = ctx.subshape_count(&first_wire, ShapeKind::Edge)?;
    let first_edge = ctx.subshape(&first_wire, ShapeKind::Edge, 0)?;
    let edge_summary = ctx.describe_shape(&first_edge)?;
    let edge_endpoints = ctx.edge_endpoints(&first_edge)?;
    let edge_geometry = ctx.edge_geometry(&first_edge)?;
    let line_payload = ctx.edge_line_payload(&first_edge)?;
    let edge_start_sample = ctx.edge_sample(&first_edge, 0.0)?;
    let edge_mid_sample = ctx.edge_sample(&first_edge, 0.5)?;
    let edge_end_sample = ctx.edge_sample(&first_edge, 1.0)?;
    let edge_mid_parameter_sample = ctx.edge_sample_at_parameter(
        &first_edge,
        0.5 * (edge_geometry.start_parameter + edge_geometry.end_parameter),
    )?;
    let edge_vertices = ctx.subshapes(&first_edge, ShapeKind::Vertex)?;
    let first_vertex_point = ctx.vertex_point(&edge_vertices[0])?;
    let face_geometry = ctx.face_geometry(&faces[0])?;
    let plane_payload = ctx.face_plane_payload(&faces[0])?;
    let face_uv_bounds = ctx.face_uv_bounds(&faces[0])?;
    let face_sample = ctx.face_sample(&faces[0], face_uv_bounds.center())?;
    let face_normalized_sample = ctx.face_sample_normalized(&faces[0], [0.5, 0.5])?;
    let topology = ctx.topology(&cut)?;
    let cut_edge_count = ctx.subshape_count(&cut, ShapeKind::Edge)?;
    let cut_vertex_count = ctx.subshape_count(&cut, ShapeKind::Vertex)?;
    let first_face_range = topology.faces[0];
    let first_face_wire_index = topology.face_wire_indices[first_face_range.offset];
    let first_wire_range = topology.wires[first_face_wire_index];
    let first_wire_vertex_range = topology.wire_vertices[first_face_wire_index];
    println!(
        "walk face_kind={:?} face_wires={} wire_kind={:?} wire_edges={} edge_kind={:?} edge_length={:.3} edge_start=({:.1},{:.1},{:.1}) first_vertex=({:.1},{:.1},{:.1})",
        face_summary.primary_kind,
        face_wire_count,
        wire_summary.primary_kind,
        wire_edge_count,
        edge_summary.primary_kind,
        edge_summary.linear_length,
        edge_endpoints.start[0],
        edge_endpoints.start[1],
        edge_endpoints.start[2],
        first_vertex_point[0],
        first_vertex_point[1],
        first_vertex_point[2],
    );
    assert_eq!(face_summary.primary_kind, ShapeKind::Face);
    assert!(face_wire_count >= 1);
    assert_eq!(wire_summary.primary_kind, ShapeKind::Wire);
    assert!(wire_edge_count >= 1);
    assert_eq!(edge_summary.primary_kind, ShapeKind::Edge);
    assert!(edge_summary.linear_length > 0.0);
    assert_eq!(edge_geometry.kind, CurveKind::Line);
    assert!(!edge_geometry.is_periodic);
    assert_eq!(face_geometry.kind, SurfaceKind::Plane);
    assert!((vector_length(line_payload.direction) - 1.0).abs() <= 1.0e-9);
    assert!(dot(line_payload.direction, edge_mid_sample.tangent).abs() >= 1.0 - 1.0e-9);
    assert!((vector_length(plane_payload.normal) - 1.0).abs() <= 1.0e-9);
    assert!((vector_length(plane_payload.x_direction) - 1.0).abs() <= 1.0e-9);
    assert!((vector_length(plane_payload.y_direction) - 1.0).abs() <= 1.0e-9);
    assert!(dot(plane_payload.normal, face_sample.normal).abs() >= 1.0 - 1.0e-9);
    assert!(approx_eq3(
        edge_start_sample.position,
        edge_endpoints.start,
        1.0e-9
    ));
    assert!(approx_eq3(
        edge_end_sample.position,
        edge_endpoints.end,
        1.0e-9
    ));
    assert!(approx_eq3(
        edge_mid_parameter_sample.position,
        edge_mid_sample.position,
        1.0e-9
    ));
    assert!((vector_length(edge_mid_sample.tangent) - 1.0).abs() <= 1.0e-9);
    assert!(face_uv_bounds.u_min <= face_uv_bounds.u_max);
    assert!(face_uv_bounds.v_min <= face_uv_bounds.v_max);
    assert!(approx_eq3(
        face_normalized_sample.position,
        face_sample.position,
        1.0e-9
    ));
    assert!((vector_length(face_sample.normal) - 1.0).abs() <= 1.0e-9);
    assert_eq!(topology.faces.len(), cut_summary.face_count);
    assert_eq!(topology.edges.len(), cut_edge_count);
    assert!(!topology.wires.is_empty());
    assert_eq!(topology.vertex_positions.len(), cut_vertex_count);
    assert_eq!(topology.edge_faces.len(), topology.edges.len());
    assert_eq!(
        topology.face_wire_roles.len(),
        topology.face_wire_indices.len()
    );
    assert_eq!(topology.wire_vertices.len(), topology.wires.len());
    assert_eq!(first_face_range.count, face_wire_count);
    assert_eq!(first_wire_range.count, wire_edge_count);
    assert_eq!(first_wire_vertex_range.count, first_wire_range.count + 1);
    assert!(matches!(
        topology.face_wire_orientations[first_face_range.offset],
        Orientation::Forward | Orientation::Reversed
    ));
    assert_eq!(
        topology.face_wire_roles[first_face_range.offset],
        LoopRole::Outer
    );
    assert!(matches!(
        topology.wire_edge_orientations[first_wire_range.offset],
        Orientation::Forward | Orientation::Reversed
    ));
    assert!(topology.edges[topology.wire_edge_indices[first_wire_range.offset]].length > 0.0);
    let first_wire_vertices = &topology.wire_vertex_indices[first_wire_vertex_range.offset
        ..first_wire_vertex_range.offset + first_wire_vertex_range.count];
    for step in 0..first_wire_range.count {
        let edge_index = topology.wire_edge_indices[first_wire_range.offset + step];
        let edge = topology.edges[edge_index];
        let (expected_start, expected_end) = if topology.wire_edge_orientations
            [first_wire_range.offset + step]
            == Orientation::Reversed
        {
            (edge.end_vertex, edge.start_vertex)
        } else {
            (edge.start_vertex, edge.end_vertex)
        };
        assert_eq!(Some(first_wire_vertices[step]), expected_start);
        assert_eq!(Some(first_wire_vertices[step + 1]), expected_end);
    }
    assert_eq!(first_wire_vertices.first(), first_wire_vertices.last());

    for face_range in &topology.faces {
        let roles =
            &topology.face_wire_roles[face_range.offset..face_range.offset + face_range.count];
        assert_eq!(
            roles
                .iter()
                .filter(|&&role| role == LoopRole::Outer)
                .count(),
            1
        );
    }

    let mut first_face_neighbors = BTreeSet::new();
    for wire_index in topology.face_wire_indices
        [first_face_range.offset..first_face_range.offset + first_face_range.count]
        .iter()
        .copied()
    {
        let wire_range = topology.wires[wire_index];
        for edge_index in topology.wire_edge_indices
            [wire_range.offset..wire_range.offset + wire_range.count]
            .iter()
            .copied()
        {
            let edge_face_range = topology.edge_faces[edge_index];
            for neighbor_face in topology.edge_face_indices
                [edge_face_range.offset..edge_face_range.offset + edge_face_range.count]
                .iter()
                .copied()
            {
                if neighbor_face != 0 {
                    first_face_neighbors.insert(neighbor_face);
                }
            }
        }
    }
    assert!(!first_face_neighbors.is_empty());

    let cut_edges = ctx.subshapes(&cut, ShapeKind::Edge)?;
    let mut saw_line_edge = false;
    let mut saw_circle_edge = false;
    for edge in &cut_edges {
        match ctx.edge_geometry(edge)?.kind {
            CurveKind::Line => saw_line_edge = true,
            CurveKind::Circle => saw_circle_edge = true,
            _ => {}
        }
    }
    assert!(saw_line_edge);
    assert!(saw_circle_edge);

    let mut saw_plane_face = false;
    let mut saw_cylinder_face = false;
    let mut cylinder_face_sample = None;
    let mut circle_payload = None;
    let mut cylinder_payload = None;
    for face in &faces {
        let geometry = ctx.face_geometry(face)?;
        match geometry.kind {
            SurfaceKind::Plane => saw_plane_face = true,
            SurfaceKind::Cylinder => {
                saw_cylinder_face = true;
                cylinder_face_sample = Some(ctx.face_sample_normalized(face, [0.5, 0.5])?);
                cylinder_payload = Some(ctx.face_cylinder_payload(face)?);
            }
            _ => {}
        }
    }
    assert!(saw_plane_face);
    assert!(saw_cylinder_face);
    let cylinder_face_sample = cylinder_face_sample.expect("expected cylindrical face sample");
    let cylinder_payload = cylinder_payload.expect("expected cylindrical face payload");
    assert!((vector_length(cylinder_face_sample.normal) - 1.0).abs() <= 1.0e-9);
    assert!((cylinder_payload.radius - 12.0).abs() <= 1.0e-9);
    assert!((vector_length(cylinder_payload.axis) - 1.0).abs() <= 1.0e-9);
    assert!(dot(cylinder_payload.axis, cylinder_face_sample.normal).abs() <= 1.0e-9);

    for edge in &cut_edges {
        if ctx.edge_geometry(edge)?.kind == CurveKind::Circle {
            circle_payload = Some(ctx.edge_circle_payload(edge)?);
            break;
        }
    }
    let circle_payload = circle_payload.expect("expected circular edge payload");
    assert!((circle_payload.radius - 12.0).abs() <= 1.0e-9);
    assert!((vector_length(circle_payload.normal) - 1.0).abs() <= 1.0e-9);

    println!(
        "geom edge={:?} line_dir=({:.3},{:.3},{:.3}) edge_mid=({:.1},{:.1},{:.1}) tangent=({:.3},{:.3},{:.3}) circle_radius={:.1} face={:?} face_uv=({:.3},{:.3}) face_normal=({:.3},{:.3},{:.3}) cylinder_radius={:.1} cylinder_axis=({:.3},{:.3},{:.3})",
        edge_geometry.kind,
        line_payload.direction[0],
        line_payload.direction[1],
        line_payload.direction[2],
        edge_mid_sample.position[0],
        edge_mid_sample.position[1],
        edge_mid_sample.position[2],
        edge_mid_sample.tangent[0],
        edge_mid_sample.tangent[1],
        edge_mid_sample.tangent[2],
        circle_payload.radius,
        face_geometry.kind,
        face_geometry.center_uv()[0],
        face_geometry.center_uv()[1],
        face_sample.normal[0],
        face_sample.normal[1],
        face_sample.normal[2],
        cylinder_payload.radius,
        cylinder_payload.axis[0],
        cylinder_payload.axis[1],
        cylinder_payload.axis[2],
    );

    let ellipse_edge = ctx.make_ellipse_edge(EllipseEdgeParams {
        origin: [30.0, 0.0, 0.0],
        axis: [0.0, 1.0, 0.0],
        x_direction: [1.0, 0.0, 0.0],
        major_radius: 10.0,
        minor_radius: 6.0,
    })?;
    let ellipse_geometry = ctx.edge_geometry(&ellipse_edge)?;
    let ellipse_payload = ctx.edge_ellipse_payload(&ellipse_edge)?;
    assert_eq!(ellipse_geometry.kind, CurveKind::Ellipse);
    assert!((ellipse_payload.major_radius - 10.0).abs() <= 1.0e-9);
    assert!((ellipse_payload.minor_radius - 6.0).abs() <= 1.0e-9);
    assert!((vector_length(ellipse_payload.normal) - 1.0).abs() <= 1.0e-9);

    let prism = ctx.make_prism(
        &ellipse_edge,
        PrismParams {
            direction: [0.0, 24.0, 0.0],
        },
    )?;
    let extrusion_face = find_first_face_by_kind(&ctx, &prism, SurfaceKind::Extrusion)?
        .expect("expected extrusion face");
    let extrusion_payload = ctx.face_extrusion_payload(&extrusion_face)?;
    assert_eq!(extrusion_payload.basis_curve_kind, CurveKind::Ellipse);
    assert!((vector_length(extrusion_payload.direction) - 1.0).abs() <= 1.0e-9);

    let revolution = ctx.make_revolution(
        &ellipse_edge,
        RevolutionParams {
            origin: [0.0, 0.0, 0.0],
            axis: [0.0, 0.0, 1.0],
            angle_radians: TAU,
        },
    )?;
    let revolution_face = find_first_face_by_kind(&ctx, &revolution, SurfaceKind::Revolution)?
        .expect("expected revolution face");
    let revolution_payload = ctx.face_revolution_payload(&revolution_face)?;
    assert_eq!(revolution_payload.basis_curve_kind, CurveKind::Ellipse);
    assert!((vector_length(revolution_payload.axis_direction) - 1.0).abs() <= 1.0e-9);

    let offset_surface = ctx.make_offset(
        &revolution_face,
        OffsetParams {
            offset: 2.5,
            tolerance: 1.0e-4,
        },
    )?;
    let offset_face = find_first_face_by_kind(&ctx, &offset_surface, SurfaceKind::Offset)?
        .expect("expected offset face");
    let offset_surface_payload = ctx.face_offset_payload(&offset_face)?;
    assert_eq!(
        offset_surface_payload.basis_surface_kind,
        SurfaceKind::Revolution
    );
    assert!((offset_surface_payload.offset_value.abs() - 2.5).abs() <= 1.0e-9);

    let cone = ctx.make_cone(ConeParams {
        origin: [0.0, 0.0, 0.0],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        base_radius: 15.0,
        top_radius: 5.0,
        height: 30.0,
    })?;
    let cone_face =
        find_first_face_by_kind(&ctx, &cone, SurfaceKind::Cone)?.expect("expected conical face");
    let cone_payload = ctx.face_cone_payload(&cone_face)?;
    assert!((cone_payload.reference_radius - 15.0).abs() <= 1.0e-9);
    assert!(
        (cone_payload.semi_angle.abs() - ((15.0_f64 - 5.0_f64) / 30.0_f64).atan()).abs() <= 1.0e-9
    );

    let sphere = ctx.make_sphere(SphereParams {
        origin: [0.0, 0.0, 0.0],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        radius: 14.0,
    })?;
    let sphere_face = find_first_face_by_kind(&ctx, &sphere, SurfaceKind::Sphere)?
        .expect("expected spherical face");
    let sphere_payload = ctx.face_sphere_payload(&sphere_face)?;
    assert!((sphere_payload.radius - 14.0).abs() <= 1.0e-9);

    let torus = ctx.make_torus(TorusParams {
        origin: [0.0, 0.0, 0.0],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        major_radius: 25.0,
        minor_radius: 6.0,
    })?;
    let torus_face =
        find_first_face_by_kind(&ctx, &torus, SurfaceKind::Torus)?.expect("expected toroidal face");
    let torus_payload = ctx.face_torus_payload(&torus_face)?;
    assert!((torus_payload.major_radius - 25.0).abs() <= 1.0e-9);
    assert!((torus_payload.minor_radius - 6.0).abs() <= 1.0e-9);

    println!(
        "analytic ellipse=({:.1},{:.1}) extrusion={:?} revolution={:?} offset={:?} cone=({:.3},{:.3}) sphere={:.1} torus=({:.1},{:.1})",
        ellipse_payload.major_radius,
        ellipse_payload.minor_radius,
        extrusion_payload.basis_curve_kind,
        revolution_payload.basis_curve_kind,
        offset_surface_payload.basis_surface_kind,
        cone_payload.reference_radius,
        cone_payload.semi_angle,
        sphere_payload.radius,
        torus_payload.major_radius,
        torus_payload.minor_radius,
    );

    let fillet_source = ctx.make_box(BoxParams {
        origin: [0.0, 0.0, 0.0],
        size: [40.0, 30.0, 20.0],
    })?;
    let fillet = ctx.make_fillet(
        &fillet_source,
        FilletParams {
            radius: 3.0,
            edge_index: 0,
        },
    )?;
    let fillet_mesh = ctx.mesh(&fillet, MeshParams::default())?;
    println!(
        "fillet solids={} faces={} triangles={} edge_segments={}",
        fillet_mesh.solid_count,
        fillet_mesh.face_count,
        fillet_mesh.triangle_indices.len() / 3,
        fillet_mesh.edge_segments.len()
    );

    let offset_source = ctx.make_box(BoxParams {
        origin: [0.0, 0.0, 0.0],
        size: [30.0, 30.0, 30.0],
    })?;
    let offset = ctx.make_offset(
        &offset_source,
        OffsetParams {
            offset: 2.0,
            tolerance: 1.0e-4,
        },
    )?;
    let offset_mesh = ctx.mesh(&offset, MeshParams::default())?;
    println!(
        "offset solids={} faces={} triangles={} edge_segments={}",
        offset_mesh.solid_count,
        offset_mesh.face_count,
        offset_mesh.triangle_indices.len() / 3,
        offset_mesh.edge_segments.len()
    );

    let feature_source = ctx.make_box(BoxParams {
        origin: [0.0, 0.0, 0.0],
        size: [40.0, 40.0, 30.0],
    })?;
    let feature = ctx.make_cylindrical_hole(
        &feature_source,
        CylindricalHoleParams {
            origin: [20.0, 20.0, -10.0],
            axis: [0.0, 0.0, 1.0],
            radius: 6.0,
        },
    )?;
    let feature_mesh = ctx.mesh(&feature, MeshParams::default())?;
    println!(
        "feature solids={} faces={} triangles={} edge_segments={}",
        feature_mesh.solid_count,
        feature_mesh.face_count,
        feature_mesh.triangle_indices.len() / 3,
        feature_mesh.edge_segments.len()
    );

    let helix = ctx.make_helix(HelixParams {
        origin: [0.0, 0.0, 0.0],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        radius: 20.0,
        height: 30.0,
        pitch: 10.0,
    })?;
    let helix_summary = ctx.describe_shape(&helix)?;
    println!(
        "helix root={:?} primary={:?} wires={} edges={} length={:.3}",
        helix_summary.root_kind,
        helix_summary.primary_kind,
        helix_summary.wire_count,
        helix_summary.edge_count,
        helix_summary.linear_length
    );
    assert_eq!(helix_summary.primary_kind, ShapeKind::Wire);
    assert_eq!(helix_summary.wire_count, 1);
    assert_eq!(helix_summary.edge_count, 3);

    let step_path = std::env::temp_dir().join("lean_occt_rust_example.step");
    ctx.write_step(&cut, &step_path)?;

    let imported = ctx.read_step(&step_path)?;
    let imported_mesh = ctx.mesh(&imported, MeshParams::default())?;
    let imported_summary = ctx.describe_shape(&imported)?;
    println!(
        "step root={:?} primary={:?} solids={} faces={} triangles={} edge_segments={}",
        imported_summary.root_kind,
        imported_summary.primary_kind,
        imported_mesh.solid_count,
        imported_mesh.face_count,
        imported_mesh.triangle_indices.len() / 3,
        imported_mesh.edge_segments.len()
    );
    assert_eq!(imported_summary.primary_kind, ShapeKind::Solid);
    assert_eq!(imported_summary.solid_count, 1);

    std::fs::remove_file(&step_path).ok();
    Ok(())
}
