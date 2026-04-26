# Next Task

Current milestone: `M4. Public Query Fallback Cleanup` from `portingMilestones.md`.

## Completed Evidence

- `rust/lean_occt/src/lib.rs` now serves `edge_line_payload()`, `edge_circle_payload()`, and `edge_ellipse_payload()` from matching `PortedCurve` descriptors. A supported but non-matching curve descriptor now returns an explicit Rust mismatch error instead of falling through to `edge_*_payload_occt()`, and only `None` reaches OCCT.
- `rust/lean_occt/src/lib.rs` applies the same rule to primitive analytic face payloads: `face_plane_payload()`, `face_cylinder_payload()`, `face_cone_payload()`, `face_sphere_payload()`, and `face_torus_payload()` return matching `PortedSurface` payloads, reject non-matching supported descriptors in Rust, and preserve OCCT fallback only for `None`.
- `rust/lean_occt/tests/ported_geometry_workflows.rs` now compares public analytic edge and face payload APIs against the ported descriptors for exercised line, circle, ellipse, plane, cylinder, cone, sphere, and torus cases, with representative mismatched supported-kind assertions proving those requests fail before OCCT.
- Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml`.

## Target

Replace or strictly narrow broad public geometry query fallbacks where a Rust descriptor already identifies a supported analytic, swept, or offset kind but the public API can still fall through to an OCCT payload helper.

## Next Bounded Cut

1. In `rust/lean_occt/src/lib.rs`, tighten `face_revolution_payload()` and `face_extrusion_payload()` so matching `PortedFaceSurface::Swept` descriptors return Rust-owned payloads, supported non-matching descriptors return explicit Rust errors, and only `None` reaches the OCCT fallback.
2. Apply the same pattern to the first offset-basis payload family backed by `ported_offset_surface()`: `face_offset_basis_plane_payload()`, `face_offset_basis_cylinder_payload()`, `face_offset_basis_revolution_payload()`, and `face_offset_basis_extrusion_payload()` should not fall through to OCCT after Rust has already identified a different supported offset basis kind.
3. Strengthen `rust/lean_occt/tests/ported_geometry_workflows.rs` so public swept and offset-basis payload APIs are asserted against the ported descriptors, including representative mismatched supported-kind requests that fail without relying on OCCT.
4. Keep offset basis curve-payload cleanup bounded for a follow-up if the surface/swept offset cut is already large; do not weaken any M2/M3 guards to make public payload tests pass.

## Guardrails

- Read `portingMilestones.md` and `nextStep.md` at the start of the next turn before editing.
- Do not reintroduce `face_bboxes_occt()`, `OffsetFaceBboxSource::OcctFaceUnion`, `offset_shape_bbox_occt()`, or `SummaryBboxSource::OffsetOcctSubshapeUnion`.
- Do not weaken `unsupported_bbox_summary_fallback_allowed()` or `unsupported_volume_summary_fallback_allowed()`.
- Preserve OCCT fallback only for `None`/unsupported descriptor cases in public query APIs. Once a Rust descriptor returns `Some(...)`, mismatched payload requests should fail explicitly in Rust instead of trying another OCCT helper.
- Keep `ModelDocument::edges()`, `ModelDocument::faces()`, `select_edge()`, and `select_face()` centered on `BrepShape`.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
