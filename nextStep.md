# Next Task

Current milestone: `M6. BRep Surface Payload Fallback Cleanup` from `portingMilestones.md`.

## Completed Evidence

- `rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs` now makes `PortedCurve::from_context_with_ported_payloads()` return only Rust-derived line, circle, and ellipse payloads. Unsupported Rust extraction returns `None`; it no longer calls `edge_line_payload_occt()`, `edge_circle_payload_occt()`, or `edge_ellipse_payload_occt()`.
- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/brep_materialize.rs` and `face_topology.rs` no longer catch public-route Rust curve extraction errors and retry `PortedCurve::from_context_with_geometry()`.
- `rust/lean_occt/tests/brep_workflows.rs` now verifies exercised line, circle, and ellipse BRep edges populate `BrepEdge::ported_curve`, derive lengths from the ported curve, match the public Rust curve route, and stay close to OCCT samples/lengths.
- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_snapshot.rs` now uses `PortedCurve::from_context_with_ported_payloads()` directly for planar root-wire area reconstruction and no longer retries `PortedCurve::from_context_with_geometry()` after a Rust extraction error.
- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/swept_face.rs::append_root_edge_sample_points()` now samples supported root-edge lines, circles, and ellipses through the Rust-owned ported payload route, keeping OCCT point sampling only for unsupported `None`.
- `rust/lean_occt/tests/brep_workflows.rs` now strengthens the holed planar single-face workflow with exact holed area, loop count parity, and ported-curve assertions on analytic loop edges.
- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_topology.rs` now makes both `FaceSurfaceRoute::Raw` and `FaceSurfaceRoute::Public` edge materialization call `PortedCurve::from_context_with_ported_payloads()` after route-specific geometry selection.
- `rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs::PortedCurve::from_context_with_geometry()` now delegates to Rust-owned ported payload extraction and no longer calls `edge_line_payload_occt()`, `edge_circle_payload_occt()`, or `edge_ellipse_payload_occt()`.
- `rust/lean_occt/tests/brep_workflows.rs` now exercises the raw-geometry `PortedCurve::from_context_with_geometry()` route for line, circle, and ellipse edges and checks midpoint sample parity with OCCT.
- `M5. BRep Curve Payload Fallback Cleanup` is complete.
- Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_exact_curve_bounding_boxes`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_simple_single_face_shapes`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`, and `git diff --check`.

## Target

Replace or strictly narrow the BRep surface materialization fallback where Rust face geometry has identified an exercised plane, cylinder, cone, sphere, or torus but `PortedSurface::from_context_with_geometry()` can still rescue through OCCT payload helpers.

## Next Bounded Cut

1. In `rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs`, add or reuse a surface `from_context_with_ported_payloads()` route for plane, cylinder, cone, sphere, and torus that returns `None` for unsupported Rust extraction instead of calling `face_*_payload_occt()`.
2. In `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_prepare.rs`, feed BRep face preparation through the Rust-owned surface payload route so supported analytic faces populate `BrepFace::ported_surface` and `ported_face_surface` without OCCT payload helper rescue.
3. Strengthen `rust/lean_occt/tests/brep_workflows.rs` around exact primitive faces, asserting the expected analytic surface variants, sample parity, and face area stability for plane, cylinder, cone, sphere, and torus.
4. Keep unsupported `None` behavior explicit; do not convert Rust surface extraction errors into OCCT payload helper retries.

## Guardrails

- Read `portingMilestones.md` and `nextStep.md` at the start of the next turn before editing.
- Do not reintroduce `face_bboxes_occt()`, `OffsetFaceBboxSource::OcctFaceUnion`, `offset_shape_bbox_occt()`, or `SummaryBboxSource::OffsetOcctSubshapeUnion`.
- Do not weaken `unsupported_bbox_summary_fallback_allowed()` or `unsupported_volume_summary_fallback_allowed()`.
- Preserve OCCT fallback only for `None`/unsupported descriptor cases in public query APIs. Once a Rust descriptor returns `Some(...)`, mismatched payload requests should fail explicitly in Rust instead of trying another OCCT helper.
- Do not replace a Rust curve extraction failure with `PortedCurve::from_context_with_geometry()` in public-route BRep code; make the unsupported/error distinction explicit.
- Do not reintroduce OCCT line, circle, or ellipse payload helper rescues into `PortedCurve::from_context_with_geometry()` or BRep edge materialization.
- Keep `ModelDocument::edges()`, `ModelDocument::faces()`, `select_edge()`, and `select_face()` centered on `BrepShape`.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
