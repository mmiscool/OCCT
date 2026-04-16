# Next Task

Keep narrowing the remaining shell-local OCCT bbox fallback in `offset_shell_bbox()`, but stay on the shell-boundary Rust path. The terminal-endpoint refinement tail now uses a reusable chooser plus an adaptive stop rule; it is no longer governed by a literal deeper-split ladder or a fixed-depth loop. The next bounded Rust-first cut is to lift that adaptive stronger-half chase into a reusable helper so earlier endpoint and terminal refinement stages can share the same signal/span-driven segment narrowing instead of keeping their own one-off segment pickers.

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
  - [`sampled_edge_interval_needs_terminal_endpoint_probe_refinement()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now iterates that same chooser only while signal and span justify continuing, with a max-step cap kept only as a safety ceiling
  - [`midpoint_edge_probe()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) centralizes midpoint sampling for that chooser
- The exercised non-solid offset shell fixture stays green on the Rust-first path.
- The exercised closed offset solid fixture stays green, including the direct shell-local parity assertion in [`ported_brep_uses_rust_owned_volume_for_offset_solids()`](rust/lean_occt/tests/brep_workflows.rs).

## Remaining Blocker

`offset_shell_bbox()` still ends at shell-local OCCT bbox for shells that fail all current validated Rust candidates. The shell-boundary candidate is now structurally capable of chasing one-sided unsupported-edge extrema much further than before, and its deepest terminal-endpoint chase now stops for Rust-owned signal/span reasons instead of a literal fixed depth. The remaining structural duplication is that the earlier endpoint and terminal stages still have their own bespoke segment-picking code before this adaptive chase begins.

The remaining blocker is no longer “add one more deeper manual split,” and it is no longer the terminal-endpoint stop rule itself. It is reusing the same adaptive stronger-half chase earlier in the refinement ladder so more of the unsupported-edge narrowing path is driven by one bounded Rust-side mechanism:

- use the same stronger-half chooser for the first terminal and endpoint narrowing steps
- use the same signal/span-driven continuation rule instead of keeping earlier one-off segment splits
- keep the existing OCCT validation boundary unchanged while reducing the remaining bespoke refinement code

## Focus

1. Keep the non-solid offset bbox win in place.
2. Keep the now-green direct shell parity check for the exercised closed offset solid.
3. Stay on loader-owned shell-local inventories; do not reintroduce fresh raw `subshapes_occt()` traversal.
4. Keep the shell-boundary candidate on the public Rust edge/vertex path.
5. Keep validating every accepted shell candidate against shell-local OCCT bbox.
6. Prefer structural Rust-side refinement improvements over adding another copied deeper split or another isolated picker.
7. Keep the verification bar unchanged:
   - `cargo check --manifest-path rust/lean_occt/Cargo.toml`
   - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
   - `cargo test --manifest-path rust/lean_occt/Cargo.toml`

## Why This Is Next

This turn replaced the terminal-endpoint fixed-depth loop with a reusable stronger-half chooser plus a signal/span-aware continuation rule. That is the right direction: the Rust/public shell-boundary path should get more general and more data-driven, not accumulate another page of copied segment selection or another hard-coded refinement depth.

The next bounded step is to reuse that adaptive narrowing earlier in the refinement chain. If the earlier terminal and endpoint stages also flow through the same chooser/continuation machinery, the shell-boundary Rust path gets broader coverage with less bespoke logic and a clearer porting boundary.
