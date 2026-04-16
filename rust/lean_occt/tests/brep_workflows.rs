mod support;

use std::f64::consts::PI;

use lean_occt::{
    BoxParams, ConeParams, CurveKind, CylinderParams, EllipseEdgeParams, HelixParams, LoopRole,
    ModelDocument, ModelKernel, OffsetParams, PortedFaceSurface, PortedOffsetBasisSurface,
    PortedSweptSurface, PrismParams, RevolutionParams, Shape, ShapeKind, SphereParams, SurfaceKind,
    ThroughHoleCut, TorusParams,
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

fn find_first_edge_by_kind(
    kernel: &ModelKernel,
    shape: &Shape,
    kind: CurveKind,
) -> Result<Shape, Box<dyn std::error::Error>> {
    for edge in kernel.context().subshapes(shape, ShapeKind::Edge)? {
        if kernel.context().edge_geometry(&edge)?.kind == kind {
            return Ok(edge);
        }
    }
    Err(std::io::Error::other(format!("expected edge with curve kind {:?}", kind)).into())
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

fn assert_vec3_close(
    lhs: [f64; 3],
    rhs: [f64; 3],
    tolerance: f64,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    for axis in 0..3 {
        if (lhs[axis] - rhs[axis]).abs() > tolerance {
            return Err(std::io::Error::other(format!(
                "{label} mismatch on axis {axis}: lhs={lhs:?} rhs={rhs:?} tol={tolerance}"
            ))
            .into());
        }
    }
    Ok(())
}

fn assert_edge_geometry_close(
    lhs: lean_occt::EdgeGeometry,
    rhs: lean_occt::EdgeGeometry,
    tolerance: f64,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if lhs.kind != rhs.kind || lhs.is_closed != rhs.is_closed || lhs.is_periodic != rhs.is_periodic
    {
        return Err(
            std::io::Error::other(format!("{label} mismatch: lhs={lhs:?} rhs={rhs:?}")).into(),
        );
    }

    for (field, lhs, rhs) in [
        ("start_parameter", lhs.start_parameter, rhs.start_parameter),
        ("end_parameter", lhs.end_parameter, rhs.end_parameter),
        ("period", lhs.period, rhs.period),
    ] {
        if (lhs - rhs).abs() > tolerance {
            return Err(std::io::Error::other(format!(
                "{label} {field} mismatch: lhs={lhs} rhs={rhs} tol={tolerance}"
            ))
            .into());
        }
    }

    Ok(())
}

fn assert_face_geometry_close(
    lhs: lean_occt::FaceGeometry,
    rhs: lean_occt::FaceGeometry,
    tolerance: f64,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if lhs.kind != rhs.kind
        || lhs.is_u_closed != rhs.is_u_closed
        || lhs.is_v_closed != rhs.is_v_closed
        || lhs.is_u_periodic != rhs.is_u_periodic
        || lhs.is_v_periodic != rhs.is_v_periodic
    {
        return Err(
            std::io::Error::other(format!("{label} mismatch: lhs={lhs:?} rhs={rhs:?}")).into(),
        );
    }

    for (field, lhs, rhs) in [
        ("u_min", lhs.u_min, rhs.u_min),
        ("u_max", lhs.u_max, rhs.u_max),
        ("v_min", lhs.v_min, rhs.v_min),
        ("v_max", lhs.v_max, rhs.v_max),
        ("u_period", lhs.u_period, rhs.u_period),
        ("v_period", lhs.v_period, rhs.v_period),
    ] {
        if (lhs - rhs).abs() > tolerance {
            return Err(std::io::Error::other(format!(
                "{label} {field} mismatch: lhs={lhs} rhs={rhs} tol={tolerance}"
            ))
            .into());
        }
    }

    Ok(())
}

fn assert_same_variant<T: std::fmt::Debug>(
    lhs: &T,
    rhs: &T,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if std::mem::discriminant(lhs) != std::mem::discriminant(rhs) {
        return Err(std::io::Error::other(format!(
            "{label} variant mismatch: lhs={lhs:?} rhs={rhs:?}"
        ))
        .into());
    }
    Ok(())
}

fn assert_ported_face_surface_matches_public(
    label: &str,
    actual: Option<PortedFaceSurface>,
    expected: Option<PortedFaceSurface>,
    actual_geometry: lean_occt::FaceGeometry,
    expected_geometry: lean_occt::FaceGeometry,
    orientation: lean_occt::Orientation,
) -> Result<(), Box<dyn std::error::Error>> {
    match (actual, expected) {
        (None, None) => Ok(()),
        (Some(actual), Some(expected)) => {
            assert_same_variant(&actual, &expected, label)?;
            match (actual, expected) {
                (
                    PortedFaceSurface::Analytic(actual_surface),
                    PortedFaceSurface::Analytic(expected_surface),
                ) => {
                    assert_same_variant(&actual_surface, &expected_surface, label)?;
                }
                (
                    PortedFaceSurface::Swept(PortedSweptSurface::Extrusion {
                        payload: actual_payload,
                        basis_curve: actual_curve,
                        basis_geometry: actual_basis_geometry,
                    }),
                    PortedFaceSurface::Swept(PortedSweptSurface::Extrusion {
                        payload: expected_payload,
                        basis_curve: expected_curve,
                        basis_geometry: expected_basis_geometry,
                    }),
                ) => {
                    if actual_payload.basis_curve_kind != expected_payload.basis_curve_kind {
                        return Err(std::io::Error::other(format!(
                            "{label} extrusion basis kind mismatch: actual={:?} expected={:?}",
                            actual_payload.basis_curve_kind, expected_payload.basis_curve_kind
                        ))
                        .into());
                    }
                    assert_same_variant(&actual_curve, &expected_curve, label)?;
                    assert_edge_geometry_close(
                        actual_basis_geometry,
                        expected_basis_geometry,
                        1.0e-9,
                        label,
                    )?;
                }
                (
                    PortedFaceSurface::Swept(PortedSweptSurface::Revolution {
                        payload: actual_payload,
                        basis_curve: actual_curve,
                        basis_geometry: actual_basis_geometry,
                    }),
                    PortedFaceSurface::Swept(PortedSweptSurface::Revolution {
                        payload: expected_payload,
                        basis_curve: expected_curve,
                        basis_geometry: expected_basis_geometry,
                    }),
                ) => {
                    if actual_payload.basis_curve_kind != expected_payload.basis_curve_kind {
                        return Err(std::io::Error::other(format!(
                            "{label} revolution basis kind mismatch: actual={:?} expected={:?}",
                            actual_payload.basis_curve_kind, expected_payload.basis_curve_kind
                        ))
                        .into());
                    }
                    assert_same_variant(&actual_curve, &expected_curve, label)?;
                    assert_edge_geometry_close(
                        actual_basis_geometry,
                        expected_basis_geometry,
                        1.0e-9,
                        label,
                    )?;
                }
                (
                    PortedFaceSurface::Offset(actual_surface),
                    PortedFaceSurface::Offset(expected_surface),
                ) => {
                    assert_face_geometry_close(
                        actual_surface.basis_geometry,
                        expected_surface.basis_geometry,
                        1.0e-9,
                        label,
                    )?;
                    assert_same_variant(&actual_surface.basis, &expected_surface.basis, label)?;
                }
                _ => {
                    return Err(std::io::Error::other(format!(
                        "{label} descriptor mismatch: actual={actual:?} expected={expected:?}"
                    ))
                    .into());
                }
            }

            let uv_t = [0.37, 0.61];
            let actual_sample =
                actual.sample_normalized_with_orientation(actual_geometry, uv_t, orientation);
            let expected_sample =
                expected.sample_normalized_with_orientation(expected_geometry, uv_t, orientation);
            assert_vec3_close(
                actual_sample.position,
                expected_sample.position,
                1.0e-8,
                &format!("{label} sample position"),
            )?;
            assert_vec3_close(
                actual_sample.normal,
                expected_sample.normal,
                1.0e-8,
                &format!("{label} sample normal"),
            )?;
            Ok(())
        }
        (actual, expected) => Err(std::io::Error::other(format!(
            "{label} descriptor presence mismatch: actual={actual:?} expected={expected:?}"
        ))
        .into()),
    }
}

fn assert_topology_matches(
    label: &str,
    rust: &lean_occt::TopologySnapshot,
    occt: &lean_occt::TopologySnapshot,
) -> Result<(), Box<dyn std::error::Error>> {
    if rust.vertex_positions.len() != occt.vertex_positions.len()
        || rust.edges.len() != occt.edges.len()
        || rust.wires.len() != occt.wires.len()
        || rust.faces.len() != occt.faces.len()
        || rust.edge_face_indices != occt.edge_face_indices
        || rust.wire_edge_indices != occt.wire_edge_indices
        || rust.wire_edge_orientations != occt.wire_edge_orientations
        || rust.wire_vertex_indices != occt.wire_vertex_indices
        || rust.face_wire_indices != occt.face_wire_indices
        || rust.face_wire_orientations != occt.face_wire_orientations
        || rust.face_wire_roles != occt.face_wire_roles
    {
        return Err(std::io::Error::other(format!(
            "{label} topology mismatch: rust={rust:?} occt={occt:?}"
        ))
        .into());
    }

    for (index, (lhs, rhs)) in rust.edge_faces.iter().zip(&occt.edge_faces).enumerate() {
        if lhs.offset != rhs.offset || lhs.count != rhs.count {
            return Err(std::io::Error::other(format!(
                "{label} edge-face range {index} mismatch: rust={lhs:?} occt={rhs:?}"
            ))
            .into());
        }
    }

    for (index, (lhs, rhs)) in rust
        .wire_vertices
        .iter()
        .zip(&occt.wire_vertices)
        .enumerate()
    {
        if lhs.offset != rhs.offset || lhs.count != rhs.count {
            return Err(std::io::Error::other(format!(
                "{label} wire-vertex range {index} mismatch: rust={lhs:?} occt={rhs:?}"
            ))
            .into());
        }
    }

    for (index, (lhs, rhs)) in rust.faces.iter().zip(&occt.faces).enumerate() {
        if lhs.offset != rhs.offset || lhs.count != rhs.count {
            return Err(std::io::Error::other(format!(
                "{label} face range {index} mismatch: rust={lhs:?} occt={rhs:?}"
            ))
            .into());
        }
    }

    for (index, (lhs, rhs)) in rust
        .vertex_positions
        .iter()
        .zip(&occt.vertex_positions)
        .enumerate()
    {
        assert_vec3_close(*lhs, *rhs, 1.0e-8, &format!("{label} vertex {index}"))?;
    }

    for (index, (lhs, rhs)) in rust.edges.iter().zip(&occt.edges).enumerate() {
        if lhs.start_vertex != rhs.start_vertex
            || lhs.end_vertex != rhs.end_vertex
            || (lhs.length - rhs.length).abs() > 1.0e-8
        {
            return Err(std::io::Error::other(format!(
                "{label} edge {index} mismatch: rust={lhs:?} occt={rhs:?}"
            ))
            .into());
        }
    }

    Ok(())
}

fn assert_topology_backed_subshape_counts_match(
    kernel: &ModelKernel,
    label: &str,
    shape: &Shape,
    topology: &lean_occt::TopologySnapshot,
) -> Result<(), Box<dyn std::error::Error>> {
    for (kind, expected_count) in [
        (ShapeKind::Face, topology.faces.len()),
        (ShapeKind::Wire, topology.wires.len()),
        (ShapeKind::Edge, topology.edges.len()),
        (ShapeKind::Vertex, topology.vertex_positions.len()),
    ] {
        let public_count = kernel.context().subshape_count(shape, kind)?;
        let occt_count = kernel.context().subshape_count_occt(shape, kind)?;
        if public_count != expected_count || public_count != occt_count {
            return Err(std::io::Error::other(format!(
                "{label} {kind:?} count mismatch: public={public_count} rust_topology={expected_count} occt={occt_count}"
            ))
            .into());
        }
    }

    Ok(())
}

fn assert_topology_backed_subshapes_match(
    kernel: &ModelKernel,
    label: &str,
    shape: &Shape,
    topology: &lean_occt::TopologySnapshot,
) -> Result<(), Box<dyn std::error::Error>> {
    for (kind, expected_count) in [
        (ShapeKind::Face, topology.faces.len()),
        (ShapeKind::Wire, topology.wires.len()),
        (ShapeKind::Edge, topology.edges.len()),
        (ShapeKind::Vertex, topology.vertex_positions.len()),
    ] {
        let public_shapes = kernel.context().subshapes(shape, kind)?;
        let occt_shapes = kernel.context().subshapes_occt(shape, kind)?;
        if public_shapes.len() != expected_count || public_shapes.len() != occt_shapes.len() {
            return Err(std::io::Error::other(format!(
                "{label} {kind:?} inventory mismatch: public={} rust_topology={expected_count} occt={}",
                public_shapes.len(),
                occt_shapes.len()
            ))
            .into());
        }

        for (index, (public_shape, occt_shape)) in
            public_shapes.iter().zip(&occt_shapes).enumerate()
        {
            let indexed_shape = kernel.context().subshape(shape, kind, index)?;
            let public_topology = kernel.context().topology_occt(public_shape)?;
            let indexed_topology = kernel.context().topology_occt(&indexed_shape)?;
            let occt_topology = kernel.context().topology_occt(occt_shape)?;
            assert_topology_matches(
                &format!("{label} {kind:?} public subshapes[{index}]"),
                &public_topology,
                &occt_topology,
            )?;
            assert_topology_matches(
                &format!("{label} {kind:?} public subshape({index})"),
                &indexed_topology,
                &occt_topology,
            )?;
        }
    }

    Ok(())
}

fn assert_summary_backed_subshape_counts_match(
    kernel: &ModelKernel,
    label: &str,
    shape: &Shape,
) -> Result<(), Box<dyn std::error::Error>> {
    let summary = kernel.context().describe_shape(shape)?;
    for (kind, expected_count) in [
        (ShapeKind::Compound, summary.compound_count),
        (ShapeKind::CompSolid, summary.compsolid_count),
        (ShapeKind::Solid, summary.solid_count),
        (ShapeKind::Shell, summary.shell_count),
    ] {
        let public_count = kernel.context().subshape_count(shape, kind)?;
        let occt_count = kernel.context().subshape_count_occt(shape, kind)?;
        if public_count != expected_count || public_count != occt_count {
            return Err(std::io::Error::other(format!(
                "{label} {kind:?} count mismatch: public={public_count} rust_summary={expected_count} occt={occt_count}"
            ))
            .into());
        }
    }

    Ok(())
}

fn assert_brep_edge_lengths_match(
    kernel: &ModelKernel,
    label: &str,
    shape: &Shape,
    brep: &lean_occt::BrepShape,
) -> Result<(), Box<dyn std::error::Error>> {
    let edge_shapes = kernel.context().subshapes(shape, ShapeKind::Edge)?;
    if edge_shapes.len() != brep.edges.len() {
        return Err(std::io::Error::other(format!(
            "{label} edge inventory mismatch: public={} brep={}",
            edge_shapes.len(),
            brep.edges.len()
        ))
        .into());
    }

    for (index, edge_shape) in edge_shapes.iter().enumerate() {
        let Some(expected_length) = kernel.context().ported_edge_length(edge_shape)? else {
            continue;
        };
        let actual_length = brep
            .edges
            .get(index)
            .ok_or_else(|| std::io::Error::other(format!("{label} missing brep edge {index}")))?
            .length;
        if (actual_length - expected_length).abs() > 1.0e-9 {
            return Err(std::io::Error::other(format!(
                "{label} edge {index} length mismatch: brep={actual_length} rust={expected_length}"
            ))
            .into());
        }
    }

    Ok(())
}

fn assert_brep_edge_geometries_match_public(
    kernel: &ModelKernel,
    label: &str,
    shape: &Shape,
    brep: &lean_occt::BrepShape,
) -> Result<(), Box<dyn std::error::Error>> {
    let edge_shapes = kernel.context().subshapes(shape, ShapeKind::Edge)?;
    if edge_shapes.len() != brep.edges.len() {
        return Err(std::io::Error::other(format!(
            "{label} edge inventory mismatch: public={} brep={}",
            edge_shapes.len(),
            brep.edges.len()
        ))
        .into());
    }

    for (index, edge_shape) in edge_shapes.iter().enumerate() {
        let expected_geometry = kernel.context().edge_geometry(edge_shape)?;
        let actual_geometry = brep
            .edges
            .get(index)
            .ok_or_else(|| std::io::Error::other(format!("{label} missing brep edge {index}")))?
            .geometry;
        assert_edge_geometry_close(
            actual_geometry,
            expected_geometry,
            1.0e-9,
            &format!("{label} edge {index} geometry"),
        )?;
    }

    Ok(())
}

fn assert_brep_faces_match_public(
    kernel: &ModelKernel,
    label: &str,
    shape: &Shape,
    brep: &lean_occt::BrepShape,
) -> Result<(), Box<dyn std::error::Error>> {
    let face_shapes = kernel.context().subshapes(shape, ShapeKind::Face)?;
    if face_shapes.len() != brep.faces.len() {
        return Err(std::io::Error::other(format!(
            "{label} face inventory mismatch: public={} brep={}",
            face_shapes.len(),
            brep.faces.len()
        ))
        .into());
    }

    for (index, face_shape) in face_shapes.iter().enumerate() {
        let expected_geometry = kernel.context().face_geometry(face_shape)?;
        let expected_descriptor = kernel
            .context()
            .ported_face_surface_descriptor(face_shape)?;
        let expected_area = kernel.context().ported_face_area(face_shape)?;
        let expected_orientation = kernel.context().shape_orientation(face_shape)?;
        let actual_face = brep
            .faces
            .get(index)
            .ok_or_else(|| std::io::Error::other(format!("{label} missing brep face {index}")))?;

        if actual_face.orientation != expected_orientation {
            return Err(std::io::Error::other(format!(
                "{label} face {index} orientation mismatch: actual={:?} expected={:?}",
                actual_face.orientation, expected_orientation
            ))
            .into());
        }

        assert_face_geometry_close(
            actual_face.geometry,
            expected_geometry,
            1.0e-9,
            &format!("{label} face {index} geometry"),
        )?;
        assert_ported_face_surface_matches_public(
            &format!("{label} face {index} descriptor"),
            actual_face.ported_face_surface,
            expected_descriptor,
            actual_face.geometry,
            expected_geometry,
            expected_orientation,
        )?;

        if let Some(expected_area) = expected_area {
            if (actual_face.area - expected_area).abs() > 1.0e-8 {
                return Err(std::io::Error::other(format!(
                    "{label} face {index} area mismatch: brep={} public={expected_area}",
                    actual_face.area
                ))
                .into());
            }
        }
    }

    Ok(())
}

fn assert_summary_matches(
    label: &str,
    rust: &lean_occt::ShapeSummary,
    expected: &lean_occt::ShapeSummary,
) -> Result<(), Box<dyn std::error::Error>> {
    if rust.root_kind != expected.root_kind
        || rust.primary_kind != expected.primary_kind
        || rust.compound_count != expected.compound_count
        || rust.compsolid_count != expected.compsolid_count
        || rust.solid_count != expected.solid_count
        || rust.shell_count != expected.shell_count
        || rust.face_count != expected.face_count
        || rust.wire_count != expected.wire_count
        || rust.edge_count != expected.edge_count
        || rust.vertex_count != expected.vertex_count
    {
        return Err(std::io::Error::other(format!(
            "{label} summary count mismatch: rust={rust:?} expected={expected:?}"
        ))
        .into());
    }

    if (rust.linear_length - expected.linear_length).abs() > 1.0e-9
        || (rust.surface_area - expected.surface_area).abs() > 1.0e-9
        || (rust.volume - expected.volume).abs() > 1.0e-9
    {
        return Err(std::io::Error::other(format!(
            "{label} summary metric mismatch: rust={rust:?} expected={expected:?}"
        ))
        .into());
    }

    assert_bbox_close(
        label,
        rust.bbox_min,
        rust.bbox_max,
        expected.bbox_min,
        expected.bbox_max,
        1.0e-9,
    )?;

    Ok(())
}

fn dot3(lhs: [f64; 3], rhs: [f64; 3]) -> f64 {
    lhs[0] * rhs[0] + lhs[1] * rhs[1] + lhs[2] * rhs[2]
}

fn norm3(value: [f64; 3]) -> f64 {
    dot3(value, value).sqrt()
}

fn normalize3(value: [f64; 3]) -> [f64; 3] {
    let length = norm3(value);
    [value[0] / length, value[1] / length, value[2] / length]
}

fn subtract3(lhs: [f64; 3], rhs: [f64; 3]) -> [f64; 3] {
    [lhs[0] - rhs[0], lhs[1] - rhs[1], lhs[2] - rhs[2]]
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
    let context_summary = document
        .kernel()
        .context()
        .describe_shape(document.shape("cut")?)?;
    let context_topology = document.topology("cut")?;
    let occt_summary = document
        .kernel()
        .context()
        .describe_shape_occt(document.shape("cut")?)?;

    assert_eq!(brep.summary.primary_kind, ShapeKind::Solid);
    assert_eq!(brep.faces.len(), 7);
    assert_eq!(brep.summary.face_count, brep.faces.len());
    assert_eq!(brep.topology.vertex_positions.len(), brep.vertices.len());
    assert!(!brep.wires.is_empty());
    assert!(brep.wires.iter().all(|wire| !wire.edge_indices.is_empty()));
    assert_eq!(kernel_summary.face_count, brep.faces.len());
    assert_eq!(kernel_summary.edge_count, brep.edges.len());
    assert_summary_matches("context summary", &context_summary, &brep.summary)?;
    assert_topology_matches("context topology", &context_topology, &brep.topology)?;
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
            .describe_shape_occt(document.shape(name)?)?;
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
fn ported_brep_uses_exact_primitive_bounding_boxes() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let kernel = ModelKernel::new()?;
    let axis = normalize3([0.0, 1.0, 1.0]);

    let cylinder = kernel.make_cylinder(CylinderParams {
        origin: [4.0, -3.0, 1.5],
        axis,
        radius: 6.0,
        height: 18.0,
    })?;
    let cone = kernel.make_cone(ConeParams {
        origin: [-6.0, 5.0, 2.0],
        axis,
        x_direction: [1.0, 0.0, 0.0],
        base_radius: 9.0,
        top_radius: 3.0,
        height: 15.0,
    })?;
    let sphere = kernel.make_sphere(SphereParams {
        origin: [5.0, -4.0, 3.0],
        axis,
        x_direction: [1.0, 0.0, 0.0],
        radius: 7.0,
    })?;
    for (label, shape, artifact_name) in [
        ("cylinder", &cylinder, "bbox_rotated_cylinder"),
        ("cone", &cone, "bbox_rotated_cone"),
        ("sphere", &sphere, "bbox_rotated_sphere"),
    ] {
        let artifact =
            support::export_kernel_shape(&kernel, shape, "brep_workflows", artifact_name)?;
        let summary = kernel.summarize(shape)?;
        let brep = kernel.brep(shape)?;
        let occt_summary = kernel.context().describe_shape_occt(shape)?;

        assert_bbox_close(
            label,
            summary.bbox_min,
            summary.bbox_max,
            occt_summary.bbox_min,
            occt_summary.bbox_max,
            5.0e-7,
        )?;
        assert_bbox_close(
            label,
            brep.summary.bbox_min,
            brep.summary.bbox_max,
            occt_summary.bbox_min,
            occt_summary.bbox_max,
            5.0e-7,
        )?;
        assert!(artifact.is_file(), "{label} artifact should exist");
    }

    Ok(())
}

#[test]
fn ported_brep_uses_exact_curve_bounding_boxes() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let kernel = ModelKernel::new()?;
    let ellipse = kernel.make_ellipse_edge(EllipseEdgeParams {
        origin: [30.0, 4.0, -2.0],
        axis: [0.0, 1.0, 0.0],
        x_direction: normalize3([1.0, 0.0, 1.0]),
        major_radius: 10.0,
        minor_radius: 6.0,
    })?;
    let cylinder = kernel.make_cylinder(CylinderParams {
        origin: [-12.0, -6.0, 3.0],
        axis: [0.0, 0.0, 1.0],
        radius: 5.0,
        height: 14.0,
    })?;
    let circle_face = find_first_face_by_kind(&kernel, &cylinder, SurfaceKind::Plane)?;
    let circle_edge = find_first_edge_by_kind(&kernel, &cylinder, CurveKind::Circle)?;
    let prism = kernel.make_prism(
        &circle_face,
        PrismParams {
            direction: [8.0, 24.0, -5.0],
        },
    )?;

    for (label, shape) in [
        ("ellipse_edge", &ellipse),
        ("circle_face", &circle_face),
        ("circle_edge", &circle_edge),
        ("oblique_circle_prism", &prism),
    ] {
        let summary = kernel.summarize(shape)?;
        let brep = kernel.brep(shape)?;
        let occt_summary = kernel.context().describe_shape_occt(shape)?;

        assert_bbox_close(
            label,
            summary.bbox_min,
            summary.bbox_max,
            occt_summary.bbox_min,
            occt_summary.bbox_max,
            5.0e-7,
        )?;
        assert_bbox_close(
            label,
            brep.summary.bbox_min,
            brep.summary.bbox_max,
            occt_summary.bbox_min,
            occt_summary.bbox_max,
            5.0e-7,
        )?;
    }

    Ok(())
}

#[test]
fn ported_brep_uses_rust_owned_topology_for_face_free_shapes(
) -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let kernel = ModelKernel::new()?;
    let ellipse = kernel.make_ellipse_edge(EllipseEdgeParams {
        origin: [30.0, 0.0, 0.0],
        axis: [0.0, 1.0, 0.0],
        x_direction: [1.0, 0.0, 0.0],
        major_radius: 10.0,
        minor_radius: 6.0,
    })?;
    let helix = kernel.make_helix(HelixParams {
        origin: [0.0, 0.0, 0.0],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        radius: 20.0,
        height: 30.0,
        pitch: 10.0,
    })?;
    let cut = kernel.box_with_through_hole(default_cut())?;

    for (label, shape) in [("ellipse", &ellipse), ("helix", &helix)] {
        let rust_topology = kernel
            .context()
            .ported_topology(shape)?
            .ok_or_else(|| std::io::Error::other(format!("expected Rust topology for {label}")))?;
        let occt_topology = kernel.context().topology_occt(shape)?;
        let brep = kernel.brep(shape)?;

        assert_topology_matches(label, &rust_topology, &occt_topology)?;
        assert_topology_backed_subshape_counts_match(&kernel, label, shape, &rust_topology)?;
        assert_topology_backed_subshapes_match(&kernel, label, shape, &rust_topology)?;
        assert_summary_backed_subshape_counts_match(&kernel, label, shape)?;
        assert_topology_matches(label, &brep.topology, &rust_topology)?;
        assert_brep_edge_geometries_match_public(&kernel, label, shape, &brep)?;
        assert_brep_faces_match_public(&kernel, label, shape, &brep)?;
        assert_brep_edge_lengths_match(&kernel, label, shape, &brep)?;
    }

    assert!(kernel.context().ported_topology(&cut)?.is_some());

    Ok(())
}

