# Next Task

Current milestone: `M8. Public Geometry and Sampling Fallback Narrowing` from `portingMilestones.md`.

## Completed Evidence

- `rust/lean_occt/src/lib.rs` public analytic, swept, offset, offset-basis, and edge payload wrappers no longer silently call direct `*_payload_occt()` helpers after Rust descriptor extraction returns `None`; explicit raw payload APIs remain opt-in parity/oracle APIs.
- `rust/lean_occt/src/lib.rs` public edge geometry/sampling wrappers (`edge_geometry()`, `edge_endpoints()`, `edge_sample()`, and `edge_sample_at_parameter()`) now require Rust `ported_edge_*` extraction for supported `Line`, `Circle`, and `Ellipse` edges. If Rust extraction returns `None` for those supported curve kinds, the wrapper returns a Rust-owned unsupported error instead of silently calling direct `edge_*_occt()` helpers.
- The edge geometry/sampling OCCT escape hatch is now strictly limited to non-ported curve kinds such as Bezier/BSpline, which keeps face-free topology workflows covered without reopening the supported line/circle/ellipse fallback path.
- `rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs::ported_edge_geometry()` no longer rescues line/circle/ellipse reconstruction through direct `edge_line_payload_occt(shape)`, `edge_circle_payload_occt(shape)`, or `edge_ellipse_payload_occt(shape)` helpers.
- `rust/lean_occt/tests/ported_geometry_workflows.rs::ported_curve_sampling_matches_occt` now requires `ported_edge_geometry()` and `ported_edge_endpoints()` before comparing public edge geometry/endpoints/samples against explicit OCCT oracles.
- Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `! rg -n 'None => self.edge_(endpoints|sample|sample_at_parameter|geometry)_occt\\(' rust/lean_occt/src/lib.rs`, `! rg -n 'edge_(line|circle|ellipse)_payload_occt\\(shape\\)' rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_curve_sampling_matches_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_analytic_curve_and_surface_payload_queries_match_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`, and `git diff --check`.

## Target

Remove or strictly narrow the next public fallback family: `rust/lean_occt/src/lib.rs::face_geometry()`, `face_sample()`, and `face_sample_normalized()` still call direct OCCT helpers when Rust face geometry or surface sampling extraction returns `None`.

## Next Bounded Cut

1. Start with the public face geometry/sampling family in `rust/lean_occt/src/lib.rs`: `face_geometry()`, `face_sample()`, and `face_sample_normalized()`.
2. Require Rust `PortedFaceSurface` descriptors before public analytic, swept, and offset face geometry/samples can succeed for supported face kinds; do not silently rescue through direct OCCT helpers after ported extraction returns `None`.
3. Preserve the explicit `face_geometry_occt()`, `face_sample_occt()`, and `face_sample_normalized_occt()` APIs as opt-in parity/oracle APIs or strictly unsupported-surface escape hatches.
4. Strengthen `ported_surface_sampling_matches_occt`, `ported_swept_surface_sampling_matches_occt`, and `ported_offset_surface_sampling_matches_occt` so public face geometry/samples are compared against Rust descriptors before explicit OCCT oracle comparison.

## Guardrails

- Read `portingMilestones.md` and `nextStep.md` at the start of the next turn before editing.
- Do not reintroduce `face_bboxes_occt()`, `OffsetFaceBboxSource::OcctFaceUnion`, `offset_shape_bbox_occt()`, or `SummaryBboxSource::OffsetOcctSubshapeUnion`.
- Do not weaken `unsupported_bbox_summary_fallback_allowed()` or `unsupported_volume_summary_fallback_allowed()`.
- Do not reintroduce direct `*_payload_occt()` fallbacks into public payload wrappers; explicit raw payload APIs may remain available only as opt-in parity/oracle APIs.
- Do not reopen supported line/circle/ellipse public edge geometry or sampling fallbacks; direct `edge_*_occt()` helpers may only remain public oracles or unsupported curve-kind escape hatches.
- Once a Rust descriptor returns `Some(...)`, mismatched payload or geometry requests should fail explicitly in Rust instead of trying another OCCT helper.
- Do not reintroduce OCCT line, circle, or ellipse payload helper rescues into `PortedCurve::from_context_with_geometry()`, `ported_edge_geometry()`, or BRep edge materialization.
- Do not reintroduce OCCT plane, cylinder, cone, sphere, or torus payload helper rescues into `PortedSurface::from_context_with_geometry()`, `ported_face_geometry()`, BRep face materialization, or planar face snapshot reconstruction.
- Do not reintroduce `face_extrusion_payload_occt()` or `face_revolution_payload_occt()` inside `brep/swept_face.rs` or the public swept payload wrappers.
- Do not reintroduce direct `face_offset_payload_occt()`, `face_offset_basis_*_occt()`, or `face_offset_basis_curve_*_occt()` fallbacks inside public wrappers narrowed under M7.
- Keep `ModelDocument::edges()`, `ModelDocument::faces()`, `select_edge()`, and `select_face()` centered on `BrepShape`.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `! rg -n 'None => self.face_(sample|sample_normalized|geometry)_occt\\(' rust/lean_occt/src/lib.rs`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_surface_sampling_matches_occt -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_swept_surface_sampling_matches_occt -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_offset_surface_sampling_matches_occt -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_face_surface_descriptors_cover_supported_faces -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- `git diff --check`
