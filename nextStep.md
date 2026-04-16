# Next Task

Keep narrowing the remaining shell-local OCCT bbox fallback in `offset_shell_bbox()`, but stay on the shell-boundary Rust path. The early unsupported-edge probe-refinement entry is now carrier-owned end to end, the interval-aware outer carrier no longer open-codes its `left/right` branch, and the winning interval-aware `RefinementSegment` now owns its own score-based creation, stronger-segment choice, local-window test, and stronger-half chase. The next bounded Rust-first cut is to collapse the remaining inline left/right descriptor assembly inside `PreparedOuterProbeChain::prepared_interval_aware_refinement_sides()`, so the outer carrier stops spelling out both eight-field side descriptors directly.

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
  - [`midpoint_edge_probe_pair()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now prepares the earlier midpoint and outer probe pairs once for the probe-refinement entry stages
  - [`sampled_edge_sample_windows_need_refinement()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now owns the shared sliding 3-sample window checks used by those early entry stages
  - [`PreparedMidpointProbeChain`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now owns the `start/first_probe/midpoint/second_probe/end` early probe carrier and decides when midpoint-stage evidence is strong enough to advance into outer probes
  - [`PreparedOuterProbeChain`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now owns the `start/left_outer_probe/first_probe/midpoint/second_probe/right_outer_probe/end` carrier passed into the interval-aware stage and now prepares both side descriptors up front before handing the winning segment to the shared refinement path
  - [`PreparedIntervalAwareRefinementSide`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now owns side choice as well as the coarse, outer, and inner segment descriptors for the chosen side, so the enum boundary is gone and the repeated side-specific remapping is gone from the segment builders
  - the last interval-aware staging carrier is gone; `PreparedOuterProbeChain` now prepares the winning interval-aware [`RefinementSegment`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) directly
  - [`RefinementSegment`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now owns score-based creation, stronger-segment choice, the local-window test, and the stronger-half chase
  - [`midpoint_edge_probe()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now serves both the interval-aware inner probe path and the later stronger-half narrowing path
  - [`half_refinement_should_continue()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now provides the shared signal/span-driven adaptive stop rule, with the max-step limit kept only as a safety ceiling
- The exercised non-solid offset shell fixture stays green on the Rust-first path.
- The exercised closed offset solid fixture stays green, including the direct shell-local parity assertion in [`ported_brep_uses_rust_owned_volume_for_offset_solids()`](rust/lean_occt/tests/brep_workflows.rs).

## Remaining Blocker

`offset_shell_bbox()` still ends at shell-local OCCT bbox for shells that fail all current validated Rust candidates. The deeper late refinement tail is no longer the structural problem: early probe preparation, local probe-window checks, interval-aware side preparation, and the later stronger-half refinement now share the same midpoint-probe and scored-segment machinery.

The remaining duplication is no longer the raw early probe chain, it is no longer the stage wrappers, it is no longer the interval-aware staging carrier, it is no longer the open-coded execution step on the prepared segment, it is no longer the old enum branch, and it is no longer the repeated per-side remapping inside the segment builders. The remaining structural gap is now the inline side-pair construction in [`PreparedOuterProbeChain::prepared_interval_aware_refinement_sides()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs). That helper still has to:

- manually assemble both side descriptors inline in one place
- pass eight sample fields into `PreparedIntervalAwareRefinementSide::new(...)` for the left side
- repeat the same eight-field constructor pattern again for the right side before side choice even starts

The next blocker is to move that remaining left/right descriptor assembly behind a prepared side-pair boundary, so the outer carrier can hand off a fully prepared interval-aware side set instead of open-coding the pair with two large constructor calls. That keeps the early shell-edge refinement path more type-owned without adding another copied probe tier or another fallback.

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

This turn carried the interval-aware cleanup one step deeper: the side enum is gone, `PreparedOuterProbeChain` now prepares both interval-aware side descriptors once, and `PreparedIntervalAwareRefinementSide` now owns both side choice and the coarse, outer, and inner segment descriptors instead of relying on a separate selector layer.

The next bounded step is to remove the remaining inline pair construction in `PreparedOuterProbeChain::prepared_interval_aware_refinement_sides()`. If that left/right descriptor assembly is collapsed behind a dedicated prepared side-pair boundary, the unsupported-edge shell-boundary path gets a cleaner Rust-owned interval-aware refinement entry without adding another fallback tier.