#[test]
fn ported_brep_uses_rust_owned_topology_for_simple_single_face_shapes(
) -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let kernel = ModelKernel::new()?;
    let cylinder = kernel.make_cylinder(CylinderParams {
        origin: [0.0, 0.0, -7.0],
        axis: [0.0, 0.0, 1.0],
        radius: 5.0,
        height: 14.0,
    })?;
    let box_shape = kernel.make_box(BoxParams {
        origin: [-10.0, -10.0, -10.0],
        size: [20.0, 20.0, 20.0],
    })?;
    let circle_face = find_first_face_by_kind(&kernel, &cylinder, SurfaceKind::Plane)?;
    let box_face = find_first_face_by_kind(&kernel, &box_shape, SurfaceKind::Plane)?;
    let cut = kernel.box_with_through_hole(default_cut())?;
    let holed_planar_face = kernel
        .context()
        .subshapes(&cut, ShapeKind::Face)?
        .into_iter()
        .find(|face| {
            kernel
                .context()
                .face_geometry(face)
                .map(|geometry| geometry.kind == SurfaceKind::Plane)
                .unwrap_or(false)
                && kernel
                    .context()
                    .topology_occt(face)
                    .map(|topology| {
                        topology
                            .faces
                            .first()
                            .map(|range| range.count > 1)
                            .unwrap_or(false)
                    })
                    .unwrap_or(false)
        })
        .ok_or_else(|| std::io::Error::other("expected a holed planar face"))?;

    for (label, shape) in [
        ("circle_face", &circle_face),
        ("box_face", &box_face),
        ("holed_planar_face", &holed_planar_face),
    ] {
        let rust_topology = kernel
            .context()
            .ported_topology(shape)?
            .ok_or_else(|| std::io::Error::other(format!("expected Rust topology for {label}")))?;
        let occt_topology = kernel.context().topology_occt(shape)?;
        let brep = kernel.brep(shape)?;

        assert_topology_matches(label, &rust_topology, &occt_topology)?;
        assert_topology_backed_subshape_counts_match(&kernel, label, shape, &rust_topology)?;
        assert_topology_backed_subshapes_match(&kernel, label, shape, &rust_topology)?;
        assert_summary_backed_subshape_counts_match(&kernel, label, shape)?;
        assert_topology_matches(label, &brep.topology, &rust_topology)?;
        assert_brep_edge_geometries_match_public(&kernel, label, shape, &brep)?;
        assert_brep_faces_match_public(&kernel, label, shape, &brep)?;
        assert_brep_edge_lengths_match(&kernel, label, shape, &brep)?;
        assert_eq!(rust_topology.faces.len(), 1);
        assert_eq!(
            rust_topology
                .face_wire_roles
                .iter()
                .filter(|&&role| role == LoopRole::Outer)
                .count(),
            1
        );
        match label {
            "holed_planar_face" => {
                assert!(rust_topology.wires.len() > 1);
                assert!(rust_topology
                    .face_wire_roles
                    .iter()
                    .any(|&role| role == LoopRole::Inner));
            }
            _ => {
                assert_eq!(rust_topology.wires.len(), 1);
                assert_eq!(rust_topology.face_wire_roles, vec![LoopRole::Outer]);
            }
        }
    }

    assert!(kernel.context().ported_topology(&cut)?.is_some());

    Ok(())
}

