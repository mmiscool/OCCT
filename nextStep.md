# Next Task

Current milestone: `M6. BRep Surface Payload Fallback Cleanup` from `portingMilestones.md`.

## Completed Evidence

- `M5. BRep Curve Payload Fallback Cleanup` is complete.
- `rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs::PortedSurface::from_context_with_geometry()` now delegates to `from_context_with_ported_payloads()`.
- `PortedSurface::from_context_with_ported_payloads()` returns only Rust-derived plane, cylinder, cone, sphere, and torus payloads; unsupported extraction returns `None`, and it no longer calls `face_plane_payload_occt()`, `face_cylinder_payload_occt()`, `face_cone_payload_occt()`, `face_sphere_payload_occt()`, or `face_torus_payload_occt()`.
- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_prepare.rs` now prepares BRep face surfaces through `PortedSurface::from_context_with_ported_payloads()` so analytic BRep faces no longer rescue through `PortedSurface::from_context_with_geometry()`.
- `rust/lean_occt/tests/brep_workflows.rs::ported_brep_uses_exact_primitive_surface_and_volume_formulas` now asserts exact primitive plane, cylinder, cone, sphere, and torus faces populate `BrepFace::ported_surface` and analytic `ported_face_surface`, exercises the raw-geometry `PortedSurface::from_context_with_geometry()` route, and checks descriptor/raw-route sample parity plus per-face area stability.
- Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_exact_primitive_surface_and_volume_formulas`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`, and `git diff --check`.

## Target

Replace or strictly narrow the next surface descriptor fallback: `rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs::Context::ported_offset_surface()` still constructs exercised offset basis descriptors with direct `face_offset_basis_*_payload_occt()` helpers after the offset face descriptor has been identified.

## Next Bounded Cut

1. In `rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry/payloads.rs`, factor the analytic surface payload builders enough to support a sampler that can produce basis-surface samples for an offset face by subtracting `offset * normal` from offset-surface samples at the basis UV.
2. In `rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs::Context::ported_offset_surface()`, replace the plane/cylinder/cone/sphere/torus `face_offset_basis_*_payload_occt()` branches with the Rust-owned offset-basis payload route. Keep unsupported or failed Rust extraction as explicit `None`; do not retry the OCCT basis payload helpers for those analytic branches.
3. Strengthen `rust/lean_occt/tests/brep_workflows.rs::ported_brep_uses_rust_owned_area_for_offset_faces` and the offset descriptor coverage in `rust/lean_occt/tests/ported_geometry_workflows.rs` so exercised `PortedFaceSurface::Offset` descriptors assert their analytic/swept basis variant, sample parity, and area stability without the direct OCCT basis payload helper rescue.
4. Leave the swept offset-basis curve payload helper cleanup as the following cut unless the analytic-basis refactor naturally exposes the same route for extrusion/revolution basis curves in this turn.

## Guardrails

- Read `portingMilestones.md` and `nextStep.md` at the start of the next turn before editing.
- Do not reintroduce `face_bboxes_occt()`, `OffsetFaceBboxSource::OcctFaceUnion`, `offset_shape_bbox_occt()`, or `SummaryBboxSource::OffsetOcctSubshapeUnion`.
- Do not weaken `unsupported_bbox_summary_fallback_allowed()` or `unsupported_volume_summary_fallback_allowed()`.
- Preserve OCCT fallback only for `None`/unsupported descriptor cases in public query APIs. Once a Rust descriptor returns `Some(...)`, mismatched payload requests should fail explicitly in Rust instead of trying another OCCT helper.
- Do not replace a Rust curve extraction failure with `PortedCurve::from_context_with_geometry()` in public-route BRep code; make the unsupported/error distinction explicit.
- Do not reintroduce OCCT line, circle, or ellipse payload helper rescues into `PortedCurve::from_context_with_geometry()` or BRep edge materialization.
- Do not reintroduce OCCT plane, cylinder, cone, sphere, or torus payload helper rescues into `PortedSurface::from_context_with_geometry()` or BRep face materialization.
- Keep `ModelDocument::edges()`, `ModelDocument::faces()`, `select_edge()`, and `select_face()` centered on `BrepShape`.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_face_surface_descriptors_cover_supported_faces`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- `git diff --check`
