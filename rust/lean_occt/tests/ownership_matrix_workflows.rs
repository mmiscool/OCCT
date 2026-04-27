mod support;

use std::f64::consts::PI;

use lean_occt::{
    BoxParams, CirclePayload, ConeParams, ConePayload, CurveKind, CylinderParams, CylinderPayload,
    EdgeGeometry, EdgeSelector, EllipseEdgeParams, EllipsePayload, ExtrusionSurfacePayload,
    FaceSelector, LinePayload, ModelDocument, ModelKernel, OffsetFaceBboxSource, OffsetParams,
    OffsetSurfacePayload, OperationRecord, PlanePayload, PortedCurve, PortedFaceSurface,
    PortedOffsetBasisSurface, PortedSurface, PortedSweptSurface, PrismParams, RevolutionParams,
    RevolutionSurfacePayload, Shape, ShapeKind, SphereParams, SpherePayload, SummaryBboxSource,
    SummaryVolumeSource, SurfaceKind, TorusParams, TorusPayload,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AuthoredFamily {
    BoxPlanar,
    Cylinder,
    Cone,
    Sphere,
    Torus,
    PrismExtrusion,
    Revolution,
    DirectOffset,
    GeneratedOffset,
    SimpleShellSolidAssembly,
    SimpleFaceWireAssembly,
    MixedAnalyticSolidAssembly,
}

#[derive(Clone, Copy, Debug)]
struct OwnershipRow {
    family: AuthoredFamily,
    construction_metadata: bool,
    normalized_snapshot_brep: bool,
    public_queries: bool,
    summary_metrics: bool,
    selectors_documents: bool,
}

const OWNERSHIP_MATRIX: &[OwnershipRow] = &[
    OwnershipRow {
        family: AuthoredFamily::BoxPlanar,
        construction_metadata: true,
        normalized_snapshot_brep: true,
        public_queries: true,
        summary_metrics: true,
        selectors_documents: true,
    },
    OwnershipRow {
        family: AuthoredFamily::Cylinder,
        construction_metadata: true,
        normalized_snapshot_brep: true,
        public_queries: true,
        summary_metrics: true,
        selectors_documents: true,
    },
    OwnershipRow {
        family: AuthoredFamily::Cone,
        construction_metadata: true,
        normalized_snapshot_brep: true,
        public_queries: true,
        summary_metrics: true,
        selectors_documents: true,
    },
    OwnershipRow {
        family: AuthoredFamily::Sphere,
        construction_metadata: true,
        normalized_snapshot_brep: true,
        public_queries: true,
        summary_metrics: true,
        selectors_documents: true,
    },
    OwnershipRow {
        family: AuthoredFamily::Torus,
        construction_metadata: true,
        normalized_snapshot_brep: true,
        public_queries: true,
        summary_metrics: true,
        selectors_documents: true,
    },
    OwnershipRow {
        family: AuthoredFamily::PrismExtrusion,
        construction_metadata: true,
        normalized_snapshot_brep: true,
        public_queries: true,
        summary_metrics: true,
        selectors_documents: true,
    },
    OwnershipRow {
        family: AuthoredFamily::Revolution,
        construction_metadata: true,
        normalized_snapshot_brep: true,
        public_queries: true,
        summary_metrics: true,
        selectors_documents: true,
    },
    OwnershipRow {
        family: AuthoredFamily::DirectOffset,
        construction_metadata: true,
        normalized_snapshot_brep: true,
        public_queries: true,
        summary_metrics: true,
        selectors_documents: true,
    },
    OwnershipRow {
        family: AuthoredFamily::GeneratedOffset,
        construction_metadata: true,
        normalized_snapshot_brep: true,
        public_queries: true,
        summary_metrics: true,
        selectors_documents: true,
    },
    OwnershipRow {
        family: AuthoredFamily::SimpleShellSolidAssembly,
        construction_metadata: true,
        normalized_snapshot_brep: true,
        public_queries: true,
        summary_metrics: true,
        selectors_documents: true,
    },
    OwnershipRow {
        family: AuthoredFamily::SimpleFaceWireAssembly,
        construction_metadata: true,
        normalized_snapshot_brep: true,
        public_queries: true,
        summary_metrics: true,
        selectors_documents: true,
    },
    OwnershipRow {
        family: AuthoredFamily::MixedAnalyticSolidAssembly,
        construction_metadata: true,
        normalized_snapshot_brep: true,
        public_queries: true,
        summary_metrics: true,
        selectors_documents: true,
    },
];

fn require_complete_ownership_row(
    family: AuthoredFamily,
    label: &str,
) -> Result<&'static OwnershipRow, Box<dyn std::error::Error>> {
    assert_eq!(OWNERSHIP_MATRIX.len(), 12);
    let row = OWNERSHIP_MATRIX
        .iter()
        .find(|row| row.family == family)
        .ok_or_else(|| std::io::Error::other(format!("missing {label} ownership row")))?;
    assert!(row.construction_metadata, "{label} metadata row incomplete");
    assert!(
        row.normalized_snapshot_brep,
        "{label} snapshot/BRep row incomplete"
    );
    assert!(row.public_queries, "{label} public query row incomplete");
    assert!(
        row.summary_metrics,
        "{label} summary metrics row incomplete"
    );
    assert!(
        row.selectors_documents,
        "{label} selector/document row incomplete"
    );
    Ok(row)
}

