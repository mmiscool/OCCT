# Next Task

Current milestone: `M2. Whole-Shape Summary Fallback Reduction` from `portingMilestones.md`.

## Completed Evidence

- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs` now caches the exact-primitive and ported topological bbox candidates up front and only lets the final root `fallback_summary()` bbox branch run for shapes that do not already expose a concrete Rust-owned root bbox path. Exact primitives, analytic/topological breps, single-face offset surfaces, and offset solids now fail loudly instead of silently dropping to the generic OCCT root bbox fallback.
- `rust/lean_occt/tests/brep_workflows.rs` now adds root bbox source assertions for `ported_brep_uses_exact_primitive_bounding_boxes()`, `ported_brep_uses_exact_curve_bounding_boxes()`, and `ported_brep_uses_rust_owned_area_for_offset_faces()` so those exercised families stay pinned to `SummaryBboxSource::ExactPrimitive`, `SummaryBboxSource::PortedBrep`, and `SummaryBboxSource::OffsetFaceUnion`.
- Focused regressions for kind classification, face-free topology, bounding boxes, and offset-solid volume all passed after the guard tightened, and the full `brep_workflows`, `cargo check`, and `cargo test` suites stayed green.

## Target

Move the exercised multi-face offset shell summaries off the shell-local root bbox fallback that still underpins `validated_shell_brep_bbox()`, while preserving the Rust-owned offset-solid root bbox and volume paths already locked in by `brep_workflows`.

## Next Bounded Cut

1. Audit the exercised offset-solid shell sub-brep path in `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs` to identify which shell summary candidate still misses OCCT validation and forces `validated_shell_brep_bbox()` to rely on a shell-local root summary fallback.
2. Promote one Rust-owned shell summary bbox path for that exercised multi-face offset shell, or make the unsupported-shell case explicit, so `context.ported_brep(shell_shape)` no longer needs the generic root `SummaryBboxSource::OcctFallback` just to keep `OffsetShellBboxSource::Brep` alive.
3. Extend `rust/lean_occt/tests/brep_workflows.rs` with a shell-level bbox source regression for the exercised offset-solid shell so the cleanup is pinned to a user-visible subshape behavior rather than only the root solid summary.

## Guardrails

- Keep loader-owned `PreparedShellShape` inventories.
- Do not reintroduce raw `subshapes_occt()` shell traversal beyond the existing prepared-shell loading path.
- Keep validating accepted Rust-owned shell bbox candidates against OCCT shell bboxes while shell-summary fallback removal is in progress.
- Keep the root bbox probe, root volume probe, and existing shell probe observable until the remaining whole-shape fallback branches are narrowed behind explicit unsupported-case guards.
- Do not spend the next turn reshuffling summary helpers unless it removes the exercised shell-local OCCT bbox fallback dependency or lands the shell-level regression that proves the exercised offset shell is already off it.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_kind_classification -- --exact`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_face_free_shapes -- --exact`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_bounding_boxes -- --exact`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_volume_for_offset_solids -- --exact`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
