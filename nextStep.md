# Next Task

Keep narrowing the remaining shell-local OCCT bbox fallback in `offset_shell_bbox()`, but stay on the shell-boundary Rust path. The early unsupported-edge probe-refinement entry is now carrier-owned end to end: `PreparedMidpointProbeChain` decides when to advance into outer probes, and `PreparedOuterProbeChain` now owns the interval-aware local-window check, stronger-segment pick, and handoff into the shared stronger-half chase. The next bounded Rust-first cut is to collapse the remaining interval-aware staging split so `PreparedOuterProbeChain` can hand the winning interval-aware `RefinementSegment` directly to the shared refinement path instead of bouncing through `PreparedIntervalAwareRefinementSide`.

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
  - [`midpoint_edge_probe_pair()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now prepares the earlier midpoint and outer probe pairs once for the probe-refinement entry stages
  - [`sampled_edge_sample_windows_need_refinement()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now owns the shared sliding 3-sample window checks used by those early entry stages
  - [`PreparedMidpointProbeChain`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now owns the `start/first_probe/midpoint/second_probe/end` early probe carrier and decides when midpoint-stage evidence is strong enough to advance into outer probes
  - [`PreparedOuterProbeChain`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now owns the `start/left_outer_probe/first_probe/midpoint/second_probe/right_outer_probe/end` carrier passed into the interval-aware stage, along with the interval-aware `left/right` choice, side-specific `outer`/`inner` segment preparation, local-window check, stronger-segment choice, and handoff into the shared stronger-half chase
  - [`sampled_edge_interval_needs_stronger_half_refinement()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) still owns the shared stronger-half chase
  - [`PreparedIntervalAwareRefinementSide`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) is now just the last interval-aware staging carrier between `PreparedOuterProbeChain` and the shared stronger-half refinement path
  - [`choose_stronger_refinement_half()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) and [`midpoint_edge_probe()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now serve the later shoulder/endpoint/terminal narrowing path through one helper with staged coarse checks
  - [`half_refinement_should_continue()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now provides the shared signal/span-driven adaptive stop rule, with the max-step limit kept only as a safety ceiling
- The exercised non-solid offset shell fixture stays green on the Rust-first path.
- The exercised closed offset solid fixture stays green, including the direct shell-local parity assertion in [`ported_brep_uses_rust_owned_volume_for_offset_solids()`](rust/lean_occt/tests/brep_workflows.rs).

## Remaining Blocker

`offset_shell_bbox()` still ends at shell-local OCCT bbox for shells that fail all current validated Rust candidates. The deeper late refinement tail is no longer the structural problem: early probe preparation, local probe-window checks, interval-aware side preparation, and the later stronger-half refinement now share the same midpoint-probe and scored-segment machinery.

The remaining duplication is no longer the raw early probe chain, and it is no longer the stage wrappers either. The remaining structural gap is the last interval-aware staging split inside the outer carrier. [`PreparedOuterProbeChain`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) still has to:

- prepare both the `outer` and `inner` interval-aware candidates under `PreparedIntervalAwareRefinementSide`
- bounce back through that extra carrier to choose the stronger prepared segment before it can continue into the shared stronger-half chase

The next blocker is to collapse that last staging carrier so the outer chain can prepare and return the winning interval-aware `RefinementSegment` directly. That keeps the early shell-edge refinement boundary type-owned without adding another copied probe tier or another isolated chooser.

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

This turn carried the early probe-entry cleanup through the remaining wrapper boundary: `PreparedMidpointProbeChain` now owns progression into outer probes, and `PreparedOuterProbeChain` now owns interval-aware execution all the way up to the shared stronger-half chase.

The next bounded step is to remove the last interval-aware staging split inside that outer carrier. If `PreparedOuterProbeChain` can prepare the winning interval-aware `RefinementSegment` directly, the unsupported-edge shell-boundary path gets a cleaner Rust-owned refinement boundary without adding another fallback tier.