fn assert_scalar_close(
    lhs: f64,
    rhs: f64,
    tolerance: f64,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if (lhs - rhs).abs() > tolerance {
        return Err(std::io::Error::other(format!(
            "{label} mismatch: lhs={lhs} rhs={rhs} tol={tolerance}"
        ))
        .into());
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
    lhs: EdgeGeometry,
    rhs: EdgeGeometry,
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

fn assert_edge_geometry_close_or_periodic_equivalent(
    lhs: EdgeGeometry,
    rhs: EdgeGeometry,
    tolerance: f64,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if assert_edge_geometry_close(lhs, rhs, tolerance, label).is_ok() {
        return Ok(());
    }

    let lhs_span = (lhs.end_parameter - lhs.start_parameter).abs();
    let rhs_span = (rhs.end_parameter - rhs.start_parameter).abs();
    if lhs.kind == rhs.kind
        && lhs.is_periodic
        && rhs.is_periodic
        && (lhs.period - rhs.period).abs() <= tolerance
        && (lhs_span - rhs_span).abs() <= tolerance
    {
        return Ok(());
    }

    Err(std::io::Error::other(format!("{label} mismatch: lhs={lhs:?} rhs={rhs:?}")).into())
}

fn assert_line_payload_close(
    lhs: LinePayload,
    rhs: LinePayload,
    tolerance: f64,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_vec3_close(
        lhs.origin,
        rhs.origin,
        tolerance,
        &format!("{label} origin"),
    )?;
    assert_vec3_close(
        lhs.direction,
        rhs.direction,
        tolerance,
        &format!("{label} direction"),
    )?;
    Ok(())
}

fn assert_circle_payload_same_carrier(
    lhs: CirclePayload,
    rhs: CirclePayload,
    tolerance: f64,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_vec3_close(
        lhs.center,
        rhs.center,
        tolerance,
        &format!("{label} center"),
    )?;
    assert_scalar_close(
        lhs.radius,
        rhs.radius,
        tolerance,
        &format!("{label} radius"),
    )?;
    let normal_dot = lhs.normal[0] * rhs.normal[0]
        + lhs.normal[1] * rhs.normal[1]
        + lhs.normal[2] * rhs.normal[2];
    if (normal_dot.abs() - 1.0).abs() > tolerance {
        return Err(std::io::Error::other(format!(
            "{label} normal axes are not parallel: lhs={:?} rhs={:?}",
            lhs.normal, rhs.normal
        ))
        .into());
    }
    Ok(())
}

fn assert_ellipse_payload_close(
    lhs: EllipsePayload,
    rhs: EllipsePayload,
    tolerance: f64,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_vec3_close(
        lhs.center,
        rhs.center,
        tolerance,
        &format!("{label} center"),
    )?;
    assert_vec3_close(
        lhs.normal,
        rhs.normal,
        tolerance,
        &format!("{label} normal"),
    )?;
    assert_vec3_close(
        lhs.x_direction,
        rhs.x_direction,
        tolerance,
        &format!("{label} x_direction"),
    )?;
    assert_vec3_close(
        lhs.y_direction,
        rhs.y_direction,
        tolerance,
        &format!("{label} y_direction"),
    )?;
    assert_scalar_close(
        lhs.major_radius,
        rhs.major_radius,
        tolerance,
        &format!("{label} major_radius"),
    )?;
    assert_scalar_close(
        lhs.minor_radius,
        rhs.minor_radius,
        tolerance,
        &format!("{label} minor_radius"),
    )?;
    Ok(())
}

fn assert_plane_payload_close(
    lhs: PlanePayload,
    rhs: PlanePayload,
    tolerance: f64,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_vec3_close(
        lhs.origin,
        rhs.origin,
        tolerance,
        &format!("{label} origin"),
    )?;
    assert_vec3_close(
        lhs.normal,
        rhs.normal,
        tolerance,
        &format!("{label} normal"),
    )?;
    assert_vec3_close(
        lhs.x_direction,
        rhs.x_direction,
        tolerance,
        &format!("{label} x_direction"),
    )?;
    assert_vec3_close(
        lhs.y_direction,
        rhs.y_direction,
        tolerance,
        &format!("{label} y_direction"),
    )?;
    Ok(())
}

fn assert_cylinder_payload_close(
    lhs: CylinderPayload,
    rhs: CylinderPayload,
    tolerance: f64,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_vec3_close(
        lhs.origin,
        rhs.origin,
        tolerance,
        &format!("{label} origin"),
    )?;
    assert_vec3_close(lhs.axis, rhs.axis, tolerance, &format!("{label} axis"))?;
    assert_vec3_close(
        lhs.x_direction,
        rhs.x_direction,
        tolerance,
        &format!("{label} x_direction"),
    )?;
    assert_vec3_close(
        lhs.y_direction,
        rhs.y_direction,
        tolerance,
        &format!("{label} y_direction"),
    )?;
    assert_scalar_close(
        lhs.radius,
        rhs.radius,
        tolerance,
        &format!("{label} radius"),
    )?;
    Ok(())
}

fn assert_cone_payload_close(
    lhs: ConePayload,
    rhs: ConePayload,
    tolerance: f64,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_vec3_close(
        lhs.origin,
        rhs.origin,
        tolerance,
        &format!("{label} origin"),
    )?;
    assert_vec3_close(lhs.axis, rhs.axis, tolerance, &format!("{label} axis"))?;
    assert_vec3_close(
        lhs.x_direction,
        rhs.x_direction,
        tolerance,
        &format!("{label} x_direction"),
    )?;
    assert_vec3_close(
        lhs.y_direction,
        rhs.y_direction,
        tolerance,
        &format!("{label} y_direction"),
    )?;
    assert_scalar_close(
        lhs.reference_radius,
        rhs.reference_radius,
        tolerance,
        &format!("{label} reference_radius"),
    )?;
    assert_scalar_close(
        lhs.semi_angle,
        rhs.semi_angle,
        tolerance,
        &format!("{label} semi_angle"),
    )?;
    Ok(())
}

fn assert_sphere_payload_close(
    lhs: SpherePayload,
    rhs: SpherePayload,
    tolerance: f64,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_vec3_close(
        lhs.center,
        rhs.center,
        tolerance,
        &format!("{label} center"),
    )?;
    assert_vec3_close(
        lhs.normal,
        rhs.normal,
        tolerance,
        &format!("{label} normal"),
    )?;
    assert_vec3_close(
        lhs.x_direction,
        rhs.x_direction,
        tolerance,
        &format!("{label} x_direction"),
    )?;
    assert_vec3_close(
        lhs.y_direction,
        rhs.y_direction,
        tolerance,
        &format!("{label} y_direction"),
    )?;
    assert_scalar_close(
        lhs.radius,
        rhs.radius,
        tolerance,
        &format!("{label} radius"),
    )?;
    Ok(())
}

fn assert_torus_payload_close(
    lhs: TorusPayload,
    rhs: TorusPayload,
    tolerance: f64,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_vec3_close(
        lhs.center,
        rhs.center,
        tolerance,
        &format!("{label} center"),
    )?;
    assert_vec3_close(lhs.axis, rhs.axis, tolerance, &format!("{label} axis"))?;
    assert_vec3_close(
        lhs.x_direction,
        rhs.x_direction,
        tolerance,
        &format!("{label} x_direction"),
    )?;
    assert_vec3_close(
        lhs.y_direction,
        rhs.y_direction,
        tolerance,
        &format!("{label} y_direction"),
    )?;
    assert_scalar_close(
        lhs.major_radius,
        rhs.major_radius,
        tolerance,
        &format!("{label} major_radius"),
    )?;
    assert_scalar_close(
        lhs.minor_radius,
        rhs.minor_radius,
        tolerance,
        &format!("{label} minor_radius"),
    )?;
    Ok(())
}

fn assert_extrusion_payload_close(
    lhs: ExtrusionSurfacePayload,
    rhs: ExtrusionSurfacePayload,
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

fn assert_revolution_payload_close(
    lhs: RevolutionSurfacePayload,
    rhs: RevolutionSurfacePayload,
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

fn assert_offset_payload_close(
    lhs: OffsetSurfacePayload,
    rhs: OffsetSurfacePayload,
    tolerance: f64,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_scalar_close(
        lhs.offset_value,
        rhs.offset_value,
        tolerance,
        &format!("{label} offset_value"),
    )?;
    assert_eq!(
        lhs.basis_surface_kind, rhs.basis_surface_kind,
        "{label} basis_surface_kind mismatch"
    );
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
        assert_scalar_close(lhs, rhs, tolerance, &format!("{label} {field}"))?;
    }

    Ok(())
}

#[test]
fn box_planar_authored_family_row_is_rust_owned() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    require_complete_ownership_row(AuthoredFamily::BoxPlanar, "box/planar")?;

    let params = BoxParams {
        origin: [-30.0, -15.0, -5.0],
        size: [60.0, 30.0, 10.0],
    };
    let expected_bbox_min = params.origin;
    let expected_bbox_max = [
        params.origin[0] + params.size[0],
        params.origin[1] + params.size[1],
        params.origin[2] + params.size[2],
    ];
    let expected_surface_area = 2.0
        * (params.size[0] * params.size[1]
            + params.size[0] * params.size[2]
            + params.size[1] * params.size[2]);
    let expected_volume = params.size[0] * params.size[1] * params.size[2];
    let expected_unique_edge_length = 4.0 * (params.size[0] + params.size[1] + params.size[2]);
    let expected_wire_occurrence_length = 2.0 * expected_unique_edge_length;

    let mut document = ModelDocument::new()?;
    document.insert_box("base", params)?;

    let base_shape = document.shape("base")?;
    assert_eq!(base_shape.rust_multi_face_analytic_source_count(), Some(3));
    let context = document.kernel().context();
    let face_shapes = context.subshapes(base_shape, ShapeKind::Face)?;
    let edge_shapes = context.subshapes(base_shape, ShapeKind::Edge)?;
    assert_eq!(face_shapes.len(), 6);
    assert_eq!(edge_shapes.len(), 12);
    assert!(
        face_shapes
            .iter()
            .all(|face| face.has_rust_analytic_surface_face_metadata()),
        "all authored box faces should retain Rust analytic plane metadata"
    );

    let ported_topology = context
        .ported_topology(base_shape)?
        .ok_or_else(|| std::io::Error::other("expected Rust-owned box topology snapshot"))?;
    let public_topology = context.topology(base_shape)?;
    let brep = document.brep("base")?;

    assert_eq!(ported_topology.faces.len(), 6);
    assert_eq!(ported_topology.wires.len(), 6);
    assert_eq!(ported_topology.edges.len(), 12);
    assert_eq!(ported_topology.vertex_positions.len(), 8);
    assert_eq!(ported_topology.wire_edge_indices.len(), 24);
    assert_eq!(ported_topology.face_wire_indices.len(), 6);
    assert_eq!(public_topology.faces.len(), ported_topology.faces.len());
    assert_eq!(public_topology.wires.len(), ported_topology.wires.len());
    assert_eq!(public_topology.edges.len(), ported_topology.edges.len());
    assert_eq!(
        public_topology.vertex_positions.len(),
        ported_topology.vertex_positions.len()
    );
    assert_eq!(brep.faces.len(), 6);
    assert_eq!(brep.wires.len(), 6);
    assert_eq!(brep.edges.len(), 12);
    assert_eq!(brep.vertices.len(), 8);
    assert!(brep.faces.iter().all(|face| {
        face.geometry.kind == SurfaceKind::Plane
            && matches!(
                face.ported_face_surface,
                Some(PortedFaceSurface::Analytic(PortedSurface::Plane(_)))
            )
    }));
    assert!(brep.edges.iter().all(|edge| {
        edge.geometry.kind == CurveKind::Line
            && matches!(edge.ported_curve, Some(PortedCurve::Line(_)))
    }));

    let plane_face = face_shapes
        .iter()
        .find(|face| {
            context
                .face_geometry(face)
                .is_ok_and(|geometry| geometry.kind == SurfaceKind::Plane)
        })
        .ok_or_else(|| std::io::Error::other("expected an authored box plane face"))?;
    let public_plane_payload = context.face_plane_payload(plane_face)?;
    let descriptor = context
        .ported_face_surface_descriptor(plane_face)?
        .ok_or_else(|| std::io::Error::other("expected Rust plane face descriptor"))?;
    let descriptor_payload = match descriptor {
        PortedFaceSurface::Analytic(PortedSurface::Plane(payload)) => payload,
        other => {
            return Err(std::io::Error::other(format!(
                "expected Rust plane descriptor, got {other:?}"
            ))
            .into());
        }
    };
    assert_plane_payload_close(
        public_plane_payload,
        descriptor_payload,
        1.0e-12,
        "public plane payload vs Rust descriptor",
    )?;
    let occt_plane_payload = context.face_plane_payload_occt(plane_face)?;
    assert_plane_payload_close(
        public_plane_payload,
        occt_plane_payload,
        1.0e-6,
        "Rust plane payload vs OCCT oracle",
    )?;
    assert!(
        context.face_cylinder_payload(plane_face).is_err(),
        "ported plane faces should reject mismatched raw cylinder payload queries"
    );

    let summary = document.summary("base")?;
    let unique_edge_length = brep.edges.iter().map(|edge| edge.length).sum::<f64>();
    assert_scalar_close(
        summary.surface_area,
        expected_surface_area,
        1.0e-9,
        "document surface area",
    )?;
    assert_scalar_close(summary.volume, expected_volume, 1.0e-9, "document volume")?;
    assert_scalar_close(
        summary.linear_length,
        expected_wire_occurrence_length,
        1.0e-9,
        "document wire-occurrence edge length",
    )?;
    assert_scalar_close(
        brep.summary.surface_area,
        expected_surface_area,
        1.0e-9,
        "BRep surface area",
    )?;
    assert_scalar_close(brep.summary.volume, expected_volume, 1.0e-9, "BRep volume")?;
    assert_scalar_close(
        unique_edge_length,
        expected_unique_edge_length,
        1.0e-9,
        "unique BRep edge length",
    )?;
    assert_vec3_close(summary.bbox_min, expected_bbox_min, 1.0e-12, "bbox min")?;
    assert_vec3_close(summary.bbox_max, expected_bbox_max, 1.0e-12, "bbox max")?;
    assert_eq!(
        brep.summary_bbox_source(),
        SummaryBboxSource::ExactPrimitive
    );
    assert_eq!(
        brep.summary_volume_source(),
        SummaryVolumeSource::ExactPrimitive
    );

    let largest_plane = document.select_face(
        "base",
        FaceSelector::LargestBySurfaceKind(SurfaceKind::Plane),
    )?;
    let top_face = document.select_face(
        "base",
        FaceSelector::BestAlignedPlane {
            normal_hint: [0.0, 0.0, 1.0],
        },
    )?;
    let longest_edge =
        document.select_edge("base", EdgeSelector::LongestByCurveKind(CurveKind::Line))?;
    let shortest_edge =
        document.select_edge("base", EdgeSelector::ShortestByCurveKind(CurveKind::Line))?;
    let faces = document.faces("base")?;
    let edges = document.edges("base")?;
    let report = document.report("base")?;

    assert_eq!(faces.len(), 6);
    assert_eq!(edges.len(), 12);
    assert_eq!(
        document.face_indices_by_surface_kind("base", SurfaceKind::Plane)?,
        vec![0, 1, 2, 3, 4, 5]
    );
    assert_eq!(
        document.edge_indices_by_curve_kind("base", CurveKind::Line)?,
        (0..12).collect::<Vec<_>>()
    );
    assert_eq!(largest_plane.geometry.kind, SurfaceKind::Plane);
    assert_scalar_close(largest_plane.area, 1800.0, 1.0e-9, "largest plane area")?;
    assert_eq!(top_face.geometry.kind, SurfaceKind::Plane);
    assert!(top_face.sample.normal[2] > 0.9);
    assert_eq!(longest_edge.geometry.kind, CurveKind::Line);
    assert_eq!(shortest_edge.geometry.kind, CurveKind::Line);
    assert_scalar_close(longest_edge.length, 60.0, 1.0e-9, "longest line edge")?;
    assert_scalar_close(shortest_edge.length, 10.0, 1.0e-9, "shortest line edge")?;
    assert_eq!(report.summary.primary_kind, ShapeKind::Solid);
    assert_eq!(report.summary.face_count, 6);
    assert_eq!(report.summary.edge_count, 12);
    assert_eq!(report.summary.vertex_count, 8);
    match document.history() {
        [OperationRecord::AddBox { output, params }] => {
            assert_eq!(output, "base");
            assert_vec3_close(params.origin, [-30.0, -15.0, -5.0], 0.0, "history origin")?;
            assert_vec3_close(params.size, [60.0, 30.0, 10.0], 0.0, "history size")?;
        }
        history => {
            return Err(std::io::Error::other(format!(
                "expected single AddBox history entry, got {history:?}"
            ))
            .into());
        }
    }

    Ok(())
}

#[test]
fn cylinder_authored_family_row_is_rust_owned() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    require_complete_ownership_row(AuthoredFamily::Cylinder, "cylinder")?;

    let params = CylinderParams {
        origin: [0.0, 0.0, -7.0],
        axis: [0.0, 0.0, 1.0],
        radius: 5.0,
        height: 14.0,
    };
    let expected_bbox_min = [-params.radius, -params.radius, params.origin[2]];
    let expected_bbox_max = [
        params.radius,
        params.radius,
        params.origin[2] + params.height,
    ];
    let expected_cap_area = PI * params.radius * params.radius;
    let expected_side_area = 2.0 * PI * params.radius * params.height;
    let expected_surface_area = expected_side_area + 2.0 * expected_cap_area;
    let expected_volume = expected_cap_area * params.height;
    let expected_circle_edge_length = 2.0 * PI * params.radius;
    let expected_unique_edge_length = 2.0 * expected_circle_edge_length + params.height;
    let expected_wire_occurrence_length = 4.0 * expected_circle_edge_length + 2.0 * params.height;

    let mut document = ModelDocument::new()?;
    document.insert_cylinder("cylinder", params)?;

    let cylinder_shape = document.shape("cylinder")?;
    assert_eq!(
        cylinder_shape.rust_multi_face_analytic_source_count(),
        Some(2)
    );
    let context = document.kernel().context();
    let face_shapes = context.subshapes(cylinder_shape, ShapeKind::Face)?;
    let edge_shapes = context.subshapes(cylinder_shape, ShapeKind::Edge)?;
    assert_eq!(face_shapes.len(), 3);
    assert_eq!(edge_shapes.len(), 3);
    assert!(
        face_shapes
            .iter()
            .all(|face| face.has_rust_analytic_surface_face_metadata()),
        "all authored cylinder faces should retain Rust analytic metadata"
    );

    let ported_topology = context
        .ported_topology(cylinder_shape)?
        .ok_or_else(|| std::io::Error::other("expected Rust-owned cylinder topology snapshot"))?;
    let public_topology = context.topology(cylinder_shape)?;
    let brep = document.brep("cylinder")?;

    assert_eq!(ported_topology.faces.len(), 3);
    assert_eq!(ported_topology.wires.len(), 3);
    assert_eq!(ported_topology.edges.len(), 3);
    assert_eq!(ported_topology.vertex_positions.len(), 2);
    assert_eq!(ported_topology.wire_edge_indices.len(), 6);
    assert_eq!(ported_topology.face_wire_indices.len(), 3);
    assert_eq!(public_topology.faces.len(), ported_topology.faces.len());
    assert_eq!(public_topology.wires.len(), ported_topology.wires.len());
    assert_eq!(public_topology.edges.len(), ported_topology.edges.len());
    assert_eq!(
        public_topology.vertex_positions.len(),
        ported_topology.vertex_positions.len()
    );
    assert_eq!(brep.faces.len(), 3);
    assert_eq!(brep.wires.len(), 3);
    assert_eq!(brep.edges.len(), 3);
    assert_eq!(brep.vertices.len(), 2);
    assert_eq!(
        brep.faces
            .iter()
            .filter(|face| {
                face.geometry.kind == SurfaceKind::Cylinder
                    && matches!(
                        face.ported_face_surface,
                        Some(PortedFaceSurface::Analytic(PortedSurface::Cylinder(_)))
                    )
            })
            .count(),
        1
    );
    assert_eq!(
        brep.faces
            .iter()
            .filter(|face| {
                face.geometry.kind == SurfaceKind::Plane
                    && matches!(
                        face.ported_face_surface,
                        Some(PortedFaceSurface::Analytic(PortedSurface::Plane(_)))
                    )
            })
            .count(),
        2
    );
    assert_eq!(
        brep.edges
            .iter()
            .filter(|edge| {
                edge.geometry.kind == CurveKind::Circle
                    && matches!(edge.ported_curve, Some(PortedCurve::Circle(_)))
            })
            .count(),
        2
    );
    assert_eq!(
        brep.edges
            .iter()
            .filter(|edge| {
                edge.geometry.kind == CurveKind::Line
                    && matches!(edge.ported_curve, Some(PortedCurve::Line(_)))
            })
            .count(),
        1
    );

    let cylinder_face = face_shapes
        .iter()
        .find(|face| {
            context
                .face_geometry(face)
                .is_ok_and(|geometry| geometry.kind == SurfaceKind::Cylinder)
        })
        .ok_or_else(|| std::io::Error::other("expected an authored cylinder side face"))?;
    let cap_face = face_shapes
        .iter()
        .find(|face| {
            context
                .face_geometry(face)
                .is_ok_and(|geometry| geometry.kind == SurfaceKind::Plane)
        })
        .ok_or_else(|| std::io::Error::other("expected an authored cylinder cap face"))?;

    let public_cylinder_payload = context.face_cylinder_payload(cylinder_face)?;
    let descriptor = context
        .ported_face_surface_descriptor(cylinder_face)?
        .ok_or_else(|| std::io::Error::other("expected Rust cylinder face descriptor"))?;
    let descriptor_payload = match descriptor {
        PortedFaceSurface::Analytic(PortedSurface::Cylinder(payload)) => payload,
        other => {
            return Err(std::io::Error::other(format!(
                "expected Rust cylinder descriptor, got {other:?}"
            ))
            .into());
        }
    };
    assert_cylinder_payload_close(
        public_cylinder_payload,
        descriptor_payload,
        1.0e-12,
        "public cylinder payload vs Rust descriptor",
    )?;
    let occt_cylinder_payload = context.face_cylinder_payload_occt(cylinder_face)?;
    assert_cylinder_payload_close(
        public_cylinder_payload,
        occt_cylinder_payload,
        1.0e-6,
        "Rust cylinder payload vs OCCT oracle",
    )?;
    assert!(
        context.face_plane_payload(cylinder_face).is_err(),
        "ported cylinder faces should reject mismatched plane payload queries"
    );

    let public_cap_payload = context.face_plane_payload(cap_face)?;
    let cap_descriptor = context
        .ported_face_surface_descriptor(cap_face)?
        .ok_or_else(|| std::io::Error::other("expected Rust cap plane descriptor"))?;
    let cap_descriptor_payload = match cap_descriptor {
        PortedFaceSurface::Analytic(PortedSurface::Plane(payload)) => payload,
        other => {
            return Err(std::io::Error::other(format!(
                "expected Rust cap plane descriptor, got {other:?}"
            ))
            .into());
        }
    };
    assert_plane_payload_close(
        public_cap_payload,
        cap_descriptor_payload,
        1.0e-12,
        "public cap plane payload vs Rust descriptor",
    )?;
    let occt_cap_payload = context.face_plane_payload_occt(cap_face)?;
    assert_plane_payload_close(
        public_cap_payload,
        occt_cap_payload,
        1.0e-6,
        "Rust cap plane payload vs OCCT oracle",
    )?;
    assert!(
        context.face_cylinder_payload(cap_face).is_err(),
        "ported cap plane faces should reject mismatched cylinder payload queries"
    );

    let summary = document.summary("cylinder")?;
    let unique_edge_length = brep.edges.iter().map(|edge| edge.length).sum::<f64>();
    assert_scalar_close(
        summary.surface_area,
        expected_surface_area,
        1.0e-9,
        "document cylinder surface area",
    )?;
    assert_scalar_close(
        summary.volume,
        expected_volume,
        1.0e-9,
        "document cylinder volume",
    )?;
    assert_scalar_close(
        summary.linear_length,
        expected_wire_occurrence_length,
        1.0e-9,
        "document cylinder wire-occurrence edge length",
    )?;
    assert_scalar_close(
        brep.summary.surface_area,
        expected_surface_area,
        1.0e-9,
        "BRep cylinder surface area",
    )?;
    assert_scalar_close(
        brep.summary.volume,
        expected_volume,
        1.0e-9,
        "BRep cylinder volume",
    )?;
    assert_scalar_close(
        unique_edge_length,
        expected_unique_edge_length,
        1.0e-9,
        "unique BRep cylinder edge length",
    )?;
    let side_face = brep
        .faces
        .iter()
        .find(|face| face.geometry.kind == SurfaceKind::Cylinder)
        .ok_or_else(|| std::io::Error::other("expected BRep cylinder side face"))?;
    assert_scalar_close(
        side_face.area,
        expected_side_area,
        1.0e-9,
        "BRep cylinder side area",
    )?;
    for cap in brep
        .faces
        .iter()
        .filter(|face| face.geometry.kind == SurfaceKind::Plane)
    {
        assert_scalar_close(cap.area, expected_cap_area, 1.0e-9, "BRep cap area")?;
    }
    assert_vec3_close(
        summary.bbox_min,
        expected_bbox_min,
        1.0e-12,
        "cylinder bbox min",
    )?;
    assert_vec3_close(
        summary.bbox_max,
        expected_bbox_max,
        1.0e-12,
        "cylinder bbox max",
    )?;
    assert_eq!(
        brep.summary_bbox_source(),
        SummaryBboxSource::ExactPrimitive
    );
    assert_eq!(
        brep.summary_volume_source(),
        SummaryVolumeSource::ExactPrimitive
    );

    let side_selector = document.select_face(
        "cylinder",
        FaceSelector::LargestBySurfaceKind(SurfaceKind::Cylinder),
    )?;
    let cap_selector = document.select_face(
        "cylinder",
        FaceSelector::LargestBySurfaceKind(SurfaceKind::Plane),
    )?;
    let top_cap = document.select_face(
        "cylinder",
        FaceSelector::BestAlignedPlane {
            normal_hint: [0.0, 0.0, 1.0],
        },
    )?;
    let circle_edge = document.select_edge(
        "cylinder",
        EdgeSelector::LongestByCurveKind(CurveKind::Circle),
    )?;
    let seam_edge = document.select_edge(
        "cylinder",
        EdgeSelector::ShortestByCurveKind(CurveKind::Line),
    )?;
    let faces = document.faces("cylinder")?;
    let edges = document.edges("cylinder")?;
    let report = document.report("cylinder")?;

    assert_eq!(faces.len(), 3);
    assert_eq!(edges.len(), 3);
    assert_eq!(
        document
            .face_indices_by_surface_kind("cylinder", SurfaceKind::Cylinder)?
            .len(),
        1
    );
    assert_eq!(
        document
            .face_indices_by_surface_kind("cylinder", SurfaceKind::Plane)?
            .len(),
        2
    );
    assert_eq!(
        document
            .edge_indices_by_curve_kind("cylinder", CurveKind::Circle)?
            .len(),
        2
    );
    assert_eq!(
        document
            .edge_indices_by_curve_kind("cylinder", CurveKind::Line)?
            .len(),
        1
    );
    assert_eq!(side_selector.geometry.kind, SurfaceKind::Cylinder);
    assert_scalar_close(
        side_selector.area,
        expected_side_area,
        1.0e-9,
        "selected cylinder side area",
    )?;
    assert_eq!(cap_selector.geometry.kind, SurfaceKind::Plane);
    assert_scalar_close(
        cap_selector.area,
        expected_cap_area,
        1.0e-9,
        "selected cap area",
    )?;
    assert_eq!(top_cap.geometry.kind, SurfaceKind::Plane);
    assert!(top_cap.sample.normal[2] > 0.9);
    assert_eq!(circle_edge.geometry.kind, CurveKind::Circle);
    assert_eq!(seam_edge.geometry.kind, CurveKind::Line);
    assert_scalar_close(
        circle_edge.length,
        expected_circle_edge_length,
        1.0e-9,
        "selected cylinder circle edge",
    )?;
    assert_scalar_close(
        seam_edge.length,
        params.height,
        1.0e-9,
        "selected cylinder seam edge",
    )?;
    assert_eq!(report.summary.primary_kind, ShapeKind::Solid);
    assert_eq!(report.summary.face_count, 3);
    assert_eq!(report.summary.edge_count, 3);
    assert_eq!(report.summary.vertex_count, 2);
    match document.history() {
        [OperationRecord::AddCylinder { output, params }] => {
            assert_eq!(output, "cylinder");
            assert_vec3_close(params.origin, [0.0, 0.0, -7.0], 0.0, "history origin")?;
            assert_vec3_close(params.axis, [0.0, 0.0, 1.0], 0.0, "history axis")?;
            assert_scalar_close(params.radius, 5.0, 0.0, "history radius")?;
            assert_scalar_close(params.height, 14.0, 0.0, "history height")?;
        }
        history => {
            return Err(std::io::Error::other(format!(
                "expected single AddCylinder history entry, got {history:?}"
            ))
            .into());
        }
    }

    Ok(())
}

#[test]
fn cone_authored_family_row_is_rust_owned() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    require_complete_ownership_row(AuthoredFamily::Cone, "cone")?;

    let params = ConeParams {
        origin: [0.0, 0.0, -8.0],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        base_radius: 9.0,
        top_radius: 3.0,
        height: 16.0,
    };
    let expected_bbox_min = [-params.base_radius, -params.base_radius, params.origin[2]];
    let expected_bbox_max = [
        params.base_radius,
        params.base_radius,
        params.origin[2] + params.height,
    ];
    let expected_slant =
        (params.height.powi(2) + (params.base_radius - params.top_radius).powi(2)).sqrt();
    let expected_base_cap_area = PI * params.base_radius * params.base_radius;
    let expected_top_cap_area = PI * params.top_radius * params.top_radius;
    let expected_side_area = PI * (params.base_radius + params.top_radius) * expected_slant;
    let expected_surface_area = expected_side_area + expected_base_cap_area + expected_top_cap_area;
    let expected_volume = PI
        * params.height
        * (params.base_radius * params.base_radius
            + params.base_radius * params.top_radius
            + params.top_radius * params.top_radius)
        / 3.0;
    let expected_base_edge_length = 2.0 * PI * params.base_radius;
    let expected_top_edge_length = 2.0 * PI * params.top_radius;
    let expected_unique_edge_length =
        expected_base_edge_length + expected_top_edge_length + expected_slant;
    let expected_wire_occurrence_length =
        2.0 * (expected_base_edge_length + expected_top_edge_length + expected_slant);

    let mut document = ModelDocument::new()?;
    document.insert_cone("cone", params)?;

    let cone_shape = document.shape("cone")?;
    assert_eq!(cone_shape.rust_multi_face_analytic_source_count(), Some(3));
    let context = document.kernel().context();
    let face_shapes = context.subshapes(cone_shape, ShapeKind::Face)?;
    let edge_shapes = context.subshapes(cone_shape, ShapeKind::Edge)?;
    assert_eq!(face_shapes.len(), 3);
    assert_eq!(edge_shapes.len(), 3);
    assert!(
        face_shapes
            .iter()
            .all(|face| face.has_rust_analytic_surface_face_metadata()),
        "all authored cone faces should retain Rust analytic metadata"
    );

    let ported_topology = context
        .ported_topology(cone_shape)?
        .ok_or_else(|| std::io::Error::other("expected Rust-owned cone topology snapshot"))?;
    let public_topology = context.topology(cone_shape)?;
    let brep = document.brep("cone")?;

    assert_eq!(ported_topology.faces.len(), 3);
    assert_eq!(ported_topology.wires.len(), 3);
    assert_eq!(ported_topology.edges.len(), 3);
    assert_eq!(ported_topology.vertex_positions.len(), 2);
    assert_eq!(ported_topology.wire_edge_indices.len(), 6);
    assert_eq!(ported_topology.face_wire_indices.len(), 3);
    assert_eq!(public_topology.faces.len(), ported_topology.faces.len());
    assert_eq!(public_topology.wires.len(), ported_topology.wires.len());
    assert_eq!(public_topology.edges.len(), ported_topology.edges.len());
    assert_eq!(
        public_topology.vertex_positions.len(),
        ported_topology.vertex_positions.len()
    );
    assert_eq!(brep.faces.len(), 3);
    assert_eq!(brep.wires.len(), 3);
    assert_eq!(brep.edges.len(), 3);
    assert_eq!(brep.vertices.len(), 2);
    assert_eq!(
        brep.faces
            .iter()
            .filter(|face| {
                face.geometry.kind == SurfaceKind::Cone
                    && matches!(
                        face.ported_face_surface,
                        Some(PortedFaceSurface::Analytic(PortedSurface::Cone(_)))
                    )
            })
            .count(),
        1
    );
    assert_eq!(
        brep.faces
            .iter()
            .filter(|face| {
                face.geometry.kind == SurfaceKind::Plane
                    && matches!(
                        face.ported_face_surface,
                        Some(PortedFaceSurface::Analytic(PortedSurface::Plane(_)))
                    )
            })
            .count(),
        2
    );
    assert_eq!(
        brep.edges
            .iter()
            .filter(|edge| {
                edge.geometry.kind == CurveKind::Circle
                    && matches!(edge.ported_curve, Some(PortedCurve::Circle(_)))
            })
            .count(),
        2
    );
    assert_eq!(
        brep.edges
            .iter()
            .filter(|edge| {
                edge.geometry.kind == CurveKind::Line
                    && matches!(edge.ported_curve, Some(PortedCurve::Line(_)))
            })
            .count(),
        1
    );

    let cone_face = face_shapes
        .iter()
        .find(|face| {
            context
                .face_geometry(face)
                .is_ok_and(|geometry| geometry.kind == SurfaceKind::Cone)
        })
        .ok_or_else(|| std::io::Error::other("expected an authored cone side face"))?;
    let cap_face = face_shapes
        .iter()
        .find(|face| {
            context
                .face_geometry(face)
                .is_ok_and(|geometry| geometry.kind == SurfaceKind::Plane)
        })
        .ok_or_else(|| std::io::Error::other("expected an authored cone cap face"))?;

    let public_cone_payload = context.face_cone_payload(cone_face)?;
    let descriptor = context
        .ported_face_surface_descriptor(cone_face)?
        .ok_or_else(|| std::io::Error::other("expected Rust cone face descriptor"))?;
    let descriptor_payload = match descriptor {
        PortedFaceSurface::Analytic(PortedSurface::Cone(payload)) => payload,
        other => {
            return Err(std::io::Error::other(format!(
                "expected Rust cone descriptor, got {other:?}"
            ))
            .into());
        }
    };
    assert_cone_payload_close(
        public_cone_payload,
        descriptor_payload,
        1.0e-12,
        "public cone payload vs Rust descriptor",
    )?;
    let occt_cone_payload = context.face_cone_payload_occt(cone_face)?;
    assert_cone_payload_close(
        public_cone_payload,
        occt_cone_payload,
        1.0e-6,
        "Rust cone payload vs OCCT oracle",
    )?;
    assert!(
        context.face_plane_payload(cone_face).is_err(),
        "ported cone faces should reject mismatched plane payload queries"
    );

    let public_cap_payload = context.face_plane_payload(cap_face)?;
    let cap_descriptor = context
        .ported_face_surface_descriptor(cap_face)?
        .ok_or_else(|| std::io::Error::other("expected Rust cone cap plane descriptor"))?;
    let cap_descriptor_payload = match cap_descriptor {
        PortedFaceSurface::Analytic(PortedSurface::Plane(payload)) => payload,
        other => {
            return Err(std::io::Error::other(format!(
                "expected Rust cone cap plane descriptor, got {other:?}"
            ))
            .into());
        }
    };
    assert_plane_payload_close(
        public_cap_payload,
        cap_descriptor_payload,
        1.0e-12,
        "public cone cap plane payload vs Rust descriptor",
    )?;
    let occt_cap_payload = context.face_plane_payload_occt(cap_face)?;
    assert_plane_payload_close(
        public_cap_payload,
        occt_cap_payload,
        1.0e-6,
        "Rust cone cap plane payload vs OCCT oracle",
    )?;
    assert!(
        context.face_cone_payload(cap_face).is_err(),
        "ported cone cap plane faces should reject mismatched cone payload queries"
    );

    let summary = document.summary("cone")?;
    let unique_edge_length = brep.edges.iter().map(|edge| edge.length).sum::<f64>();
    assert_scalar_close(
        summary.surface_area,
        expected_surface_area,
        1.0e-9,
        "document cone surface area",
    )?;
    assert_scalar_close(
        summary.volume,
        expected_volume,
        1.0e-9,
        "document cone volume",
    )?;
    assert_scalar_close(
        summary.linear_length,
        expected_wire_occurrence_length,
        1.0e-9,
        "document cone wire-occurrence edge length",
    )?;
    assert_scalar_close(
        brep.summary.surface_area,
        expected_surface_area,
        1.0e-9,
        "BRep cone surface area",
    )?;
    assert_scalar_close(
        brep.summary.volume,
        expected_volume,
        1.0e-9,
        "BRep cone volume",
    )?;
    assert_scalar_close(
        unique_edge_length,
        expected_unique_edge_length,
        1.0e-9,
        "unique BRep cone edge length",
    )?;
    let side_face = brep
        .faces
        .iter()
        .find(|face| face.geometry.kind == SurfaceKind::Cone)
        .ok_or_else(|| std::io::Error::other("expected BRep cone side face"))?;
    assert_scalar_close(
        side_face.area,
        expected_side_area,
        2.0e-2,
        "BRep cone side area",
    )?;
    let cap_areas = brep
        .faces
        .iter()
        .filter(|face| face.geometry.kind == SurfaceKind::Plane)
        .map(|face| face.area)
        .collect::<Vec<_>>();
    assert_eq!(cap_areas.len(), 2);
    assert!(
        cap_areas
            .iter()
            .any(|area| (*area - expected_base_cap_area).abs() <= 1.0e-9),
        "expected base cone cap area {expected_base_cap_area}, got {cap_areas:?}"
    );
    assert!(
        cap_areas
            .iter()
            .any(|area| (*area - expected_top_cap_area).abs() <= 1.0e-9),
        "expected top cone cap area {expected_top_cap_area}, got {cap_areas:?}"
    );
    assert_vec3_close(
        summary.bbox_min,
        expected_bbox_min,
        1.0e-12,
        "cone bbox min",
    )?;
    assert_vec3_close(
        summary.bbox_max,
        expected_bbox_max,
        1.0e-12,
        "cone bbox max",
    )?;
    assert_eq!(
        brep.summary_bbox_source(),
        SummaryBboxSource::ExactPrimitive
    );
    assert_eq!(
        brep.summary_volume_source(),
        SummaryVolumeSource::ExactPrimitive
    );

    let side_selector = document.select_face(
        "cone",
        FaceSelector::LargestBySurfaceKind(SurfaceKind::Cone),
    )?;
    let base_cap_selector = document.select_face(
        "cone",
        FaceSelector::LargestBySurfaceKind(SurfaceKind::Plane),
    )?;
    let top_cap = document.select_face(
        "cone",
        FaceSelector::BestAlignedPlane {
            normal_hint: [0.0, 0.0, 1.0],
        },
    )?;
    let base_circle =
        document.select_edge("cone", EdgeSelector::LongestByCurveKind(CurveKind::Circle))?;
    let top_circle =
        document.select_edge("cone", EdgeSelector::ShortestByCurveKind(CurveKind::Circle))?;
    let seam_edge =
        document.select_edge("cone", EdgeSelector::ShortestByCurveKind(CurveKind::Line))?;
    let faces = document.faces("cone")?;
    let edges = document.edges("cone")?;
    let report = document.report("cone")?;

    assert_eq!(faces.len(), 3);
    assert_eq!(edges.len(), 3);
    assert_eq!(
        document
            .face_indices_by_surface_kind("cone", SurfaceKind::Cone)?
            .len(),
        1
    );
    assert_eq!(
        document
            .face_indices_by_surface_kind("cone", SurfaceKind::Plane)?
            .len(),
        2
    );
    assert_eq!(
        document
            .edge_indices_by_curve_kind("cone", CurveKind::Circle)?
            .len(),
        2
    );
    assert_eq!(
        document
            .edge_indices_by_curve_kind("cone", CurveKind::Line)?
            .len(),
        1
    );
    assert_eq!(side_selector.geometry.kind, SurfaceKind::Cone);
    assert_scalar_close(
        side_selector.area,
        expected_side_area,
        2.0e-2,
        "selected cone side area",
    )?;
    assert_eq!(base_cap_selector.geometry.kind, SurfaceKind::Plane);
    assert_scalar_close(
        base_cap_selector.area,
        expected_base_cap_area,
        1.0e-9,
        "selected cone base cap area",
    )?;
    assert_eq!(top_cap.geometry.kind, SurfaceKind::Plane);
    assert!(top_cap.sample.normal[2] > 0.9);
    assert_scalar_close(
        top_cap.area,
        expected_top_cap_area,
        1.0e-9,
        "selected cone top cap area",
    )?;
    assert_eq!(base_circle.geometry.kind, CurveKind::Circle);
    assert_eq!(top_circle.geometry.kind, CurveKind::Circle);
    assert_eq!(seam_edge.geometry.kind, CurveKind::Line);
    assert_scalar_close(
        base_circle.length,
        expected_base_edge_length,
        1.0e-9,
        "selected cone base circle edge",
    )?;
    assert_scalar_close(
        top_circle.length,
        expected_top_edge_length,
        1.0e-9,
        "selected cone top circle edge",
    )?;
    assert_scalar_close(
        seam_edge.length,
        expected_slant,
        1.0e-9,
        "selected cone seam edge",
    )?;
    assert_eq!(report.summary.primary_kind, ShapeKind::Solid);
    assert_eq!(report.summary.face_count, 3);
    assert_eq!(report.summary.edge_count, 3);
    assert_eq!(report.summary.vertex_count, 2);
    match document.history() {
        [OperationRecord::AddCone { output, params }] => {
            assert_eq!(output, "cone");
            assert_vec3_close(params.origin, [0.0, 0.0, -8.0], 0.0, "history origin")?;
            assert_vec3_close(params.axis, [0.0, 0.0, 1.0], 0.0, "history axis")?;
            assert_vec3_close(
                params.x_direction,
                [1.0, 0.0, 0.0],
                0.0,
                "history x_direction",
            )?;
            assert_scalar_close(params.base_radius, 9.0, 0.0, "history base_radius")?;
            assert_scalar_close(params.top_radius, 3.0, 0.0, "history top_radius")?;
            assert_scalar_close(params.height, 16.0, 0.0, "history height")?;
        }
        history => {
            return Err(std::io::Error::other(format!(
                "expected single AddCone history entry, got {history:?}"
            ))
            .into());
        }
    }

    Ok(())
}

