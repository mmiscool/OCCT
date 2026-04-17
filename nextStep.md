# Next Task

Current milestone: `M2. Whole-Shape Summary Fallback Reduction` from `portingMilestones.md`.

## Completed Evidence

- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs` now lets the exercised multi-face offset-shell summary accept an `OffsetFaceUnion` bbox after axis-by-axis offset-margin expansion toward the shell's OCCT-validated extent, which removes the exercised shell path's dependence on the shell-local root `SummaryBboxSource::OcctFallback`.
- `rust/lean_occt/tests/brep_workflows.rs` now asserts that each shell in `ported_brep_uses_rust_owned_volume_for_offset_solids()` resolves its root summary bbox through `SummaryBboxSource::OffsetFaceUnion`, pinning the cleaned-up subshape behavior alongside the existing Rust-owned root solid bbox and volume checks.
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_volume_for_offset_solids -- --exact`, the full `brep_workflows` suite, `cargo check`, and the full `cargo test` suite all passed after the shell summary source flipped.

## Target

Remove the remaining OCCT-described face bbox union inside the exercised multi-face `OffsetFaceUnion` shell-summary path, while preserving the Rust-owned offset-solid root bbox and volume paths already locked in by `brep_workflows`.

## Next Bounded Cut

1. Audit why `face_breps_bbox(context, face_shapes)` still comes back empty for the exercised multi-face offset shell in `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs`, and identify which face-level validation step still drops the Rust-owned candidate.
2. Promote a Rust-owned per-face bbox source for that exercised shell, or narrow any remaining miss to an explicit unsupported-face guard, so `SummaryBboxSource::OffsetFaceUnion` no longer depends on `face_bboxes_occt()`.
3. Extend `rust/lean_occt/tests/brep_workflows.rs` with a face-level or shell-level regression that proves the exercised offset-shell path stays off the OCCT-described per-face bbox union once the Rust-owned candidate lands.

## Guardrails

- Keep loader-owned `PreparedShellShape` inventories.
- Do not reintroduce raw `subshapes_occt()` shell traversal beyond the existing prepared-shell loading path.
- Keep validating accepted Rust-owned shell bbox candidates against OCCT shell bboxes while the per-face OCCT union remains in place.
- Keep the root bbox probe, root volume probe, and shell probe observable until the remaining whole-shape fallback branches are narrowed behind explicit unsupported-case guards.
- Do not spend the next turn reshuffling summary helpers unless it removes the exercised `face_bboxes_occt()` dependency or lands the regression that proves the exercised offset shell is already off it.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_volume_for_offset_solids -- --exact`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
