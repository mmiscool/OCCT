# Next Task

Current milestone: `M46. Rust-Owned Metadata-Backed Offset Face Geometry Entry` from `portingMilestones.md`.

## Completed Evidence

- `M45. Rust-Owned Offset Result Attachment Kind Gate` is complete.
- `attach_single_face_offset_metadata()` no longer calls `context.face_geometry_occt(&face_shape)?.kind`; it now validates the retained metadata with `Context::ported_offset_surface_from_metadata()` and attaches it only when the generated face reconstructs a Rust-owned offset descriptor.
- `attach_multi_face_offset_metadata()` no longer performs a raw generated-face kind gate. Every retained metadata candidate, including signed-offset variants, reaches the existing Rust descriptor validation and deterministic match scoring.
- Non-offset, unsupported, invalid, or tied generated faces remain unchanged because candidate validation returns no unique match.
- Single-face regression coverage was strengthened: `ported_brep_uses_rust_owned_area_for_offset_faces` now asserts the generated offset subshape carries Rust-retained metadata before public payload queries and that the public payload mirrors the attached descriptor.
- Source guard passed: no `face_geometry_occt(` call remains between `attach_single_face_offset_metadata()` and `offset_metadata_match_score()`.
- Verification passed with `(cd rust/lean_occt && cargo fmt)`, `cmake --build build --target LeanOcctCAPI`, `(cd rust/lean_occt && cargo check)`, focused offset attachment/basis/sampling/BRep regressions, the attachment source guard, full `(cd rust/lean_occt && cargo test)`, and `git diff --check`.

## Target

Move the next exercised metadata-backed offset face geometry entry out of the raw face-geometry classifier.

`Context::ported_face_geometry()` in `rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs` still starts with:

```rust
let raw_geometry = self.face_geometry_occt(shape)?;
```

After M45, direct offset faces and generated single-/multi-face offset faces carry retained Rust offset metadata only after Rust descriptor validation. Those metadata-backed offset faces should be able to return `FaceGeometry { kind: SurfaceKind::Offset, ... }` from retained Rust basis geometry before `ported_face_geometry()` reaches the raw classifier.

## Next Bounded Cut

1. Read `portingMilestones.md` and `nextStep.md` before editing.
2. Inspect `Context::ported_face_geometry()`, `Context::ported_offset_surface_from_metadata()`, and the offset geometry assertions in `ported_geometry_workflows.rs` and `brep_workflows.rs`.
3. Add a helper in `ported_geometry.rs` that checks `shape.offset_surface_face_metadata()`, validates it with `Context::ported_offset_surface_from_metadata()`, and derives an offset `FaceGeometry` from the validated `PortedOffsetSurface::basis_geometry` with `kind: SurfaceKind::Offset`.
4. Call that helper at the top of `Context::ported_face_geometry()` before `self.face_geometry_occt(shape)?`.
5. Treat invalid metadata, non-face roots, unsupported generated faces, and metadata-free shapes as `Ok(None)` from the new helper so the existing behavior remains available outside the ported metadata-backed family.
6. Preserve the remaining raw classifier for metadata-free and currently unsupported families; do not remove explicit raw/oracle APIs.
7. Strengthen runtime coverage if current offset geometry tests do not prove direct, single-face generated, and multi-face generated metadata-backed offset faces still expose public/ported offset geometry.
8. Add source guards proving the pre-raw branch contains metadata-backed validation and no direct `face_geometry_occt()` call.
9. Update both control files with completed evidence, active milestone, next bounded cut, and exact verification commands.

## Guardrails

- Do not call `face_geometry_occt()` before serving metadata-backed offset face geometry.
- Do not classify metadata-free, imported, invalid, or unsupported faces as Rust-owned offsets just because metadata validation failed or the shape has offset result metadata on a non-face root.
- Derive the returned offset geometry from validated Rust metadata, not from raw generated-face kind reads.
- Keep explicit raw face geometry, offset payload, offset basis, and sampling oracle APIs available.
- Preserve public face geometry parity for all currently exercised analytic, swept, direct offset, single-face generated offset, and multi-face generated offset fixtures.

## Verification

- `(cd rust/lean_occt && cargo fmt)`
- `cmake --build build --target LeanOcctCAPI`
- `(cd rust/lean_occt && cargo check)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows ported_face_surface_descriptors_cover_supported_faces -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows public_offset_basis_queries_match_occt -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows ported_offset_surface_sampling_matches_occt -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test brep_workflows ported_brep_maps_multi_source_swept_offsets_in_rust -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test brep_workflows ported_brep_uses_rust_owned_volume_for_offset_solids -- --nocapture)`
- `perl -0ne 'print $1 if /(pub fn ported_face_geometry[\s\S]*?)\n        let raw_geometry = self\.face_geometry_occt/' rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs | rg -n 'offset_surface_face_metadata|ported_offset_surface_from_metadata'`
- `! perl -0ne 'print $1 if /(pub fn ported_face_geometry[\s\S]*?)\n        let raw_geometry = self\.face_geometry_occt/' rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs | rg -n 'face_geometry_occt\('`
- `(cd rust/lean_occt && cargo test)`
- `git diff --check`
