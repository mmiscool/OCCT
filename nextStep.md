# Next Task

Current milestone: `M2. Whole-Shape Summary Fallback Reduction` from `portingMilestones.md`.

## Completed Evidence

- `capi/include/lean_occt_capi.h`, `capi/src/lean_occt_capi.cxx`, and `rust/lean_occt/src/lib.rs` now expose narrow bbox primitives for edge curves, face pcurve boundaries, and face surfaces. These replace the exercised need for a whole-face OCCT bbox union on the multi-face offset shell with smaller shape-local data that Rust can validate and union.
- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs` now feeds a guarded degenerate plane-cap surface candidate into `face_brep_bbox_candidate(...)` only when the face is a plane and its current summary bbox came from mesh data. Multi-face offset roots return `None` before the old `face_bboxes_occt()` branch, so that branch no longer handles the exercised multi-face family.
- `rust/lean_occt/tests/brep_workflows.rs` now keeps the single-face offset shell pinned to `Some(OffsetFaceBboxSource::ValidatedMesh)` and proves each exercised multi-face offset-solid shell resolves through `Some(OffsetFaceBboxSource::SummaryFaceBrep)` instead of `OcctFaceUnion`.
- Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cmake --build build --target LeanOcctCAPI`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces -- --exact`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_volume_for_offset_solids -- --exact`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml`.
- The remaining `M2` bbox cleanup is now narrower: `face_bboxes_occt()` can only be reached for a single-face offset root after mesh and face-BRep candidates fail, and `OffsetFaceBboxSource::OcctFaceUnion` is no longer exercised by the multi-face offset shell regression.

## Target

Remove the remaining single-face `face_bboxes_occt()`/`OffsetFaceBboxSource::OcctFaceUnion` branch inside `offset_faces_bbox()` or isolate it behind an explicit unsupported-shape guard, without regressing the now-green single-face `ValidatedMesh` path, the multi-face `SummaryFaceBrep` path, the offset-solid root path, or the exact-primitive roots.

## Next Bounded Cut

1. Trace the remaining `face_bboxes_occt()` call in `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs` and determine which supported single-face offset roots can already be accepted by `ValidatedMesh`, `ValidatedFaceBrep`, or `SummaryFaceBrep`.
2. Delete the normal single-face `OcctFaceUnion` winner by promoting the nearest Rust-owned candidate, or move the OCCT branch behind an explicit unsupported guard that cannot be reached by the exercised single-face offset-shell regression.
3. Strengthen `ported_brep_uses_rust_owned_area_for_offset_faces` if needed so it fails on `Some(OffsetFaceBboxSource::OcctFaceUnion)`, and keep `ported_brep_uses_rust_owned_volume_for_offset_solids` asserting `Some(OffsetFaceBboxSource::SummaryFaceBrep)`.
4. After the face-union branch is narrowed, inspect the remaining `fallback_summary()` sites in `ported_shape_summary()` and choose the next supported bbox or volume family instead of spending a turn on helper reshuffling.

## Guardrails

- Keep loader-owned `PreparedShellShape` inventories.
- Do not reintroduce raw `subshapes_occt()` shell traversal beyond the existing prepared-shell loading path.
- Keep validating accepted Rust-owned shell bbox candidates against OCCT shell bboxes while the explicit per-face guard remains in place.
- Keep the root bbox probe, root volume probe, and shell probe observable until the remaining whole-shape fallback branches are narrowed behind explicit unsupported-case guards.
- Do not reintroduce multi-face `face_bboxes_occt()` handling; the exercised multi-face shell must stay on `SummaryFaceBrep`.
- Do not spend the next turn reshuffling summary helpers unless it removes or strictly narrows the remaining single-face `face_bboxes_occt()` dependency or lands a regression that proves a supported branch is off an OCCT fallback.
- Do not stop after adding another probe. The next productive cut should carry a Rust bbox candidate through validation, remove or strictly narrow the OCCT face-union branch, and keep the focused regressions green in the same turn.
- Prefer one coherent multi-file porting change over several tiny preparatory edits if the Rust-owned path needs data model, summary, and test updates together.
- Keep the exercised single-face offset shell on `OffsetFaceBboxSource::ValidatedMesh`; do not let it slide back to `OcctFaceUnion`.
- Keep the rotated torus on `SummaryBboxSource::ExactPrimitive`; do not let it fall back to the generic mesh bbox path just to match OCCT's looser torus envelope.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cmake --build build --target LeanOcctCAPI`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces -- --exact`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_volume_for_offset_solids -- --exact`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