#[test]
fn sphere_authored_family_row_is_rust_owned() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    require_complete_ownership_row(AuthoredFamily::Sphere, "sphere")?;

    let params = SphereParams {
        origin: [4.0, -6.0, 3.0],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        radius: 7.0,
    };
    let expected_bbox_min = [
        params.origin[0] - params.radius,
        params.origin[1] - params.radius,
        params.origin[2] - params.radius,
    ];
    let expected_bbox_max = [
        params.origin[0] + params.radius,
        params.origin[1] + params.radius,
        params.origin[2] + params.radius,
    ];
    let expected_surface_area = 4.0 * PI * params.radius * params.radius;
    let expected_volume = 4.0 * PI * params.radius * params.radius * params.radius / 3.0;

    let mut document = ModelDocument::new()?;
    document.insert_sphere("sphere", params)?;

    let sphere_shape = document.shape("sphere")?;
    assert_eq!(
        sphere_shape.rust_multi_face_analytic_source_count(),
        Some(1)
    );
    let context = document.kernel().context();
    let face_shapes = context.subshapes(sphere_shape, ShapeKind::Face)?;
    let edge_shapes = context.subshapes(sphere_shape, ShapeKind::Edge)?;
    assert_eq!(face_shapes.len(), 1);
    assert_eq!(edge_shapes.len(), 0);
    assert!(
        face_shapes
            .iter()
            .all(|face| face.has_rust_analytic_surface_face_metadata()),
        "all authored sphere faces should retain Rust analytic metadata"
    );

    let ported_topology = context
        .ported_topology(sphere_shape)?
        .ok_or_else(|| std::io::Error::other("expected Rust-owned sphere topology snapshot"))?;
    let public_topology = context.topology(sphere_shape)?;
    let brep = document.brep("sphere")?;

    assert_eq!(ported_topology.faces.len(), 1);
    assert_eq!(ported_topology.wires.len(), 0);
    assert_eq!(ported_topology.edges.len(), 0);
    assert_eq!(ported_topology.vertex_positions.len(), 0);
    assert_eq!(ported_topology.wire_edge_indices.len(), 0);
    assert_eq!(ported_topology.face_wire_indices.len(), 0);
    assert_eq!(public_topology.faces.len(), ported_topology.faces.len());
    assert_eq!(public_topology.wires.len(), ported_topology.wires.len());
    assert_eq!(public_topology.edges.len(), ported_topology.edges.len());
    assert_eq!(
        public_topology.vertex_positions.len(),
        ported_topology.vertex_positions.len()
    );
    assert_eq!(brep.faces.len(), 1);
    assert_eq!(brep.wires.len(), 0);
    assert_eq!(brep.edges.len(), 0);
    assert_eq!(brep.vertices.len(), 0);
    assert!(brep.faces.iter().all(|face| {
        face.geometry.kind == SurfaceKind::Sphere
            && matches!(
                face.ported_face_surface,
                Some(PortedFaceSurface::Analytic(PortedSurface::Sphere(_)))
            )
    }));
    assert!(brep.edges.is_empty());

    let sphere_face = face_shapes
        .iter()
        .find(|face| {
            context
                .face_geometry(face)
                .is_ok_and(|geometry| geometry.kind == SurfaceKind::Sphere)
        })
        .ok_or_else(|| std::io::Error::other("expected an authored sphere face"))?;

    let public_sphere_payload = context.face_sphere_payload(sphere_face)?;
    let descriptor = context
        .ported_face_surface_descriptor(sphere_face)?
        .ok_or_else(|| std::io::Error::other("expected Rust sphere face descriptor"))?;
    let descriptor_payload = match descriptor {
        PortedFaceSurface::Analytic(PortedSurface::Sphere(payload)) => payload,
        other => {
            return Err(std::io::Error::other(format!(
                "expected Rust sphere descriptor, got {other:?}"
            ))
            .into());
        }
    };
    assert_sphere_payload_close(
        public_sphere_payload,
        descriptor_payload,
        1.0e-12,
        "public sphere payload vs Rust descriptor",
    )?;
    let occt_sphere_payload = context.face_sphere_payload_occt(sphere_face)?;
    assert_sphere_payload_close(
        public_sphere_payload,
        occt_sphere_payload,
        1.0e-6,
        "Rust sphere payload vs OCCT oracle",
    )?;
    assert!(
        context.face_plane_payload(sphere_face).is_err(),
        "ported sphere faces should reject mismatched plane payload queries"
    );

    let summary = document.summary("sphere")?;
    let unique_edge_length = brep.edges.iter().map(|edge| edge.length).sum::<f64>();
    assert_scalar_close(
        summary.surface_area,
        expected_surface_area,
        1.0e-9,
        "document sphere surface area",
    )?;
    assert_scalar_close(
        summary.volume,
        expected_volume,
        1.0e-9,
        "document sphere volume",
    )?;
    assert_scalar_close(
        summary.linear_length,
        0.0,
        1.0e-12,
        "document sphere edge length",
    )?;
    assert_scalar_close(
        brep.summary.surface_area,
        expected_surface_area,
        1.0e-9,
        "BRep sphere surface area",
    )?;
    assert_scalar_close(
        brep.summary.volume,
        expected_volume,
        1.0e-9,
        "BRep sphere volume",
    )?;
    assert_scalar_close(
        unique_edge_length,
        0.0,
        1.0e-12,
        "unique BRep sphere edge length",
    )?;
    let brep_sphere_face = brep
        .faces
        .iter()
        .find(|face| face.geometry.kind == SurfaceKind::Sphere)
        .ok_or_else(|| std::io::Error::other("expected BRep sphere face"))?;
    assert_scalar_close(
        brep_sphere_face.area,
        expected_surface_area,
        1.0e-9,
        "BRep sphere face area",
    )?;
    assert_vec3_close(
        summary.bbox_min,
        expected_bbox_min,
        1.0e-12,
        "sphere bbox min",
    )?;
    assert_vec3_close(
        summary.bbox_max,
        expected_bbox_max,
        1.0e-12,
        "sphere bbox max",
    )?;
    assert_eq!(
        brep.summary_bbox_source(),
        SummaryBboxSource::ExactPrimitive
    );
    assert_eq!(
        brep.summary_volume_source(),
        SummaryVolumeSource::ExactPrimitive
    );

    let selected_sphere = document.select_face(
        "sphere",
        FaceSelector::LargestBySurfaceKind(SurfaceKind::Sphere),
    )?;
    let faces = document.faces("sphere")?;
    let edges = document.edges("sphere")?;
    let report = document.report("sphere")?;

    assert_eq!(faces.len(), 1);
    assert_eq!(edges.len(), 0);
    assert_eq!(
        document
            .face_indices_by_surface_kind("sphere", SurfaceKind::Sphere)?
            .len(),
        1
    );
    assert!(document
        .face_indices_by_surface_kind("sphere", SurfaceKind::Plane)?
        .is_empty());
    assert!(document
        .edge_indices_by_curve_kind("sphere", CurveKind::Line)?
        .is_empty());
    assert!(document
        .edge_indices_by_curve_kind("sphere", CurveKind::Circle)?
        .is_empty());
    assert_eq!(selected_sphere.geometry.kind, SurfaceKind::Sphere);
    assert_scalar_close(
        selected_sphere.area,
        expected_surface_area,
        1.0e-9,
        "selected sphere face area",
    )?;
    assert!(
        document
            .select_edge("sphere", EdgeSelector::LongestByCurveKind(CurveKind::Line),)
            .is_err(),
        "edge-free sphere should reject line edge selectors"
    );
    assert!(
        document
            .select_edge(
                "sphere",
                EdgeSelector::ShortestByCurveKind(CurveKind::Circle),
            )
            .is_err(),
        "edge-free sphere should reject circle edge selectors"
    );
    assert_eq!(report.summary.primary_kind, ShapeKind::Solid);
    assert_eq!(report.summary.face_count, 1);
    assert_eq!(report.summary.edge_count, 0);
    assert_eq!(report.summary.vertex_count, 0);
    match document.history() {
        [OperationRecord::AddSphere { output, params }] => {
            assert_eq!(output, "sphere");
            assert_vec3_close(params.origin, [4.0, -6.0, 3.0], 0.0, "history origin")?;
            assert_vec3_close(params.axis, [0.0, 0.0, 1.0], 0.0, "history axis")?;
            assert_vec3_close(
                params.x_direction,
                [1.0, 0.0, 0.0],
                0.0,
                "history x_direction",
            )?;
            assert_scalar_close(params.radius, 7.0, 0.0, "history radius")?;
        }
        history => {
            return Err(std::io::Error::other(format!(
                "expected single AddSphere history entry, got {history:?}"
            ))
            .into());
        }
    }

    Ok(())
}

#[test]
fn torus_authored_family_row_is_rust_owned() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    require_complete_ownership_row(AuthoredFamily::Torus, "torus")?;

    let params = TorusParams {
        origin: [-5.0, 8.0, 2.5],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        major_radius: 11.0,
        minor_radius: 3.0,
    };
    let radial_extent = params.major_radius + params.minor_radius;
    let expected_bbox_min = [
        params.origin[0] - radial_extent,
        params.origin[1] - radial_extent,
        params.origin[2] - params.minor_radius,
    ];
    let expected_bbox_max = [
        params.origin[0] + radial_extent,
        params.origin[1] + radial_extent,
        params.origin[2] + params.minor_radius,
    ];
    let expected_surface_area = 4.0 * PI * PI * params.major_radius * params.minor_radius;
    let expected_volume =
        2.0 * PI * PI * params.major_radius * params.minor_radius * params.minor_radius;

    let mut document = ModelDocument::new()?;
    document.insert_torus("torus", params)?;

    let torus_shape = document.shape("torus")?;
    assert_eq!(torus_shape.rust_multi_face_analytic_source_count(), Some(1));
    let context = document.kernel().context();
    let face_shapes = context.subshapes(torus_shape, ShapeKind::Face)?;
    let edge_shapes = context.subshapes(torus_shape, ShapeKind::Edge)?;
    assert_eq!(face_shapes.len(), 1);
    assert_eq!(edge_shapes.len(), 0);
    assert!(
        face_shapes
            .iter()
            .all(|face| face.has_rust_analytic_surface_face_metadata()),
        "all authored torus faces should retain Rust analytic metadata"
    );

    let ported_topology = context
        .ported_topology(torus_shape)?
        .ok_or_else(|| std::io::Error::other("expected Rust-owned torus topology snapshot"))?;
    let public_topology = context.topology(torus_shape)?;
    let brep = document.brep("torus")?;

    assert_eq!(ported_topology.faces.len(), 1);
    assert_eq!(ported_topology.wires.len(), 0);
    assert_eq!(ported_topology.edges.len(), 0);
    assert_eq!(ported_topology.vertex_positions.len(), 0);
    assert_eq!(ported_topology.wire_edge_indices.len(), 0);
    assert_eq!(ported_topology.face_wire_indices.len(), 0);
    assert_eq!(public_topology.faces.len(), ported_topology.faces.len());
    assert_eq!(public_topology.wires.len(), ported_topology.wires.len());
    assert_eq!(public_topology.edges.len(), ported_topology.edges.len());
    assert_eq!(
        public_topology.vertex_positions.len(),
        ported_topology.vertex_positions.len()
    );
    assert_eq!(brep.faces.len(), 1);
    assert_eq!(brep.wires.len(), 0);
    assert_eq!(brep.edges.len(), 0);
    assert_eq!(brep.vertices.len(), 0);
    assert!(brep.faces.iter().all(|face| {
        face.geometry.kind == SurfaceKind::Torus
            && matches!(
                face.ported_face_surface,
                Some(PortedFaceSurface::Analytic(PortedSurface::Torus(_)))
            )
    }));
    assert!(brep.edges.is_empty());

    let torus_face = face_shapes
        .iter()
        .find(|face| {
            context
                .face_geometry(face)
                .is_ok_and(|geometry| geometry.kind == SurfaceKind::Torus)
        })
        .ok_or_else(|| std::io::Error::other("expected an authored torus face"))?;

    let public_torus_payload = context.face_torus_payload(torus_face)?;
    let descriptor = context
        .ported_face_surface_descriptor(torus_face)?
        .ok_or_else(|| std::io::Error::other("expected Rust torus face descriptor"))?;
    let descriptor_payload = match descriptor {
        PortedFaceSurface::Analytic(PortedSurface::Torus(payload)) => payload,
        other => {
            return Err(std::io::Error::other(format!(
                "expected Rust torus descriptor, got {other:?}"
            ))
            .into());
        }
    };
    assert_torus_payload_close(
        public_torus_payload,
        descriptor_payload,
        1.0e-12,
        "public torus payload vs Rust descriptor",
    )?;
    let occt_torus_payload = context.face_torus_payload_occt(torus_face)?;
    assert_torus_payload_close(
        public_torus_payload,
        occt_torus_payload,
        1.0e-6,
        "Rust torus payload vs OCCT oracle",
    )?;
    assert!(
        context.face_sphere_payload(torus_face).is_err(),
        "ported torus faces should reject mismatched sphere payload queries"
    );

    let summary = document.summary("torus")?;
    let unique_edge_length = brep.edges.iter().map(|edge| edge.length).sum::<f64>();
    assert_scalar_close(
        summary.surface_area,
        expected_surface_area,
        1.0e-8,
        "document torus surface area",
    )?;
    assert_scalar_close(
        summary.volume,
        expected_volume,
        1.0e-8,
        "document torus volume",
    )?;
    assert_scalar_close(
        summary.linear_length,
        0.0,
        1.0e-12,
        "document torus edge length",
    )?;
    assert_scalar_close(
        brep.summary.surface_area,
        expected_surface_area,
        1.0e-8,
        "BRep torus surface area",
    )?;
    assert_scalar_close(
        brep.summary.volume,
        expected_volume,
        1.0e-8,
        "BRep torus volume",
    )?;
    assert_scalar_close(
        unique_edge_length,
        0.0,
        1.0e-12,
        "unique BRep torus edge length",
    )?;
    let brep_torus_face = brep
        .faces
        .iter()
        .find(|face| face.geometry.kind == SurfaceKind::Torus)
        .ok_or_else(|| std::io::Error::other("expected BRep torus face"))?;
    assert_scalar_close(
        brep_torus_face.area,
        expected_surface_area,
        1.0e-8,
        "BRep torus face area",
    )?;
    assert_vec3_close(
        summary.bbox_min,
        expected_bbox_min,
        1.0e-12,
        "torus bbox min",
    )?;
    assert_vec3_close(
        summary.bbox_max,
        expected_bbox_max,
        1.0e-12,
        "torus bbox max",
    )?;
    assert_eq!(
        brep.summary_bbox_source(),
        SummaryBboxSource::ExactPrimitive
    );
    assert_eq!(
        brep.summary_volume_source(),
        SummaryVolumeSource::ExactPrimitive
    );

    let selected_torus = document.select_face(
        "torus",
        FaceSelector::LargestBySurfaceKind(SurfaceKind::Torus),
    )?;
    let faces = document.faces("torus")?;
    let edges = document.edges("torus")?;
    let report = document.report("torus")?;

    assert_eq!(faces.len(), 1);
    assert_eq!(edges.len(), 0);
    assert_eq!(
        document
            .face_indices_by_surface_kind("torus", SurfaceKind::Torus)?
            .len(),
        1
    );
    assert!(document
        .face_indices_by_surface_kind("torus", SurfaceKind::Plane)?
        .is_empty());
    assert!(document
        .edge_indices_by_curve_kind("torus", CurveKind::Line)?
        .is_empty());
    assert!(document
        .edge_indices_by_curve_kind("torus", CurveKind::Circle)?
        .is_empty());
    assert_eq!(selected_torus.geometry.kind, SurfaceKind::Torus);
    assert_scalar_close(
        selected_torus.area,
        expected_surface_area,
        1.0e-8,
        "selected torus face area",
    )?;
    assert!(
        document
            .select_edge("torus", EdgeSelector::LongestByCurveKind(CurveKind::Line),)
            .is_err(),
        "edge-free torus should reject line edge selectors"
    );
    assert!(
        document
            .select_edge(
                "torus",
                EdgeSelector::ShortestByCurveKind(CurveKind::Circle),
            )
            .is_err(),
        "edge-free torus should reject circle edge selectors"
    );
    assert_eq!(report.summary.primary_kind, ShapeKind::Solid);
    assert_eq!(report.summary.face_count, 1);
    assert_eq!(report.summary.edge_count, 0);
    assert_eq!(report.summary.vertex_count, 0);
    match document.history() {
        [OperationRecord::AddTorus { output, params }] => {
            assert_eq!(output, "torus");
            assert_vec3_close(params.origin, [-5.0, 8.0, 2.5], 0.0, "history origin")?;
            assert_vec3_close(params.axis, [0.0, 0.0, 1.0], 0.0, "history axis")?;
            assert_vec3_close(
                params.x_direction,
                [1.0, 0.0, 0.0],
                0.0,
                "history x_direction",
            )?;
            assert_scalar_close(params.major_radius, 11.0, 0.0, "history major_radius")?;
            assert_scalar_close(params.minor_radius, 3.0, 0.0, "history minor_radius")?;
        }
        history => {
            return Err(std::io::Error::other(format!(
                "expected single AddTorus history entry, got {history:?}"
            ))
            .into());
        }
    }

    Ok(())
}

