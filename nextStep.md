# Next Task

Current milestone: `M2. Whole-Shape Summary Fallback Reduction` from `portingMilestones.md`.

## Completed Evidence

- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs` now routes offset-shell bbox selection through a shared `OffsetShellBboxSource` resolver and removes the unconditional shell-local `or(Some(shell_occt_bbox))` fallback for shells that already have prepared topology inventory.
- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep.rs` now exposes offset-shell bbox winners on `BrepShape`.
- `rust/lean_occt/tests/brep_workflows.rs` now proves the exercised offset-solid fixture resolves shell bbox through `OffsetShellBboxSource::Brep`, not the former shell-local OCCT fallback.

## Target

Make the supported offset-solid bbox path in `ported_shape_summary()` stop depending on the root-level `fallback_summary()` branch now that the shell-local offset bbox winner is proven Rust-owned.

## Next Bounded Cut

1. Refactor the offset-solid bbox branch in `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs` so supported offset solids return `offset_solid_shell_bbox(...)` directly and only use `fallback_summary()` behind an explicit unsupported-shape guard.
2. Add a regression in `rust/lean_occt/tests/brep_workflows.rs` that proves the exercised offset-solid summary stays off that whole-shape OCCT fallback.
3. Keep the current shell winner observable so the next cut can distinguish a root-summary fallback from a shell-local fallback regression.

## Guardrails

- Keep loader-owned `PreparedShellShape` inventories.
- Do not reintroduce raw `subshapes_occt()` shell traversal beyond the existing prepared-shell loading path.
- Keep validating accepted Rust-owned shell bbox candidates against OCCT shell bboxes while whole-shape fallback removal is in progress.
- Do not spend the next turn reshuffling shell probe helpers unless it deletes the root `fallback_summary()` branch or lands a new regression that proves the branch is no longer exercised.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_volume_for_offset_solids -- --exact`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
