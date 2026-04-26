# Next Task

Current milestone: `M45. Rust-Owned Offset Result Attachment Kind Gate` from `portingMilestones.md`.

## Completed Evidence

- `M44. Rust-Owned Offset Metadata Basis Support Gate` is complete.
- `Context::offset_surface_face_metadata_candidate()` no longer calls `self.face_geometry_occt(basis_face)` before deciding whether retained `OffsetSurfaceFaceMetadata` applies.
- The metadata candidate path now starts with `self.ported_face_geometry(basis_face)`, returns `Ok(None)` for non-face roots or unsupported/unported families, and uses the ported geometry kind for `offset_surface_face_metadata_supports_basis()`.
- Analytic basis metadata still validates through `PortedSurface::from_context_with_ported_payloads()`, swept basis metadata still validates through `brep::ported_face_surface_descriptor()`, and mismatch/unsupported errors are based on the Rust-owned descriptor kind.
- Direct and generated offset metadata coverage was strengthened: `public_offset_basis_queries_match_occt` now requires Rust-owned source basis geometry for direct analytic, direct swept, and generated swept offset metadata sources.
- The full suite initially exposed the whole-source `make_offset()` caller path; the final candidate preserves the old `Ok(None)` behavior for non-face source roots so multi-face offset inventory discovery remains intact.
- Source guard passed: no `face_geometry_occt(` call remains in `offset_surface_face_metadata_candidate()`.
- Verification passed with `(cd rust/lean_occt && cargo fmt)`, `cmake --build build --target LeanOcctCAPI`, `(cd rust/lean_occt && cargo check)`, focused offset basis/sampling/BRep regressions, the multi-source swept-offset and offset-solid volume regressions, the source guard, full `(cd rust/lean_occt && cargo test)`, and `git diff --check`.

## Target

Remove the next raw face geometry generated-face kind gates in offset result metadata attachment:

`attach_single_face_offset_metadata()` and `attach_multi_face_offset_metadata()` in `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs` currently call `context.face_geometry_occt(&face_shape)?.kind` before attaching retained Rust offset metadata to generated faces. Those reads decide whether a generated face is eligible for Rust-owned offset metadata; they are not explicit raw oracle APIs.

The attachment path should validate candidate metadata through the Rust offset descriptor/match path instead of first asking raw OCCT for the face kind. Explicit raw offset, face geometry, and sampling APIs should remain available as oracle APIs.

## Next Bounded Cut

1. Read `portingMilestones.md` and `nextStep.md` before editing.
2. Inspect `attach_single_face_offset_metadata()`, `attach_multi_face_offset_metadata()`, and `offset_metadata_match_score()` in `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs`.
3. Replace the single-face raw generated-kind check with Rust-owned validation of the sole generated face against the retained metadata, likely by using `Context::ported_offset_surface_from_metadata()`.
4. Replace the multi-face raw generated-kind check by letting candidate scoring/validation decide whether a generated face can accept offset metadata.
5. Preserve deterministic multi-face assignment, tie rejection, and signed-offset handling.
6. Preserve non-offset or unsupported generated faces as unchanged shapes without widening Rust-required support.
7. Keep explicit raw face geometry, offset payload, offset basis, and sampling oracle APIs available.
8. Strengthen regression coverage if the existing multi-source swept-offset and offset-solid tests do not fail on this attachment gate.
9. Add a source guard proving the attachment gate contains no `face_geometry_occt(` call.
10. Update both control files with completed evidence, active milestone, next bounded cut, and exact verification commands.

## Guardrails

- Do not classify generated offset faces by calling `face_geometry_occt()` in the attachment functions.
- Do not attach retained metadata merely because a result has one face; validate that the candidate metadata reconstructs a ported offset surface for that generated face.
- Keep unsupported generated faces outside retained Rust offset metadata unless the same turn ports their behavior.
- Preserve explicit raw OCCT helpers as oracle APIs or unsupported fallback paths outside this attachment gate.
- Keep multi-face matching deterministic: ambiguous ties should continue to leave the face unmodified.

## Verification

- `(cd rust/lean_occt && cargo fmt)`
- `cmake --build build --target LeanOcctCAPI`
- `(cd rust/lean_occt && cargo check)`
- `(cd rust/lean_occt && cargo test --test brep_workflows ported_brep_maps_multi_source_swept_offsets_in_rust -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test brep_workflows ported_brep_uses_rust_owned_volume_for_offset_solids -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows public_offset_basis_queries_match_occt -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows ported_offset_surface_sampling_matches_occt -- --nocapture)`
- `! awk '/fn attach_single_face_offset_metadata/,/^fn offset_metadata_match_score/' rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs | rg -n 'face_geometry_occt\('`
- `(cd rust/lean_occt && cargo test)`
- `git diff --check`
