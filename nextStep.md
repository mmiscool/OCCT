# Next Task

Current milestone: `M8. Public Geometry and Sampling Fallback Narrowing` from `portingMilestones.md`.

## Completed Evidence

- `rust/lean_occt/src/lib.rs` public analytic face payload wrappers (`face_plane_payload()`, `face_cylinder_payload()`, `face_cone_payload()`, `face_sphere_payload()`, and `face_torus_payload()`) use `ported_analytic_face_surface_payload()` and no longer silently call direct analytic `face_*_payload_occt()` helpers after Rust descriptor extraction returns `None`.
- `rust/lean_occt/src/lib.rs` public swept face payload wrappers (`face_revolution_payload()` and `face_extrusion_payload()`) use `ported_swept_face_surface_payload()` and no longer call direct swept `face_*_payload_occt()` helpers after Rust descriptor extraction returns `None`.
- `rust/lean_occt/src/lib.rs` public top-level offset payload wrapper (`face_offset_payload()`) uses `ported_offset_face_surface_payload()` and no longer calls direct `face_offset_payload_occt()` after Rust descriptor extraction returns `None`.
- `rust/lean_occt/src/lib.rs` public offset-basis face-surface wrappers (`face_offset_basis_geometry()`, analytic basis payloads, and swept basis payloads) use `ported_offset_face_surface_payload()` and no longer call direct `face_offset_basis_*_occt()` helpers after Rust descriptor extraction returns `None`.
- `rust/lean_occt/src/lib.rs` public offset-basis curve wrappers (`face_offset_basis_curve_geometry()`, `face_offset_basis_curve_line_payload()`, `face_offset_basis_curve_circle_payload()`, and `face_offset_basis_curve_ellipse_payload()`) use `ported_offset_face_surface_payload()` and no longer call direct `face_offset_basis_curve_*_occt()` helpers after Rust descriptor extraction returns `None`.
- `rust/lean_occt/src/lib.rs` public edge payload wrappers (`edge_line_payload()`, `edge_circle_payload()`, and `edge_ellipse_payload()`) now use `ported_edge_curve_payload()` and `PortedCurve::from_context_with_ported_payloads()`. Supported line/circle/ellipse edge payload queries require a Rust `PortedCurve` descriptor and no longer call direct `edge_*_payload_occt()` helpers after descriptor extraction returns `None`.
- Explicit raw `*_payload_occt()` methods remain available as opt-in parity/oracle APIs, and `rust/lean_occt/tests/ported_geometry_workflows.rs` uses them only after requiring matching Rust descriptors for the public payload paths.
- `rust/lean_occt/tests/ported_geometry_workflows.rs::public_analytic_curve_and_surface_payload_queries_match_occt` now requires Rust line/circle/ellipse descriptors before public edge payload comparisons, compares public edge payloads to those descriptors before explicit OCCT oracles, and covers line/circle/ellipse mismatch errors.
- A full payload-fallback scan found no remaining `None => self.*payload_occt(shape)` branches in `rust/lean_occt/src/lib.rs`.
- Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `! rg -n 'None => self.edge_(line|circle|ellipse)_payload_occt\\(shape\\)|edge_(line|circle|ellipse)_payload_occt\\(shape\\)' rust/lean_occt/src/lib.rs`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_analytic_curve_and_surface_payload_queries_match_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_curve_sampling_matches_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`, `git diff --check`, and `! rg -n 'None => self\\.[A-Za-z0-9_]*payload_occt\\(shape\\)|payload_occt\\(shape\\)' rust/lean_occt/src/lib.rs`.

## Target

Remove or strictly narrow the next public fallback family: `rust/lean_occt/src/lib.rs::edge_geometry()`, `edge_endpoints()`, `edge_sample()`, and `edge_sample_at_parameter()` still call direct OCCT helpers when Rust edge geometry, curve sampling, or topology extraction returns `None`.

## Next Bounded Cut

1. Start with the public edge geometry/sampling family in `rust/lean_occt/src/lib.rs`: `edge_geometry()`, `edge_endpoints()`, `edge_sample()`, and `edge_sample_at_parameter()`.
2. Require Rust `PortedCurve` and BRep topology descriptors before public line/circle/ellipse edge geometry, endpoints, and samples can succeed for supported edge kinds; do not silently rescue through direct OCCT helpers after ported extraction returns `None`.
3. Preserve the explicit `edge_geometry_occt()`, `edge_endpoints_occt()`, `edge_sample_occt()`, and `edge_sample_at_parameter_occt()` APIs as opt-in parity/oracle APIs.
4. Strengthen `ported_curve_sampling_matches_occt` so public edge geometry/endpoints/samples are compared against Rust descriptors before explicit OCCT oracle comparison.

## Guardrails

- Read `portingMilestones.md` and `nextStep.md` at the start of the next turn before editing.
- Do not reintroduce `face_bboxes_occt()`, `OffsetFaceBboxSource::OcctFaceUnion`, `offset_shape_bbox_occt()`, or `SummaryBboxSource::OffsetOcctSubshapeUnion`.
- Do not weaken `unsupported_bbox_summary_fallback_allowed()` or `unsupported_volume_summary_fallback_allowed()`.
- Do not reintroduce direct `*_payload_occt()` fallbacks into public payload wrappers; explicit raw payload APIs may remain available only as opt-in parity/oracle APIs.
- Once a Rust descriptor returns `Some(...)`, mismatched payload or geometry requests should fail explicitly in Rust instead of trying another OCCT helper.
- Do not reintroduce OCCT line, circle, or ellipse payload helper rescues into `PortedCurve::from_context_with_geometry()` or BRep edge materialization.
- Do not reintroduce OCCT plane, cylinder, cone, sphere, or torus payload helper rescues into `PortedSurface::from_context_with_geometry()`, `ported_face_geometry()`, BRep face materialization, or planar face snapshot reconstruction.
- Do not reintroduce `face_extrusion_payload_occt()` or `face_revolution_payload_occt()` inside `brep/swept_face.rs` or the public swept payload wrappers.
- Do not reintroduce direct `face_offset_payload_occt()`, `face_offset_basis_*_occt()`, or `face_offset_basis_curve_*_occt()` fallbacks inside public wrappers narrowed under M7.
- Keep `ModelDocument::edges()`, `ModelDocument::faces()`, `select_edge()`, and `select_face()` centered on `BrepShape`.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `! rg -n 'None => self.edge_(endpoints|sample|sample_at_parameter|geometry)_occt\\(' rust/lean_occt/src/lib.rs`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_curve_sampling_matches_occt -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_analytic_curve_and_surface_payload_queries_match_occt -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- `git diff --check`
