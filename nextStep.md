# Next Task

Current milestone: `M5. BRep Curve Payload Fallback Cleanup` from `portingMilestones.md`.

## Completed Evidence

- `rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs` now makes `PortedCurve::from_context_with_ported_payloads()` return only Rust-derived line, circle, and ellipse payloads. Unsupported Rust extraction returns `None`; it no longer calls `edge_line_payload_occt()`, `edge_circle_payload_occt()`, or `edge_ellipse_payload_occt()`.
- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/brep_materialize.rs` and `face_topology.rs` no longer catch public-route Rust curve extraction errors and retry `PortedCurve::from_context_with_geometry()`.
- `rust/lean_occt/tests/brep_workflows.rs` now verifies exercised line, circle, and ellipse BRep edges populate `BrepEdge::ported_curve`, derive lengths from the ported curve, match the public Rust curve route, and stay close to OCCT samples/lengths.
- Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_exact_curve_bounding_boxes`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml`.

## Target

Replace or strictly narrow the remaining BRep curve materialization fallbacks where Rust edge geometry has identified an exercised line, circle, or ellipse but root-edge reconstruction can still rescue through `PortedCurve::from_context_with_geometry()`.

## Next Bounded Cut

1. In `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_snapshot.rs`, remove the planar wire reconstruction rescue that catches `PortedCurve::from_context_with_ported_payloads()` errors and retries `PortedCurve::from_context_with_geometry()`.
2. In `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/swept_face.rs`, replace `append_root_edge_sample_points()` direct use of `PortedCurve::from_context_with_geometry()` with the Rust-owned payload route where line/circle/ellipse extraction succeeds, keeping sample fallback only for unsupported `None`.
3. Strengthen `rust/lean_occt/tests/brep_workflows.rs` around the holed planar face and swept/offset roots so root-edge area/sample reconstruction remains green without OCCT payload rescue.
4. Leave the explicitly raw `FaceSurfaceRoute::Raw` branch in `face_topology.rs` for a later raw-route cut unless this bounded work needs the same helper to stay coherent.

## Guardrails

- Read `portingMilestones.md` and `nextStep.md` at the start of the next turn before editing.
- Do not reintroduce `face_bboxes_occt()`, `OffsetFaceBboxSource::OcctFaceUnion`, `offset_shape_bbox_occt()`, or `SummaryBboxSource::OffsetOcctSubshapeUnion`.
- Do not weaken `unsupported_bbox_summary_fallback_allowed()` or `unsupported_volume_summary_fallback_allowed()`.
- Preserve OCCT fallback only for `None`/unsupported descriptor cases in public query APIs. Once a Rust descriptor returns `Some(...)`, mismatched payload requests should fail explicitly in Rust instead of trying another OCCT helper.
- Do not replace a Rust curve extraction failure with `PortedCurve::from_context_with_geometry()` in public-route BRep code; make the unsupported/error distinction explicit.
- Keep `ModelDocument::edges()`, `ModelDocument::faces()`, `select_edge()`, and `select_face()` centered on `BrepShape`.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
