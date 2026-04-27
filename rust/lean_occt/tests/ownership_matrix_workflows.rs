mod support;

use lean_occt::{
    BoxParams, CurveKind, EdgeSelector, FaceSelector, ModelDocument, OperationRecord, PlanePayload,
    PortedCurve, PortedFaceSurface, PortedSurface, ShapeKind, SummaryBboxSource,
    SummaryVolumeSource, SurfaceKind,
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
        construction_metadata: false,
        normalized_snapshot_brep: false,
        public_queries: false,
        summary_metrics: false,
        selectors_documents: false,
    },
    OwnershipRow {
        family: AuthoredFamily::Cone,
        construction_metadata: false,
        normalized_snapshot_brep: false,
        public_queries: false,
        summary_metrics: false,
        selectors_documents: false,
    },
    OwnershipRow {
        family: AuthoredFamily::Sphere,
        construction_metadata: false,
        normalized_snapshot_brep: false,
        public_queries: false,
        summary_metrics: false,
        selectors_documents: false,
    },
    OwnershipRow {
        family: AuthoredFamily::Torus,
        construction_metadata: false,
        normalized_snapshot_brep: false,
        public_queries: false,
        summary_metrics: false,
        selectors_documents: false,
    },
    OwnershipRow {
        family: AuthoredFamily::PrismExtrusion,
        construction_metadata: false,
        normalized_snapshot_brep: false,
        public_queries: false,
        summary_metrics: false,
        selectors_documents: false,
    },
    OwnershipRow {
        family: AuthoredFamily::Revolution,
        construction_metadata: false,
        normalized_snapshot_brep: false,
        public_queries: false,
        summary_metrics: false,
        selectors_documents: false,
    },
    OwnershipRow {
        family: AuthoredFamily::DirectOffset,
        construction_metadata: false,
        normalized_snapshot_brep: false,
        public_queries: false,
        summary_metrics: false,
        selectors_documents: false,
    },
    OwnershipRow {
        family: AuthoredFamily::GeneratedOffset,
        construction_metadata: false,
        normalized_snapshot_brep: false,
        public_queries: false,
        summary_metrics: false,
        selectors_documents: false,
    },
];

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

#[test]
fn box_planar_authored_family_row_is_rust_owned() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = support::test_guard();
    let row = OWNERSHIP_MATRIX
        .iter()
        .find(|row| row.family == AuthoredFamily::BoxPlanar)
        .ok_or_else(|| std::io::Error::other("missing box/planar ownership row"))?;
    assert_eq!(OWNERSHIP_MATRIX.len(), 9);
    assert!(row.construction_metadata);
    assert!(row.normalized_snapshot_brep);
    assert!(row.public_queries);
    assert!(row.summary_metrics);
    assert!(row.selectors_documents);

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
