# Next Task

Current milestone: `M2. Whole-Shape Summary Fallback Reduction` from `portingMilestones.md`.

## Completed Evidence

- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep.rs` no longer exposes `SummaryBboxSource::OffsetOcctSubshapeUnion`, so callers cannot observe the offset-specific OCCT subshape-union bbox as a normal summary winner.
- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs` deleted `offset_shape_bbox_occt()` and the `union_shape_bboxes_occt()` helper. Loaded offset-face inventories now set `requires_rust_owned_bbox`, forcing supported offset roots through `offset_faces_bbox()`, offset validated mesh, or offset solid shell unions.
- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep.rs` and `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs` no longer carry the root `vertex_shapes` payload or perform the fallback root vertex walk that only fed the deleted OCCT bbox branch. Shell-local prepared vertex inventories remain in place for shell bbox validation.
- The exercised single-face offset shell remains pinned to `Some(OffsetFaceBboxSource::ValidatedMesh)`, and each exercised multi-face offset-solid shell remains pinned to `Some(OffsetFaceBboxSource::SummaryFaceBrep)`.
- Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cmake --build build --target LeanOcctCAPI`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces -- --exact`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_volume_for_offset_solids -- --exact`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml`.
- The remaining `M2` fallback inventory is now just the generic `fallback_summary()` bbox branch and the generic `fallback_summary()` volume branch in `ported_shape_summary()`.

## Target

Narrow the remaining generic `fallback_summary()` bbox and volume branches behind explicit unsupported-shape guards, without regressing the now-green single-face offset `ValidatedMesh` path, multi-face offset `SummaryFaceBrep` path, offset-solid root path, swept roots, exact-primitive roots, or topology-owned bbox paths.

## Next Bounded Cut

1. In `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs`, make the generic bbox fallback conditional on a named unsupported-shape predicate, and promote any loaded analytic, swept, or offset face inventory already covered by Rust bbox candidates into `requires_rust_owned_bbox`.
2. Do the same for the generic volume fallback: keep it only for explicitly unsupported open or unclassified shapes, while supported offset solids and swept solids continue to require `SummaryVolumeSource::FaceContributions`.
3. If tightening those guards exposes a supported fixture that still reaches `OcctFallback`, implement the Rust-owned bbox or volume candidate in the same turn instead of weakening the guard.
4. Keep the existing source assertions for exact primitives, exact curves, swept solids, single-face offsets, and multi-face offset-solid shells green; add a targeted assertion if a newly guarded supported family was previously unobserved.

## Guardrails

- Keep loader-owned `PreparedShellShape` inventories.
- Do not reintroduce raw `subshapes_occt()` shell traversal beyond the existing prepared-shell loading path.
- Keep validating accepted Rust-owned shell bbox candidates against OCCT shell bboxes while the explicit per-face guard remains in place.
- Keep the root bbox probe, root volume probe, and shell probe observable until the remaining whole-shape fallback branches are narrowed behind explicit unsupported-case guards.
- Do not reintroduce `face_bboxes_occt()`, `OffsetFaceBboxSource::OcctFaceUnion`, `offset_shape_bbox_occt()`, or `SummaryBboxSource::OffsetOcctSubshapeUnion`.
- Do not spend the next turn reshuffling summary helpers unless it narrows one of the two generic `fallback_summary()` branches or lands a regression that proves a supported branch is off an OCCT fallback.
- Do not stop after adding another probe. The next productive cut should either make a supported family require Rust-owned bbox/volume behavior or put the generic fallback behind an explicit unsupported-shape guard, then keep the focused regressions green in the same turn.
- Prefer one coherent multi-file porting change over several tiny preparatory edits if the Rust-owned path needs data model, summary, and test updates together.
- Keep the exercised single-face offset shell on `OffsetFaceBboxSource::ValidatedMesh`; do not let it slide to any OCCT bbox source.
- Keep the rotated torus on `SummaryBboxSource::ExactPrimitive`; do not let it fall back to the generic mesh bbox path just to match OCCT's looser torus envelope.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cmake --build build --target LeanOcctCAPI`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces -- --exact`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_volume_for_offset_solids -- --exact`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
