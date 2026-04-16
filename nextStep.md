# Next Task

Keep narrowing the remaining shell-local OCCT bbox fallback in `offset_shell_bbox()`, but stay on the shell-boundary Rust path. The late unsupported-edge refinement tail now runs through one shared stronger-half mechanism instead of a staircase of terminal-endpoint-specific segment pickers. The next bounded Rust-first cut is to move the remaining bespoke segment choice in `sampled_edge_interval_needs_interval_aware_probe_refinement()` onto a reusable scored chooser so the shell-boundary path enters the shared stronger-half chase earlier.

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
- The late refinement ladder is structurally simpler now:
  - [`sampled_edge_interval_needs_stronger_half_refinement()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now owns the shared stronger-half chase
  - [`choose_stronger_refinement_half()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) and [`midpoint_edge_probe()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now serve the shoulder, endpoint, terminal, and terminal-endpoint narrowing path through one helper with staged coarse checks
  - [`half_refinement_should_continue()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now provides the shared signal/span-driven adaptive stop rule, with the max-step limit kept only as a safety ceiling
- The exercised non-solid offset shell fixture stays green on the Rust-first path.
- The exercised closed offset solid fixture stays green, including the direct shell-local parity assertion in [`ported_brep_uses_rust_owned_volume_for_offset_solids()`](rust/lean_occt/tests/brep_workflows.rs).

## Remaining Blocker

`offset_shell_bbox()` still ends at shell-local OCCT bbox for shells that fail all current validated Rust candidates. The deepest late refinement tail is no longer the structural problem: shoulder, endpoint, terminal, and terminal-endpoint narrowing now share one stronger-half chase with one adaptive stop rule.

The remaining structural duplication is earlier in the unsupported-edge narrowing path. [`sampled_edge_interval_needs_interval_aware_probe_refinement()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) still open-codes two scored segment decisions before the shared stronger-half chase begins:

- left-biased vs right-biased suspicious side
- outer suspicious triple vs inner suspicious triple on that chosen side

The next blocker is to move that outer/inner and left/right scored segment selection onto a reusable chooser so more of the unsupported-edge narrowing path flows through one bounded Rust-side selection mechanism before later mesh or OCCT fallback tiers.

## Focus

1. Keep the non-solid offset bbox win in place.
2. Keep the now-green direct shell parity check for the exercised closed offset solid.
3. Stay on loader-owned shell-local inventories; do not reintroduce fresh raw `subshapes_occt()` traversal.
4. Keep the shell-boundary candidate on the public Rust edge/vertex path.
5. Keep validating every accepted shell candidate against shell-local OCCT bbox.
6. Prefer structural Rust-side refinement improvements over adding another copied probe tier or another isolated chooser.
7. Keep the verification bar unchanged:
   - `cargo check --manifest-path rust/lean_occt/Cargo.toml`
   - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
   - `cargo test --manifest-path rust/lean_occt/Cargo.toml`

## Why This Is Next

This turn finished the intended reuse step: the adaptive stronger-half chase is no longer terminal-endpoint-only, and the earlier shoulder/endpoint/terminal narrowing stages now flow through the same bounded helper instead of repeating their own midpoint-side pickers.

The next bounded step is to push that same consolidation one stage earlier. If the interval-aware stage also stops open-coding its left/right and outer/inner segment choice, the unsupported-edge shell-boundary path gets broader Rust-owned coverage with less bespoke refinement code and a cleaner porting boundary.
