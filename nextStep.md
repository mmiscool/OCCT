# Next Task

Current milestone: `M47. Rust-Owned Offset Classifier Fallback Narrowing` from `portingMilestones.md`.

## Completed Evidence

- `M46. Rust-Owned Metadata-Backed Offset Face Geometry Entry` is complete.
- `Context::ported_face_geometry()` now calls `ported_offset_surface_from_metadata_face_geometry()` before `self.face_geometry_occt(shape)?`, so retained metadata-backed offset faces can return Rust-derived `FaceGeometry { kind: SurfaceKind::Offset, ... }` before the raw classifier.
- The new helper reads `shape.offset_surface_face_metadata()`, validates the candidate with `Context::ported_offset_surface_from_metadata()`, and derives the returned geometry from the validated `PortedOffsetSurface::basis_geometry`.
- Invalid metadata, non-face roots, unsupported generated faces, and metadata-free shapes return `Ok(None)` from the helper, preserving existing behavior outside the retained-metadata offset family.
- `OffsetSurfaceFaceMetadata` now carries `direct_surface_face` provenance. Direct `make_offset_surface_face()` metadata recomputes rectangular-trimmed periodic/closed flags from the retained parameter span, while generated offset-result metadata keeps generated-offset semantics.
- Swept offset geometry clears V-closed for extrusion and revolution bases to match OCCT offset-surface behavior.
- `public_offset_basis_queries_match_occt` now asserts retained metadata before ported offset geometry queries and covers revolution/direct-revolution closed and periodic semantics.
- `ported_brep_maps_multi_source_swept_offsets_in_rust` now asserts public/ported offset geometry parity for mapped multi-source generated offset faces.
- Verification passed with `(cd rust/lean_occt && cargo fmt)`, `cmake --build build --target LeanOcctCAPI`, `(cd rust/lean_occt && cargo check)`, focused M46 geometry/BRep regressions, the pre-raw source guards, and full `(cd rust/lean_occt && cargo test)`.

## Target

Narrow the remaining metadata-free offset classifier in `Context::ported_face_geometry()`.

M46 intentionally preserved this post-raw descriptor validation arm:

```rust
| (SurfaceKind::Offset, Some(PortedFaceSurface::Offset(_))) => Ok(Some(raw_geometry))
```

Direct removal currently breaks `ported_brep_uses_rust_owned_area_for_offset_faces` with a missing Rust-owned bbox summary, which means at least one exercised BRep offset summary path still enters `ported_face_geometry()` without usable retained `OffsetSurfaceFaceMetadata`.

## Next Bounded Cut

1. Read `portingMilestones.md` and `nextStep.md` before editing.
2. Temporarily remove or instrument the post-raw `SurfaceKind::Offset` arm in `Context::ported_face_geometry()` and reproduce the focused failure with `ported_brep_uses_rust_owned_area_for_offset_faces`.
3. Trace the failing face or subshape to determine whether it lacks `OffsetSurfaceFaceMetadata`, loses attached metadata during topology/subshape propagation, or represents a genuinely metadata-free unsupported/imported offset.
4. If the face is part of an exercised generated offset result, propagate or attach validated Rust metadata before BRep summary geometry is queried.
5. If the face is genuinely metadata-free, split the fallback so it remains an explicit unsupported/raw escape hatch instead of a generic Rust-owned offset classifier.
6. Remove or strictly narrow the raw `(SurfaceKind::Offset, Some(PortedFaceSurface::Offset(_)))` arm only after the offset-area, multi-source offset, offset-solid, and offset-geometry regressions stay green.
7. Strengthen runtime coverage around the formerly metadata-free path so the test fails if it silently falls back through the generic raw offset classifier again.
8. Add a source guard proving the raw offset arm is removed or explicitly conditioned on the documented metadata-free unsupported path.
9. Update both control files with completed evidence, active milestone, next bounded cut, and exact verification commands.

## Guardrails

- Do not remove the raw offset arm without replacing the exercised behavior that currently needs it.
- Do not classify metadata-free, imported, invalid, or unsupported faces as Rust-owned offsets unless validated Rust metadata or an explicit Rust descriptor path proves support.
- Do not call `face_geometry_occt()` before serving retained metadata-backed offset face geometry.
- Preserve explicit raw/oracle face geometry, offset payload, offset basis, and sampling APIs.
- Keep direct, single-face generated, multi-face generated, BRep offset-area, and offset-solid volume regressions green.

## Verification

- `(cd rust/lean_occt && cargo fmt)`
- `cmake --build build --target LeanOcctCAPI`
- `(cd rust/lean_occt && cargo check)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows public_offset_basis_queries_match_occt -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows ported_offset_surface_sampling_matches_occt -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test brep_workflows ported_brep_maps_multi_source_swept_offsets_in_rust -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test brep_workflows ported_brep_uses_rust_owned_volume_for_offset_solids -- --nocapture)`
- A source guard proving the raw offset classifier is removed or explicitly narrowed to the documented metadata-free unsupported path.
- `(cd rust/lean_occt && cargo test)`
- `git diff --check`