#[test]
fn prism_extrusion_authored_family_row_is_rust_owned() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    require_complete_ownership_row(AuthoredFamily::PrismExtrusion, "prism/extrusion")?;

    let profile_params = EllipseEdgeParams {
        origin: [30.0, 0.0, 0.0],
        axis: [0.0, 1.0, 0.0],
        x_direction: [1.0, 0.0, 0.0],
        major_radius: 10.0,
        minor_radius: 6.0,
    };
    let prism_params = PrismParams {
        direction: [0.0, 24.0, 0.0],
    };
    let expected_bbox_min = [20.0, 0.0, -6.0];
    let expected_bbox_max = [40.0, 24.0, 6.0];

    let mut document = ModelDocument::new()?;
    document.insert_ellipse_edge("profile", profile_params)?;
    document.prism("prism", "profile", prism_params)?;

    let prism_shape = document.shape("prism")?;
    assert!(
        prism_shape.has_rust_swept_surface_face_metadata(),
        "authored edge-source prism should retain Rust single-face swept metadata"
    );
    let context = document.kernel().context();
    let face_shapes = context.subshapes(prism_shape, ShapeKind::Face)?;
    let edge_shapes = context.subshapes(prism_shape, ShapeKind::Edge)?;
    assert_eq!(face_shapes.len(), 1);
    assert!(!edge_shapes.is_empty());
    assert!(
        face_shapes
            .iter()
            .all(|face| face.has_rust_swept_surface_face_metadata()),
        "generated prism extrusion faces should retain Rust swept metadata"
    );

    let extrusion_face = face_shapes
        .iter()
        .find(|face| {
            context
                .face_geometry(face)
                .is_ok_and(|geometry| geometry.kind == SurfaceKind::Extrusion)
        })
        .ok_or_else(|| std::io::Error::other("expected an authored extrusion face"))?;

    let ported_topology = context
        .ported_topology(prism_shape)?
        .ok_or_else(|| std::io::Error::other("expected Rust-owned prism topology snapshot"))?;
    let public_topology = context.topology(prism_shape)?;
    let brep = document.brep("prism")?;

    assert_eq!(ported_topology.faces.len(), 1);
    assert!(!ported_topology.wires.is_empty());
    assert!(!ported_topology.edges.is_empty());
    assert_eq!(public_topology.faces.len(), ported_topology.faces.len());
    assert_eq!(public_topology.wires.len(), ported_topology.wires.len());
    assert_eq!(public_topology.edges.len(), ported_topology.edges.len());
    assert_eq!(
        public_topology.vertex_positions.len(),
        ported_topology.vertex_positions.len()
    );
    assert_eq!(brep.faces.len(), ported_topology.faces.len());
    assert_eq!(brep.wires.len(), ported_topology.wires.len());
    assert_eq!(brep.edges.len(), ported_topology.edges.len());
    assert_eq!(brep.vertices.len(), ported_topology.vertex_positions.len());
    assert_eq!(
        brep.faces
            .iter()
            .filter(|face| {
                face.geometry.kind == SurfaceKind::Extrusion
                    && matches!(
                        face.ported_face_surface,
                        Some(PortedFaceSurface::Swept(
                            PortedSweptSurface::Extrusion { .. }
                        ))
                    )
            })
            .count(),
        1
    );
    assert!(brep.edges.iter().all(|edge| {
        matches!(edge.geometry.kind, CurveKind::Line | CurveKind::Ellipse)
            && edge.ported_curve.is_some()
    }));
    assert!(
        brep.edges
            .iter()
            .any(|edge| matches!(edge.ported_curve, Some(PortedCurve::Ellipse(_)))),
        "prism profile boundary should materialize Rust-owned ellipse edges"
    );

    let public_extrusion_payload = context.face_extrusion_payload(extrusion_face)?;
    assert_eq!(
        public_extrusion_payload.basis_curve_kind,
        CurveKind::Ellipse
    );
    assert_vec3_close(
        public_extrusion_payload.direction,
        [0.0, -1.0, 0.0],
        1.0e-12,
        "public extrusion direction",
    )?;
    let descriptor = context
        .ported_face_surface_descriptor(extrusion_face)?
        .ok_or_else(|| std::io::Error::other("expected Rust extrusion face descriptor"))?;
    let descriptor_payload = match descriptor {
        PortedFaceSurface::Swept(PortedSweptSurface::Extrusion {
            payload,
            basis_curve,
            basis_geometry,
        }) => {
            assert!(matches!(basis_curve, PortedCurve::Ellipse(_)));
            assert_eq!(basis_geometry.kind, CurveKind::Ellipse);
            payload
        }
        other => {
            return Err(std::io::Error::other(format!(
                "expected Rust extrusion descriptor, got {other:?}"
            ))
            .into());
        }
    };
    assert_extrusion_payload_close(
        public_extrusion_payload,
        descriptor_payload,
        1.0e-12,
        "public extrusion payload vs Rust descriptor",
    )?;
    let occt_extrusion_payload = context.face_extrusion_payload_occt(extrusion_face)?;
    assert_extrusion_payload_close(
        public_extrusion_payload,
        occt_extrusion_payload,
        1.0e-12,
        "Rust extrusion payload vs OCCT oracle",
    )?;
    assert!(
        context.face_plane_payload(extrusion_face).is_err(),
        "ported extrusion faces should reject mismatched plane payload queries"
    );

    let rust_sample = context
        .ported_face_sample_normalized(extrusion_face, [0.25, 0.75])?
        .ok_or_else(|| std::io::Error::other("expected Rust extrusion face sample"))?;
    let occt_sample = context.face_sample_normalized_occt(extrusion_face, [0.25, 0.75])?;
    assert_vec3_close(
        rust_sample.position,
        occt_sample.position,
        1.0e-9,
        "extrusion sample position vs OCCT oracle",
    )?;
    assert_vec3_close(
        rust_sample.normal,
        occt_sample.normal,
        1.0e-9,
        "extrusion sample normal vs OCCT oracle",
    )?;

    let summary = document.summary("prism")?;
    let occt_summary = context.describe_shape_occt(prism_shape)?;
    let unique_edge_length = brep.edges.iter().map(|edge| edge.length).sum::<f64>();
    let wire_occurrence_length = if ported_topology.wire_edge_indices.is_empty() {
        unique_edge_length
    } else {
        ported_topology
            .wire_edge_indices
            .iter()
            .filter_map(|&edge_index| brep.edges.get(edge_index))
            .map(|edge| edge.length)
            .sum()
    };
    let brep_surface_area = brep.faces.iter().map(|face| face.area).sum::<f64>();
    assert_scalar_close(
        summary.surface_area,
        brep_surface_area,
        1.0e-9,
        "document prism surface area from BRep faces",
    )?;
    assert_scalar_close(
        summary.surface_area,
        occt_summary.surface_area,
        1.0e-2,
        "Rust prism surface area vs OCCT oracle",
    )?;
    assert_scalar_close(summary.volume, 0.0, 1.0e-12, "document prism volume")?;
    assert_scalar_close(
        summary.linear_length,
        wire_occurrence_length,
        1.0e-9,
        "document prism wire-occurrence edge length",
    )?;
    assert_scalar_close(
        unique_edge_length,
        brep.edges.iter().map(|edge| edge.length).sum::<f64>(),
        1.0e-12,
        "unique BRep prism edge length",
    )?;
    assert_vec3_close(
        summary.bbox_min,
        expected_bbox_min,
        1.0e-7,
        "prism bbox min",
    )?;
    assert_vec3_close(
        summary.bbox_max,
        expected_bbox_max,
        1.0e-7,
        "prism bbox max",
    )?;
    assert_eq!(brep.summary_bbox_source(), SummaryBboxSource::PortedBrep);
    assert_eq!(brep.summary_volume_source(), SummaryVolumeSource::Zero);

    let selected_extrusion = document.select_face(
        "prism",
        FaceSelector::LargestBySurfaceKind(SurfaceKind::Extrusion),
    )?;
    let selected_profile = document.select_edge(
        "prism",
        EdgeSelector::LongestByCurveKind(CurveKind::Ellipse),
    )?;
    let faces = document.faces("prism")?;
    let edges = document.edges("prism")?;
    let report = document.report("prism")?;

    assert_eq!(faces.len(), 1);
    assert_eq!(edges.len(), brep.edges.len());
    assert_eq!(
        document.face_indices_by_surface_kind("prism", SurfaceKind::Extrusion)?,
        vec![0]
    );
    assert_eq!(
        document
            .edge_indices_by_curve_kind("prism", CurveKind::Ellipse)?
            .len(),
        brep.edges
            .iter()
            .filter(|edge| edge.geometry.kind == CurveKind::Ellipse)
            .count()
    );
    assert_eq!(selected_extrusion.geometry.kind, SurfaceKind::Extrusion);
    assert_scalar_close(
        selected_extrusion.area,
        brep_surface_area,
        1.0e-9,
        "selected extrusion face area",
    )?;
    assert_eq!(selected_profile.geometry.kind, CurveKind::Ellipse);
    assert_eq!(report.summary.primary_kind, ShapeKind::Face);
    assert_eq!(report.summary.face_count, 1);
    assert_eq!(report.summary.edge_count, brep.edges.len());
    match document.history() {
        [OperationRecord::AddEllipseEdge { output, params }, OperationRecord::Prism {
            output: prism_output,
            input,
            params: prism_history,
        }] => {
            assert_eq!(output, "profile");
            assert_vec3_close(
                params.origin,
                profile_params.origin,
                0.0,
                "history ellipse origin",
            )?;
            assert_vec3_close(
                params.axis,
                profile_params.axis,
                0.0,
                "history ellipse axis",
            )?;
            assert_vec3_close(
                params.x_direction,
                profile_params.x_direction,
                0.0,
                "history ellipse x_direction",
            )?;
            assert_scalar_close(
                params.major_radius,
                profile_params.major_radius,
                0.0,
                "history ellipse major_radius",
            )?;
            assert_scalar_close(
                params.minor_radius,
                profile_params.minor_radius,
                0.0,
                "history ellipse minor_radius",
            )?;
            assert_eq!(prism_output, "prism");
            assert_eq!(input, "profile");
            assert_vec3_close(
                prism_history.direction,
                prism_params.direction,
                0.0,
                "history prism direction",
            )?;
        }
        history => {
            return Err(std::io::Error::other(format!(
                "expected AddEllipseEdge + Prism history entries, got {history:?}"
            ))
            .into());
        }
    }

    Ok(())
}

#[test]
fn revolution_authored_family_row_is_rust_owned() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    require_complete_ownership_row(AuthoredFamily::Revolution, "revolution")?;

    let profile_params = EllipseEdgeParams {
        origin: [30.0, 0.0, 0.0],
        axis: [0.0, 1.0, 0.0],
        x_direction: [1.0, 0.0, 0.0],
        major_radius: 10.0,
        minor_radius: 6.0,
    };
    let revolution_params = RevolutionParams {
        origin: [0.0, 0.0, 0.0],
        axis: [0.0, 0.0, 1.0],
        angle_radians: PI,
    };

    let mut document = ModelDocument::new()?;
    document.insert_ellipse_edge("profile", profile_params)?;
    document.revolution("revolution", "profile", revolution_params)?;

    let revolution_shape = document.shape("revolution")?;
    assert!(
        revolution_shape.has_rust_swept_surface_face_metadata(),
        "authored edge-source revolution should retain Rust single-face swept metadata"
    );
    assert_eq!(
        revolution_shape.rust_multi_face_swept_source_count(),
        None,
        "edge-source revolution should not rely on multi-face swept metadata"
    );

    let context = document.kernel().context();
    let face_shapes = context.subshapes(revolution_shape, ShapeKind::Face)?;
    let edge_shapes = context.subshapes(revolution_shape, ShapeKind::Edge)?;
    assert_eq!(face_shapes.len(), 1);
    assert!(!edge_shapes.is_empty());
    assert!(
        face_shapes
            .iter()
            .all(|face| face.has_rust_swept_surface_face_metadata()),
        "generated revolution faces should retain Rust swept metadata"
    );

    let revolution_face = face_shapes
        .iter()
        .find(|face| {
            context
                .face_geometry(face)
                .is_ok_and(|geometry| geometry.kind == SurfaceKind::Revolution)
        })
        .ok_or_else(|| std::io::Error::other("expected an authored revolution face"))?;

    let ported_topology = context
        .ported_topology(revolution_shape)?
        .ok_or_else(|| std::io::Error::other("expected Rust-owned revolution topology snapshot"))?;
    let public_topology = context.topology(revolution_shape)?;
    let brep = document.brep("revolution")?;

    assert_eq!(ported_topology.faces.len(), 1);
    assert!(!ported_topology.wires.is_empty());
    assert!(!ported_topology.edges.is_empty());
    assert_eq!(public_topology.faces.len(), ported_topology.faces.len());
    assert_eq!(public_topology.wires.len(), ported_topology.wires.len());
    assert_eq!(public_topology.edges.len(), ported_topology.edges.len());
    assert_eq!(
        public_topology.vertex_positions.len(),
        ported_topology.vertex_positions.len()
    );
    assert_eq!(brep.faces.len(), ported_topology.faces.len());
    assert_eq!(brep.wires.len(), ported_topology.wires.len());
    assert_eq!(brep.edges.len(), ported_topology.edges.len());
    assert_eq!(brep.vertices.len(), ported_topology.vertex_positions.len());
    assert_eq!(
        brep.faces
            .iter()
            .filter(|face| {
                face.geometry.kind == SurfaceKind::Revolution
                    && matches!(
                        face.ported_face_surface,
                        Some(PortedFaceSurface::Swept(
                            PortedSweptSurface::Revolution { .. }
                        ))
                    )
            })
            .count(),
        1
    );
    assert!(brep.edges.iter().all(|edge| {
        matches!(
            edge.geometry.kind,
            CurveKind::Line | CurveKind::Circle | CurveKind::Ellipse
        ) && edge.ported_curve.is_some()
    }));
    assert!(
        brep.edges
            .iter()
            .any(|edge| matches!(edge.ported_curve, Some(PortedCurve::Ellipse(_)))),
        "revolution profile boundary should materialize Rust-owned ellipse edges"
    );

    let public_revolution_payload = context.face_revolution_payload(revolution_face)?;
    assert_eq!(
        public_revolution_payload.basis_curve_kind,
        CurveKind::Ellipse
    );
    let descriptor = context
        .ported_face_surface_descriptor(revolution_face)?
        .ok_or_else(|| std::io::Error::other("expected Rust revolution face descriptor"))?;
    let descriptor_payload = match descriptor {
        PortedFaceSurface::Swept(PortedSweptSurface::Revolution {
            payload,
            basis_curve,
            basis_geometry,
        }) => {
            assert!(matches!(basis_curve, PortedCurve::Ellipse(_)));
            assert_eq!(basis_geometry.kind, CurveKind::Ellipse);
            payload
        }
        other => {
            return Err(std::io::Error::other(format!(
                "expected Rust revolution descriptor, got {other:?}"
            ))
            .into());
        }
    };
    assert_revolution_payload_close(
        public_revolution_payload,
        descriptor_payload,
        1.0e-12,
        "public revolution payload vs Rust descriptor",
    )?;
    let occt_revolution_payload = context.face_revolution_payload_occt(revolution_face)?;
    assert_revolution_payload_close(
        public_revolution_payload,
        occt_revolution_payload,
        1.0e-12,
        "Rust revolution payload vs OCCT oracle",
    )?;
    assert!(
        context.face_extrusion_payload(revolution_face).is_err(),
        "ported revolution faces should reject mismatched extrusion payload queries"
    );

    let rust_sample = context
        .ported_face_sample_normalized(revolution_face, [0.25, 0.75])?
        .ok_or_else(|| std::io::Error::other("expected Rust revolution face sample"))?;
    let occt_sample = context.face_sample_normalized_occt(revolution_face, [0.25, 0.75])?;
    assert_vec3_close(
        rust_sample.position,
        occt_sample.position,
        1.0e-9,
        "revolution sample position vs OCCT oracle",
    )?;
    assert_vec3_close(
        rust_sample.normal,
        occt_sample.normal,
        1.0e-9,
        "revolution sample normal vs OCCT oracle",
    )?;

    let summary = document.summary("revolution")?;
    let occt_summary = context.describe_shape_occt(revolution_shape)?;
    let unique_edge_length = brep.edges.iter().map(|edge| edge.length).sum::<f64>();
    let wire_occurrence_length = if ported_topology.wire_edge_indices.is_empty() {
        unique_edge_length
    } else {
        ported_topology
            .wire_edge_indices
            .iter()
            .filter_map(|&edge_index| brep.edges.get(edge_index))
            .map(|edge| edge.length)
            .sum()
    };
    let brep_surface_area = brep.faces.iter().map(|face| face.area).sum::<f64>();
    assert_scalar_close(
        summary.surface_area,
        brep_surface_area,
        1.0e-9,
        "document revolution surface area from BRep faces",
    )?;
    assert_scalar_close(
        summary.surface_area,
        occt_summary.surface_area,
        2.0e-1,
        "Rust revolution surface area vs OCCT oracle",
    )?;
    assert_scalar_close(summary.volume, 0.0, 1.0e-12, "document revolution volume")?;
    assert_scalar_close(
        summary.linear_length,
        wire_occurrence_length,
        1.0e-9,
        "document revolution wire-occurrence edge length",
    )?;
    assert_vec3_close(
        summary.bbox_min,
        occt_summary.bbox_min,
        1.0e-6,
        "revolution bbox min vs OCCT oracle",
    )?;
    assert_vec3_close(
        summary.bbox_max,
        occt_summary.bbox_max,
        1.0e-6,
        "revolution bbox max vs OCCT oracle",
    )?;
    assert_eq!(brep.summary_bbox_source(), SummaryBboxSource::PortedBrep);
    assert_eq!(brep.summary_volume_source(), SummaryVolumeSource::Zero);

    let selected_revolution = document.select_face(
        "revolution",
        FaceSelector::LargestBySurfaceKind(SurfaceKind::Revolution),
    )?;
    let selected_profile = document.select_edge(
        "revolution",
        EdgeSelector::LongestByCurveKind(CurveKind::Ellipse),
    )?;
    let faces = document.faces("revolution")?;
    let edges = document.edges("revolution")?;
    let report = document.report("revolution")?;

    assert_eq!(faces.len(), 1);
    assert_eq!(edges.len(), brep.edges.len());
    assert_eq!(
        document.face_indices_by_surface_kind("revolution", SurfaceKind::Revolution)?,
        vec![0]
    );
    assert_eq!(
        document
            .edge_indices_by_curve_kind("revolution", CurveKind::Ellipse)?
            .len(),
        brep.edges
            .iter()
            .filter(|edge| edge.geometry.kind == CurveKind::Ellipse)
            .count()
    );
    assert_eq!(selected_revolution.geometry.kind, SurfaceKind::Revolution);
    assert_scalar_close(
        selected_revolution.area,
        brep_surface_area,
        1.0e-9,
        "selected revolution face area",
    )?;
    assert_eq!(selected_profile.geometry.kind, CurveKind::Ellipse);
    assert_eq!(report.summary.primary_kind, ShapeKind::Face);
    assert_eq!(report.summary.face_count, 1);
    assert_eq!(report.summary.edge_count, brep.edges.len());
    match document.history() {
        [OperationRecord::AddEllipseEdge { output, params }, OperationRecord::Revolution {
            output: revolution_output,
            input,
            params: revolution_history,
        }] => {
            assert_eq!(output, "profile");
            assert_vec3_close(
                params.origin,
                profile_params.origin,
                0.0,
                "history ellipse origin",
            )?;
            assert_vec3_close(
                params.axis,
                profile_params.axis,
                0.0,
                "history ellipse axis",
            )?;
            assert_vec3_close(
                params.x_direction,
                profile_params.x_direction,
                0.0,
                "history ellipse x_direction",
            )?;
            assert_scalar_close(
                params.major_radius,
                profile_params.major_radius,
                0.0,
                "history ellipse major_radius",
            )?;
            assert_scalar_close(
                params.minor_radius,
                profile_params.minor_radius,
                0.0,
                "history ellipse minor_radius",
            )?;
            assert_eq!(revolution_output, "revolution");
            assert_eq!(input, "profile");
            assert_vec3_close(
                revolution_history.origin,
                revolution_params.origin,
                0.0,
                "history revolution origin",
            )?;
            assert_vec3_close(
                revolution_history.axis,
                revolution_params.axis,
                0.0,
                "history revolution axis",
            )?;
            assert_scalar_close(
                revolution_history.angle_radians,
                revolution_params.angle_radians,
                0.0,
                "history revolution angle",
            )?;
        }
        history => {
            return Err(std::io::Error::other(format!(
                "expected AddEllipseEdge + Revolution history entries, got {history:?}"
            ))
            .into());
        }
    }

    Ok(())
}

