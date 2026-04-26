# Next Task

Current milestone: `M2. Whole-Shape Summary Fallback Reduction` from `portingMilestones.md`.

## Completed Evidence

- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep.rs` no longer exposes `OffsetFaceBboxSource::OcctFaceUnion`, so tests and downstream callers cannot treat a whole-face OCCT bbox union as a normal offset-face winner.
- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs` deleted `face_bboxes_occt()` and the final single-face OCCT summary union branch in `offset_faces_bbox()`. Supported offset roots now resolve through validated mesh, validated face BReps, summary face BReps, or a single-face surface/pcurve/control-curve bbox candidate that is still validated against the root bbox and offset margin.
- `ported_shape_summary()` now prevents supported single-face offset roots from dropping to `SummaryBboxSource::OffsetOcctSubshapeUnion`; if the Rust-owned candidates fail, the summary returns the existing supported-shape error instead of falling through to the whole-shape OCCT summary.
- The exercised single-face offset shell remains pinned to `Some(OffsetFaceBboxSource::ValidatedMesh)`, and each exercised multi-face offset-solid shell remains pinned to `Some(OffsetFaceBboxSource::SummaryFaceBrep)`.
- Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cmake --build build --target LeanOcctCAPI`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces -- --exact`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_volume_for_offset_solids -- --exact`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml`.
- The remaining `M2` bbox cleanup is now narrower: `rg` only finds the generic `fallback_summary()` bbox/volume branches and the offset-specific `offset_shape_bbox_occt()`/`SummaryBboxSource::OffsetOcctSubshapeUnion` branch.

## Target

Remove or strictly narrow `offset_shape_bbox_occt()`/`SummaryBboxSource::OffsetOcctSubshapeUnion` for loaded non-solid offset face inventories, without regressing the now-green single-face `ValidatedMesh` path, the multi-face `SummaryFaceBrep` path, the offset-solid root path, swept roots, or exact-primitive roots.

## Next Bounded Cut

1. In `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs`, promote non-solid offset roots with loaded face inventory into the `requires_rust_owned_bbox` guard so they cannot drop to `OffsetOcctSubshapeUnion` or the generic bbox `fallback_summary()`.
2. Delete `offset_shape_bbox_occt()` if the existing mesh, face-BRep, summary-face, and single-face surface/pcurve candidates cover the exercised cases; otherwise isolate it behind an explicit unsupported-empty-inventory guard that cannot be reached by loaded offset faces.
3. If a test exposes another supported multi-face offset shell gap, extend `offset_faces_bbox()` with a Rust-owned candidate and validate it against the root bbox in the same turn.
4. Keep `ported_brep_uses_rust_owned_area_for_offset_faces` pinned to `Some(OffsetFaceBboxSource::ValidatedMesh)` and `ported_brep_uses_rust_owned_volume_for_offset_solids` pinned to `Some(OffsetFaceBboxSource::SummaryFaceBrep)`.

## Guardrails

- Keep loader-owned `PreparedShellShape` inventories.
- Do not reintroduce raw `subshapes_occt()` shell traversal beyond the existing prepared-shell loading path.
- Keep validating accepted Rust-owned shell bbox candidates against OCCT shell bboxes while the explicit per-face guard remains in place.
- Keep the root bbox probe, root volume probe, and shell probe observable until the remaining whole-shape fallback branches are narrowed behind explicit unsupported-case guards.
- Do not reintroduce `face_bboxes_occt()` or `OffsetFaceBboxSource::OcctFaceUnion`; the exercised multi-face shell must stay on `SummaryFaceBrep`.
- Do not spend the next turn reshuffling summary helpers unless it removes or strictly narrows `offset_shape_bbox_occt()` or lands a regression that proves a supported branch is off an OCCT fallback.
- Do not stop after adding another probe. The next productive cut should carry a Rust bbox candidate through validation, remove or strictly narrow the offset-specific OCCT subshape-union branch, and keep the focused regressions green in the same turn.
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
