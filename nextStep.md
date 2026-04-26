# Next Task

Current milestone: `M44. Rust-Owned Offset Metadata Basis Support Gate` from `portingMilestones.md`.

## Completed Evidence

- `M43. Rust-Owned Public Face Payload Mismatch Gates` is complete.
- `Context::ported_analytic_face_surface_payload()` no longer calls `self.face_geometry_occt(shape)?` after `ported_face_surface()` returns `None`.
- `Context::ported_swept_face_surface_payload()` no longer calls `self.face_geometry_occt(shape)?` after `ported_face_surface_descriptor()` returns `None`.
- Both helpers now use `Context::ported_face_geometry()` for the remaining support/kind decision, preserving explicit mismatch errors for supported wrong-kind faces and returning an unsupported Rust payload error for unsupported families.
- The analytic helper reconstructs payloads through `PortedSurface::from_context_with_ported_payloads()` instead of the generic geometry constructor.
- `public_swept_and_offset_payload_queries_match_occt` now verifies that an extrusion face rejects a public analytic plane payload with the Rust-owned `requested Plane payload for ported Extrusion face` mismatch.
- Source guards passed for both helper bodies: no `face_geometry_occt(` call remains inside `ported_analytic_face_surface_payload()` or `ported_swept_face_surface_payload()`.
- Verification passed with `(cd rust/lean_occt && cargo fmt)`, `cmake --build build --target LeanOcctCAPI`, `(cd rust/lean_occt && cargo check)`, the focused public analytic/swept descriptor workflows, both source guards, full `(cd rust/lean_occt && cargo test)`, and `git diff --check`.

## Target

Remove the next raw face geometry support classifier in direct offset metadata construction:

`Context::offset_surface_face_metadata_candidate()` in `rust/lean_occt/src/lib.rs` currently calls `self.face_geometry_occt(basis_face)` before deciding whether a basis face can carry retained Rust `OffsetSurfaceFaceMetadata`. That read decides analytic/swept basis support for direct `make_offset_surface_face()` and multi-face offset result metadata; it is not an explicit raw oracle API.

The metadata candidate path should use Rust-owned ported face-geometry and descriptor validation for the support/kind decision. Explicit raw offset and face geometry APIs should remain available as oracle APIs.

## Next Bounded Cut

1. Read `portingMilestones.md` and `nextStep.md` before editing.
2. Inspect `offset_surface_face_metadata_candidate()` in `rust/lean_occt/src/lib.rs`.
3. Replace the `self.face_geometry_occt(basis_face)` support classifier with `self.ported_face_geometry(basis_face)?`.
4. Use the ported geometry kind for `offset_surface_face_metadata_supports_basis()` and for analytic/swept mismatch errors.
5. Preserve direct offset metadata attachment for supported plane, cylinder, cone, sphere, torus, extrusion, and revolution bases.
6. Preserve unsupported-family behavior as `Ok(None)` without widening Rust-required support.
7. Keep explicit raw `face_geometry_occt()`, offset payload, and offset basis oracle APIs available.
8. Strengthen offset metadata regression coverage if existing offset basis/sampling/BRep assertions do not fail on this support gate.
9. Add a source guard proving `offset_surface_face_metadata_candidate()` contains no `face_geometry_occt(` call.
10. Update both control files with completed evidence, active milestone, next bounded cut, and exact verification commands.

## Guardrails

- Do not route the metadata support decision through public `face_geometry()` if that would obscure whether the basis is Rust-owned.
- Do not use raw `face_geometry_occt()` as a support classifier in `offset_surface_face_metadata_candidate()`.
- Keep unsupported basis families outside retained Rust offset metadata unless the same turn ports their geometry and payload behavior.
- Preserve explicit mismatch and unsupported errors where Rust descriptors expose a supported actual kind.
- Keep raw OCCT helpers available only as explicit oracle APIs or unsupported-shape fallbacks outside this support gate.

## Verification

- `(cd rust/lean_occt && cargo fmt)`
- `cmake --build build --target LeanOcctCAPI`
- `(cd rust/lean_occt && cargo check)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows public_offset_basis_queries_match_occt -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows ported_offset_surface_sampling_matches_occt -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows ported_face_surface_descriptors_cover_supported_faces -- --nocapture)`
- `! awk '/fn offset_surface_face_metadata_candidate/,/^    }/' rust/lean_occt/src/lib.rs | rg -n 'face_geometry_occt\('`
- `(cd rust/lean_occt && cargo test)`
- `git diff --check`