#[test]
fn direct_offset_authored_family_row_is_rust_owned() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    require_complete_ownership_row(AuthoredFamily::DirectOffset, "direct offset")?;

    let source_params = BoxParams {
        origin: [-20.0, -10.0, -3.0],
        size: [40.0, 20.0, 6.0],
    };
    let offset_params = OffsetParams {
        offset: 2.25,
        tolerance: 1.0e-4,
    };
    let basis_selector = FaceSelector::BestAlignedPlane {
        normal_hint: [0.0, 0.0, 1.0],
    };

    let mut document = ModelDocument::new()?;
    document.insert_box("basis_box", source_params)?;
    let selected_basis = document.direct_offset_surface_face(
        "offset_top",
        "basis_box",
        basis_selector,
        offset_params,
    )?;

    assert_eq!(selected_basis.geometry.kind, SurfaceKind::Plane);
    assert!(
        selected_basis.sample.normal[2] > 0.9,
        "document selector should choose the top planar basis face"
    );

    let source_shape = document.shape("basis_box")?;
    let offset_shape = document.shape("offset_top")?;
    assert!(
        offset_shape.has_rust_offset_surface_face_metadata(),
        "direct offset root should retain Rust offset metadata"
    );
    assert_eq!(
        offset_shape.rust_multi_face_offset_source_count(),
        None,
        "direct offset face should not rely on generated multi-face offset metadata"
    );

    let context = document.kernel().context();
    let source_face_shapes = context.subshapes(source_shape, ShapeKind::Face)?;
    let basis_face_shape = source_face_shapes
        .get(selected_basis.index)
        .ok_or_else(|| std::io::Error::other("selected basis face index missing"))?;
    let face_shapes = context.subshapes(offset_shape, ShapeKind::Face)?;
    let edge_shapes = context.subshapes(offset_shape, ShapeKind::Edge)?;
    assert_eq!(face_shapes.len(), 1);
    assert_eq!(edge_shapes.len(), 0);
    assert!(
        face_shapes
            .iter()
            .all(|face| face.has_rust_offset_surface_face_metadata()),
        "enumerated direct offset faces should retain Rust offset metadata"
    );

    let offset_face_shape = face_shapes
        .iter()
        .find(|face| {
            context
                .face_geometry(face)
                .is_ok_and(|geometry| geometry.kind == SurfaceKind::Offset)
        })
        .ok_or_else(|| std::io::Error::other("expected an authored direct offset face"))?;

    let ported_topology = context
        .ported_topology(offset_shape)?
        .ok_or_else(|| std::io::Error::other("expected Rust-owned direct offset topology"))?;
    let public_topology = context.topology(offset_shape)?;
    let brep = document.brep("offset_top")?;

    assert_eq!(ported_topology.faces.len(), 1);
    assert_eq!(ported_topology.wires.len(), 0);
    assert_eq!(ported_topology.edges.len(), 0);
    assert_eq!(ported_topology.vertex_positions.len(), 0);
    assert_eq!(ported_topology.wire_edge_indices.len(), 0);
    assert_eq!(ported_topology.face_wire_indices.len(), 0);
    assert_eq!(public_topology.faces.len(), ported_topology.faces.len());
    assert_eq!(public_topology.wires.len(), ported_topology.wires.len());
    assert_eq!(public_topology.edges.len(), ported_topology.edges.len());
    assert_eq!(
        public_topology.vertex_positions.len(),
        ported_topology.vertex_positions.len()
    );
    assert_eq!(brep.faces.len(), ported_topology.faces.len());
    assert_eq!(brep.wires.len(), ported_topology.wires.len());
    assert_eq!(brep.edges.len(), ported_topology.edges.len());
    assert_eq!(brep.vertices.len(), ported_topology.vertex_positions.len());
    assert_eq!(
        brep.faces
            .iter()
            .filter(|face| {
                face.geometry.kind == SurfaceKind::Offset
                    && matches!(face.ported_face_surface, Some(PortedFaceSurface::Offset(_)))
            })
            .count(),
        1
    );
    assert!(brep.edges.is_empty());

    let public_offset_payload = context.face_offset_payload(offset_face_shape)?;
    assert_eq!(public_offset_payload.basis_surface_kind, SurfaceKind::Plane);
    assert_scalar_close(
        public_offset_payload.offset_value,
        offset_params.offset,
        1.0e-12,
        "public direct offset value",
    )?;
    let descriptor = context
        .ported_face_surface_descriptor(offset_face_shape)?
        .ok_or_else(|| std::io::Error::other("expected Rust direct offset descriptor"))?;
    let offset_surface = match descriptor {
        PortedFaceSurface::Offset(surface) => surface,
        other => {
            return Err(std::io::Error::other(format!(
                "expected Rust direct offset descriptor, got {other:?}"
            ))
            .into());
        }
    };
    assert_offset_payload_close(
        public_offset_payload,
        offset_surface.payload,
        1.0e-12,
        "public direct offset payload vs Rust descriptor",
    )?;
    let occt_offset_payload = context.face_offset_payload_occt(offset_face_shape)?;
    assert_offset_payload_close(
        public_offset_payload,
        occt_offset_payload,
        1.0e-12,
        "Rust direct offset payload vs OCCT oracle",
    )?;
    assert_eq!(offset_surface.basis_geometry.kind, SurfaceKind::Plane);
    assert_plane_payload_close(
        context.face_offset_basis_plane_payload(offset_face_shape)?,
        context.face_plane_payload(basis_face_shape)?,
        1.0e-12,
        "public direct offset basis payload mirrors source basis",
    )?;
    match offset_surface.basis {
        PortedOffsetBasisSurface::Analytic(PortedSurface::Plane(payload)) => {
            assert_plane_payload_close(
                context.face_offset_basis_plane_payload(offset_face_shape)?,
                payload,
                1.0e-12,
                "public direct offset basis payload vs Rust descriptor",
            )?;
        }
        other => {
            return Err(std::io::Error::other(format!(
                "expected plane direct offset basis descriptor, got {other:?}"
            ))
            .into());
        }
    }
    assert!(
        context
            .face_offset_basis_cylinder_payload(offset_face_shape)
            .is_err(),
        "ported direct plane offsets should reject mismatched cylinder basis payload queries"
    );
    assert!(
        context.face_plane_payload(offset_face_shape).is_err(),
        "offset faces should not masquerade as direct plane payloads"
    );

    let rust_sample = context
        .ported_face_sample_normalized(offset_face_shape, [0.25, 0.75])?
        .ok_or_else(|| std::io::Error::other("expected Rust direct offset face sample"))?;
    let occt_sample = context.face_sample_normalized_occt(offset_face_shape, [0.25, 0.75])?;
    assert_vec3_close(
        rust_sample.position,
        occt_sample.position,
        1.0e-6,
        "direct offset sample position vs OCCT oracle",
    )?;
    assert_vec3_close(
        rust_sample.normal,
        occt_sample.normal,
        1.0e-6,
        "direct offset sample normal vs OCCT oracle",
    )?;

    let summary = document.summary("offset_top")?;
    let occt_summary = context.describe_shape_occt(offset_shape)?;
    let unique_edge_length = brep.edges.iter().map(|edge| edge.length).sum::<f64>();
    let wire_occurrence_length = if ported_topology.wire_edge_indices.is_empty() {
        unique_edge_length
    } else {
        ported_topology
            .wire_edge_indices
            .iter()
            .filter_map(|&edge_index| brep.edges.get(edge_index))
            .map(|edge| edge.length)
            .sum()
    };
    let brep_surface_area = brep.faces.iter().map(|face| face.area).sum::<f64>();
    assert_scalar_close(
        summary.surface_area,
        brep_surface_area,
        1.0e-9,
        "document direct offset surface area from BRep faces",
    )?;
    assert_scalar_close(
        summary.surface_area,
        occt_summary.surface_area,
        1.0e-6,
        "Rust direct offset surface area vs OCCT oracle",
    )?;
    assert_scalar_close(
        summary.volume,
        0.0,
        1.0e-12,
        "document direct offset volume",
    )?;
    assert_scalar_close(
        summary.linear_length,
        0.0,
        1.0e-12,
        "document direct offset wire-occurrence edge length",
    )?;
    assert_scalar_close(
        wire_occurrence_length,
        0.0,
        1.0e-12,
        "direct offset wire-occurrence edge length",
    )?;
    assert_scalar_close(
        unique_edge_length,
        0.0,
        1.0e-12,
        "unique BRep direct offset edge length",
    )?;
    assert_vec3_close(
        summary.bbox_min,
        occt_summary.bbox_min,
        5.0e-2,
        "direct offset bbox min vs OCCT oracle",
    )?;
    assert_vec3_close(
        summary.bbox_max,
        occt_summary.bbox_max,
        5.0e-2,
        "direct offset bbox max vs OCCT oracle",
    )?;
    assert_eq!(
        brep.summary_bbox_source(),
        SummaryBboxSource::OffsetFaceUnion
    );
    assert!(
        matches!(
            brep.offset_face_bbox_source(),
            Some(OffsetFaceBboxSource::ValidatedMesh | OffsetFaceBboxSource::SummaryFaceBrep)
        ),
        "direct offset bbox should resolve through Rust-owned offset face data, got {:?}",
        brep.offset_face_bbox_source()
    );
    assert_eq!(brep.summary_volume_source(), SummaryVolumeSource::Zero);

    let selected_offset = document.select_face(
        "offset_top",
        FaceSelector::LargestBySurfaceKind(SurfaceKind::Offset),
    )?;
    let document_topology = document.topology("offset_top")?;
    let faces = document.faces("offset_top")?;
    let edges = document.edges("offset_top")?;

    assert_eq!(document_topology.faces.len(), 1);
    assert_eq!(document_topology.edges.len(), 0);
    assert_eq!(
        document_topology.vertex_positions.len(),
        ported_topology.vertex_positions.len()
    );
    assert_eq!(faces.len(), 1);
    assert_eq!(edges.len(), 0);
    assert_eq!(
        document.face_indices_by_surface_kind("offset_top", SurfaceKind::Offset)?,
        vec![0]
    );
    assert_eq!(
        document
            .edge_indices_by_curve_kind("offset_top", CurveKind::Line)?
            .len(),
        0
    );
    assert_eq!(selected_offset.geometry.kind, SurfaceKind::Offset);
    assert!(matches!(
        selected_offset.ported_face_surface,
        Some(PortedFaceSurface::Offset(_))
    ));
    assert_scalar_close(
        selected_offset.area,
        brep_surface_area,
        1.0e-9,
        "selected direct offset face area",
    )?;
    assert!(
        document
            .select_edge(
                "offset_top",
                EdgeSelector::LongestByCurveKind(CurveKind::Line),
            )
            .is_err(),
        "boundary-free direct offset face should reject line edge selectors"
    );
    assert_eq!(summary.primary_kind, ShapeKind::Face);
    assert_eq!(summary.face_count, 1);
    assert_eq!(summary.edge_count, 0);
    match document.history() {
        [OperationRecord::AddBox { output, params }, OperationRecord::DirectOffsetSurfaceFace {
            output: offset_output,
            input,
            selector: FaceSelector::BestAlignedPlane { normal_hint },
            params: offset_history,
        }] => {
            assert_eq!(output, "basis_box");
            assert_vec3_close(
                params.origin,
                source_params.origin,
                0.0,
                "history basis box origin",
            )?;
            assert_vec3_close(
                params.size,
                source_params.size,
                0.0,
                "history basis box size",
            )?;
            assert_eq!(offset_output, "offset_top");
            assert_eq!(input, "basis_box");
            assert_vec3_close(
                *normal_hint,
                [0.0, 0.0, 1.0],
                0.0,
                "history direct offset selector normal",
            )?;
            assert_scalar_close(
                offset_history.offset,
                offset_params.offset,
                0.0,
                "history direct offset value",
            )?;
            assert_scalar_close(
                offset_history.tolerance,
                offset_params.tolerance,
                0.0,
                "history direct offset tolerance",
            )?;
        }
        history => {
            return Err(std::io::Error::other(format!(
                "expected AddBox + DirectOffsetSurfaceFace history entries, got {history:?}"
            ))
            .into());
        }
    }

    Ok(())
}

