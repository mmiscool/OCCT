# Next Task

Current milestone: `M4. Public Query Fallback Cleanup` from `portingMilestones.md`.

## Completed Evidence

- `rust/lean_occt/src/lib.rs` now serves `face_revolution_payload()` and `face_extrusion_payload()` from matching `PortedFaceSurface::Swept` descriptors. A supported but non-matching face descriptor now returns an explicit Rust mismatch error instead of falling through to `face_*_payload_occt()`, and only `None` reaches OCCT.
- `rust/lean_occt/src/lib.rs` applies the same rule to offset-basis payloads backed by `ported_offset_surface()`: `face_offset_basis_plane_payload()`, `face_offset_basis_cylinder_payload()`, `face_offset_basis_cone_payload()`, `face_offset_basis_sphere_payload()`, `face_offset_basis_torus_payload()`, `face_offset_basis_revolution_payload()`, and `face_offset_basis_extrusion_payload()` return matching `PortedOffsetBasisSurface` payloads, reject non-matching supported basis descriptors in Rust, and preserve OCCT fallback only for `None`.
- `rust/lean_occt/tests/ported_geometry_workflows.rs` now compares public swept and offset-basis payload APIs against the ported descriptors and OCCT for exercised extrusion/revolution and swept-offset faces, with representative mismatched supported-kind assertions proving top-level swept requests and plane/cylinder/revolution/extrusion offset-basis requests fail before OCCT.
- Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml`.

## Target

Replace or strictly narrow broad public geometry query fallbacks where a Rust descriptor already identifies supported offset basis curve data but the public API can still fall through to an OCCT curve geometry or curve payload helper.

## Next Bounded Cut

1. In `rust/lean_occt/src/lib.rs`, tighten `face_offset_basis_curve_geometry()` so swept offset bases return `basis_geometry` from Rust, supported analytic bases or unsupported curve-bearing shapes return an explicit Rust error, and only `None` reaches the OCCT fallback.
2. Apply the same pattern to `face_offset_basis_curve_line_payload()`, `face_offset_basis_curve_circle_payload()`, and `face_offset_basis_curve_ellipse_payload()`: matching swept basis curves return Rust-owned payloads, supported non-matching curve kinds fail in Rust, and only `None` reaches OCCT.
3. Strengthen `rust/lean_occt/tests/ported_geometry_workflows.rs` so exercised swept offset faces assert matching basis curve geometry and ellipse payloads against ported descriptors and OCCT, including representative line/circle mismatch requests that fail without relying on OCCT.
4. Keep any future analytic offset-basis fixture expansion bounded for a follow-up; do not weaken any M2/M3 guards to make public payload tests pass.

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
