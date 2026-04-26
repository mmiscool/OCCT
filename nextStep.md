# Next Task

Current milestone: `M49. Rust-Owned Face UV Bounds Seed Narrowing` from `portingMilestones.md`.

## Completed Evidence

- M49 direct single-face swept seed cut remains complete for exercised edge-source `make_prism()` and `make_revolution()` faces.
- M49 multi-face swept seed cut is now complete for the exercised face-source `make_revolution()` side-face path. Face-source sweeps retain conservative `MultiFaceSweptResult` inventories from Rust-owned ellipse edge geometry while edge-source sweeps keep `SingleFaceSweptResult`.
- Ported topology now propagates multi-face swept inventories through root and shell loading, validates each generated face against candidate swept seeds, and attaches `SweptSurfaceFaceMetadata` only when exactly one seed matches.
- The swept metadata helper validates normalized OCCT sample positions and oriented normals before accepting a Rust UV seed. This keeps caps, analytic faces, unsupported faces, imported faces, and ambiguous/domain-mismatched faces off swept metadata.
- `Context::ported_face_geometry()` still consumes swept metadata before `face_uv_bounds_occt(shape)`. The face-source revolution side-face regression now proves the tagged face returns geometry and samples through the Rust path; the face-source prism fixture explicitly guards that its current caps/analytic side faces remain untagged.
- Verification passed with the exact commands listed below, including the full Rust suite and `git diff --check`.

## Target

Narrow the remaining M49 raw bounds fallback beyond swept side-face metadata.

`face_uv_bounds_occt(shape)` still seeds constructor-owned analytic faces, caps, imported faces, unsupported faces, and metadata-free faces. The next useful cut is to carry Rust-owned analytic UV seeds onto a validated generated face inventory while preserving the raw bounds path for unsupported or ambiguous cases.

## Next Bounded Cut

1. Read `portingMilestones.md` and `nextStep.md` before editing.
2. Extend constructor-owned face metadata to the first analytic UV-seed family, preferably exercised `make_box()` planar faces or the planar/cylindrical cap-side faces that still hit `face_uv_bounds_occt(shape)`.
3. Derive the generated face geometry seeds from Rust construction data, propagate the inventory through topology loading, and attach metadata only after normalized sample/orientation validation against the generated face.
4. Keep imported, unsupported, cap/side cases not covered by the new analytic seed, and ambiguous matches on the existing raw bounds path.
5. Strengthen regression/source coverage so the newly tagged analytic faces classify before `face_uv_bounds_occt(shape)` without weakening the swept metadata guards.
6. Update both control files with completed evidence, active milestone, next bounded cut, and exact verification commands.

## Guardrails

- Do not reintroduce `face_geometry_occt(shape)` inside `Context::ported_face_geometry()`.
- Keep offset metadata first, swept metadata before `face_uv_bounds_occt(shape)`, and analytic candidates before the generic swept recognizer.
- Do not attach swept or analytic metadata to metadata-free, imported, invalid, unsupported, cap/side faces outside the current cut, or ambiguous faces.
- Preserve explicit raw/oracle face geometry, UV bounds, swept payload, offset payload, basis, and sampling APIs.
- Keep direct swept, multi-face swept, public payload, swept BRep solid, swept-offset metadata, multi-source swept offset, offset-solid volume, source-guard, and full-suite regressions green.

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