#[test]
fn generated_offset_authored_family_row_is_rust_owned() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    require_complete_ownership_row(AuthoredFamily::GeneratedOffset, "generated offset")?;

    let profile_params = EllipseEdgeParams {
        origin: [30.0, 0.0, 0.0],
        axis: [0.0, 1.0, 0.0],
        x_direction: [1.0, 0.0, 0.0],
        major_radius: 10.0,
        minor_radius: 6.0,
    };
    let prism_params = PrismParams {
        direction: [0.0, 24.0, 0.0],
    };
    let revolution_params = RevolutionParams {
        origin: [0.0, 0.0, 0.0],
        axis: [0.0, 0.0, 1.0],
        angle_radians: PI,
    };
    let offset_params = OffsetParams {
        offset: 2.5,
        tolerance: 1.0e-4,
    };

    let kernel = ModelKernel::new()?;
    let context = kernel.context();
    let profile = kernel.make_ellipse_edge(profile_params)?;
    let prism = kernel.make_prism(&profile, prism_params)?;
    let extrusion_face = context
        .subshapes(&prism, ShapeKind::Face)?
        .into_iter()
        .find_map(|face_shape| match context.face_geometry(&face_shape) {
            Ok(geometry) if geometry.kind == SurfaceKind::Extrusion => Some(Ok(face_shape)),
            Ok(_) => None,
            Err(error) => Some(Err(error)),
        })
        .transpose()?
        .ok_or_else(|| std::io::Error::other("expected prism extrusion face"))?;
    let revolved = kernel.make_revolution(&extrusion_face, revolution_params)?;
    let offset_shape = kernel.make_offset(&revolved, offset_params)?;

    assert_eq!(
        offset_shape.rust_multi_face_offset_source_count(),
        Some(4),
        "generated offset root should retain the Rust source-face metadata inventory"
    );
    assert!(
        !offset_shape.has_rust_offset_surface_face_metadata(),
        "generated multi-face offset roots should not masquerade as direct offset faces"
    );

    let face_shapes = context.subshapes(&offset_shape, ShapeKind::Face)?;
    let shell_shapes = context.subshapes(&offset_shape, ShapeKind::Shell)?;
    let offset_face_shapes = face_shapes
        .iter()
        .filter_map(|face_shape| match context.face_geometry(face_shape) {
            Ok(geometry) if geometry.kind == SurfaceKind::Offset => Some(Ok(face_shape)),
            Ok(_) => None,
            Err(error) => Some(Err(error)),
        })
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(
        offset_face_shapes.len(),
        4,
        "generated offset should expose the mapped OFFSET_SURFACE faces"
    );
    assert!(
        offset_face_shapes
            .iter()
            .all(|face| face.has_rust_offset_surface_face_metadata()),
        "generated offset faces should carry resolved Rust offset metadata"
    );
    for (shell_index, shell_shape) in shell_shapes.iter().enumerate() {
        assert_eq!(
            shell_shape.rust_multi_face_offset_source_count(),
            Some(4),
            "generated offset shell {shell_index} should retain the multi-face metadata inventory"
        );
    }

    let ported_topology = context
        .ported_topology(&offset_shape)?
        .ok_or_else(|| std::io::Error::other("expected Rust-owned generated offset topology"))?;
    let public_topology = context.topology(&offset_shape)?;

    assert_eq!(public_topology.faces.len(), ported_topology.faces.len());
    assert_eq!(public_topology.wires.len(), ported_topology.wires.len());
    assert_eq!(public_topology.edges.len(), ported_topology.edges.len());
    assert_eq!(
        public_topology.vertex_positions.len(),
        ported_topology.vertex_positions.len()
    );

    let mut saw_revolution_basis = false;
    let mut saw_extrusion_basis = false;
    for (face_index, face_shape) in offset_face_shapes.iter().enumerate() {
        let public_geometry = context.face_geometry(face_shape)?;
        let ported_geometry = context.ported_face_geometry(face_shape)?.ok_or_else(|| {
            std::io::Error::other(format!(
                "generated offset face {face_index} should expose Rust geometry"
            ))
        })?;
        assert_eq!(public_geometry.kind, SurfaceKind::Offset);
        assert_face_geometry_close(
            public_geometry,
            ported_geometry,
            1.0e-12,
            &format!("generated offset face {face_index} public vs Rust geometry"),
        )?;

        let descriptor = match context.ported_face_surface_descriptor(face_shape)? {
            Some(PortedFaceSurface::Offset(surface)) => surface,
            other => {
                return Err(std::io::Error::other(format!(
                    "generated offset face {face_index} did not expose a Rust offset descriptor: {other:?}"
                ))
                .into());
            }
        };
        let public_payload = context.face_offset_payload(face_shape)?;
        assert_offset_payload_close(
            public_payload,
            descriptor.payload,
            1.0e-12,
            &format!("generated offset face {face_index} public payload vs Rust descriptor"),
        )?;
        let occt_payload = context.face_offset_payload_occt(face_shape)?;
        assert_eq!(
            public_payload.basis_surface_kind, occt_payload.basis_surface_kind,
            "generated offset face {face_index} basis kind drifted from OCCT oracle"
        );
        assert_scalar_close(
            public_payload.offset_value.abs(),
            occt_payload.offset_value.abs(),
            1.0e-9,
            &format!("generated offset face {face_index} offset magnitude vs OCCT oracle"),
        )?;
        assert_scalar_close(
            public_payload.offset_value.abs(),
            offset_params.offset,
            1.0e-9,
            &format!("generated offset face {face_index} offset magnitude"),
        )?;
        assert_eq!(
            context.face_offset_basis_geometry(face_shape)?.kind,
            public_payload.basis_surface_kind,
            "generated offset face {face_index} public basis geometry kind drifted"
        );

        match descriptor.basis {
            PortedOffsetBasisSurface::Swept(PortedSweptSurface::Revolution {
                payload,
                basis_curve,
                basis_geometry,
            }) => {
                saw_revolution_basis = true;
                assert!(matches!(basis_curve, PortedCurve::Ellipse(_)));
                assert_eq!(basis_geometry.kind, CurveKind::Ellipse);
                assert_revolution_payload_close(
                    context.face_offset_basis_revolution_payload(face_shape)?,
                    payload,
                    1.0e-12,
                    &format!("generated offset face {face_index} revolution basis payload"),
                )?;
                assert_revolution_payload_close(
                    context.face_offset_basis_revolution_payload(face_shape)?,
                    context.face_offset_basis_revolution_payload_occt(face_shape)?,
                    1.0e-12,
                    &format!("generated offset face {face_index} revolution basis vs OCCT oracle"),
                )?;
                assert!(
                    context
                        .face_offset_basis_extrusion_payload(face_shape)
                        .is_err(),
                    "generated revolution-offset faces should reject extrusion basis payloads"
                );
            }
            PortedOffsetBasisSurface::Swept(PortedSweptSurface::Extrusion {
                payload,
                basis_curve,
                basis_geometry,
            }) => {
                saw_extrusion_basis = true;
                assert!(matches!(basis_curve, PortedCurve::Ellipse(_)));
                assert_eq!(basis_geometry.kind, CurveKind::Ellipse);
                assert_extrusion_payload_close(
                    context.face_offset_basis_extrusion_payload(face_shape)?,
                    payload,
                    1.0e-12,
                    &format!("generated offset face {face_index} extrusion basis payload"),
                )?;
                assert_extrusion_payload_close(
                    context.face_offset_basis_extrusion_payload(face_shape)?,
                    context.face_offset_basis_extrusion_payload_occt(face_shape)?,
                    1.0e-12,
                    &format!("generated offset face {face_index} extrusion basis vs OCCT oracle"),
                )?;
                assert!(
                    context
                        .face_offset_basis_revolution_payload(face_shape)
                        .is_err(),
                    "generated extrusion-offset faces should reject revolution basis payloads"
                );
            }
            other => {
                return Err(std::io::Error::other(format!(
                    "generated offset face {face_index} should keep a swept basis descriptor, got {other:?}"
                ))
                .into());
            }
        }
        assert_eq!(
            context.face_offset_basis_curve_geometry(face_shape)?.kind,
            CurveKind::Ellipse,
            "generated offset face {face_index} swept basis curve should remain ellipse-backed"
        );
        assert!(
            context
                .face_offset_basis_curve_ellipse_payload(face_shape)
                .is_ok(),
            "generated offset face {face_index} should expose the swept ellipse basis payload"
        );
        assert!(
            context.face_revolution_payload(face_shape).is_err(),
            "offset faces should reject top-level revolution payload requests"
        );
        assert!(
            context.face_extrusion_payload(face_shape).is_err(),
            "offset faces should reject top-level extrusion payload requests"
        );

        let rust_sample = context
            .ported_face_sample_normalized(face_shape, [0.37, 0.61])?
            .ok_or_else(|| {
                std::io::Error::other(format!(
                    "expected Rust sample for generated offset face {face_index}"
                ))
            })?;
        let public_sample = context.face_sample_normalized(face_shape, [0.37, 0.61])?;
        let occt_sample = context.face_sample_normalized_occt(face_shape, [0.37, 0.61])?;
        assert_vec3_close(
            public_sample.position,
            rust_sample.position,
            1.0e-12,
            &format!("generated offset face {face_index} public sample position"),
        )?;
        assert_vec3_close(
            public_sample.normal,
            rust_sample.normal,
            1.0e-12,
            &format!("generated offset face {face_index} public sample normal"),
        )?;
        assert_vec3_close(
            rust_sample.position,
            occt_sample.position,
            1.0e-6,
            &format!("generated offset face {face_index} sample position vs OCCT oracle"),
        )?;
        assert_vec3_close(
            rust_sample.normal,
            occt_sample.normal,
            1.0e-6,
            &format!("generated offset face {face_index} sample normal vs OCCT oracle"),
        )?;

        let face_brep = kernel.brep(face_shape)?;
        assert_eq!(
            face_brep.faces.len(),
            1,
            "generated offset face {face_index} BRep should materialize the selected face only"
        );
        let brep_face = &face_brep.faces[0];
        assert_eq!(brep_face.geometry.kind, SurfaceKind::Offset);
        assert!(matches!(
            brep_face.ported_face_surface,
            Some(PortedFaceSurface::Offset(_))
        ));
        assert_eq!(face_brep.topology.faces.len(), 1);
        assert_eq!(face_brep.wires.len(), face_brep.topology.wires.len());
        assert_eq!(face_brep.edges.len(), face_brep.topology.edges.len());
        assert_eq!(
            face_brep.vertices.len(),
            face_brep.topology.vertex_positions.len()
        );

        let face_summary = kernel.summarize(face_shape)?;
        let face_wire_length = if face_brep.topology.wire_edge_indices.is_empty() {
            face_brep.edges.iter().map(|edge| edge.length).sum()
        } else {
            face_brep
                .topology
                .wire_edge_indices
                .iter()
                .filter_map(|&edge_index| face_brep.edges.get(edge_index))
                .map(|edge| edge.length)
                .sum()
        };
        assert_eq!(face_summary.primary_kind, ShapeKind::Face);
        assert_scalar_close(
            face_summary.surface_area,
            brep_face.area,
            1.0e-9,
            &format!("generated offset face {face_index} summary surface area"),
        )?;
        assert_scalar_close(
            face_summary.linear_length,
            face_wire_length,
            1.0e-9,
            &format!("generated offset face {face_index} summary wire length"),
        )?;
        assert_vec3_close(
            face_summary.bbox_min,
            face_brep.summary.bbox_min,
            1.0e-12,
            &format!("generated offset face {face_index} public vs BRep bbox min"),
        )?;
        assert_vec3_close(
            face_summary.bbox_max,
            face_brep.summary.bbox_max,
            1.0e-12,
            &format!("generated offset face {face_index} public vs BRep bbox max"),
        )?;
        assert!(
            matches!(
                face_brep.summary_bbox_source(),
                SummaryBboxSource::PortedBrep | SummaryBboxSource::OffsetFaceUnion
            ),
            "generated offset face {face_index} summary bbox should stay Rust-owned, got {:?}",
            face_brep.summary_bbox_source()
        );
    }
    assert!(
        saw_revolution_basis && saw_extrusion_basis,
        "generated offset row should cover both revolution and extrusion swept bases"
    );

    let mut document = ModelDocument::new()?;
    document.insert_ellipse_edge("doc_profile", profile_params)?;
    document.revolution("doc_revolution", "doc_profile", revolution_params)?;
    document.offset("doc_generated_offset", "doc_revolution", offset_params)?;

    let selected_offset = document.select_face(
        "doc_generated_offset",
        FaceSelector::LargestBySurfaceKind(SurfaceKind::Offset),
    )?;
    let document_topology = document.topology("doc_generated_offset")?;
    let faces = document.faces("doc_generated_offset")?;
    let edges = document.edges("doc_generated_offset")?;
    let report = document.report("doc_generated_offset")?;
    let offset_indices =
        document.face_indices_by_surface_kind("doc_generated_offset", SurfaceKind::Offset)?;

    assert!(!document_topology.faces.is_empty());
    assert_eq!(faces.len(), document_topology.faces.len());
    assert_eq!(edges.len(), document_topology.edges.len());
    assert_eq!(
        offset_indices.len(),
        faces
            .iter()
            .filter(|face| face.geometry.kind == SurfaceKind::Offset)
            .count()
    );
    assert_eq!(selected_offset.geometry.kind, SurfaceKind::Offset);
    assert!(matches!(
        selected_offset.ported_face_surface,
        Some(PortedFaceSurface::Offset(_))
    ));
    assert_eq!(report.summary.face_count, faces.len());
    assert_eq!(report.summary.edge_count, edges.len());
    match document.history() {
        [OperationRecord::AddEllipseEdge { output, params }, OperationRecord::Revolution {
            output: revolution_output,
            input: revolution_input,
            params: revolution_history,
        }, OperationRecord::Offset {
            output: offset_output,
            input: offset_input,
            params: offset_history,
        }] => {
            assert_eq!(output, "doc_profile");
            assert_vec3_close(
                params.origin,
                profile_params.origin,
                0.0,
                "history ellipse origin",
            )?;
            assert_vec3_close(
                params.axis,
                profile_params.axis,
                0.0,
                "history ellipse axis",
            )?;
            assert_vec3_close(
                params.x_direction,
                profile_params.x_direction,
                0.0,
                "history ellipse x_direction",
            )?;
            assert_scalar_close(
                params.major_radius,
                profile_params.major_radius,
                0.0,
                "history ellipse major_radius",
            )?;
            assert_scalar_close(
                params.minor_radius,
                profile_params.minor_radius,
                0.0,
                "history ellipse minor_radius",
            )?;
            assert_eq!(revolution_output, "doc_revolution");
            assert_eq!(revolution_input, "doc_profile");
            assert_vec3_close(
                revolution_history.origin,
                revolution_params.origin,
                0.0,
                "history revolution origin",
            )?;
            assert_vec3_close(
                revolution_history.axis,
                revolution_params.axis,
                0.0,
                "history revolution axis",
            )?;
            assert_scalar_close(
                revolution_history.angle_radians,
                revolution_params.angle_radians,
                0.0,
                "history revolution angle",
            )?;
            assert_eq!(offset_output, "doc_generated_offset");
            assert_eq!(offset_input, "doc_revolution");
            assert_scalar_close(
                offset_history.offset,
                offset_params.offset,
                0.0,
                "history generated offset value",
            )?;
            assert_scalar_close(
                offset_history.tolerance,
                offset_params.tolerance,
                0.0,
                "history generated offset tolerance",
            )?;
        }
        history => {
            return Err(std::io::Error::other(format!(
                "expected AddEllipseEdge + Revolution + Offset history entries, got {history:?}"
            ))
            .into());
        }
    }

    Ok(())
}

#[derive(Clone, Copy, Debug)]
struct AssemblySummaryExpectation {
    root_kind: ShapeKind,
    primary_kind: ShapeKind,
    compound_count: usize,
    compsolid_count: usize,
    solid_count: usize,
    shell_count: usize,
    face_count: usize,
    wire_count: usize,
    edge_count: usize,
    vertex_count: usize,
    summary_bbox_source: SummaryBboxSource,
    summary_volume_source: SummaryVolumeSource,
}

fn assert_assembly_metadata(
    shape: &Shape,
    assembly_kind: ShapeKind,
    child_root_kinds: &[ShapeKind],
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    assert!(
        shape.has_rust_assembly_metadata(),
        "{label} should carry Rust assembly construction metadata"
    );
    assert_eq!(
        shape.rust_assembly_kind(),
        Some(assembly_kind),
        "{label} Rust assembly kind"
    );
    assert_eq!(
        shape.rust_assembly_source_count(),
        Some(child_root_kinds.len()),
        "{label} Rust assembly source count"
    );
    let observed_child_kinds = shape.rust_assembly_child_root_kinds().ok_or_else(|| {
        std::io::Error::other(format!(
            "{label} missing Rust assembly child root kind inventory"
        ))
    })?;
    assert_eq!(
        observed_child_kinds, child_root_kinds,
        "{label} Rust assembly child root kinds"
    );
    Ok(())
}

fn assert_assembly_analytic_source_counts(
    shape: &Shape,
    expected_counts: &[Option<usize>],
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let observed_counts = shape
        .rust_assembly_child_analytic_source_counts()
        .ok_or_else(|| {
            std::io::Error::other(format!(
                "{label} missing Rust assembly analytic source inventory"
            ))
        })?;
    assert_eq!(
        observed_counts, expected_counts,
        "{label} Rust assembly analytic source counts"
    );
    Ok(())
}

fn assert_simple_assembly_snapshot(
    kernel: &ModelKernel,
    shape: &Shape,
    expected: AssemblySummaryExpectation,
    expected_face_kind_counts: &[(SurfaceKind, usize)],
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = kernel.context();
    let public_topology = kernel.topology(shape)?;
    let ported_topology = context
        .ported_topology(shape)?
        .ok_or_else(|| std::io::Error::other(format!("{label} missing Rust topology")))?;
    let brep = kernel.brep(shape)?;
    let summary = kernel.summarize(shape)?;
    let report = if expected.face_count > 0 {
        Some(kernel.inspect(shape)?)
    } else {
        None
    };
    let occt_summary = context.describe_shape_occt(shape)?;

    assert_eq!(summary.root_kind, expected.root_kind, "{label} root kind");
    assert_eq!(
        summary.primary_kind, expected.primary_kind,
        "{label} primary kind"
    );
    assert_eq!(
        summary.compound_count, expected.compound_count,
        "{label} compound count"
    );
    assert_eq!(
        summary.compsolid_count, expected.compsolid_count,
        "{label} compsolid count"
    );
    assert_eq!(
        summary.solid_count, expected.solid_count,
        "{label} solid count"
    );
    assert_eq!(
        summary.shell_count, expected.shell_count,
        "{label} shell count"
    );
    assert_eq!(
        summary.face_count, expected.face_count,
        "{label} face count"
    );
    assert_eq!(
        summary.wire_count, expected.wire_count,
        "{label} wire count"
    );
    assert_eq!(
        summary.edge_count, expected.edge_count,
        "{label} edge count"
    );
    assert_eq!(
        summary.vertex_count, expected.vertex_count,
        "{label} vertex count"
    );
    if let Some(report) = report {
        assert_eq!(report.summary.face_count, summary.face_count);
        assert_eq!(report.summary.wire_count, summary.wire_count);
        assert_eq!(report.summary.edge_count, summary.edge_count);
        assert_eq!(report.summary.vertex_count, summary.vertex_count);
        assert_eq!(report.topology.faces.len(), public_topology.faces.len());
        assert_eq!(report.topology.wires.len(), public_topology.wires.len());
        assert_eq!(report.topology.edges.len(), public_topology.edges.len());
        assert_eq!(
            report.topology.vertex_positions.len(),
            public_topology.vertex_positions.len()
        );
    }

    assert_eq!(public_topology.faces.len(), ported_topology.faces.len());
    assert_eq!(public_topology.wires.len(), ported_topology.wires.len());
    assert_eq!(public_topology.edges.len(), ported_topology.edges.len());
    assert_eq!(
        public_topology.vertex_positions.len(),
        ported_topology.vertex_positions.len()
    );
    assert_eq!(brep.topology.faces.len(), public_topology.faces.len());
    assert_eq!(brep.topology.wires.len(), public_topology.wires.len());
    assert_eq!(brep.topology.edges.len(), public_topology.edges.len());
    assert_eq!(
        brep.topology.vertex_positions.len(),
        public_topology.vertex_positions.len()
    );
    assert_eq!(brep.faces.len(), expected.face_count);
    assert_eq!(brep.wires.len(), expected.wire_count);
    assert_eq!(brep.edges.len(), expected.edge_count);
    assert_eq!(
        brep.topology.vertex_positions.len(),
        expected.vertex_count,
        "{label} BRep vertex count"
    );
    assert_eq!(
        context.subshape_count(shape, ShapeKind::Face)?,
        expected.face_count
    );
    assert_eq!(
        context.subshape_count(shape, ShapeKind::Wire)?,
        expected.wire_count
    );
    assert_eq!(
        context.subshape_count(shape, ShapeKind::Edge)?,
        expected.edge_count
    );
    assert_eq!(
        context.subshape_count(shape, ShapeKind::Vertex)?,
        expected.vertex_count
    );
    assert_eq!(
        context.subshape_count(shape, ShapeKind::Shell)?,
        expected.shell_count
    );
    assert_eq!(
        context.subshape_count(shape, ShapeKind::Solid)?,
        expected.solid_count
    );

    let face_area_sum = brep.faces.iter().map(|face| face.area).sum::<f64>();
    let wire_occurrence_length = if brep.topology.wire_edge_indices.is_empty() {
        brep.edges.iter().map(|edge| edge.length).sum()
    } else {
        brep.topology
            .wire_edge_indices
            .iter()
            .filter_map(|&edge_index| brep.edges.get(edge_index))
            .map(|edge| edge.length)
            .sum()
    };
    assert_scalar_close(
        summary.surface_area,
        face_area_sum,
        1.0e-9,
        &format!("{label} summary surface area from BRep faces"),
    )?;
    assert_scalar_close(
        summary.linear_length,
        wire_occurrence_length,
        1.0e-9,
        &format!("{label} summary wire-occurrence edge length"),
    )?;
    assert_scalar_close(
        summary.surface_area,
        occt_summary.surface_area,
        1.0e-7,
        &format!("{label} surface area vs OCCT oracle"),
    )?;
    assert_scalar_close(
        summary.volume,
        occt_summary.volume,
        1.0e-7,
        &format!("{label} volume vs OCCT oracle"),
    )?;
    assert_vec3_close(
        summary.bbox_min,
        occt_summary.bbox_min,
        1.0e-6,
        &format!("{label} bbox min vs OCCT oracle"),
    )?;
    assert_vec3_close(
        summary.bbox_max,
        occt_summary.bbox_max,
        1.0e-6,
        &format!("{label} bbox max vs OCCT oracle"),
    )?;
    assert_eq!(
        brep.summary_bbox_source(),
        expected.summary_bbox_source,
        "{label} summary bbox source"
    );
    assert_eq!(
        brep.summary_volume_source(),
        expected.summary_volume_source,
        "{label} summary volume source"
    );

    let face_shapes = context.subshapes(shape, ShapeKind::Face)?;
    assert_eq!(face_shapes.len(), expected.face_count);
    for &(kind, count) in expected_face_kind_counts {
        assert_eq!(
            face_shapes
                .iter()
                .filter(|face_shape| context
                    .face_geometry(face_shape)
                    .is_ok_and(|geometry| geometry.kind == kind))
                .count(),
            count,
            "{label} {kind:?} face count"
        );
        assert_eq!(
            brep.faces
                .iter()
                .filter(|face| face.geometry.kind == kind
                    && ported_face_surface_matches_kind(face.ported_face_surface, kind))
                .count(),
            count,
            "{label} {kind:?} BRep ported face count"
        );
    }
    for (face_index, face_shape) in face_shapes.iter().enumerate() {
        let public_geometry = context.face_geometry(face_shape)?;
        let ported_geometry = context.ported_face_geometry(face_shape)?.ok_or_else(|| {
            std::io::Error::other(format!(
                "{label} face {face_index} missing Rust face geometry"
            ))
        })?;
        assert_face_geometry_close(
            public_geometry,
            ported_geometry,
            1.0e-12,
            &format!("{label} face {face_index} public vs Rust geometry"),
        )?;
        assert_supported_face_payload_query(
            context,
            face_shape,
            public_geometry.kind,
            &format!("{label} face {face_index}"),
        )?;
    }

    Ok(())
}

fn ported_face_surface_matches_kind(surface: Option<PortedFaceSurface>, kind: SurfaceKind) -> bool {
    matches!(
        (surface, kind),
        (
            Some(PortedFaceSurface::Analytic(PortedSurface::Plane(_))),
            SurfaceKind::Plane
        ) | (
            Some(PortedFaceSurface::Analytic(PortedSurface::Cylinder(_))),
            SurfaceKind::Cylinder
        ) | (
            Some(PortedFaceSurface::Analytic(PortedSurface::Cone(_))),
            SurfaceKind::Cone
        ) | (
            Some(PortedFaceSurface::Analytic(PortedSurface::Sphere(_))),
            SurfaceKind::Sphere
        ) | (
            Some(PortedFaceSurface::Analytic(PortedSurface::Torus(_))),
            SurfaceKind::Torus
        )
    )
}

fn assert_supported_face_payload_query(
    context: &lean_occt::Context,
    face_shape: &Shape,
    kind: SurfaceKind,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let descriptor = context
        .ported_face_surface_descriptor(face_shape)?
        .ok_or_else(|| std::io::Error::other(format!("{label} missing Rust face descriptor")))?;

    match kind {
        SurfaceKind::Plane => {
            let public_payload = context.face_plane_payload(face_shape)?;
            let descriptor_payload = match descriptor {
                PortedFaceSurface::Analytic(PortedSurface::Plane(payload)) => payload,
                other => {
                    return Err(std::io::Error::other(format!(
                        "{label} expected Rust plane descriptor, got {other:?}"
                    ))
                    .into());
                }
            };
            assert_plane_payload_close(
                public_payload,
                descriptor_payload,
                1.0e-12,
                &format!("{label} public plane payload vs Rust descriptor"),
            )?;
            assert_plane_payload_close(
                public_payload,
                context.face_plane_payload_occt(face_shape)?,
                1.0e-6,
                &format!("{label} plane payload vs OCCT oracle"),
            )?;
        }
        SurfaceKind::Cylinder => {
            let public_payload = context.face_cylinder_payload(face_shape)?;
            let descriptor_payload = match descriptor {
                PortedFaceSurface::Analytic(PortedSurface::Cylinder(payload)) => payload,
                other => {
                    return Err(std::io::Error::other(format!(
                        "{label} expected Rust cylinder descriptor, got {other:?}"
                    ))
                    .into());
                }
            };
            assert_cylinder_payload_close(
                public_payload,
                descriptor_payload,
                1.0e-12,
                &format!("{label} public cylinder payload vs Rust descriptor"),
            )?;
            assert_cylinder_payload_close(
                public_payload,
                context.face_cylinder_payload_occt(face_shape)?,
                1.0e-6,
                &format!("{label} cylinder payload vs OCCT oracle"),
            )?;
        }
        SurfaceKind::Cone => {
            let public_payload = context.face_cone_payload(face_shape)?;
            let descriptor_payload = match descriptor {
                PortedFaceSurface::Analytic(PortedSurface::Cone(payload)) => payload,
                other => {
                    return Err(std::io::Error::other(format!(
                        "{label} expected Rust cone descriptor, got {other:?}"
                    ))
                    .into());
                }
            };
            assert_cone_payload_close(
                public_payload,
                descriptor_payload,
                1.0e-12,
                &format!("{label} public cone payload vs Rust descriptor"),
            )?;
            assert_cone_payload_close(
                public_payload,
                context.face_cone_payload_occt(face_shape)?,
                1.0e-6,
                &format!("{label} cone payload vs OCCT oracle"),
            )?;
        }
        SurfaceKind::Sphere => {
            let public_payload = context.face_sphere_payload(face_shape)?;
            let descriptor_payload = match descriptor {
                PortedFaceSurface::Analytic(PortedSurface::Sphere(payload)) => payload,
                other => {
                    return Err(std::io::Error::other(format!(
                        "{label} expected Rust sphere descriptor, got {other:?}"
                    ))
                    .into());
                }
            };
            assert_sphere_payload_close(
                public_payload,
                descriptor_payload,
                1.0e-12,
                &format!("{label} public sphere payload vs Rust descriptor"),
            )?;
            assert_sphere_payload_close(
                public_payload,
                context.face_sphere_payload_occt(face_shape)?,
                1.0e-6,
                &format!("{label} sphere payload vs OCCT oracle"),
            )?;
        }
        SurfaceKind::Torus => {
            let public_payload = context.face_torus_payload(face_shape)?;
            let descriptor_payload = match descriptor {
                PortedFaceSurface::Analytic(PortedSurface::Torus(payload)) => payload,
                other => {
                    return Err(std::io::Error::other(format!(
                        "{label} expected Rust torus descriptor, got {other:?}"
                    ))
                    .into());
                }
            };
            assert_torus_payload_close(
                public_payload,
                descriptor_payload,
                1.0e-12,
                &format!("{label} public torus payload vs Rust descriptor"),
            )?;
            assert_torus_payload_close(
                public_payload,
                context.face_torus_payload_occt(face_shape)?,
                1.0e-6,
                &format!("{label} torus payload vs OCCT oracle"),
            )?;
        }
        other => {
            return Err(std::io::Error::other(format!(
                "{label} unsupported assembly face kind {other:?}"
            ))
            .into());
        }
    }

    Ok(())
}

