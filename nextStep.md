# Next Task

Current milestone: `M2. Whole-Shape Summary Fallback Reduction` from `portingMilestones.md`.

## Completed Evidence

- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs` now marks supported swept solids as Rust-owned whole-shape volume candidates, lets them reach `analytic_shape_volume(...)` even when they miss the old closed-topology gate, and blocks the shared `fallback_summary().map(|summary| summary.volume)` branch for that exercised swept-solid family.
- `rust/lean_occt/tests/brep_workflows.rs` now proves `ported_brep_summarizes_swept_revolution_solids_in_rust()` resolves its root bbox through `SummaryBboxSource::PortedBrep`, resolves its root volume through `SummaryVolumeSource::FaceContributions`, and keeps a deterministic Rust-owned volume regression anchor at `35530.57584392169` while OCCT still reports zero for the same whole-shape summary volume.
- Full verification stayed green after widening the swept-solid Rust path: `cargo check`, the focused swept-solid regression, `brep_workflows`, and the full `cargo test` suite all passed.

## Target

Narrow the last generic whole-shape bbox `fallback_summary()` escape hatch in `ported_shape_summary()` so the exercised supported families keep resolving root bbox from Rust-owned data and any remaining OCCT summary fallback is explicit unsupported-shape handling.

## Next Bounded Cut

1. Audit which exercised supported families can still reach the root bbox `fallback_summary()` branch in `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs`, starting with the swept, offset, and exact-primitive fixtures already covered in `brep_workflows`.
2. Replace that generic bbox fallback with an explicit unsupported-shape guard so supported exercised families either resolve from Rust-owned bbox paths or fail loudly instead of silently dropping to OCCT.
3. Add or extend a regression in `rust/lean_occt/tests/brep_workflows.rs` if another exercised family needs a root bbox source assertion to keep the narrowed guard from regressing.

## Guardrails

- Keep loader-owned `PreparedShellShape` inventories.
- Do not reintroduce raw `subshapes_occt()` shell traversal beyond the existing prepared-shell loading path.
- Keep validating accepted Rust-owned shell bbox candidates against OCCT shell bboxes while whole-shape fallback removal is in progress.
- Keep the root bbox probe, root volume probe, and existing shell probe observable until the remaining whole-shape fallback branches are narrowed behind explicit unsupported-case guards.
- Do not spend the next turn reshuffling summary helpers unless it deletes the remaining generic OCCT summary bbox fallback or lands a regression that proves another exercised family is already off it.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_summarizes_swept_revolution_solids_in_rust -- --exact`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
