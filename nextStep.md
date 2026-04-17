# Next Task

Current milestone: `M2. Whole-Shape Summary Fallback Reduction` from `portingMilestones.md`.

## Completed Evidence

- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs` now threads `SummaryVolumeSource`, keeps the supported offset-solid branch off `fallback_summary().map(|summary| summary.volume)`, and resolves the exercised offset-solid volume through a Rust-owned face-contribution path with a targeted planar-cap mesh promotion when the analytic planar integral collapses numerically.
- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep.rs` now exposes the root summary volume source on `BrepShape`, and `rust/lean_occt/src/lib.rs` re-exports `SummaryVolumeSource`.
- `rust/lean_occt/tests/brep_workflows.rs` now proves the exercised offset-solid fixture resolves its root volume through `SummaryVolumeSource::FaceContributions` while preserving the existing shell bbox winner checks and OCCT parity tolerance.

## Target

Move the exercised swept-revolution solid in `ported_shape_summary()` off the remaining shared `fallback_summary()` summary path now that the exercised offset-solid bbox and volume paths are both proven Rust-owned.

## Next Bounded Cut

1. Extend `rust/lean_occt/tests/brep_workflows.rs` so `ported_brep_summarizes_swept_revolution_solids_in_rust()` records and asserts the root summary bbox and volume sources for the exercised swept solid.
2. Remove the supported swept-solid branch from the shared `fallback_summary()` volume path in `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs`, keeping any remaining OCCT summary use behind an explicit unsupported-case guard.
3. If the swept-solid bbox still lands on the generic whole-shape fallback, narrow that bbox fallback to unsupported shapes only and keep the regression proving the swept fixture stays on a Rust-owned root source.

## Guardrails

- Keep loader-owned `PreparedShellShape` inventories.
- Do not reintroduce raw `subshapes_occt()` shell traversal beyond the existing prepared-shell loading path.
- Keep validating accepted Rust-owned shell bbox candidates against OCCT shell bboxes while whole-shape fallback removal is in progress.
- Keep the new root bbox probe, root volume probe, and existing shell probe observable until the shared whole-shape fallback branches are narrowed behind explicit unsupported-case guards.
- Do not spend the next turn reshuffling summary helpers unless it deletes one of the remaining shared OCCT summary fallback branches for the exercised swept path or lands a regression that proves that path is already off them.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_volume_for_offset_solids -- --exact`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
