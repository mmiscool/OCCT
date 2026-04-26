# Next Task

Current milestone: `M49. Rust-Owned Face UV Bounds Seed Narrowing` from `portingMilestones.md`.

## Completed Evidence

- M49 direct single-face swept seed cut is complete for exercised edge-source `make_prism()` and `make_revolution()` faces.
- `make_prism()` and `make_revolution()` now retain `SingleFaceSweptResult` metadata when the source is a supported Rust-owned line/circle/ellipse edge. Extrusion seeds use the source edge parameter span on U and OCCT-compatible `-direction_length..0` bounds on V; revolution seeds use `0..angle_radians` on U and the source edge parameter span on V.
- Ported topology face enumeration attaches `SweptSurfaceFaceMetadata` only when a swept result exposes exactly one face, avoiding ambiguous multi-face side/cap inventories for this cut.
- `Context::ported_face_geometry()` now tries `ported_swept_surface_from_metadata_face_geometry(self, shape)?` before `face_uv_bounds_occt(shape)`. The metadata helper validates the swept descriptor from samples using the Rust seed and does not call the raw UV-bounds helper.
- `ported_swept_surface_sampling_matches_occt` asserts constructor-owned direct swept faces carry Rust swept metadata. `ported_face_geometry_classifies_swept_before_raw_geometry` proves metadata classification runs before the raw UV-bounds seed and blocks raw face-geometry/swept descriptor regressions.
- Verification passed with the exact commands listed below, including the full Rust suite and `git diff --check`.

## Target

Narrow the remaining M49 raw bounds fallback beyond direct single-face edge sweeps.

`face_uv_bounds_occt(shape)` still seeds metadata-free, imported, unsupported, analytic, and multi-face constructor-owned swept faces. The next useful cut is to carry Rust-owned swept UV seeds onto validated side faces from multi-face constructor-owned swept results while leaving caps on their current analytic/raw-seed path.

## Next Bounded Cut

1. Read `portingMilestones.md` and `nextStep.md` before editing.
2. Extend swept metadata inventory to validated multi-face constructor-owned swept side faces from face-source `make_revolution()` and `make_prism()` fixtures.
3. Derive source edge parameter spans and sweep ranges in Rust, match generated side faces by descriptor/sample validation, and attach `SweptSurfaceFaceMetadata` only to uniquely validated side faces.
4. Keep cap and analytic faces on the existing path until a separate analytic seed cut; do not attach swept metadata to caps, imported faces, unsupported faces, or ambiguous matches.
5. Strengthen regression/source coverage so matched multi-face swept side faces classify before `face_uv_bounds_occt(shape)`.
6. Update both control files with completed evidence, active milestone, next bounded cut, and exact verification commands.

## Guardrails

- Do not reintroduce `face_geometry_occt(shape)` inside `Context::ported_face_geometry()`.
- Keep offset metadata first, swept metadata before `face_uv_bounds_occt(shape)`, and analytic candidates before the generic swept recognizer.
- Do not silently attach swept metadata to metadata-free, imported, invalid, unsupported, cap, or ambiguous faces.
- Preserve explicit raw/oracle face geometry, UV bounds, swept payload, offset payload, basis, and sampling APIs.
- Keep direct swept, public payload, swept BRep solid, swept-offset metadata, multi-source swept offset, offset-solid volume, source-guard, and full-suite regressions green.

## Verification

- `(cd rust/lean_occt && cargo fmt)`
- `cmake --build build --target LeanOcctCAPI`
- `(cd rust/lean_occt && cargo check)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows ported_face_geometry_classifies_swept_before_raw_geometry -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows ported_face_surface_descriptors_cover_supported_faces -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows public_swept_and_offset_payload_queries_match_occt -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows ported_swept_surface_sampling_matches_occt -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test brep_workflows ported_brep_summarizes_swept_revolution_solids_in_rust -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test brep_workflows ported_brep_maps_multi_source_swept_offsets_in_rust -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test brep_workflows ported_brep_uses_rust_owned_volume_for_offset_solids -- --nocapture)`
- `perl -0ne 'print $1 if /(pub fn ported_face_geometry[\s\S]*?)\n\n    pub fn ported_edge_curve/' rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs | rg -n 'ported_swept_surface_from_metadata_face_geometry|face_uv_bounds_occt|ported_swept_face_geometry_candidate'`
- `! perl -0ne 'print $1 if /(pub fn ported_face_geometry[\s\S]*?)\n\n    pub fn ported_edge_curve/' rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs | rg -n 'face_geometry_occt\(shape\)|SurfaceKind::Revolution \| SurfaceKind::Extrusion|PortedFaceSurface::Swept|ported_face_surface_descriptor_value'`
- `(cd rust/lean_occt && cargo test)`
- `git diff --check`
