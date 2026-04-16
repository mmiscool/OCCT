# Next Task

Keep narrowing the remaining shell-local OCCT bbox fallback in `offset_shell_bbox()`, but stay on the shell-boundary Rust path. The terminal-endpoint refinement tail is no longer a hand-unrolled chain of deeper `inner` vs `outer` splits; it now goes through a bounded reusable chooser in `summary.rs`. The next bounded Rust-first cut is to replace the fixed `TERMINAL_ENDPOINT_HALF_REFINEMENT_STEPS` depth with a signal-aware or span-aware stop condition, so the public-edge refinement depth follows the edge evidence instead of a hard-coded loop count.

## Current State

- [`ported_shape_summary()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) still keeps the narrowed offset bbox tiers:
  - non-solid offset shapes validate Rust mesh, expanded Rust mesh, Rust face-BRep union, then only later use narrower OCCT bbox fallbacks
  - offset solids/compsolids use [`offset_solid_shell_bbox()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs), and each shell now tries validated Rust face-BRep, Rust shell-boundary, Rust mesh, expanded Rust mesh, Rust `ported_brep(shell).summary`, expanded Rust `ported_brep(shell).summary`, and only then shell-local OCCT bbox
- [`load_ported_topology()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs) preserves loader-owned `PreparedShellShape { shell_shape, shell_vertex_shapes, shell_edge_shapes, shell_face_shapes }` on the successful Rust-topology path.
- [`shell_boundary_shape_bbox()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) remains a mixed Rust/public shell-boundary union:
  - it always starts from loader-owned shell vertices
  - it unions exact public edge bbox results when available
  - unsupported shell edges no longer kill the candidate immediately
  - unsupported shell edges now get adaptive public-edge sampling, recursive interval refinement, tangent-root polish, near-flat tangent-dip probing, local axis-position extremum search, and run-based seeded axis-position search before mesh or OCCT fallback tiers
- The terminal refinement tail is now structurally simpler:
  - [`sampled_edge_interval_needs_terminal_probe_refinement()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now uses [`choose_stronger_refinement_half()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) instead of open-coding another half-choice
  - [`sampled_edge_interval_needs_terminal_endpoint_probe_refinement()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now iterates that same chooser through a bounded loop controlled by `TERMINAL_ENDPOINT_HALF_REFINEMENT_STEPS`
  - [`midpoint_edge_probe()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) centralizes midpoint sampling for that chooser
- The exercised non-solid offset shell fixture stays green on the Rust-first path.
- The exercised closed offset solid fixture stays green, including the direct shell-local parity assertion in [`ported_brep_uses_rust_owned_volume_for_offset_solids()`](rust/lean_occt/tests/brep_workflows.rs).

## Remaining Blocker

`offset_shell_bbox()` still ends at shell-local OCCT bbox for shells that fail all current validated Rust candidates. The shell-boundary candidate is now structurally capable of chasing one-sided unsupported-edge extrema much further than before, but its deepest terminal-endpoint chase is still governed by a fixed loop depth. That means some edges will still stop too early, while others may spend refinement budget where the signal has already gone flat.

The remaining blocker is no longer “add one more deeper manual split.” It is making the bounded terminal-endpoint chooser stop for Rust-owned reasons:

- stop when the chosen sub-interval is already too small in `t` or spatial span to matter
- stop when refinement signal strength has decayed below a useful threshold
- keep going when the signal remains strong enough that the current fixed-depth loop may still undershoot the decisive extrema

## Focus

1. Keep the non-solid offset bbox win in place.
2. Keep the now-green direct shell parity check for the exercised closed offset solid.
3. Stay on loader-owned shell-local inventories; do not reintroduce fresh raw `subshapes_occt()` traversal.
4. Keep the shell-boundary candidate on the public Rust edge/vertex path.
5. Keep validating every accepted shell candidate against shell-local OCCT bbox.
6. Prefer structural Rust-side refinement improvements over adding another copied deeper split.
7. Keep the verification bar unchanged:
   - `cargo check --manifest-path rust/lean_occt/Cargo.toml`
   - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
   - `cargo test --manifest-path rust/lean_occt/Cargo.toml`

## Why This Is Next

This turn replaced the worst remaining hand-unrolled terminal-endpoint refinement ladder with a reusable stronger-half chooser plus a bounded loop. That is the right direction: the Rust/public shell-boundary path should get more general and more data-driven, not accumulate another page of copied `sub_sub_sub...` segment selection.

The next bounded step is to make that loop adaptive. If refinement depth follows signal and interval size instead of a literal constant, the shell-boundary Rust path can cover more real offset shells without turning the code back into a manual deeper-split tracker.
