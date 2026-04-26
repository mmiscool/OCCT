# Next Task

Current milestone: `M5. BRep Curve Payload Fallback Cleanup` from `portingMilestones.md`.

## Completed Evidence

- `rust/lean_occt/src/lib.rs` now serves `face_offset_basis_curve_geometry()`, `face_offset_basis_curve_line_payload()`, `face_offset_basis_curve_circle_payload()`, and `face_offset_basis_curve_ellipse_payload()` from matching `PortedOffsetSurface` swept-basis descriptors. Supported analytic offset bases now return explicit Rust-owned "no basis curve" errors, supported swept bases with non-matching curve kinds return explicit Rust mismatch errors, and only `None` reaches OCCT.
- `rust/lean_occt/tests/ported_geometry_workflows.rs` now compares exercised extrusion/revolution offset basis curve geometry and ellipse payloads against ported descriptors and OCCT, with line/circle mismatch assertions proving those requests fail before OCCT.
- `M4. Public Query Fallback Cleanup` is complete: public descriptor-backed analytic, swept, offset, offset-basis, and swept offset-basis curve query APIs now keep OCCT helper fallbacks isolated to `None` from the ported loaders.
- Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_swept_offset_basis_queries_match_occt`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml`.

## Target

Replace or strictly narrow BRep curve materialization fallbacks where Rust edge geometry has identified an exercised line, circle, or ellipse but `PortedCurve` construction can still rescue through OCCT payload helpers.

## Next Bounded Cut

1. In `rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs`, make `PortedCurve::from_context_with_ported_payloads()` return Rust-derived line/circle/ellipse payloads when the local extractor succeeds and return `None` when it cannot, instead of calling `edge_*_payload_occt()`.
2. In `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/brep_materialize.rs` and `face_topology.rs`, stop swallowing `from_context_with_ported_payloads()` errors into `PortedCurve::from_context_with_geometry()` for public-route BRep edge materialization. Unsupported curve kinds can stay `None`; extraction errors should stay visible.
3. Strengthen `rust/lean_occt/tests/brep_workflows.rs` so exercised line/circle/ellipse edges still populate `BrepEdge::ported_curve`, derived lengths match OCCT/topology expectations, and the path remains green without the OCCT payload rescue.
4. Keep raw-route fallback users of `PortedCurve::from_context_with_geometry()` bounded for a later cut if public-route BRep materialization is already a large change.

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
