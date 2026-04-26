# Next Task

Current milestone: `M6. BRep Surface Payload Fallback Cleanup` from `portingMilestones.md`.

## Completed Evidence

- `M5. BRep Curve Payload Fallback Cleanup` is complete.
- The M6 analytic BRep face materialization cut is complete: `face_prepare.rs` prepares analytic faces through `PortedSurface::from_context_with_ported_payloads()`, and `PortedSurface::from_context_with_geometry()` no longer rescues plane, cylinder, cone, sphere, or torus payload extraction through direct `face_*_payload_occt()` helpers.
- The analytic and swept offset-basis descriptor cuts are complete: `Context::ported_offset_surface()` builds supported analytic and swept basis descriptors through Rust-owned sample reconstruction and no longer retries the direct offset-basis payload helpers for exercised plane/cylinder/cone/sphere/torus/extrusion/revolution branches.
- The non-offset swept BRep descriptor cut is complete: `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/swept_face.rs::ported_extrusion_face_surface()` and `ported_revolution_face_surface()` now derive extrusion direction, revolution axis, basis curve kind, and line/circle/ellipse basis curves from Rust-owned BRep topology plus face samples instead of `face_extrusion_payload_occt()` and `face_revolution_payload_occt()`.
- Full `TAU` revolution faces now resolve the swept basis/sweep parameter tie in Rust, keeping document-level revolution descriptors on the ported evaluator path.
- `rust/lean_occt/tests/brep_workflows.rs::ported_brep_summarizes_swept_revolution_solids_in_rust` now asserts swept extrusion and revolution descriptor payload parity, public descriptor parity, OCCT sample parity, selected basis kind, and area stability.
- Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_summarizes_swept_revolution_solids_in_rust -- --nocapture`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_swept_and_offset_payload_queries_match_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_face_surface_descriptors_cover_supported_faces -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows document_runs_analytic_shape_pipeline -- --nocapture`, `cmake --build build --target LeanOcctCAPI`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`, and `git diff --check`.

## Target

Remove or strictly narrow the next remaining M6 surface-payload fallback: `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_snapshot.rs` still accepts `face_plane_payload_occt()` for supported planar multi-wire root-face snapshot reconstruction after Rust plane extraction fails.

## Next Bounded Cut

1. Reuse the Rust-owned plane payload extraction path for planar multi-wire `PreparedFaceTopologyBuilder` setup in `brep/face_snapshot.rs`.
2. Replace the `context.face_plane_payload(face_shape).is_err() && context.face_plane_payload_occt(face_shape).is_err()` gate with an explicit Rust-owned supported/unsupported distinction, and do not retry `face_plane_payload_occt()` for supported planar faces.
3. Strengthen document or BRep coverage around a supported multi-wire planar face so face snapshot packing still resolves wire roles and areas without the OCCT plane payload helper.
4. Keep the cut behavior-owned: if a prerequisite helper is needed, land it in the same turn as the fallback removal and regression coverage.

## Guardrails

- Read `portingMilestones.md` and `nextStep.md` at the start of the next turn before editing.
- Do not reintroduce `face_bboxes_occt()`, `OffsetFaceBboxSource::OcctFaceUnion`, `offset_shape_bbox_occt()`, or `SummaryBboxSource::OffsetOcctSubshapeUnion`.
- Do not weaken `unsupported_bbox_summary_fallback_allowed()` or `unsupported_volume_summary_fallback_allowed()`.
- Preserve OCCT fallback only for `None`/unsupported descriptor cases in public query APIs. Once a Rust descriptor returns `Some(...)`, mismatched payload requests should fail explicitly in Rust instead of trying another OCCT helper.
- Do not replace a Rust curve extraction failure with `PortedCurve::from_context_with_geometry()` in public-route BRep code; make the unsupported/error distinction explicit.
- Do not reintroduce OCCT line, circle, or ellipse payload helper rescues into `PortedCurve::from_context_with_geometry()` or BRep edge materialization.
- Do not reintroduce OCCT plane, cylinder, cone, sphere, or torus payload helper rescues into `PortedSurface::from_context_with_geometry()` or BRep face materialization.
- Do not reintroduce `face_extrusion_payload_occt()` or `face_revolution_payload_occt()` inside `brep/swept_face.rs`.
- Keep `ModelDocument::edges()`, `ModelDocument::faces()`, `select_edge()`, and `select_face()` centered on `BrepShape`.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cmake --build build --target LeanOcctCAPI`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_summarizes_swept_revolution_solids_in_rust -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows document_runs_analytic_shape_pipeline -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_face_surface_descriptors_cover_supported_faces`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_swept_and_offset_payload_queries_match_occt`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- `git diff --check`