fn assert_line_edge_queries(
    kernel: &ModelKernel,
    shape: &Shape,
    expected_edge_count: usize,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = kernel.context();
    let edge_shapes = context.subshapes(shape, ShapeKind::Edge)?;
    assert_eq!(
        edge_shapes.len(),
        expected_edge_count,
        "{label} edge subshape count"
    );

    for (edge_index, edge_shape) in edge_shapes.iter().enumerate() {
        let public_geometry = context.edge_geometry(edge_shape)?;
        let oracle_geometry = context.edge_geometry_occt(edge_shape)?;
        assert_eq!(
            public_geometry.kind,
            CurveKind::Line,
            "{label} edge {edge_index} should remain linear"
        );
        assert_edge_geometry_close(
            public_geometry,
            oracle_geometry,
            1.0e-8,
            &format!("{label} edge {edge_index} public geometry vs OCCT oracle"),
        )?;
        assert_line_payload_close(
            context.edge_line_payload(edge_shape)?,
            context.edge_line_payload_occt(edge_shape)?,
            1.0e-6,
            &format!("{label} edge {edge_index} line payload vs OCCT oracle"),
        )?;
    }

    Ok(())
}

fn assert_supported_edge_queries(
    kernel: &ModelKernel,
    shape: &Shape,
    expected_curve_kind_counts: &[(CurveKind, usize)],
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = kernel.context();
    let edge_shapes = context.subshapes(shape, ShapeKind::Edge)?;
    let brep = kernel.brep(shape)?;
    assert_eq!(
        edge_shapes.len(),
        expected_curve_kind_counts
            .iter()
            .map(|(_, count)| *count)
            .sum::<usize>(),
        "{label} edge subshape count"
    );

    for &(kind, count) in expected_curve_kind_counts {
        assert_eq!(
            edge_shapes
                .iter()
                .filter(|edge_shape| context
                    .edge_geometry(edge_shape)
                    .is_ok_and(|geometry| geometry.kind == kind))
                .count(),
            count,
            "{label} {kind:?} edge count"
        );
        assert_eq!(
            brep.edges
                .iter()
                .filter(|edge| {
                    edge.geometry.kind == kind
                        && match kind {
                            CurveKind::Line | CurveKind::Circle | CurveKind::Ellipse => {
                                ported_curve_matches_kind(edge.ported_curve, kind)
                            }
                            CurveKind::Other => edge.ported_curve.is_none(),
                            _ => false,
                        }
                })
                .count(),
            count,
            "{label} {kind:?} BRep ported edge count"
        );
    }

    for (edge_index, edge_shape) in edge_shapes.iter().enumerate() {
        let public_geometry = context.edge_geometry(edge_shape)?;
        let ported_geometry = context.ported_edge_geometry(edge_shape)?;
        if public_geometry.kind == CurveKind::Other {
            assert!(
                ported_geometry.is_none(),
                "{label} edge {edge_index} unsupported Other curve should not masquerade as Rust-owned geometry"
            );
        } else {
            let ported_geometry = ported_geometry.ok_or_else(|| {
                std::io::Error::other(format!(
                    "{label} edge {edge_index} missing Rust edge geometry"
                ))
            })?;
            assert_edge_geometry_close(
                public_geometry,
                ported_geometry,
                1.0e-8,
                &format!("{label} edge {edge_index} public vs Rust geometry"),
            )?;
        }
        assert_edge_geometry_close_or_periodic_equivalent(
            public_geometry,
            context.edge_geometry_occt(edge_shape)?,
            1.0e-8,
            &format!("{label} edge {edge_index} public geometry vs OCCT oracle"),
        )?;
        match public_geometry.kind {
            CurveKind::Line => {
                assert_line_payload_close(
                    context.edge_line_payload(edge_shape)?,
                    context.edge_line_payload_occt(edge_shape)?,
                    1.0e-6,
                    &format!("{label} edge {edge_index} line payload vs OCCT oracle"),
                )?;
            }
            CurveKind::Circle => {
                assert_circle_payload_same_carrier(
                    context.edge_circle_payload(edge_shape)?,
                    context.edge_circle_payload_occt(edge_shape)?,
                    1.0e-6,
                    &format!("{label} edge {edge_index} circle carrier vs OCCT oracle"),
                )?;
            }
            CurveKind::Ellipse => {
                assert_ellipse_payload_close(
                    context.edge_ellipse_payload(edge_shape)?,
                    context.edge_ellipse_payload_occt(edge_shape)?,
                    1.0e-6,
                    &format!("{label} edge {edge_index} ellipse payload vs OCCT oracle"),
                )?;
            }
            CurveKind::Other => {
                assert!(
                    context.ported_edge_curve(edge_shape)?.is_none(),
                    "{label} edge {edge_index} unsupported Other curve should not expose a Rust analytic payload"
                );
                assert!(
                    context.edge_line_payload(edge_shape).is_err(),
                    "{label} edge {edge_index} unsupported Other curve should reject line payload queries"
                );
                assert!(
                    context.edge_circle_payload(edge_shape).is_err(),
                    "{label} edge {edge_index} unsupported Other curve should reject circle payload queries"
                );
                assert!(
                    context.edge_ellipse_payload(edge_shape).is_err(),
                    "{label} edge {edge_index} unsupported Other curve should reject ellipse payload queries"
                );
            }
            other => {
                return Err(std::io::Error::other(format!(
                    "{label} edge {edge_index} unsupported assembly curve kind {other:?}"
                ))
                .into());
            }
        }
    }

    Ok(())
}

fn ported_curve_matches_kind(curve: Option<PortedCurve>, kind: CurveKind) -> bool {
    matches!(
        (curve, kind),
        (Some(PortedCurve::Line(_)), CurveKind::Line)
            | (Some(PortedCurve::Circle(_)), CurveKind::Circle)
            | (Some(PortedCurve::Ellipse(_)), CurveKind::Ellipse)
    )
}

fn assert_vertex_queries(
    kernel: &ModelKernel,
    shape: &Shape,
    expected_vertex_count: usize,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = kernel.context();
    let vertex_shapes = context.subshapes(shape, ShapeKind::Vertex)?;
    assert_eq!(
        vertex_shapes.len(),
        expected_vertex_count,
        "{label} vertex subshape count"
    );
    for (vertex_index, vertex_shape) in vertex_shapes.iter().enumerate() {
        assert_vec3_close(
            context.vertex_point(vertex_shape)?,
            context.vertex_point_occt(vertex_shape)?,
            1.0e-9,
            &format!("{label} vertex {vertex_index} point vs OCCT oracle"),
        )?;
    }

    Ok(())
}

#[test]
fn simple_shell_solid_assembly_authored_family_row_is_rust_owned(
) -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    require_complete_ownership_row(
        AuthoredFamily::SimpleShellSolidAssembly,
        "simple shell/solid assembly",
    )?;

    let kernel = ModelKernel::new()?;
    let left_box_params = BoxParams {
        origin: [-18.0, -6.0, -5.0],
        size: [10.0, 12.0, 8.0],
    };
    let right_box_params = BoxParams {
        origin: [8.0, -6.0, -5.0],
        size: [10.0, 12.0, 8.0],
    };
    let left_box = kernel.make_box(left_box_params)?;
    let right_box = kernel.make_box(right_box_params)?;
    let expected_solid_volume =
        kernel.summarize(&left_box)?.volume + kernel.summarize(&right_box)?.volume;
    let solid_assembly = kernel.make_compsolid(&[left_box, right_box])?;

    assert_assembly_metadata(
        &solid_assembly,
        ShapeKind::CompSolid,
        &[ShapeKind::Solid, ShapeKind::Solid],
        "solid compsolid assembly",
    )?;
    assert_simple_assembly_snapshot(
        &kernel,
        &solid_assembly,
        AssemblySummaryExpectation {
            root_kind: ShapeKind::CompSolid,
            primary_kind: ShapeKind::Solid,
            compound_count: 0,
            compsolid_count: 1,
            solid_count: 2,
            shell_count: 2,
            face_count: 12,
            wire_count: 12,
            edge_count: 24,
            vertex_count: 16,
            summary_bbox_source: SummaryBboxSource::PortedBrep,
            summary_volume_source: SummaryVolumeSource::FaceContributions,
        },
        &[(SurfaceKind::Plane, 12)],
        "solid compsolid assembly",
    )?;
    assert_scalar_close(
        kernel.summarize(&solid_assembly)?.volume,
        expected_solid_volume,
        1.0e-8,
        "solid assembly volume from child solids",
    )?;

    let context = kernel.context();
    let shell_left_box = kernel.make_box(BoxParams {
        origin: [-18.0, 18.0, -5.0],
        size: [10.0, 12.0, 8.0],
    })?;
    let shell_right_box = kernel.make_box(BoxParams {
        origin: [8.0, 18.0, -5.0],
        size: [10.0, 12.0, 8.0],
    })?;
    let left_shell = context.subshape(&shell_left_box, ShapeKind::Shell, 0)?;
    let right_shell = context.subshape(&shell_right_box, ShapeKind::Shell, 0)?;
    let shell_compound = kernel.make_compound(&[left_shell, right_shell])?;

    assert_assembly_metadata(
        &shell_compound,
        ShapeKind::Compound,
        &[ShapeKind::Shell, ShapeKind::Shell],
        "shell compound assembly",
    )?;
    assert_simple_assembly_snapshot(
        &kernel,
        &shell_compound,
        AssemblySummaryExpectation {
            root_kind: ShapeKind::Compound,
            primary_kind: ShapeKind::Shell,
            compound_count: 1,
            compsolid_count: 0,
            solid_count: 0,
            shell_count: 2,
            face_count: 12,
            wire_count: 12,
            edge_count: 24,
            vertex_count: 16,
            summary_bbox_source: SummaryBboxSource::PortedBrep,
            summary_volume_source: SummaryVolumeSource::Zero,
        },
        &[(SurfaceKind::Plane, 12)],
        "shell compound assembly",
    )?;

    let nested_shell_compound = kernel.make_compound(&[shell_compound])?;
    assert_assembly_metadata(
        &nested_shell_compound,
        ShapeKind::Compound,
        &[ShapeKind::Compound],
        "nested shell compound assembly",
    )?;
    assert_simple_assembly_snapshot(
        &kernel,
        &nested_shell_compound,
        AssemblySummaryExpectation {
            root_kind: ShapeKind::Compound,
            primary_kind: ShapeKind::Shell,
            compound_count: 1,
            compsolid_count: 0,
            solid_count: 0,
            shell_count: 2,
            face_count: 12,
            wire_count: 12,
            edge_count: 24,
            vertex_count: 16,
            summary_bbox_source: SummaryBboxSource::PortedBrep,
            summary_volume_source: SummaryVolumeSource::Zero,
        },
        &[(SurfaceKind::Plane, 12)],
        "nested shell compound assembly",
    )?;

    let mut document = ModelDocument::new()?;
    document.insert_box("doc_left", left_box_params)?;
    document.insert_box("doc_right", right_box_params)?;
    document.compsolid("doc_solid_assembly", &["doc_left", "doc_right"])?;
    document.compound("doc_compound_assembly", &["doc_left", "doc_right"])?;

    let doc_solid = document.shape("doc_solid_assembly")?;
    assert_assembly_metadata(
        doc_solid,
        ShapeKind::CompSolid,
        &[ShapeKind::Solid, ShapeKind::Solid],
        "document solid assembly",
    )?;
    let doc_report = document.report("doc_solid_assembly")?;
    let doc_faces = document.faces("doc_solid_assembly")?;
    let doc_edges = document.edges("doc_solid_assembly")?;
    let doc_plane_indices =
        document.face_indices_by_surface_kind("doc_solid_assembly", SurfaceKind::Plane)?;
    let selected_face = document.select_face(
        "doc_solid_assembly",
        FaceSelector::LargestBySurfaceKind(SurfaceKind::Plane),
    )?;
    let selected_edge = document.select_edge(
        "doc_solid_assembly",
        EdgeSelector::LongestByCurveKind(CurveKind::Line),
    )?;

    assert_eq!(doc_report.summary.root_kind, ShapeKind::CompSolid);
    assert_eq!(doc_report.summary.solid_count, 2);
    assert_eq!(doc_report.summary.shell_count, 2);
    assert_eq!(doc_report.summary.face_count, 12);
    assert_eq!(doc_faces.len(), 12);
    assert_eq!(doc_plane_indices.len(), 12);
    assert_eq!(selected_face.geometry.kind, SurfaceKind::Plane);
    assert_eq!(selected_edge.geometry.kind, CurveKind::Line);
    assert!(selected_edge.length > 0.0);
    assert_eq!(doc_edges.len(), doc_report.summary.edge_count);
    assert_assembly_metadata(
        document.shape("doc_compound_assembly")?,
        ShapeKind::Compound,
        &[ShapeKind::Solid, ShapeKind::Solid],
        "document compound assembly",
    )?;

    match document.history() {
        [OperationRecord::AddBox {
            output: left_output,
            params: left_params,
        }, OperationRecord::AddBox {
            output: right_output,
            params: right_params,
        }, OperationRecord::CompSolid {
            output: compsolid_output,
            inputs: compsolid_inputs,
        }, OperationRecord::Compound {
            output: compound_output,
            inputs: compound_inputs,
        }] => {
            assert_eq!(left_output, "doc_left");
            assert_eq!(right_output, "doc_right");
            assert_vec3_close(
                left_params.origin,
                left_box_params.origin,
                0.0,
                "document left box history origin",
            )?;
            assert_vec3_close(
                right_params.origin,
                right_box_params.origin,
                0.0,
                "document right box history origin",
            )?;
            assert_eq!(compsolid_output, "doc_solid_assembly");
            assert_eq!(
                compsolid_inputs,
                &vec!["doc_left".to_owned(), "doc_right".to_owned()]
            );
            assert_eq!(compound_output, "doc_compound_assembly");
            assert_eq!(
                compound_inputs,
                &vec!["doc_left".to_owned(), "doc_right".to_owned()]
            );
        }
        history => {
            return Err(std::io::Error::other(format!(
                "expected AddBox + AddBox + CompSolid + Compound history entries, got {history:?}"
            ))
            .into());
        }
    }

    Ok(())
}

