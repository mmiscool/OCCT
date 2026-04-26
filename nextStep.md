# Next Task

Current milestone: `M6. BRep Surface Payload Fallback Cleanup` from `portingMilestones.md`.

## Completed Evidence

- `M5. BRep Curve Payload Fallback Cleanup` is complete.
- `rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs::PortedSurface::from_context_with_geometry()` now delegates to `from_context_with_ported_payloads()`.
- `PortedSurface::from_context_with_ported_payloads()` returns only Rust-derived plane, cylinder, cone, sphere, and torus payloads; unsupported extraction returns `None`, and it no longer calls `face_plane_payload_occt()`, `face_cylinder_payload_occt()`, `face_cone_payload_occt()`, `face_sphere_payload_occt()`, or `face_torus_payload_occt()`.
- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_prepare.rs` now prepares BRep face surfaces through `PortedSurface::from_context_with_ported_payloads()` so analytic BRep faces no longer rescue through `PortedSurface::from_context_with_geometry()`.
- `rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs::Context::ported_offset_surface()` now routes plane, cylinder, cone, sphere, and torus offset-basis descriptors through Rust-owned `ported_offset_basis_surface_payload()` instead of direct `face_offset_basis_*_payload_occt()` helpers.
- `rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry/payloads.rs` now shares sampler-driven analytic payload builders so offset-basis samples are reconstructed from the offset face by subtracting the signed offset along the natural normal.
- Raw `Geom_OffsetSurface` faces are preserved as `SurfaceKind::Offset` by `ported_face_geometry()` instead of being reclassified by analytic probing.
- `Context::make_offset_surface_face()` and the matching C ABI fixture create natural trimmed offset-surface faces for plane/cylinder/cone/sphere/torus regression coverage.
- Natural no-loop BRep faces now compute rectangular analytic areas in Rust.
- `rust/lean_occt/tests/brep_workflows.rs::ported_brep_uses_rust_owned_area_for_offset_faces`, `rust/lean_occt/tests/ported_geometry_workflows.rs::ported_face_surface_descriptors_cover_supported_faces`, and `rust/lean_occt/tests/ported_geometry_workflows.rs::public_offset_basis_queries_match_occt` now cover analytic offset basis descriptors, sample parity, and area parity.
- Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cmake --build build --target LeanOcctCAPI`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_face_surface_descriptors_cover_supported_faces`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_offset_basis_queries_match_occt`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`, and `git diff --check`.

## Target

Replace or strictly narrow the remaining swept offset-basis surface descriptor fallback: `rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs::Context::ported_offset_surface()` still constructs exercised revolution and extrusion offset basis descriptors with direct `face_offset_basis_revolution_payload_occt()`, `face_offset_basis_extrusion_payload_occt()`, and `face_offset_basis_curve_*_payload_occt()` helpers after the offset face descriptor has been identified.

## Next Bounded Cut

1. Factor or reuse swept basis reconstruction so an offset face can produce revolution/extrusion basis payloads and basis-curve payloads from offset samples plus basis geometry without calling the direct offset-basis OCCT payload helpers.
2. In `rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs::Context::ported_offset_surface()`, replace the revolution and extrusion branches with the Rust-owned swept offset-basis route. Keep unsupported or failed Rust extraction as explicit `None`; do not retry the OCCT swept offset-basis payload helpers for exercised swept branches.
3. Strengthen `rust/lean_occt/tests/ported_geometry_workflows.rs::public_offset_basis_queries_match_occt` and descriptor coverage around offset revolution/extrusion so exercised swept offset descriptors assert basis variant, basis-curve payload parity, descriptor sample parity, and area stability.
4. If BRep offset coverage exposes the same swept direct-offset fixtures cleanly, extend `rust/lean_occt/tests/brep_workflows.rs::ported_brep_uses_rust_owned_area_for_offset_faces`; otherwise record the precise fixture blocker and keep the tested public descriptor cut focused.

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
- `cmake --build build --target LeanOcctCAPI`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_face_surface_descriptors_cover_supported_faces`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_offset_basis_queries_match_occt`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- `git diff --check`
