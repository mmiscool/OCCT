# Next Task

Keep narrowing the remaining shell-local OCCT bbox fallback in `offset_shell_bbox()`, but stay on the shell-boundary Rust path. The unsupported-edge interval-aware stage now uses the same reusable scored-segment chooser as the later stronger-half chase. The next bounded Rust-first cut is to remove the remaining bespoke side-specific segment assembly in `sampled_edge_interval_needs_interval_aware_probe_refinement()` so that stage hands off a prepared segment to the shared refinement path instead of rebuilding `outer` and `inner` triples inline.

## Current State

- [`ported_shape_summary()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) still keeps the narrowed offset bbox tiers:
  - non-solid offset shapes validate Rust mesh, expanded Rust mesh, Rust face-BRep union, then only later use narrower OCCT bbox fallbacks
  - offset solids/compsolids use [`offset_solid_shell_bbox()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs), and each shell now tries validated Rust face-BRep, Rust shell-boundary, Rust mesh, expanded Rust mesh, Rust `ported_brep(shell).summary`, expanded Rust `ported_brep(shell).summary`, and only then shell-local OCCT bbox
- [`load_ported_topology()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs) preserves loader-owned `PreparedShellShape { shell_shape, shell_vertex_shapes, shell_edge_shapes, shell_face_shapes }` on the successful Rust-topology path.
- [`shell_boundary_shape_bbox()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) remains a mixed Rust/public shell-boundary union:
  - it always starts from loader-owned shell vertices
  - it unions exact public edge bbox results when available
  - unsupported shell edges no longer kill the candidate immediately
  - unsupported shell edges now get adaptive public-edge sampling, recursive interval refinement, tangent-root polish, near-flat tangent-dip probing, local axis-position extremum search, run-based seeded axis-position search, and the shared stronger-half refinement chase before mesh or OCCT fallback tiers
- The late refinement ladder is structurally tighter now:
  - [`scored_refinement_segment()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now builds the reusable scored segment carrier used by both interval-aware and stronger-half refinement
  - [`choose_stronger_refinement_segment()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now owns the shared score-based segment choice used by the interval-aware `left/right` and `outer/inner` stage as well as the later stronger-half chase
  - [`sampled_edge_interval_needs_stronger_half_refinement()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) still owns the shared stronger-half chase
  - [`choose_stronger_refinement_half()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) and [`midpoint_edge_probe()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now serve the later shoulder/endpoint/terminal narrowing path through one helper with staged coarse checks
  - [`half_refinement_should_continue()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now provides the shared signal/span-driven adaptive stop rule, with the max-step limit kept only as a safety ceiling
- The exercised non-solid offset shell fixture stays green on the Rust-first path.
- The exercised closed offset solid fixture stays green, including the direct shell-local parity assertion in [`ported_brep_uses_rust_owned_volume_for_offset_solids()`](rust/lean_occt/tests/brep_workflows.rs).

## Remaining Blocker

`offset_shell_bbox()` still ends at shell-local OCCT bbox for shells that fail all current validated Rust candidates. The deeper late refinement tail is no longer the structural problem: interval-aware scored choice and later stronger-half refinement now share the same scored segment machinery.

The remaining duplication is the side-specific segment assembly at the interval-aware entry. [`sampled_edge_interval_needs_interval_aware_probe_refinement()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) still manually:

- maps the chosen suspicious side onto ad hoc `outer_start`, `outer_mid`, `outer_end`, `inner_start`, and `inner_end` references
- computes the chosen side’s inner midpoint probe inline instead of handing off a prepared segment descriptor

The next blocker is to move that side-to-segment assembly onto a reusable helper or carrier so the interval-aware stage becomes: choose the stronger side, prepare its `outer` and `inner` candidate segments once, reuse the shared scored chooser, then hand the winning segment to the shared stronger-half chase.

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

This turn finished the intended segment-choice reuse step: the interval-aware stage now uses the same reusable scored segment chooser as the later stronger-half chase, and the shared score carrier is no longer terminal-only.

The next bounded step is to push that same consolidation into the remaining side-specific assembly immediately before the chooser. If the interval-aware stage stops rebuilding its `outer` and `inner` triples inline, the unsupported-edge shell-boundary path gets broader Rust-owned coverage with less bespoke refinement code and a cleaner porting boundary.
