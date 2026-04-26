mod support;

use std::f64::consts::PI;

use lean_occt::{
    BoxParams, ConeParams, CurveKind, CylinderParams, EllipseEdgeParams, HelixParams, LoopRole,
    ModelDocument, ModelKernel, OffsetFaceBboxSource, OffsetParams, OffsetShellBboxSource,
    PortedCurve, PortedFaceSurface, PortedOffsetBasisSurface, PortedSurface, PortedSweptSurface,
    PrismParams, RevolutionParams, Shape, ShapeKind, SphereParams, SummaryBboxSource,
    SummaryVolumeSource, SurfaceKind, ThroughHoleCut, TorusParams,
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

fn assert_revolution_payload_close(
    lhs: lean_occt::RevolutionSurfacePayload,
    rhs: lean_occt::RevolutionSurfacePayload,
    tolerance: f64,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_vec3_close(
        lhs.axis_origin,
        rhs.axis_origin,
        tolerance,
        &format!("{label} axis_origin"),
    )?;
    assert_vec3_close(
        lhs.axis_direction,
        rhs.axis_direction,
        tolerance,
        &format!("{label} axis_direction"),
    )?;
    if lhs.basis_curve_kind != rhs.basis_curve_kind {
        return Err(std::io::Error::other(format!(
            "{label} basis_curve_kind mismatch: lhs={:?} rhs={:?}",
            lhs.basis_curve_kind, rhs.basis_curve_kind
        ))
        .into());
    }
    Ok(())
}

fn assert_extrusion_payload_close(
    lhs: lean_occt::ExtrusionSurfacePayload,
    rhs: lean_occt::ExtrusionSurfacePayload,
    tolerance: f64,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_vec3_close(
        lhs.direction,
        rhs.direction,
        tolerance,
        &format!("{label} direction"),
    )?;
    if lhs.basis_curve_kind != rhs.basis_curve_kind {
        return Err(std::io::Error::other(format!(
            "{label} basis_curve_kind mismatch: lhs={:?} rhs={:?}",
            lhs.basis_curve_kind, rhs.basis_curve_kind
        ))
        .into());
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
                    assert_extrusion_payload_close(
                        actual_payload,
                        expected_payload,
                        1.0e-8,
                        &format!("{label} extrusion payload"),
                    )?;
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
                    assert_revolution_payload_close(
                        actual_payload,
                        expected_payload,
                        1.0e-8,
                        &format!("{label} revolution payload"),
                    )?;
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

fn assert_ported_surface_kind(
    surface: PortedSurface,
    expected_kind: SurfaceKind,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    match (expected_kind, surface) {
        (SurfaceKind::Plane, PortedSurface::Plane(_))
        | (SurfaceKind::Cylinder, PortedSurface::Cylinder(_))
        | (SurfaceKind::Cone, PortedSurface::Cone(_))
        | (SurfaceKind::Sphere, PortedSurface::Sphere(_))
        | (SurfaceKind::Torus, PortedSurface::Torus(_)) => Ok(()),
        _ => Err(std::io::Error::other(format!(
            "{label} expected {expected_kind:?} analytic surface, got {surface:?}"
        ))
        .into()),
    }
}

fn assert_offset_basis_kind(
    basis: PortedOffsetBasisSurface,
    expected_kind: SurfaceKind,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    match (expected_kind, basis) {
        (SurfaceKind::Plane, PortedOffsetBasisSurface::Analytic(PortedSurface::Plane(_)))
        | (SurfaceKind::Cylinder, PortedOffsetBasisSurface::Analytic(PortedSurface::Cylinder(_)))
        | (SurfaceKind::Cone, PortedOffsetBasisSurface::Analytic(PortedSurface::Cone(_)))
        | (SurfaceKind::Sphere, PortedOffsetBasisSurface::Analytic(PortedSurface::Sphere(_)))
        | (SurfaceKind::Torus, PortedOffsetBasisSurface::Analytic(PortedSurface::Torus(_)))
        | (
            SurfaceKind::Extrusion,
            PortedOffsetBasisSurface::Swept(PortedSweptSurface::Extrusion { .. }),
        )
        | (
            SurfaceKind::Revolution,
            PortedOffsetBasisSurface::Swept(PortedSweptSurface::Revolution { .. }),
        ) => Ok(()),
        _ => Err(std::io::Error::other(format!(
            "{label} expected {expected_kind:?} offset basis, got {basis:?}"
        ))
        .into()),
    }
}

fn assert_brep_analytic_faces_use_rust_surface_route(
    kernel: &ModelKernel,
    label: &str,
    shape: &Shape,
    brep: &lean_occt::BrepShape,
    expected_kind: SurfaceKind,
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

    let mut matched_faces = 0usize;
    for (index, face_shape) in face_shapes.iter().enumerate() {
        let public_geometry = kernel.context().face_geometry(face_shape)?;
        if public_geometry.kind != expected_kind {
            continue;
        }
        matched_faces += 1;

        let actual_face = brep
            .faces
            .get(index)
            .ok_or_else(|| std::io::Error::other(format!("{label} missing brep face {index}")))?;
        let surface = actual_face.ported_surface.ok_or_else(|| {
            std::io::Error::other(format!(
                "{label} face {index} did not populate BrepFace::ported_surface"
            ))
        })?;
        assert_ported_surface_kind(
            surface,
            expected_kind,
            &format!("{label} face {index} ported surface"),
        )?;

        let descriptor_surface = match actual_face.ported_face_surface {
            Some(PortedFaceSurface::Analytic(surface)) => surface,
            other => {
                return Err(std::io::Error::other(format!(
                    "{label} face {index} expected analytic face descriptor, got {other:?}"
                ))
                .into())
            }
        };
        assert_same_variant(
            &descriptor_surface,
            &surface,
            &format!("{label} face {index} analytic descriptor"),
        )?;

        let raw_geometry = kernel.context().face_geometry_occt(face_shape)?;
        if raw_geometry.kind != expected_kind {
            return Err(std::io::Error::other(format!(
                "{label} face {index} raw geometry kind mismatch: raw={:?} expected={expected_kind:?}",
                raw_geometry.kind
            ))
            .into());
        }
        assert_face_geometry_close(
            actual_face.geometry,
            raw_geometry,
            1.0e-9,
            &format!("{label} face {index} raw geometry"),
        )?;

        let raw_surface =
            PortedSurface::from_context_with_geometry(kernel.context(), face_shape, raw_geometry)?
                .ok_or_else(|| {
                    std::io::Error::other(format!(
                        "{label} face {index} raw geometry route returned no ported surface"
                    ))
                })?;
        assert_same_variant(
            &raw_surface,
            &surface,
            &format!("{label} face {index} raw surface route"),
        )?;

        let uv_t = [0.37, 0.61];
        let surface_sample = surface.sample_normalized_with_orientation(
            actual_face.geometry,
            uv_t,
            actual_face.orientation,
        );
        let descriptor_sample = descriptor_surface.sample_normalized_with_orientation(
            actual_face.geometry,
            uv_t,
            actual_face.orientation,
        );
        let raw_sample = raw_surface.sample_normalized_with_orientation(
            raw_geometry,
            uv_t,
            actual_face.orientation,
        );
        let occt_sample = kernel
            .context()
            .face_sample_normalized_occt(face_shape, uv_t)?;
        assert_vec3_close(
            surface_sample.position,
            descriptor_sample.position,
            1.0e-12,
            &format!("{label} face {index} descriptor sample position"),
        )?;
        assert_vec3_close(
            surface_sample.normal,
            descriptor_sample.normal,
            1.0e-12,
            &format!("{label} face {index} descriptor sample normal"),
        )?;
        assert_vec3_close(
            surface_sample.position,
            raw_sample.position,
            1.0e-8,
            &format!("{label} face {index} raw route sample position"),
        )?;
        assert_vec3_close(
            surface_sample.normal,
            raw_sample.normal,
            1.0e-8,
            &format!("{label} face {index} raw route sample normal"),
        )?;
        assert_vec3_close(
            surface_sample.position,
            occt_sample.position,
            1.0e-6,
            &format!("{label} face {index} OCCT sample position"),
        )?;
        assert_vec3_close(
            surface_sample.normal,
            occt_sample.normal,
            1.0e-6,
            &format!("{label} face {index} OCCT sample normal"),
        )?;

        let expected_area = kernel
            .context()
            .ported_face_area(face_shape)?
            .ok_or_else(|| {
                std::io::Error::other(format!(
                    "{label} face {index} did not produce a ported face area"
                ))
            })?;
        if (actual_face.area - expected_area).abs() > 1.0e-8 {
            return Err(std::io::Error::other(format!(
                "{label} face {index} area mismatch: brep={} public={expected_area}",
                actual_face.area
            ))
            .into());
        }
    }

    if matched_faces == 0 {
        return Err(std::io::Error::other(format!(
            "{label} did not expose any {expected_kind:?} faces"
        ))
        .into());
    }

    Ok(())
}

fn assert_offset_brep_uses_rust_basis(
    kernel: &ModelKernel,
    label: &str,
    source: &Shape,
    expected_kind: SurfaceKind,
) -> Result<(), Box<dyn std::error::Error>> {
    let source_face = find_first_face_by_kind(kernel, source, expected_kind)?;
    let offset_shape = kernel.context().make_offset_surface_face(
        &source_face,
        OffsetParams {
            offset: 1.25,
            tolerance: 1.0e-4,
        },
    )?;
    let brep = kernel.brep(&offset_shape)?;
    let offset_face = brep
        .faces
        .iter()
        .find(|face| face.geometry.kind == SurfaceKind::Offset)
        .ok_or_else(|| std::io::Error::other(format!("{label} expected an offset brep face")))?;
    let occt_area = kernel
        .context()
        .describe_shape_occt(&offset_shape)?
        .surface_area;
    let occt_sample = kernel
        .context()
        .face_sample_normalized_occt(&offset_shape, [0.37, 0.61])?;

    assert!(
        offset_face.ported_surface.is_none(),
        "{label} offset face should use the dedicated offset descriptor path"
    );
    let surface = match offset_face.ported_face_surface {
        Some(PortedFaceSurface::Offset(surface)) => surface,
        other => {
            return Err(std::io::Error::other(format!(
                "{label} expected offset face descriptor, got {other:?}"
            ))
            .into())
        }
    };
    assert_eq!(
        surface.payload.basis_surface_kind, expected_kind,
        "{label} offset basis kind mismatch"
    );
    assert_offset_basis_kind(
        surface.basis,
        expected_kind,
        &format!("{label} offset descriptor basis"),
    )?;
    assert!(
        (offset_face.area - occt_area).abs() <= 5.0e-1,
        "{label} offset face area drifted from OCCT: rust={} occt={}",
        offset_face.area,
        occt_area
    );
    let rust_sample = PortedFaceSurface::Offset(surface).sample_normalized_with_orientation(
        offset_face.geometry,
        [0.37, 0.61],
        offset_face.orientation,
    );
    assert_vec3_close(
        rust_sample.position,
        occt_sample.position,
        1.0e-6,
        &format!("{label} offset descriptor sample position"),
    )?;
    assert_vec3_close(
        rust_sample.normal,
        occt_sample.normal,
        1.0e-6,
        &format!("{label} offset descriptor sample normal"),
    )?;

    Ok(())
}

fn assert_topology_matches(
    label: &str,
    rust: &lean_occt::TopologySnapshot,
    occt: &lean_occt::TopologySnapshot,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_topology_matches_with_edge_length_mode(label, rust, occt, true)
}

fn assert_topology_matches_ignoring_edge_lengths(
    label: &str,
    rust: &lean_occt::TopologySnapshot,
    occt: &lean_occt::TopologySnapshot,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_topology_matches_with_edge_length_mode(label, rust, occt, false)
}

fn assert_topology_matches_with_edge_length_mode(
    label: &str,
    rust: &lean_occt::TopologySnapshot,
    occt: &lean_occt::TopologySnapshot,
    compare_edge_lengths: bool,
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
            || (compare_edge_lengths && (lhs.length - rhs.length).abs() > 1.0e-8)
        {
            return Err(std::io::Error::other(format!(
                "{label} edge {index} mismatch: rust={lhs:?} occt={rhs:?}"
            ))
            .into());
        }
    }

    Ok(())
}

fn assert_supported_brep_materializes_from_ported_topology(
    kernel: &ModelKernel,
    label: &str,
    shape: &Shape,
) -> Result<(), Box<dyn std::error::Error>> {
    let rust_topology = kernel
        .context()
        .ported_topology(shape)?
        .ok_or_else(|| std::io::Error::other(format!("{label} missing Rust topology")))?;
    let occt_topology = kernel.context().topology_occt(shape)?;
    let brep = kernel.brep(shape)?;

    assert_topology_matches_ignoring_edge_lengths(
        &format!("{label} ported topology parity"),
        &rust_topology,
        &occt_topology,
    )?;
    assert_topology_matches(
        &format!("{label} BRep topology source"),
        &brep.topology,
        &rust_topology,
    )?;

    Ok(())
}

fn topology_has_repeated_wire_edge_occurrence(topology: &lean_occt::TopologySnapshot) -> bool {
    topology.wires.iter().any(|wire| {
        let edge_indices = &topology.wire_edge_indices[wire.offset..wire.offset + wire.count];
        edge_indices.iter().enumerate().any(|(index, edge_index)| {
            edge_indices
                .iter()
                .skip(index + 1)
                .any(|other_edge_index| other_edge_index == edge_index)
        })
    })
}

fn assert_topology_backed_subshape_counts_match(
    kernel: &ModelKernel,
    label: &str,
    shape: &Shape,
    topology: &lean_occt::TopologySnapshot,
) -> Result<(), Box<dyn std::error::Error>> {
    let public_topology = kernel.context().topology(shape)?;
    assert_topology_matches(
        &format!("{label} public topology"),
        &public_topology,
        topology,
    )?;

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

fn topology_vertex_position(
    topology: &lean_occt::TopologySnapshot,
    vertex_index: usize,
    label: &str,
) -> Result<[f64; 3], Box<dyn std::error::Error>> {
    topology
        .vertex_positions
        .get(vertex_index)
        .copied()
        .ok_or_else(|| {
            std::io::Error::other(format!(
                "{label} referenced missing topology vertex {vertex_index}"
            ))
            .into()
        })
}

fn assert_topology_edges_match_public_queries(
    kernel: &ModelKernel,
    label: &str,
    shape: &Shape,
    topology: &lean_occt::TopologySnapshot,
    require_supported_edges: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let edge_shapes = kernel.context().subshapes(shape, ShapeKind::Edge)?;
    if edge_shapes.len() != topology.edges.len() {
        return Err(std::io::Error::other(format!(
            "{label} topology edge inventory mismatch: public={} topology={}",
            edge_shapes.len(),
            topology.edges.len()
        ))
        .into());
    }

    let mut supported_edges = 0usize;
    for (index, edge_shape) in edge_shapes.iter().enumerate() {
        let public_geometry = kernel.context().edge_geometry(edge_shape)?;
        let public_endpoints = kernel.context().edge_endpoints(edge_shape)?;
        if matches!(
            public_geometry.kind,
            CurveKind::Line | CurveKind::Circle | CurveKind::Ellipse
        ) {
            supported_edges += 1;
        }

        let topology_edge = topology.edges.get(index).ok_or_else(|| {
            std::io::Error::other(format!("{label} missing topology edge {index}"))
        })?;
        let start_vertex = topology_edge.start_vertex.ok_or_else(|| {
            std::io::Error::other(format!(
                "{label} topology edge {index} missing start vertex"
            ))
        })?;
        let end_vertex = topology_edge.end_vertex.ok_or_else(|| {
            std::io::Error::other(format!("{label} topology edge {index} missing end vertex"))
        })?;
        let start = topology_vertex_position(
            topology,
            start_vertex,
            &format!("{label} topology edge {index} start"),
        )?;
        let end = topology_vertex_position(
            topology,
            end_vertex,
            &format!("{label} topology edge {index} end"),
        )?;
        assert_vec3_close(
            start,
            public_endpoints.start,
            1.0e-7,
            &format!("{label} topology edge {index} public start"),
        )?;
        assert_vec3_close(
            end,
            public_endpoints.end,
            1.0e-7,
            &format!("{label} topology edge {index} public end"),
        )?;
        if let Some(expected_length) = kernel.context().ported_edge_length(edge_shape)? {
            if (topology_edge.length - expected_length).abs() > 1.0e-9 {
                return Err(std::io::Error::other(format!(
                    "{label} topology edge {index} length mismatch: topology={} rust={expected_length}",
                    topology_edge.length
                ))
                .into());
            }
        }
    }

    if require_supported_edges && supported_edges == 0 {
        return Err(std::io::Error::other(format!(
            "{label} topology did not exercise supported line/circle/ellipse edges"
        ))
        .into());
    }

    for (wire_index, wire_range) in topology.wires.iter().enumerate() {
        let vertex_range = topology.wire_vertices.get(wire_index).ok_or_else(|| {
            std::io::Error::other(format!("{label} missing wire vertex range {wire_index}"))
        })?;
        if wire_range.count == 0 {
            continue;
        }
        if vertex_range.count != wire_range.count + 1 {
            return Err(std::io::Error::other(format!(
                "{label} wire {wire_index} vertex chain length mismatch: vertices={} edges={}",
                vertex_range.count, wire_range.count
            ))
            .into());
        }

        for occurrence_offset in 0..wire_range.count {
            let edge_offset = wire_range.offset + occurrence_offset;
            let edge_index = *topology.wire_edge_indices.get(edge_offset).ok_or_else(|| {
                std::io::Error::other(format!(
                    "{label} wire {wire_index} missing edge occurrence {occurrence_offset}"
                ))
            })?;
            let orientation = *topology
                .wire_edge_orientations
                .get(edge_offset)
                .ok_or_else(|| {
                    std::io::Error::other(format!(
                        "{label} wire {wire_index} missing edge orientation {occurrence_offset}"
                    ))
                })?;
            let edge_shape = edge_shapes.get(edge_index).ok_or_else(|| {
                std::io::Error::other(format!(
                    "{label} wire {wire_index} referenced missing edge {edge_index}"
                ))
            })?;
            let endpoints = kernel.context().edge_endpoints(edge_shape)?;
            let (expected_start, expected_end) =
                if matches!(orientation, lean_occt::Orientation::Reversed) {
                    (endpoints.end, endpoints.start)
                } else {
                    (endpoints.start, endpoints.end)
                };

            let start_vertex_index = *topology
                .wire_vertex_indices
                .get(vertex_range.offset + occurrence_offset)
                .ok_or_else(|| {
                    std::io::Error::other(format!(
                        "{label} wire {wire_index} missing start vertex occurrence {occurrence_offset}"
                    ))
                })?;
            let end_vertex_index = *topology
                .wire_vertex_indices
                .get(vertex_range.offset + occurrence_offset + 1)
                .ok_or_else(|| {
                    std::io::Error::other(format!(
                        "{label} wire {wire_index} missing end vertex occurrence {occurrence_offset}"
                    ))
                })?;
            let start = topology_vertex_position(
                topology,
                start_vertex_index,
                &format!("{label} wire {wire_index} occurrence {occurrence_offset} start"),
            )?;
            let end = topology_vertex_position(
                topology,
                end_vertex_index,
                &format!("{label} wire {wire_index} occurrence {occurrence_offset} end"),
            )?;
            assert_vec3_close(
                start,
                expected_start,
                1.0e-7,
                &format!("{label} wire {wire_index} occurrence {occurrence_offset} public start"),
            )?;
            assert_vec3_close(
                end,
                expected_end,
                1.0e-7,
                &format!("{label} wire {wire_index} occurrence {occurrence_offset} public end"),
            )?;
        }
    }

    Ok(())
}

fn assert_single_face_ported_area_matches_brep(
    kernel: &ModelKernel,
    label: &str,
    shape: &Shape,
    brep: &lean_occt::BrepShape,
) -> Result<(), Box<dyn std::error::Error>> {
    let ported_area = kernel
        .context()
        .ported_face_area(shape)?
        .ok_or_else(|| std::io::Error::other(format!("{label} missing public ported area")))?;
    let brep_face = brep
        .faces
        .first()
        .ok_or_else(|| std::io::Error::other(format!("{label} missing BRep face")))?;
    if (brep_face.area - ported_area).abs() > 1.0e-8 {
        return Err(std::io::Error::other(format!(
            "{label} single-face public area mismatch: brep={} public={ported_area}",
            brep_face.area
        ))
        .into());
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

fn assert_brep_ported_curve_matches_public(
    kernel: &ModelKernel,
    label: &str,
    shape: &Shape,
    brep: &lean_occt::BrepShape,
    expected_kind: CurveKind,
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

    let mut matched = 0usize;
    for (index, edge_shape) in edge_shapes.iter().enumerate() {
        let geometry = kernel.context().edge_geometry(edge_shape)?;
        if geometry.kind != expected_kind {
            continue;
        }

        let actual_edge = brep
            .edges
            .get(index)
            .ok_or_else(|| std::io::Error::other(format!("{label} missing brep edge {index}")))?;
        assert_edge_geometry_close(
            actual_edge.geometry,
            geometry,
            1.0e-9,
            &format!("{label} edge {index} geometry"),
        )?;

        let actual_curve = actual_edge.ported_curve.ok_or_else(|| {
            std::io::Error::other(format!(
                "{label} edge {index} expected ported {expected_kind:?} curve"
            ))
        })?;
        let expected_curve = kernel
            .context()
            .ported_edge_curve(edge_shape)?
            .ok_or_else(|| {
                std::io::Error::other(format!(
                    "{label} edge {index} public path expected ported {expected_kind:?} curve"
                ))
            })?;
        assert_same_variant(
            &actual_curve,
            &expected_curve,
            &format!("{label} edge {index} curve"),
        )?;

        let raw_geometry = kernel.context().edge_geometry_occt(edge_shape)?;
        if raw_geometry.kind != expected_kind {
            return Err(std::io::Error::other(format!(
                "{label} edge {index} raw geometry kind mismatch: raw={:?} expected={expected_kind:?}",
                raw_geometry.kind
            ))
            .into());
        }
        let Some(raw_geometry_curve) =
            PortedCurve::from_context_with_geometry(kernel.context(), edge_shape, raw_geometry)?
        else {
            return Err(std::io::Error::other(format!(
                "{label} edge {index} raw geometry route expected ported {expected_kind:?} curve"
            ))
            .into());
        };
        assert_same_variant(
            &raw_geometry_curve,
            &expected_curve,
            &format!("{label} edge {index} raw geometry curve"),
        )?;

        let derived_length = actual_curve.length_with_geometry(actual_edge.geometry);
        if (actual_edge.length - derived_length).abs() > 1.0e-9 {
            return Err(std::io::Error::other(format!(
                "{label} edge {index} length should come from the ported curve: brep={} derived={derived_length}",
                actual_edge.length
            ))
            .into());
        }

        let public_length = expected_curve.length_with_geometry(geometry);
        if (actual_edge.length - public_length).abs() > 1.0e-9 {
            return Err(std::io::Error::other(format!(
                "{label} edge {index} length mismatch: brep={} public={public_length}",
                actual_edge.length
            ))
            .into());
        }

        let parameter = 0.5 * (geometry.start_parameter + geometry.end_parameter);
        let actual_sample = actual_curve.sample_with_geometry(actual_edge.geometry, parameter);
        let occt_sample = kernel
            .context()
            .edge_sample_at_parameter_occt(edge_shape, parameter)?;
        assert_vec3_close(
            actual_sample.position,
            occt_sample.position,
            1.0e-8,
            &format!("{label} edge {index} sample position"),
        )?;
        assert_vec3_close(
            actual_sample.tangent,
            occt_sample.tangent,
            1.0e-8,
            &format!("{label} edge {index} sample tangent"),
        )?;

        let raw_parameter = 0.5 * (raw_geometry.start_parameter + raw_geometry.end_parameter);
        let raw_sample = raw_geometry_curve.sample_with_geometry(raw_geometry, raw_parameter);
        let raw_occt_sample = kernel
            .context()
            .edge_sample_at_parameter_occt(edge_shape, raw_parameter)?;
        assert_vec3_close(
            raw_sample.position,
            raw_occt_sample.position,
            1.0e-8,
            &format!("{label} edge {index} raw geometry sample position"),
        )?;
        assert_vec3_close(
            raw_sample.tangent,
            raw_occt_sample.tangent,
            1.0e-8,
            &format!("{label} edge {index} raw geometry sample tangent"),
        )?;

        let occt_length = kernel
            .context()
            .describe_shape_occt(edge_shape)?
            .linear_length;
        let occt_tolerance = if expected_kind == CurveKind::Ellipse {
            5.0e-2
        } else {
            1.0e-7
        };
        if (actual_edge.length - occt_length).abs() > occt_tolerance {
            return Err(std::io::Error::other(format!(
                "{label} edge {index} length drifted from OCCT: brep={} occt={} tol={occt_tolerance}",
                actual_edge.length,
                occt_length
            ))
            .into());
        }

        match (expected_kind, actual_curve) {
            (CurveKind::Line, PortedCurve::Line(_))
            | (CurveKind::Circle, PortedCurve::Circle(_))
            | (CurveKind::Ellipse, PortedCurve::Ellipse(_)) => {}
            _ => {
                return Err(std::io::Error::other(format!(
                    "{label} edge {index} expected {expected_kind:?} curve, got {actual_curve:?}"
                ))
                .into());
            }
        }
        matched += 1;
    }

    if matched == 0 {
        return Err(std::io::Error::other(format!(
            "{label} did not expose any {expected_kind:?} BRep edges"
        ))
        .into());
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
        let actual_edge = brep
            .edges
            .get(index)
            .ok_or_else(|| std::io::Error::other(format!("{label} missing brep edge {index}")))?;
        assert_edge_geometry_close(
            actual_edge.geometry,
            expected_geometry,
            1.0e-9,
            &format!("{label} edge {index} geometry"),
        )?;

        if matches!(
            expected_geometry.kind,
            CurveKind::Line | CurveKind::Circle | CurveKind::Ellipse
        ) {
            let actual_curve = actual_edge.ported_curve.ok_or_else(|| {
                std::io::Error::other(format!(
                    "{label} edge {index} expected Rust-owned {:?} BRep curve",
                    expected_geometry.kind
                ))
            })?;
            let expected_curve =
                kernel
                    .context()
                    .ported_edge_curve(edge_shape)?
                    .ok_or_else(|| {
                        std::io::Error::other(format!(
                            "{label} edge {index} public Rust edge curve missing for {:?}",
                            expected_geometry.kind
                        ))
                    })?;
            assert_same_variant(
                &actual_curve,
                &expected_curve,
                &format!("{label} edge {index} BRep/public curve"),
            )?;
        }
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
            SurfaceKind::Plane,
        ),
        (
            "cylinder",
            2.0 * PI * 6.0 * (18.0 + 6.0),
            PI * 6.0 * 6.0 * 18.0,
            "primitive_cylinder",
            SurfaceKind::Cylinder,
        ),
        (
            "cone",
            PI * (9.0 + 3.0) * cone_slant + PI * (9.0 * 9.0 + 3.0 * 3.0),
            PI * 15.0 * (9.0 * 9.0 + 9.0 * 3.0 + 3.0 * 3.0) / 3.0,
            "primitive_cone",
            SurfaceKind::Cone,
        ),
        (
            "sphere",
            4.0 * PI * 7.0 * 7.0,
            4.0 * PI * 7.0 * 7.0 * 7.0 / 3.0,
            "primitive_sphere",
            SurfaceKind::Sphere,
        ),
        (
            "torus",
            4.0 * PI * PI * 15.0 * 4.0,
            2.0 * PI * PI * 15.0 * 4.0 * 4.0,
            "primitive_torus",
            SurfaceKind::Torus,
        ),
    ];

    for (name, expected_area, expected_volume, artifact_name, expected_surface_kind) in expected {
        let artifact =
            support::export_document_shape(&mut document, name, "brep_workflows", artifact_name)?;
        let brep = document.brep(name)?;
        assert_brep_analytic_faces_use_rust_surface_route(
            document.kernel(),
            name,
            document.shape(name)?,
            &brep,
            expected_surface_kind,
        )?;
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
        assert_eq!(
            brep.summary_bbox_source(),
            SummaryBboxSource::ExactPrimitive,
            "{name} root summary bbox should resolve through the exact primitive path"
        );
        assert_eq!(
            brep.summary_volume_source(),
            SummaryVolumeSource::ExactPrimitive,
            "{name} root summary volume should resolve through the exact primitive path"
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
    let torus = kernel.make_torus(TorusParams {
        origin: [-8.0, 6.0, -1.5],
        axis,
        x_direction: [1.0, 0.0, 0.0],
        major_radius: 15.0,
        minor_radius: 4.0,
    })?;
    let torus_center = [-8.0, 6.0, -1.5];
    let torus_half_extent = [
        15.0 * (1.0 - axis[0] * axis[0]).sqrt() + 4.0,
        15.0 * (1.0 - axis[1] * axis[1]).sqrt() + 4.0,
        15.0 * (1.0 - axis[2] * axis[2]).sqrt() + 4.0,
    ];
    let torus_expected_bbox = (
        [
            torus_center[0] - torus_half_extent[0],
            torus_center[1] - torus_half_extent[1],
            torus_center[2] - torus_half_extent[2],
        ],
        [
            torus_center[0] + torus_half_extent[0],
            torus_center[1] + torus_half_extent[1],
            torus_center[2] + torus_half_extent[2],
        ],
    );
    for (label, shape, artifact_name) in [
        ("cylinder", &cylinder, "bbox_rotated_cylinder"),
        ("cone", &cone, "bbox_rotated_cone"),
        ("sphere", &sphere, "bbox_rotated_sphere"),
        ("torus", &torus, "bbox_rotated_torus"),
    ] {
        let artifact =
            support::export_kernel_shape(&kernel, shape, "brep_workflows", artifact_name)?;
        let summary = kernel.summarize(shape)?;
        let brep = kernel.brep(shape)?;
        let occt_summary = kernel.context().describe_shape_occt(shape)?;

        assert_eq!(
            brep.summary_bbox_source(),
            SummaryBboxSource::ExactPrimitive,
            "{label} root summary bbox should resolve through the exact primitive path"
        );
        if label == "torus" {
            assert_bbox_close(
                label,
                summary.bbox_min,
                summary.bbox_max,
                torus_expected_bbox.0,
                torus_expected_bbox.1,
                5.0e-7,
            )?;
            assert_bbox_close(
                label,
                brep.summary.bbox_min,
                brep.summary.bbox_max,
                torus_expected_bbox.0,
                torus_expected_bbox.1,
                5.0e-7,
            )?;
        } else {
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
    let box_shape = kernel.make_box(BoxParams {
        origin: [-6.0, -4.0, -2.0],
        size: [12.0, 8.0, 6.0],
    })?;
    let line_edge = find_first_edge_by_kind(&kernel, &box_shape, CurveKind::Line)?;
    let circle_face = find_first_face_by_kind(&kernel, &cylinder, SurfaceKind::Plane)?;
    let circle_edge = find_first_edge_by_kind(&kernel, &cylinder, CurveKind::Circle)?;
    let prism = kernel.make_prism(
        &circle_face,
        PrismParams {
            direction: [8.0, 24.0, -5.0],
        },
    )?;

    for (label, shape) in [
        ("line_edge", &line_edge),
        ("ellipse_edge", &ellipse),
        ("circle_face", &circle_face),
        ("circle_edge", &circle_edge),
        ("oblique_circle_prism", &prism),
    ] {
        let summary = kernel.summarize(shape)?;
        let brep = kernel.brep(shape)?;
        let occt_summary = kernel.context().describe_shape_occt(shape)?;

        assert_eq!(
            brep.summary_bbox_source(),
            SummaryBboxSource::PortedBrep,
            "{label} root summary bbox should resolve through the Rust-owned brep path"
        );
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

    for (label, shape, expected_kind) in [
        ("line_edge", &line_edge, CurveKind::Line),
        ("circle_edge", &circle_edge, CurveKind::Circle),
        ("ellipse_edge", &ellipse, CurveKind::Ellipse),
    ] {
        let brep = kernel.brep(shape)?;
        assert_brep_ported_curve_matches_public(&kernel, label, shape, &brep, expected_kind)?;
    }

    Ok(())
}

#[test]
fn supported_brep_materialization_requires_ported_topology(
) -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let kernel = ModelKernel::new()?;
    let box_shape = kernel.make_box(BoxParams {
        origin: [-10.0, -10.0, -10.0],
        size: [20.0, 20.0, 20.0],
    })?;
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
    let prism = kernel.make_prism(
        &ellipse,
        PrismParams {
            direction: [0.0, 24.0, 0.0],
        },
    )?;
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
        ("analytic_box", &box_shape),
        ("face_free_ellipse", &ellipse),
        ("face_free_helix", &helix),
        ("swept_extrusion", &prism),
        ("swept_revolution", &revolution),
        ("offset_surface", &offset_surface),
    ] {
        assert_supported_brep_materializes_from_ported_topology(&kernel, label, shape)?;
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

        assert_topology_matches_ignoring_edge_lengths(label, &rust_topology, &occt_topology)?;
        assert_topology_backed_subshape_counts_match(&kernel, label, shape, &rust_topology)?;
        assert_topology_backed_subshapes_match(&kernel, label, shape, &rust_topology)?;
        assert_topology_edges_match_public_queries(
            &kernel,
            label,
            shape,
            &rust_topology,
            label != "helix",
        )?;
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
        assert_topology_edges_match_public_queries(&kernel, label, shape, &rust_topology, true)?;
        assert_summary_backed_subshape_counts_match(&kernel, label, shape)?;
        assert_topology_matches(label, &brep.topology, &rust_topology)?;
        assert_brep_edge_geometries_match_public(&kernel, label, shape, &brep)?;
        assert_brep_faces_match_public(&kernel, label, shape, &brep)?;
        assert_single_face_ported_area_matches_brep(&kernel, label, shape, &brep)?;
        assert_brep_edge_lengths_match(&kernel, label, shape, &brep)?;
        assert!(
            matches!(
                brep.summary_bbox_source(),
                SummaryBboxSource::PortedBrep | SummaryBboxSource::Mesh
            ),
            "{label} supported single-face bbox should stay Rust-owned, not {:?}",
            brep.summary_bbox_source()
        );
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
                let face = brep
                    .faces
                    .first()
                    .ok_or_else(|| std::io::Error::other("missing holed planar BRep face"))?;
                let expected_holed_area = 60.0 * 60.0 - PI * 12.0 * 12.0;
                assert!(
                    (face.area - expected_holed_area).abs() <= 1.0e-6,
                    "holed planar face area should come from reconstructed Rust loops: brep={} expected={expected_holed_area}",
                    face.area
                );
                let public_geometry = kernel.context().face_geometry(shape)?;
                assert_eq!(public_geometry.kind, SurfaceKind::Plane);
                let rust_plane_surface = PortedSurface::from_context_with_geometry(
                    kernel.context(),
                    shape,
                    public_geometry,
                )?
                .ok_or_else(|| {
                    std::io::Error::other(
                        "holed planar face public geometry route returned no Rust plane payload",
                    )
                })?;
                assert_ported_surface_kind(
                    rust_plane_surface,
                    SurfaceKind::Plane,
                    "holed planar face snapshot plane payload",
                )?;
                let brep_plane_surface = face.ported_surface.ok_or_else(|| {
                    std::io::Error::other(
                        "holed planar BRep face did not retain the Rust plane payload",
                    )
                })?;
                assert_same_variant(
                    &rust_plane_surface,
                    &brep_plane_surface,
                    "holed planar face snapshot plane route",
                )?;
                assert_eq!(face.loops.len(), rust_topology.wires.len());

                let mut analytic_loop_edges = 0usize;
                for face_loop in &face.loops {
                    let wire = brep.wires.get(face_loop.wire_index).ok_or_else(|| {
                        std::io::Error::other(format!(
                            "missing BRep wire {} for holed planar face loop",
                            face_loop.wire_index
                        ))
                    })?;
                    for &edge_index in &wire.edge_indices {
                        let edge = brep.edges.get(edge_index).ok_or_else(|| {
                            std::io::Error::other(format!(
                                "missing BRep edge {edge_index} for holed planar face loop"
                            ))
                        })?;
                        if matches!(
                            edge.geometry.kind,
                            CurveKind::Line | CurveKind::Circle | CurveKind::Ellipse
                        ) {
                            analytic_loop_edges += 1;
                            assert!(
                                edge.ported_curve.is_some(),
                                "holed planar face edge {edge_index} should use a Rust-owned ported curve"
                            );
                        }
                    }
                }
                assert!(
                    analytic_loop_edges > 0,
                    "holed planar face did not expose analytic loop edges"
                );
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
fn ported_brep_orders_repeated_wire_edge_occurrences_in_rust(
) -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let kernel = ModelKernel::new()?;
    let cylinder = kernel.make_cylinder(CylinderParams {
        origin: [0.0, 0.0, -7.0],
        axis: [0.0, 0.0, 1.0],
        radius: 5.0,
        height: 14.0,
    })?;

    let rust_topology = kernel
        .context()
        .ported_topology(&cylinder)?
        .ok_or_else(|| {
            std::io::Error::other("expected Rust topology for repeated cylinder wire occurrences")
        })?;
    let occt_topology = kernel.context().topology_occt(&cylinder)?;
    let brep = kernel.brep(&cylinder)?;

    assert_topology_matches(
        "repeated cylinder wire occurrences",
        &rust_topology,
        &occt_topology,
    )?;
    assert!(
        topology_has_repeated_wire_edge_occurrence(&rust_topology),
        "expected the cylinder topology to include a wire that repeats a root edge occurrence"
    );
    assert_topology_matches(
        "repeated cylinder wire BRep topology",
        &brep.topology,
        &rust_topology,
    )?;

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
        assert_topology_edges_match_public_queries(&kernel, label, shape, &rust_topology, true)?;
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
        assert!(
            matches!(
                brep.summary_bbox_source(),
                SummaryBboxSource::ExactPrimitive
                    | SummaryBboxSource::PortedBrep
                    | SummaryBboxSource::Mesh
            ),
            "{label} supported analytic solid bbox should stay Rust-owned, not {:?}",
            brep.summary_bbox_source()
        );
        if label != "through_hole_cut" {
            assert!(
                matches!(
                    brep.summary_volume_source(),
                    SummaryVolumeSource::ExactPrimitive
                        | SummaryVolumeSource::FaceContributions
                        | SummaryVolumeSource::WholeShapeMesh
                ),
                "{label} supported analytic solid volume should stay Rust-owned, not {:?}",
                brep.summary_volume_source()
            );
        }
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
    let face_shapes = kernel.context().subshapes(&revolved, ShapeKind::Face)?;
    let expected_rust_volume = 35_530.575_843_921_69;

    assert_eq!(summary.primary_kind, ShapeKind::Solid);
    assert_eq!(summary.solid_count, 1);
    assert_eq!(
        face_shapes.len(),
        brep.faces.len(),
        "swept revolution face inventory mismatch"
    );
    assert!(brep
        .faces
        .iter()
        .any(|face| face.geometry.kind == SurfaceKind::Extrusion));
    assert!(brep
        .faces
        .iter()
        .any(|face| face.geometry.kind == SurfaceKind::Revolution));
    let extrusion_index = brep
        .faces
        .iter()
        .position(|face| face.geometry.kind == SurfaceKind::Extrusion)
        .ok_or_else(|| std::io::Error::other("expected an extrusion face in the revolved solid"))?;
    let extrusion_face = &brep.faces[extrusion_index];
    let extrusion_face_shape = &face_shapes[extrusion_index];
    let extrusion_surface = extrusion_face
        .ported_face_surface
        .ok_or_else(|| std::io::Error::other("expected a ported extrusion descriptor"))?;
    let (extrusion_payload, extrusion_basis_geometry) = match extrusion_surface {
        PortedFaceSurface::Swept(PortedSweptSurface::Extrusion {
            payload,
            basis_curve: PortedCurve::Ellipse(_),
            basis_geometry,
        }) => (payload, basis_geometry),
        other => {
            return Err(std::io::Error::other(format!(
                "expected Rust-owned ellipse extrusion descriptor, got {other:?}"
            ))
            .into())
        }
    };
    assert_eq!(
        extrusion_payload.basis_curve_kind, extrusion_basis_geometry.kind,
        "extrusion payload should record the selected Rust topology basis kind"
    );
    assert_extrusion_payload_close(
        extrusion_payload,
        kernel
            .context()
            .face_extrusion_payload(extrusion_face_shape)?,
        1.0e-8,
        "swept revolution brep extrusion public payload",
    )?;
    assert_extrusion_payload_close(
        extrusion_payload,
        kernel
            .context()
            .face_extrusion_payload_occt(extrusion_face_shape)?,
        1.0e-8,
        "swept revolution brep extrusion occt payload",
    )?;
    assert_ported_face_surface_matches_public(
        "swept revolution brep extrusion descriptor",
        Some(extrusion_surface),
        kernel
            .context()
            .ported_face_surface_descriptor(extrusion_face_shape)?,
        extrusion_face.geometry,
        kernel.context().face_geometry(extrusion_face_shape)?,
        extrusion_face.orientation,
    )?;
    let uv_t = [0.37, 0.61];
    let extrusion_sample = extrusion_surface.sample_normalized_with_orientation(
        extrusion_face.geometry,
        uv_t,
        extrusion_face.orientation,
    );
    let extrusion_occt_sample = kernel
        .context()
        .face_sample_normalized_occt(extrusion_face_shape, uv_t)?;
    assert_vec3_close(
        extrusion_sample.position,
        extrusion_occt_sample.position,
        1.0e-6,
        "swept revolution brep extrusion sample position",
    )?;
    assert_vec3_close(
        extrusion_sample.normal,
        extrusion_occt_sample.normal,
        1.0e-6,
        "swept revolution brep extrusion sample normal",
    )?;
    let extrusion_public_area = kernel
        .context()
        .ported_face_area(extrusion_face_shape)?
        .ok_or_else(|| std::io::Error::other("expected ported extrusion face area"))?;
    let extrusion_occt_area = kernel
        .context()
        .describe_shape_occt(extrusion_face_shape)?
        .surface_area;
    assert!(
        (extrusion_face.area - extrusion_public_area).abs() <= 1.0e-8,
        "swept revolution brep extrusion area drifted from public ported area: brep={} public={}",
        extrusion_face.area,
        extrusion_public_area
    );
    assert!(
        (extrusion_face.area - extrusion_occt_area).abs() <= 2.0e-1,
        "swept revolution brep extrusion area drifted from OCCT: brep={} occt={}",
        extrusion_face.area,
        extrusion_occt_area
    );

    let revolution_index = brep
        .faces
        .iter()
        .position(|face| face.geometry.kind == SurfaceKind::Revolution)
        .ok_or_else(|| std::io::Error::other("expected a revolution face in the revolved solid"))?;
    let revolution_face = &brep.faces[revolution_index];
    let revolution_face_shape = &face_shapes[revolution_index];
    let revolution_surface = revolution_face
        .ported_face_surface
        .ok_or_else(|| std::io::Error::other("expected a ported revolution descriptor"))?;
    let (revolution_payload, revolution_basis_geometry) = match revolution_surface {
        PortedFaceSurface::Swept(PortedSweptSurface::Revolution {
            payload,
            basis_curve: PortedCurve::Ellipse(_),
            basis_geometry,
        }) => (payload, basis_geometry),
        other => {
            return Err(std::io::Error::other(format!(
                "expected Rust-owned ellipse revolution descriptor, got {other:?}"
            ))
            .into())
        }
    };
    assert_eq!(
        revolution_payload.basis_curve_kind, revolution_basis_geometry.kind,
        "revolution payload should record the selected Rust topology basis kind"
    );
    assert_revolution_payload_close(
        revolution_payload,
        kernel
            .context()
            .face_revolution_payload(revolution_face_shape)?,
        1.0e-8,
        "swept revolution brep revolution public payload",
    )?;
    assert_revolution_payload_close(
        revolution_payload,
        kernel
            .context()
            .face_revolution_payload_occt(revolution_face_shape)?,
        1.0e-8,
        "swept revolution brep revolution occt payload",
    )?;
    assert_ported_face_surface_matches_public(
        "swept revolution brep revolution descriptor",
        Some(revolution_surface),
        kernel
            .context()
            .ported_face_surface_descriptor(revolution_face_shape)?,
        revolution_face.geometry,
        kernel.context().face_geometry(revolution_face_shape)?,
        revolution_face.orientation,
    )?;
    let revolution_sample = revolution_surface.sample_normalized_with_orientation(
        revolution_face.geometry,
        uv_t,
        revolution_face.orientation,
    );
    let revolution_occt_sample = kernel
        .context()
        .face_sample_normalized_occt(revolution_face_shape, uv_t)?;
    assert_vec3_close(
        revolution_sample.position,
        revolution_occt_sample.position,
        1.0e-6,
        "swept revolution brep revolution sample position",
    )?;
    assert_vec3_close(
        revolution_sample.normal,
        revolution_occt_sample.normal,
        1.0e-6,
        "swept revolution brep revolution sample normal",
    )?;
    let revolution_public_area = kernel
        .context()
        .ported_face_area(revolution_face_shape)?
        .ok_or_else(|| std::io::Error::other("expected ported revolution face area"))?;
    let revolution_occt_area = kernel
        .context()
        .describe_shape_occt(revolution_face_shape)?
        .surface_area;
    assert!(
        (revolution_face.area - revolution_public_area).abs() <= 1.0e-8,
        "swept revolution brep revolution area drifted from public ported area: brep={} public={}",
        revolution_face.area,
        revolution_public_area
    );
    assert!(
        (revolution_face.area - revolution_occt_area).abs() <= 2.0e-1,
        "swept revolution brep revolution area drifted from OCCT: brep={} occt={}",
        revolution_face.area,
        revolution_occt_area
    );
    assert_eq!(
        brep.summary_bbox_source(),
        SummaryBboxSource::PortedBrep,
        "swept revolution root summary bbox should resolve through the Rust-owned brep path"
    );
    assert_eq!(
        brep.summary_volume_source(),
        SummaryVolumeSource::FaceContributions,
        "swept revolution root summary volume should resolve through Rust-owned face contributions"
    );
    assert!(
        (summary.surface_area - occt_summary.surface_area).abs() <= 2.0e-1,
        "swept revolution surface area drifted from OCCT: rust={} occt={}",
        summary.surface_area,
        occt_summary.surface_area
    );
    assert!(
        (summary.volume - expected_rust_volume).abs() <= 1.0e-6,
        "swept revolution volume drifted from the Rust-owned regression anchor: rust={} expected={}",
        summary.volume,
        expected_rust_volume
    );
    assert!(
        summary.volume > occt_summary.volume + 1.0e4,
        "swept revolution volume should stay on the Rust-owned path instead of the OCCT zero fallback: rust={} occt={}",
        summary.volume,
        occt_summary.volume
    );
    assert_bbox_close(
        "swept revolution kernel summary",
        summary.bbox_min,
        summary.bbox_max,
        occt_summary.bbox_min,
        occt_summary.bbox_max,
        1.0e-6,
    )?;
    assert_bbox_close(
        "swept revolution brep summary",
        brep.summary.bbox_min,
        brep.summary.bbox_max,
        occt_summary.bbox_min,
        occt_summary.bbox_max,
        1.0e-6,
    )?;
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
    let prism = kernel.make_prism(
        &ellipse,
        PrismParams {
            direction: [0.0, 24.0, 0.0],
        },
    )?;
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
    let summary = kernel.summarize(&offset_surface)?;
    let occt_summary = kernel.context().describe_shape_occt(&offset_surface)?;
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
    assert_eq!(
        brep.summary_bbox_source(),
        SummaryBboxSource::OffsetFaceUnion,
        "offset shell root summary bbox should resolve through the dedicated offset-face union path"
    );
    assert_eq!(
        brep.offset_face_bbox_source(),
        Some(OffsetFaceBboxSource::ValidatedMesh),
        "single-face offset shell root summary bbox should resolve through the validated Rust mesh path, not {:?}",
        brep.offset_face_bbox_source()
    );
    assert_bbox_close(
        "offset shell kernel summary",
        summary.bbox_min,
        summary.bbox_max,
        occt_summary.bbox_min,
        occt_summary.bbox_max,
        5.0e-2,
    )?;
    assert_bbox_close(
        "offset shell brep summary",
        brep.summary.bbox_min,
        brep.summary.bbox_max,
        occt_summary.bbox_min,
        occt_summary.bbox_max,
        5.0e-2,
    )?;
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

    let plane_source = kernel.make_box(BoxParams {
        origin: [-8.0, -6.0, -4.0],
        size: [16.0, 12.0, 8.0],
    })?;
    let cylinder = kernel.make_cylinder(CylinderParams {
        origin: [4.0, -3.0, 1.5],
        axis: [0.0, 0.0, 1.0],
        radius: 6.0,
        height: 18.0,
    })?;
    let cone = kernel.make_cone(ConeParams {
        origin: [-6.0, 5.0, 2.0],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        base_radius: 9.0,
        top_radius: 3.0,
        height: 15.0,
    })?;
    let sphere = kernel.make_sphere(SphereParams {
        origin: [5.0, -4.0, 3.0],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        radius: 7.0,
    })?;
    let torus = kernel.make_torus(TorusParams {
        origin: [-8.0, 6.0, -1.5],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        major_radius: 15.0,
        minor_radius: 4.0,
    })?;
    for (label, source, expected_kind) in [
        ("plane", &plane_source, SurfaceKind::Plane),
        ("cylinder", &cylinder, SurfaceKind::Cylinder),
        ("cone", &cone, SurfaceKind::Cone),
        ("sphere", &sphere, SurfaceKind::Sphere),
        ("torus", &torus, SurfaceKind::Torus),
        ("extrusion", &prism, SurfaceKind::Extrusion),
        ("revolution", &revolution, SurfaceKind::Revolution),
    ] {
        assert_offset_brep_uses_rust_basis(&kernel, label, source, expected_kind)?;
    }

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
    let shell_shapes = kernel.context().subshapes(&offset, ShapeKind::Shell)?;

    assert_eq!(summary.primary_kind, ShapeKind::Solid);
    assert_eq!(
        kernel.context().subshape_count(&offset, ShapeKind::Shell)?,
        shell_shapes.len(),
        "offset solid shell count should come from the loaded Rust topology inventory"
    );
    assert_eq!(
        kernel
            .context()
            .subshape_count_occt(&offset, ShapeKind::Shell)?,
        shell_shapes.len(),
        "Rust-owned shell inventory should match OCCT's raw shell count"
    );
    for (shell_index, shell_shape) in shell_shapes.iter().enumerate() {
        let indexed_shell = kernel
            .context()
            .subshape(&offset, ShapeKind::Shell, shell_index)?;
        let public_topology = kernel.context().topology(shell_shape)?;
        let indexed_topology = kernel.context().topology(&indexed_shell)?;
        assert_topology_matches(
            &format!("offset solid shell {shell_index} public subshape"),
            &indexed_topology,
            &public_topology,
        )?;
    }
    assert!(
        !offset_faces.is_empty(),
        "expected offset solid to retain offset faces"
    );
    let offset_face_shapes = kernel
        .context()
        .subshapes(&offset, ShapeKind::Face)?
        .into_iter()
        .filter_map(
            |face_shape| match kernel.context().face_geometry(&face_shape) {
                Ok(geometry) if geometry.kind == SurfaceKind::Offset => Some(Ok(face_shape)),
                Ok(_) => None,
                Err(error) => Some(Err(error)),
            },
        )
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(
        offset_face_shapes.len(),
        offset_faces.len(),
        "public Rust topology should expose the same generated offset faces as BRep materialization"
    );
    let mut public_swept_offset_descriptors = 0usize;
    for (offset_index, face_shape) in offset_face_shapes.iter().enumerate() {
        let descriptor = match kernel
            .context()
            .ported_face_surface_descriptor(face_shape)?
        {
            Some(PortedFaceSurface::Offset(surface)) => surface,
            other => {
                return Err(std::io::Error::other(format!(
                    "offset solid public face {offset_index} did not expose a ported offset descriptor: {other:?}"
                ))
                .into());
            }
        };
        assert!(
            (descriptor.payload.offset_value.abs() - 2.5).abs() <= 1.0e-9,
            "offset solid public face {offset_index} payload offset drifted: {:?}",
            descriptor.payload
        );
        assert!(
            matches!(
                descriptor.payload.basis_surface_kind,
                SurfaceKind::Revolution | SurfaceKind::Extrusion
            ),
            "offset solid public face {offset_index} should map to the swept source family, got {:?}",
            descriptor.payload.basis_surface_kind
        );

        let public_payload = kernel.context().face_offset_payload(face_shape)?;
        assert!(
            (public_payload.offset_value - descriptor.payload.offset_value).abs() <= 1.0e-9,
            "offset solid public face {offset_index} public payload offset drifted from descriptor"
        );
        assert_eq!(
            public_payload.basis_surface_kind, descriptor.payload.basis_surface_kind,
            "offset solid public face {offset_index} public payload basis drifted from descriptor"
        );
        assert_eq!(
            kernel
                .context()
                .face_offset_basis_geometry(face_shape)?
                .kind,
            descriptor.payload.basis_surface_kind,
            "offset solid public face {offset_index} public basis geometry drifted from descriptor"
        );

        if matches!(descriptor.basis, PortedOffsetBasisSurface::Swept(_)) {
            public_swept_offset_descriptors += 1;
            assert_eq!(
                kernel
                    .context()
                    .face_offset_basis_curve_geometry(face_shape)?
                    .kind,
                CurveKind::Ellipse,
                "offset solid public face {offset_index} swept basis curve should mirror the source ellipse"
            );
        }

        let orientation = kernel.context().shape_orientation(face_shape)?;
        let descriptor_sample =
            descriptor.sample_normalized_with_orientation([0.37, 0.61], orientation);
        let occt_sample = kernel
            .context()
            .face_sample_normalized_occt(face_shape, [0.37, 0.61])?;
        assert_vec3_close(
            descriptor_sample.position,
            occt_sample.position,
            1.0e-6,
            &format!("offset solid public face {offset_index} descriptor sample position"),
        )?;
        assert_vec3_close(
            descriptor_sample.normal,
            occt_sample.normal,
            1.0e-6,
            &format!("offset solid public face {offset_index} descriptor sample normal"),
        )?;
    }
    assert!(
        public_swept_offset_descriptors > 0,
        "expected public generated offset faces to expose swept offset descriptors"
    );
    assert!(
        offset_faces.iter().any(|face| matches!(
            face.ported_face_surface,
            Some(PortedFaceSurface::Offset(surface))
                if matches!(surface.basis, PortedOffsetBasisSurface::Swept(_))
        )),
        "expected at least one Rust-owned swept offset face descriptor in offset solid"
    );
    assert_eq!(
        brep.offset_shell_bbox_sources().len(),
        shell_shapes.len(),
        "expected one shell bbox winner per offset shell: {:?}",
        brep.offset_shell_bbox_sources()
    );
    assert_eq!(
        brep.summary_bbox_source(),
        SummaryBboxSource::OffsetSolidShellUnion,
        "offset solid root summary bbox should resolve through the shell-union path"
    );
    assert!(
        matches!(
            brep.summary_volume_source(),
            SummaryVolumeSource::FaceContributions
        ),
        "offset solid root summary volume should resolve through a Rust-owned path, not {:?}",
        brep.summary_volume_source()
    );
    assert!(
        brep.offset_shell_bbox_sources()
            .iter()
            .all(|&source| source == OffsetShellBboxSource::Brep),
        "offset solid shell bbox should resolve through the validated shell brep path: {:?}",
        brep.offset_shell_bbox_sources()
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
    assert_bbox_close(
        "offset solid kernel summary",
        summary.bbox_min,
        summary.bbox_max,
        occt_summary.bbox_min,
        occt_summary.bbox_max,
        5.0e-2,
    )?;
    assert_bbox_close(
        "offset solid brep summary",
        brep.summary.bbox_min,
        brep.summary.bbox_max,
        occt_summary.bbox_min,
        occt_summary.bbox_max,
        5.0e-2,
    )?;
    let mut shell_swept_offset_descriptors = 0usize;
    for (shell_index, shell_shape) in shell_shapes.iter().enumerate() {
        let shell_brep = kernel.context().ported_brep(shell_shape)?;
        let shell_occt = kernel.context().describe_shape_occt(shell_shape)?;
        let label = format!("offset solid shell {shell_index} brep summary");
        assert_eq!(
            shell_brep.summary_bbox_source(),
            SummaryBboxSource::OffsetFaceUnion,
            "offset solid shell {shell_index} root summary bbox should resolve through the expanded offset-face union path"
        );
        assert_eq!(
            shell_brep.offset_face_bbox_source(),
            Some(OffsetFaceBboxSource::SummaryFaceBrep),
            "offset solid shell {shell_index} root summary bbox should resolve through the Rust-owned summary face BRep path, not {:?}",
            shell_brep.offset_face_bbox_source()
        );
        assert_bbox_close(
            &label,
            shell_brep.summary.bbox_min,
            shell_brep.summary.bbox_max,
            shell_occt.bbox_min,
            shell_occt.bbox_max,
            5.0e-2,
        )?;
        for (face_index, shell_face_shape) in kernel
            .context()
            .subshapes(shell_shape, ShapeKind::Face)?
            .into_iter()
            .enumerate()
        {
            if kernel.context().face_geometry(&shell_face_shape)?.kind != SurfaceKind::Offset {
                continue;
            }
            let descriptor = match kernel
                .context()
                .ported_face_surface_descriptor(&shell_face_shape)?
            {
                Some(PortedFaceSurface::Offset(surface)) => surface,
                other => {
                    return Err(std::io::Error::other(format!(
                        "offset solid shell {shell_index} face {face_index} did not expose a ported offset descriptor: {other:?}"
                    ))
                    .into());
                }
            };
            assert!(
                (kernel
                    .context()
                    .face_offset_payload(&shell_face_shape)?
                    .offset_value
                    .abs()
                    - 2.5)
                    .abs()
                    <= 1.0e-9,
                "offset solid shell {shell_index} face {face_index} public offset payload drifted"
            );
            if matches!(descriptor.basis, PortedOffsetBasisSurface::Swept(_)) {
                shell_swept_offset_descriptors += 1;
            }
        }
    }
    assert!(
        shell_swept_offset_descriptors >= public_swept_offset_descriptors,
        "shell subshape topology should preserve the generated swept offset descriptors"
    );
    assert!(artifact.is_file());

    Ok(())
}

#[test]
fn ported_brep_maps_multi_source_swept_offsets_in_rust() -> Result<(), Box<dyn std::error::Error>> {
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

    let context = kernel.context();
    assert_eq!(
        offset.rust_multi_face_offset_source_count(),
        Some(4),
        "multi-source swept offset should carry the full Rust source-face metadata inventory"
    );
    let offset_faces = context
        .subshapes(&offset, ShapeKind::Face)?
        .into_iter()
        .filter_map(|face_shape| match context.face_geometry(&face_shape) {
            Ok(geometry) if geometry.kind == SurfaceKind::Offset => Some(Ok(face_shape)),
            Ok(_) => None,
            Err(error) => Some(Err(error)),
        })
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(
        offset_faces.len(),
        4,
        "multi-source swept offset should expose the generated OFFSET_SURFACE faces"
    );

    for (face_index, face_shape) in offset_faces.iter().enumerate() {
        assert!(
            face_shape.has_rust_offset_surface_face_metadata(),
            "multi-source offset face {face_index} should be mapped to a Rust source metadata candidate before public payload queries"
        );
        let descriptor = match context.ported_face_surface_descriptor(face_shape)? {
            Some(PortedFaceSurface::Offset(surface)) => surface,
            other => {
                return Err(std::io::Error::other(format!(
                    "multi-source offset face {face_index} did not expose a Rust descriptor: {other:?}"
                ))
                .into());
            }
        };
        assert!(
            matches!(
                descriptor.payload.basis_surface_kind,
                SurfaceKind::Revolution | SurfaceKind::Extrusion
            ),
            "multi-source offset face {face_index} should keep the generated swept basis in Rust, got {:?}",
            descriptor.payload.basis_surface_kind
        );
        assert!(
            matches!(
                descriptor.basis,
                PortedOffsetBasisSurface::Swept(PortedSweptSurface::Revolution { .. })
                    | PortedOffsetBasisSurface::Swept(PortedSweptSurface::Extrusion { .. })
            ),
            "multi-source offset face {face_index} should expose a swept basis descriptor"
        );
        assert!(
            (descriptor.payload.offset_value.abs() - 2.5).abs() <= 1.0e-9,
            "multi-source offset face {face_index} descriptor offset drifted"
        );

        let public_payload = context.face_offset_payload(face_shape)?;
        assert!(
            (public_payload.offset_value - descriptor.payload.offset_value).abs() <= 1.0e-9,
            "multi-source offset face {face_index} public payload should come from the attached Rust descriptor"
        );
        assert_eq!(
            public_payload.basis_surface_kind, descriptor.payload.basis_surface_kind,
            "multi-source offset face {face_index} public payload basis drifted"
        );
        assert_eq!(
            context.face_offset_basis_geometry(face_shape)?.kind,
            descriptor.payload.basis_surface_kind,
            "multi-source offset face {face_index} public basis geometry drifted"
        );

        let orientation = context.shape_orientation(face_shape)?;
        let rust_sample = descriptor.sample_normalized_with_orientation([0.37, 0.61], orientation);
        let occt_sample = context.face_sample_normalized_occt(face_shape, [0.37, 0.61])?;
        assert_vec3_close(
            rust_sample.position,
            occt_sample.position,
            1.0e-6,
            &format!("multi-source offset face {face_index} descriptor sample position"),
        )?;
        assert_vec3_close(
            rust_sample.normal,
            occt_sample.normal,
            1.0e-6,
            &format!("multi-source offset face {face_index} descriptor sample normal"),
        )?;
    }

    for (shell_index, shell_shape) in context
        .subshapes(&offset, ShapeKind::Shell)?
        .iter()
        .enumerate()
    {
        assert_eq!(
            shell_shape.rust_multi_face_offset_source_count(),
            Some(4),
            "offset shell {shell_index} should retain the Rust multi-source metadata inventory"
        );
        for (face_index, shell_face_shape) in context
            .subshapes(shell_shape, ShapeKind::Face)?
            .iter()
            .enumerate()
        {
            if context.face_geometry(shell_face_shape)?.kind != SurfaceKind::Offset {
                continue;
            }
            assert!(
                shell_face_shape.has_rust_offset_surface_face_metadata(),
                "offset shell {shell_index} face {face_index} should carry resolved Rust metadata"
            );
            assert!(matches!(
                context.ported_face_surface_descriptor(shell_face_shape)?,
                Some(PortedFaceSurface::Offset(_))
            ));
        }
    }

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
    let cut_brep = kernel.brep(&cut)?;
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
    assert_bbox_close(
        "cut",
        cut_brep.summary.bbox_min,
        cut_brep.summary.bbox_max,
        cut_occt.bbox_min,
        cut_occt.bbox_max,
        5.0e-7,
    )?;

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