#[test]
fn ported_brep_uses_rust_owned_topology_for_simple_multi_face_solids(
) -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let kernel = ModelKernel::new()?;
    let box_shape = kernel.make_box(BoxParams {
        origin: [-10.0, -10.0, -10.0],
        size: [20.0, 20.0, 20.0],
    })?;
    let cylinder = kernel.make_cylinder(CylinderParams {
        origin: [0.0, 0.0, -7.0],
        axis: [0.0, 0.0, 1.0],
        radius: 5.0,
        height: 14.0,
    })?;
    let cone = kernel.make_cone(ConeParams {
        origin: [0.0, 0.0, -8.0],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        base_radius: 6.0,
        top_radius: 2.0,
        height: 16.0,
    })?;
    let cut = kernel.box_with_through_hole(default_cut())?;

    for (label, shape, expected_face_count) in [
        ("box_solid", &box_shape, 6usize),
        ("cylinder_solid", &cylinder, 3usize),
        ("cone_solid", &cone, 3usize),
        ("through_hole_cut", &cut, 7usize),
    ] {
        let rust_topology = kernel
            .context()
            .ported_topology(shape)?
            .ok_or_else(|| std::io::Error::other(format!("expected Rust topology for {label}")))?;
        let occt_topology = kernel.context().topology_occt(shape)?;
        let brep = kernel.brep(shape)?;

        assert_topology_matches(label, &rust_topology, &occt_topology)?;
        assert_topology_backed_subshape_counts_match(&kernel, label, shape, &rust_topology)?;
        assert_topology_backed_subshapes_match(&kernel, label, shape, &rust_topology)?;
        assert_summary_backed_subshape_counts_match(&kernel, label, shape)?;
        assert_topology_matches(label, &brep.topology, &rust_topology)?;
        assert_brep_edge_geometries_match_public(&kernel, label, shape, &brep)?;
        assert_brep_faces_match_public(&kernel, label, shape, &brep)?;
        assert_brep_edge_lengths_match(&kernel, label, shape, &brep)?;
        assert_eq!(rust_topology.faces.len(), expected_face_count);
        assert!(rust_topology
            .edge_faces
            .iter()
            .all(|range| range.count >= 1 && range.count <= 2));
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
    let occt_summary = kernel.context().describe_shape_occt(&revolved)?;
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
    let extrusion_face = brep
        .faces
        .iter()
        .find(|face| face.geometry.kind == SurfaceKind::Extrusion)
        .ok_or_else(|| std::io::Error::other("expected an extrusion face in the revolved solid"))?;
    assert!(matches!(
        extrusion_face.ported_face_surface,
        Some(PortedFaceSurface::Swept(
            PortedSweptSurface::Extrusion { .. }
        ))
    ));
    let revolution_face = brep
        .faces
        .iter()
        .find(|face| face.geometry.kind == SurfaceKind::Revolution)
        .ok_or_else(|| std::io::Error::other("expected a revolution face in the revolved solid"))?;
    assert!(matches!(
        revolution_face.ported_face_surface,
        Some(PortedFaceSurface::Swept(
            PortedSweptSurface::Revolution { .. }
        ))
    ));
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
fn ported_brep_uses_rust_owned_area_for_offset_faces() -> Result<(), Box<dyn std::error::Error>> {
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
        "offset_surface_rust_area",
    )?;
    let brep = kernel.brep(&offset_surface)?;
    let offset_face = brep
        .faces
        .iter()
        .find(|face| face.geometry.kind == SurfaceKind::Offset)
        .ok_or_else(|| std::io::Error::other("expected an offset face"))?;
    let occt_face = find_first_face_by_kind(&kernel, &offset_surface, SurfaceKind::Offset)?;
    let occt_area = kernel
        .context()
        .describe_shape_occt(&occt_face)?
        .surface_area;
    let occt_sample = kernel
        .context()
        .face_sample_normalized_occt(&occt_face, [0.5, 0.5])?;

    assert_eq!(brep.summary.primary_kind, ShapeKind::Shell);
    assert_eq!(brep.summary.face_count, 1);
    assert!(
        offset_face.ported_surface.is_none(),
        "offset face should stay on the dedicated offset surface path"
    );
    assert!(matches!(
        offset_face.ported_face_surface,
        Some(PortedFaceSurface::Offset(_))
    ));
    assert!(
        (offset_face.area - occt_area).abs() <= 5.0e-1,
        "offset face area drifted from OCCT: rust={} occt={}",
        offset_face.area,
        occt_area
    );
    assert!(
        (norm3(offset_face.sample.normal) - 1.0).abs() <= 1.0e-3,
        "offset face sample normal was not unit length: {:?}",
        offset_face.sample.normal
    );
    assert!(
        dot3(offset_face.sample.normal, occt_sample.normal).abs() >= 0.6,
        "offset face sample normal drifted from OCCT center sample: rust={:?} occt={:?}",
        offset_face.sample.normal,
        occt_sample.normal
    );
    assert!(
        norm3(subtract3(offset_face.sample.position, occt_sample.position)) <= 8.0,
        "offset face sample position drifted too far from OCCT center sample: rust={:?} occt={:?}",
        offset_face.sample.position,
        occt_sample.position
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
fn ported_brep_uses_rust_owned_volume_for_offset_solids() -> Result<(), Box<dyn std::error::Error>>
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
    let offset = kernel.make_offset(
        &revolved,
        OffsetParams {
            offset: 2.5,
            tolerance: 1.0e-4,
        },
    )?;

    let artifact = support::export_kernel_shape(
        &kernel,
        &offset,
        "brep_workflows",
        "offset_solid_rust_volume",
    )?;
    let summary = kernel.summarize(&offset)?;
    let occt_summary = kernel.context().describe_shape_occt(&offset)?;
    let brep = kernel.brep(&offset)?;
    let offset_faces = brep
        .faces
        .iter()
        .filter(|face| face.geometry.kind == SurfaceKind::Offset)
        .collect::<Vec<_>>();

    assert_eq!(summary.primary_kind, ShapeKind::Solid);
    assert!(
        !offset_faces.is_empty(),
        "expected offset solid to retain offset faces"
    );
    assert!(
        offset_faces.iter().any(|face| matches!(
            face.ported_face_surface,
            Some(PortedFaceSurface::Offset(surface))
                if matches!(surface.basis, PortedOffsetBasisSurface::Swept(_))
        )),
        "expected at least one Rust-owned swept offset face descriptor in offset solid"
    );
    assert!(
        (summary.volume - occt_summary.volume).abs() <= 3.0e2,
        "offset solid volume drifted from OCCT: rust={} occt={}",
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
    let cut_occt = kernel.context().describe_shape_occt(&cut)?;
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
    let sphere_occt = kernel.context().describe_shape_occt(&sphere)?;
    assert_bbox_close(
        "sphere",
        sphere_summary.bbox_min,
        sphere_summary.bbox_max,
        sphere_occt.bbox_min,
        sphere_occt.bbox_max,
        5.0e-7,
    )?;

    let helix_summary = kernel.summarize(&helix)?;
    let helix_occt = kernel.context().describe_shape_occt(&helix)?;
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
        let occt_summary = kernel.context().describe_shape_occt(shape)?;
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
