# Next Task

Current milestone: `M2. Whole-Shape Summary Fallback Reduction` from `portingMilestones.md`.

## Completed Evidence

- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs` now removes the branch-local `fallback_summary()` wrapper around the supported offset-solid bbox path and tags the winning root bbox route with `SummaryBboxSource`.
- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep.rs` now exposes the root summary bbox source on `BrepShape` while keeping the per-shell `OffsetShellBboxSource` probe.
- `rust/lean_occt/tests/brep_workflows.rs` now proves the exercised offset-solid fixture resolves its root bbox through `SummaryBboxSource::OffsetSolidShellUnion` and its shell bboxes through `OffsetShellBboxSource::Brep`.

## Target

Make the supported offset-solid volume path in `ported_shape_summary()` stop depending on `fallback_summary().map(|summary| summary.volume)` now that the exercised offset-solid bbox path is already proven Rust-owned.

## Next Bounded Cut

1. Thread the winning root volume source out of `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs` and onto `BrepShape`, mirroring the current bbox probe only as far as needed to tell Rust-owned volume from `fallback_summary()`.
2. Remove the exercised offset-solid volume branch from the shared `fallback_summary().map(|summary| summary.volume)` path, keeping any remaining OCCT summary use behind an explicit unsupported-case guard.
3. Extend `rust/lean_occt/tests/brep_workflows.rs` so the offset-solid regression proves both root bbox and root volume stay off OCCT fallback while preserving the current shell winner checks.

## Guardrails

- Keep loader-owned `PreparedShellShape` inventories.
- Do not reintroduce raw `subshapes_occt()` shell traversal beyond the existing prepared-shell loading path.
- Keep validating accepted Rust-owned shell bbox candidates against OCCT shell bboxes while whole-shape fallback removal is in progress.
- Keep the new root bbox probe and existing shell probe observable until the volume fallback cut lands.
- Do not spend the next turn reshuffling summary helpers unless it deletes the shared OCCT volume fallback for the exercised path or lands a regression that proves the path is already off that fallback.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_volume_for_offset_solids -- --exact`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