#[test]
fn simple_face_wire_assembly_authored_family_row_is_rust_owned(
) -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    require_complete_ownership_row(
        AuthoredFamily::SimpleFaceWireAssembly,
        "simple face/wire assembly",
    )?;

    let kernel = ModelKernel::new()?;
    let context = kernel.context();

    let face_lhs = kernel.make_box(BoxParams {
        origin: [-36.0, -8.0, -4.0],
        size: [8.0, 10.0, 6.0],
    })?;
    let face_rhs = kernel.make_box(BoxParams {
        origin: [-18.0, -8.0, -4.0],
        size: [8.0, 10.0, 6.0],
    })?;
    let lhs_face = context.subshape(&face_lhs, ShapeKind::Face, 0)?;
    let rhs_face = context.subshape(&face_rhs, ShapeKind::Face, 0)?;
    let face_compound = kernel.make_compound(&[lhs_face, rhs_face])?;

    assert_assembly_metadata(
        &face_compound,
        ShapeKind::Compound,
        &[ShapeKind::Face, ShapeKind::Face],
        "face compound assembly",
    )?;
    assert_simple_assembly_snapshot(
        &kernel,
        &face_compound,
        AssemblySummaryExpectation {
            root_kind: ShapeKind::Compound,
            primary_kind: ShapeKind::Face,
            compound_count: 1,
            compsolid_count: 0,
            solid_count: 0,
            shell_count: 0,
            face_count: 2,
            wire_count: 2,
            edge_count: 8,
            vertex_count: 8,
            summary_bbox_source: SummaryBboxSource::PortedBrep,
            summary_volume_source: SummaryVolumeSource::Zero,
        },
        &[(SurfaceKind::Plane, 2)],
        "face compound assembly",
    )?;
    assert_line_edge_queries(&kernel, &face_compound, 8, "face compound assembly")?;
    assert_vertex_queries(&kernel, &face_compound, 8, "face compound assembly")?;

    let wire_lhs = kernel.make_box(BoxParams {
        origin: [2.0, -8.0, -4.0],
        size: [8.0, 10.0, 6.0],
    })?;
    let wire_rhs = kernel.make_box(BoxParams {
        origin: [20.0, -8.0, -4.0],
        size: [8.0, 10.0, 6.0],
    })?;
    let lhs_wire_face = context.subshape(&wire_lhs, ShapeKind::Face, 0)?;
    let rhs_wire_face = context.subshape(&wire_rhs, ShapeKind::Face, 0)?;
    let lhs_wire = context.subshape(&lhs_wire_face, ShapeKind::Wire, 0)?;
    let rhs_wire = context.subshape(&rhs_wire_face, ShapeKind::Wire, 0)?;
    let wire_compound = kernel.make_compound(&[lhs_wire, rhs_wire])?;

    assert_assembly_metadata(
        &wire_compound,
        ShapeKind::Compound,
        &[ShapeKind::Wire, ShapeKind::Wire],
        "wire compound assembly",
    )?;
    assert_simple_assembly_snapshot(
        &kernel,
        &wire_compound,
        AssemblySummaryExpectation {
            root_kind: ShapeKind::Compound,
            primary_kind: ShapeKind::Wire,
            compound_count: 1,
            compsolid_count: 0,
            solid_count: 0,
            shell_count: 0,
            face_count: 0,
            wire_count: 2,
            edge_count: 8,
            vertex_count: 8,
            summary_bbox_source: SummaryBboxSource::PortedBrep,
            summary_volume_source: SummaryVolumeSource::Zero,
        },
        &[],
        "wire compound assembly",
    )?;
    assert_line_edge_queries(&kernel, &wire_compound, 8, "wire compound assembly")?;
    assert_vertex_queries(&kernel, &wire_compound, 8, "wire compound assembly")?;

    let edge_lhs = kernel.make_box(BoxParams {
        origin: [-36.0, 18.0, -4.0],
        size: [8.0, 10.0, 6.0],
    })?;
    let edge_rhs = kernel.make_box(BoxParams {
        origin: [-18.0, 18.0, -4.0],
        size: [8.0, 10.0, 6.0],
    })?;
    let lhs_edge = context.subshape(&edge_lhs, ShapeKind::Edge, 0)?;
    let rhs_edge = context.subshape(&edge_rhs, ShapeKind::Edge, 0)?;
    let edge_compound = kernel.make_compound(&[lhs_edge, rhs_edge])?;

    assert_assembly_metadata(
        &edge_compound,
        ShapeKind::Compound,
        &[ShapeKind::Edge, ShapeKind::Edge],
        "edge compound assembly",
    )?;
    assert_simple_assembly_snapshot(
        &kernel,
        &edge_compound,
        AssemblySummaryExpectation {
            root_kind: ShapeKind::Compound,
            primary_kind: ShapeKind::Edge,
            compound_count: 1,
            compsolid_count: 0,
            solid_count: 0,
            shell_count: 0,
            face_count: 0,
            wire_count: 0,
            edge_count: 2,
            vertex_count: 4,
            summary_bbox_source: SummaryBboxSource::PortedBrep,
            summary_volume_source: SummaryVolumeSource::Zero,
        },
        &[],
        "edge compound assembly",
    )?;
    assert_line_edge_queries(&kernel, &edge_compound, 2, "edge compound assembly")?;
    assert_vertex_queries(&kernel, &edge_compound, 4, "edge compound assembly")?;

    let vertex_lhs = kernel.make_box(BoxParams {
        origin: [2.0, 18.0, -4.0],
        size: [8.0, 10.0, 6.0],
    })?;
    let vertex_rhs = kernel.make_box(BoxParams {
        origin: [20.0, 18.0, -4.0],
        size: [8.0, 10.0, 6.0],
    })?;
    let lhs_vertex = context.subshape(&vertex_lhs, ShapeKind::Vertex, 0)?;
    let rhs_vertex = context.subshape(&vertex_rhs, ShapeKind::Vertex, 0)?;
    let vertex_compound = kernel.make_compound(&[lhs_vertex, rhs_vertex])?;

    assert_assembly_metadata(
        &vertex_compound,
        ShapeKind::Compound,
        &[ShapeKind::Vertex, ShapeKind::Vertex],
        "vertex compound assembly",
    )?;
    assert_simple_assembly_snapshot(
        &kernel,
        &vertex_compound,
        AssemblySummaryExpectation {
            root_kind: ShapeKind::Compound,
            primary_kind: ShapeKind::Vertex,
            compound_count: 1,
            compsolid_count: 0,
            solid_count: 0,
            shell_count: 0,
            face_count: 0,
            wire_count: 0,
            edge_count: 0,
            vertex_count: 2,
            summary_bbox_source: SummaryBboxSource::PortedBrep,
            summary_volume_source: SummaryVolumeSource::Zero,
        },
        &[],
        "vertex compound assembly",
    )?;
    assert_vertex_queries(&kernel, &vertex_compound, 2, "vertex compound assembly")?;

    let mut document = ModelDocument::new()?;
    let doc_face_params = BoxParams {
        origin: [-48.0, 44.0, -3.0],
        size: [7.0, 9.0, 5.0],
    };
    let doc_wire_params = BoxParams {
        origin: [-28.0, 44.0, -3.0],
        size: [7.0, 9.0, 5.0],
    };
    let doc_edge_params = BoxParams {
        origin: [-8.0, 44.0, -3.0],
        size: [7.0, 9.0, 5.0],
    };
    let doc_vertex_params = BoxParams {
        origin: [12.0, 44.0, -3.0],
        size: [7.0, 9.0, 5.0],
    };
    document.insert_box("doc_face_source", doc_face_params)?;
    document.insert_box("doc_wire_source", doc_wire_params)?;
    document.insert_box("doc_edge_source", doc_edge_params)?;
    document.insert_box("doc_vertex_source", doc_vertex_params)?;
    document.subshape("doc_face", "doc_face_source", ShapeKind::Face, 0)?;
    document.subshape("doc_wire_face", "doc_wire_source", ShapeKind::Face, 0)?;
    document.subshape("doc_wire", "doc_wire_face", ShapeKind::Wire, 0)?;
    document.subshape("doc_edge", "doc_edge_source", ShapeKind::Edge, 0)?;
    document.subshape("doc_vertex", "doc_vertex_source", ShapeKind::Vertex, 0)?;
    document.compound(
        "doc_face_wire_assembly",
        &["doc_face", "doc_wire", "doc_edge", "doc_vertex"],
    )?;

    let doc_shape = document.shape("doc_face_wire_assembly")?;
    assert_assembly_metadata(
        doc_shape,
        ShapeKind::Compound,
        &[
            ShapeKind::Face,
            ShapeKind::Wire,
            ShapeKind::Edge,
            ShapeKind::Vertex,
        ],
        "document face/wire compound assembly",
    )?;
    assert_simple_assembly_snapshot(
        document.kernel(),
        doc_shape,
        AssemblySummaryExpectation {
            root_kind: ShapeKind::Compound,
            primary_kind: ShapeKind::Face,
            compound_count: 1,
            compsolid_count: 0,
            solid_count: 0,
            shell_count: 0,
            face_count: 1,
            wire_count: 2,
            edge_count: 9,
            vertex_count: 11,
            summary_bbox_source: SummaryBboxSource::PortedBrep,
            summary_volume_source: SummaryVolumeSource::Zero,
        },
        &[(SurfaceKind::Plane, 1)],
        "document face/wire compound assembly",
    )?;
    assert_line_edge_queries(
        document.kernel(),
        doc_shape,
        9,
        "document face/wire compound assembly",
    )?;
    assert_vertex_queries(
        document.kernel(),
        doc_shape,
        11,
        "document face/wire compound assembly",
    )?;

    let doc_report = document.report("doc_face_wire_assembly")?;
    let doc_faces = document.faces("doc_face_wire_assembly")?;
    let doc_edges = document.edges("doc_face_wire_assembly")?;
    let doc_plane_indices =
        document.face_indices_by_surface_kind("doc_face_wire_assembly", SurfaceKind::Plane)?;
    let doc_line_indices =
        document.edge_indices_by_curve_kind("doc_face_wire_assembly", CurveKind::Line)?;
    let selected_face = document.select_face(
        "doc_face_wire_assembly",
        FaceSelector::FirstBySurfaceKind(SurfaceKind::Plane),
    )?;
    let selected_edge = document.select_edge(
        "doc_face_wire_assembly",
        EdgeSelector::LongestByCurveKind(CurveKind::Line),
    )?;

    assert_eq!(doc_report.summary.root_kind, ShapeKind::Compound);
    assert_eq!(doc_report.summary.primary_kind, ShapeKind::Face);
    assert_eq!(doc_report.summary.face_count, 1);
    assert_eq!(doc_report.summary.wire_count, 2);
    assert_eq!(doc_report.summary.edge_count, 9);
    assert_eq!(doc_report.summary.vertex_count, 11);
    assert_eq!(doc_faces.len(), 1);
    assert_eq!(doc_edges.len(), 9);
    assert_eq!(doc_plane_indices.len(), 1);
    assert_eq!(doc_line_indices.len(), 9);
    assert_eq!(selected_face.geometry.kind, SurfaceKind::Plane);
    assert_eq!(selected_edge.geometry.kind, CurveKind::Line);
    assert!(selected_edge.length > 0.0);

    match document.history() {
        [OperationRecord::AddBox {
            output: face_source,
            params: face_params,
        }, OperationRecord::AddBox {
            output: wire_source,
            params: wire_params,
        }, OperationRecord::AddBox {
            output: edge_source,
            params: edge_params,
        }, OperationRecord::AddBox {
            output: vertex_source,
            params: vertex_params,
        }, OperationRecord::Subshape {
            output: face_output,
            input: face_input,
            kind: face_kind,
            index: face_index,
        }, OperationRecord::Subshape {
            output: wire_face_output,
            input: wire_face_input,
            kind: wire_face_kind,
            index: wire_face_index,
        }, OperationRecord::Subshape {
            output: wire_output,
            input: wire_input,
            kind: wire_kind,
            index: wire_index,
        }, OperationRecord::Subshape {
            output: edge_output,
            input: edge_input,
            kind: edge_kind,
            index: edge_index,
        }, OperationRecord::Subshape {
            output: vertex_output,
            input: vertex_input,
            kind: vertex_kind,
            index: vertex_index,
        }, OperationRecord::Compound {
            output: assembly_output,
            inputs: assembly_inputs,
        }] => {
            assert_eq!(face_source, "doc_face_source");
            assert_eq!(wire_source, "doc_wire_source");
            assert_eq!(edge_source, "doc_edge_source");
            assert_eq!(vertex_source, "doc_vertex_source");
            assert_vec3_close(
                face_params.origin,
                doc_face_params.origin,
                0.0,
                "document face source history origin",
            )?;
            assert_vec3_close(
                wire_params.origin,
                doc_wire_params.origin,
                0.0,
                "document wire source history origin",
            )?;
            assert_vec3_close(
                edge_params.origin,
                doc_edge_params.origin,
                0.0,
                "document edge source history origin",
            )?;
            assert_vec3_close(
                vertex_params.origin,
                doc_vertex_params.origin,
                0.0,
                "document vertex source history origin",
            )?;
            assert_eq!(face_output, "doc_face");
            assert_eq!(face_input, "doc_face_source");
            assert_eq!(*face_kind, ShapeKind::Face);
            assert_eq!(*face_index, 0);
            assert_eq!(wire_face_output, "doc_wire_face");
            assert_eq!(wire_face_input, "doc_wire_source");
            assert_eq!(*wire_face_kind, ShapeKind::Face);
            assert_eq!(*wire_face_index, 0);
            assert_eq!(wire_output, "doc_wire");
            assert_eq!(wire_input, "doc_wire_face");
            assert_eq!(*wire_kind, ShapeKind::Wire);
            assert_eq!(*wire_index, 0);
            assert_eq!(edge_output, "doc_edge");
            assert_eq!(edge_input, "doc_edge_source");
            assert_eq!(*edge_kind, ShapeKind::Edge);
            assert_eq!(*edge_index, 0);
            assert_eq!(vertex_output, "doc_vertex");
            assert_eq!(vertex_input, "doc_vertex_source");
            assert_eq!(*vertex_kind, ShapeKind::Vertex);
            assert_eq!(*vertex_index, 0);
            assert_eq!(assembly_output, "doc_face_wire_assembly");
            assert_eq!(
                assembly_inputs,
                &vec![
                    "doc_face".to_owned(),
                    "doc_wire".to_owned(),
                    "doc_edge".to_owned(),
                    "doc_vertex".to_owned()
                ]
            );
        }
        history => {
            return Err(std::io::Error::other(format!(
                "expected AddBox/AddBox/AddBox/AddBox/Subshape/Subshape/Subshape/Subshape/Subshape/Compound history entries, got {history:?}"
            ))
            .into());
        }
    }

    Ok(())
}

fn assert_assembly_summary_matches_children(
    kernel: &ModelKernel,
    shape: &Shape,
    children: &[&Shape],
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut expected_bbox_min = [f64::INFINITY; 3];
    let mut expected_bbox_max = [f64::NEG_INFINITY; 3];
    let mut expected_surface_area = 0.0;
    let mut expected_volume = 0.0;

    for child in children {
        let summary = kernel.summarize(child)?;
        for axis in 0..3 {
            expected_bbox_min[axis] = expected_bbox_min[axis].min(summary.bbox_min[axis]);
            expected_bbox_max[axis] = expected_bbox_max[axis].max(summary.bbox_max[axis]);
        }
        expected_surface_area += summary.surface_area;
        expected_volume += summary.volume;
    }

    let summary = kernel.summarize(shape)?;
    assert_vec3_close(
        summary.bbox_min,
        expected_bbox_min,
        1.0e-8,
        &format!("{label} bbox min from child summaries"),
    )?;
    assert_vec3_close(
        summary.bbox_max,
        expected_bbox_max,
        1.0e-8,
        &format!("{label} bbox max from child summaries"),
    )?;
    assert_scalar_close(
        summary.surface_area,
        expected_surface_area,
        1.0e-8,
        &format!("{label} surface area from child summaries"),
    )?;
    assert_scalar_close(
        summary.volume,
        expected_volume,
        1.0e-8,
        &format!("{label} volume from child summaries"),
    )?;

    Ok(())
}

fn assert_mixed_analytic_solid_assembly(
    kernel: &ModelKernel,
    shape: &Shape,
    child_shapes: &[&Shape],
    root_kind: ShapeKind,
    compound_count: usize,
    compsolid_count: usize,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let expected_child_kinds = [
        ShapeKind::Solid,
        ShapeKind::Solid,
        ShapeKind::Solid,
        ShapeKind::Solid,
        ShapeKind::Solid,
    ];
    let expected_child_counts = [Some(3), Some(2), Some(3), Some(1), Some(1)];
    let expected_face_kinds = [
        (SurfaceKind::Plane, 10),
        (SurfaceKind::Cylinder, 1),
        (SurfaceKind::Cone, 1),
        (SurfaceKind::Sphere, 1),
        (SurfaceKind::Torus, 1),
    ];
    let expected_curve_kinds = [
        (CurveKind::Line, 14),
        (CurveKind::Circle, 7),
        (CurveKind::Other, 2),
    ];

    assert_eq!(
        child_shapes.len(),
        expected_child_counts.len(),
        "{label} child shape inventory"
    );
    for (index, (child, expected_count)) in
        child_shapes.iter().zip(expected_child_counts).enumerate()
    {
        assert_eq!(
            child.rust_multi_face_analytic_source_count(),
            expected_count,
            "{label} child {index} analytic source count"
        );
    }

    assert_assembly_metadata(shape, root_kind, &expected_child_kinds, label)?;
    assert_assembly_analytic_source_counts(shape, &expected_child_counts, label)?;
    assert_simple_assembly_snapshot(
        kernel,
        shape,
        AssemblySummaryExpectation {
            root_kind,
            primary_kind: ShapeKind::Solid,
            compound_count,
            compsolid_count,
            solid_count: 5,
            shell_count: 5,
            face_count: 14,
            wire_count: 14,
            edge_count: 23,
            vertex_count: 15,
            summary_bbox_source: SummaryBboxSource::PortedBrep,
            summary_volume_source: SummaryVolumeSource::FaceContributions,
        },
        &expected_face_kinds,
        label,
    )?;
    assert_supported_edge_queries(kernel, shape, &expected_curve_kinds, label)?;
    assert_vertex_queries(kernel, shape, 15, label)?;
    assert_assembly_summary_matches_children(kernel, shape, child_shapes, label)?;

    Ok(())
}

#[test]
fn mixed_analytic_solid_assembly_authored_family_row_is_rust_owned(
) -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    require_complete_ownership_row(
        AuthoredFamily::MixedAnalyticSolidAssembly,
        "mixed analytic solid assembly",
    )?;

    let kernel = ModelKernel::new()?;
    let box_params = BoxParams {
        origin: [-84.0, -12.0, -5.0],
        size: [10.0, 24.0, 10.0],
    };
    let cylinder_params = CylinderParams {
        origin: [-54.0, 0.0, -6.0],
        axis: [0.0, 0.0, 1.0],
        radius: 5.0,
        height: 12.0,
    };
    let cone_params = ConeParams {
        origin: [-24.0, 0.0, -6.0],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        base_radius: 5.0,
        top_radius: 2.5,
        height: 12.0,
    };
    let sphere_params = SphereParams {
        origin: [8.0, 0.0, 0.0],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        radius: 5.0,
    };
    let torus_params = TorusParams {
        origin: [-8.0, 0.0, 0.0],
        axis: [0.0, 0.0, 1.0],
        x_direction: [1.0, 0.0, 0.0],
        major_radius: 6.0,
        minor_radius: 2.0,
    };

    let solids = [
        kernel.make_box(box_params)?,
        kernel.make_cylinder(cylinder_params)?,
        kernel.make_cone(cone_params)?,
        kernel.make_sphere(sphere_params)?,
        kernel.make_torus(torus_params)?,
    ];
    let solid_refs = solids.iter().collect::<Vec<_>>();
    let mixed_compound = kernel.make_compound(&solids)?;
    let mixed_compsolid = kernel.make_compsolid(&solids)?;

    assert_mixed_analytic_solid_assembly(
        &kernel,
        &mixed_compound,
        &solid_refs,
        ShapeKind::Compound,
        1,
        0,
        "mixed analytic solid compound",
    )?;
    assert_mixed_analytic_solid_assembly(
        &kernel,
        &mixed_compsolid,
        &solid_refs,
        ShapeKind::CompSolid,
        0,
        1,
        "mixed analytic solid compsolid",
    )?;

    let mut document = ModelDocument::new()?;
    document.insert_box("doc_box", box_params)?;
    document.insert_cylinder("doc_cylinder", cylinder_params)?;
    document.insert_cone("doc_cone", cone_params)?;
    document.insert_sphere("doc_sphere", sphere_params)?;
    document.insert_torus("doc_torus", torus_params)?;
    let doc_input_names = [
        "doc_box",
        "doc_cylinder",
        "doc_cone",
        "doc_sphere",
        "doc_torus",
    ];
    document.compsolid("doc_mixed_compsolid", &doc_input_names)?;
    document.compound("doc_mixed_compound", &doc_input_names)?;

    let doc_child_refs = doc_input_names
        .iter()
        .map(|name| document.shape(*name))
        .collect::<Result<Vec<_>, _>>()?;
    let doc_compsolid = document.shape("doc_mixed_compsolid")?;
    assert_mixed_analytic_solid_assembly(
        document.kernel(),
        doc_compsolid,
        &doc_child_refs,
        ShapeKind::CompSolid,
        0,
        1,
        "document mixed analytic solid compsolid",
    )?;
    let doc_compound = document.shape("doc_mixed_compound")?;
    assert_mixed_analytic_solid_assembly(
        document.kernel(),
        doc_compound,
        &doc_child_refs,
        ShapeKind::Compound,
        1,
        0,
        "document mixed analytic solid compound",
    )?;

    let doc_report = document.report("doc_mixed_compsolid")?;
    let doc_faces = document.faces("doc_mixed_compsolid")?;
    let doc_edges = document.edges("doc_mixed_compsolid")?;
    assert_eq!(doc_report.summary.root_kind, ShapeKind::CompSolid);
    assert_eq!(doc_report.summary.primary_kind, ShapeKind::Solid);
    assert_eq!(doc_report.summary.solid_count, 5);
    assert_eq!(doc_report.summary.face_count, 14);
    assert_eq!(doc_report.summary.edge_count, 23);
    assert_eq!(doc_faces.len(), 14);
    assert_eq!(doc_edges.len(), 23);
    for (kind, expected) in [
        (SurfaceKind::Plane, 10),
        (SurfaceKind::Cylinder, 1),
        (SurfaceKind::Cone, 1),
        (SurfaceKind::Sphere, 1),
        (SurfaceKind::Torus, 1),
    ] {
        assert_eq!(
            document
                .face_indices_by_surface_kind("doc_mixed_compsolid", kind)?
                .len(),
            expected,
            "document mixed analytic face selector inventory for {kind:?}"
        );
        let selected = document.select_face(
            "doc_mixed_compsolid",
            FaceSelector::LargestBySurfaceKind(kind),
        )?;
        assert_eq!(selected.geometry.kind, kind);
        assert!(selected.area > 0.0);
    }
    for (kind, expected) in [
        (CurveKind::Line, 14),
        (CurveKind::Circle, 7),
        (CurveKind::Other, 2),
    ] {
        assert_eq!(
            document
                .edge_indices_by_curve_kind("doc_mixed_compsolid", kind)?
                .len(),
            expected,
            "document mixed analytic edge selector inventory for {kind:?}"
        );
        let selected = document.select_edge(
            "doc_mixed_compsolid",
            EdgeSelector::LongestByCurveKind(kind),
        )?;
        assert_eq!(selected.geometry.kind, kind);
        assert!(selected.length > 0.0);
    }

    assert_eq!(document.history().len(), 7);
    match &document.history()[0] {
        OperationRecord::AddBox { output, params } => {
            assert_eq!(output, "doc_box");
            assert_vec3_close(params.origin, box_params.origin, 0.0, "history box origin")?;
        }
        other => {
            return Err(std::io::Error::other(format!(
                "expected AddBox history entry, got {other:?}"
            ))
            .into());
        }
    }
    match &document.history()[1] {
        OperationRecord::AddCylinder { output, params } => {
            assert_eq!(output, "doc_cylinder");
            assert_vec3_close(
                params.origin,
                cylinder_params.origin,
                0.0,
                "history cylinder origin",
            )?;
            assert_scalar_close(
                params.radius,
                cylinder_params.radius,
                0.0,
                "history cylinder radius",
            )?;
        }
        other => {
            return Err(std::io::Error::other(format!(
                "expected AddCylinder history entry, got {other:?}"
            ))
            .into());
        }
    }
    match &document.history()[2] {
        OperationRecord::AddCone { output, params } => {
            assert_eq!(output, "doc_cone");
            assert_vec3_close(
                params.origin,
                cone_params.origin,
                0.0,
                "history cone origin",
            )?;
            assert_scalar_close(
                params.top_radius,
                cone_params.top_radius,
                0.0,
                "history cone top radius",
            )?;
        }
        other => {
            return Err(std::io::Error::other(format!(
                "expected AddCone history entry, got {other:?}"
            ))
            .into());
        }
    }
    match &document.history()[3] {
        OperationRecord::AddSphere { output, params } => {
            assert_eq!(output, "doc_sphere");
            assert_vec3_close(
                params.origin,
                sphere_params.origin,
                0.0,
                "history sphere origin",
            )?;
            assert_scalar_close(
                params.radius,
                sphere_params.radius,
                0.0,
                "history sphere radius",
            )?;
        }
        other => {
            return Err(std::io::Error::other(format!(
                "expected AddSphere history entry, got {other:?}"
            ))
            .into());
        }
    }
    match &document.history()[4] {
        OperationRecord::AddTorus { output, params } => {
            assert_eq!(output, "doc_torus");
            assert_vec3_close(
                params.origin,
                torus_params.origin,
                0.0,
                "history torus origin",
            )?;
            assert_scalar_close(
                params.major_radius,
                torus_params.major_radius,
                0.0,
                "history torus major radius",
            )?;
        }
        other => {
            return Err(std::io::Error::other(format!(
                "expected AddTorus history entry, got {other:?}"
            ))
            .into());
        }
    }
    let expected_doc_inputs = doc_input_names
        .iter()
        .map(|name| (*name).to_owned())
        .collect::<Vec<_>>();
    match &document.history()[5] {
        OperationRecord::CompSolid { output, inputs } => {
            assert_eq!(output, "doc_mixed_compsolid");
            assert_eq!(inputs, &expected_doc_inputs);
        }
        other => {
            return Err(std::io::Error::other(format!(
                "expected CompSolid history entry, got {other:?}"
            ))
            .into());
        }
    }
    match &document.history()[6] {
        OperationRecord::Compound { output, inputs } => {
            assert_eq!(output, "doc_mixed_compound");
            assert_eq!(inputs, &expected_doc_inputs);
        }
        other => {
            return Err(std::io::Error::other(format!(
                "expected Compound history entry, got {other:?}"
            ))
            .into());
        }
    }

    Ok(())
}
