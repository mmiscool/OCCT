# Next Task

Keep narrowing the remaining shell-local OCCT bbox fallback in `offset_shell_bbox()`, but stay on the shell-boundary Rust path. The next bounded Rust-first cut is to collapse the remaining array-only `EarlyProbeStageLayout::refinement_result()` bounce, so both early unsupported-edge probe stages hand typed `EarlyProbeStageSource` values directly into the shared stage-source path instead of still routing the outer stage through a one-use array wrapper.

## Current State

- [`ported_shape_summary()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) still keeps the narrowed offset bbox tiers:
  - non-solid offset shapes validate Rust mesh, expanded Rust mesh, Rust face-BRep union, then only later use narrower OCCT bbox fallbacks
  - offset solids/compsolids use [`offset_solid_shell_bbox()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs), and each shell still tries validated Rust face-BRep, Rust shell-boundary, Rust mesh, expanded Rust mesh, Rust `ported_brep(shell).summary`, expanded Rust `ported_brep(shell).summary`, and only then shell-local OCCT bbox
- [`load_ported_topology()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs) preserves loader-owned `PreparedShellShape { shell_shape, shell_vertex_shapes, shell_edge_shapes, shell_face_shapes }` on the successful Rust-topology path.
- [`shell_boundary_shape_bbox()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) remains a mixed Rust/public shell-boundary union:
  - it always starts from loader-owned shell vertices
  - it unions exact public edge bbox results when available
  - unsupported shell edges no longer kill the candidate immediately
  - unsupported shell edges still go through adaptive public-edge sampling, recursive interval refinement, tangent-root polish, near-flat tangent-dip probing, local axis-position extremum search, seeded axis-position search, and the shared stronger-half refinement chase before mesh or OCCT fallback tiers
- [`sampled_edge_interval_needs_probe_refinement()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now keeps each early probe stage on an array-only Rust boundary:
  - [`EarlyProbeStageLayout`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now works directly on `[NormalizedEdgeSample; N]` inputs instead of generic source traits
  - the old `EarlyProbeStageRole` trait and `EarlyProbeStageRoleLayout` helper are gone
  - [`EarlyProbeSampleRole`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now only describes stage-local sample ordering over a supplied `source index -> sample` resolver
  - [`EarlyProbeStageLayout::refinement_result_from_source()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now owns the shared typed Rust-owned `Result<[NormalizedEdgeSample; N], bool>` stage result path for both array-backed and triplet-backed midpoint inputs
  - the midpoint-only `EarlyProbeStageLayout<3, _>` specialization is gone
  - the temporary `EarlyProbeStageProgress` enum and the one-use `continue_stage()` bounce are gone
  - [`MIDPOINT_EARLY_PROBE_STAGE_LAYOUT`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) and [`OUTER_EARLY_PROBE_STAGE_LAYOUT`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now hold direct request-source indices plus sample roles
  - [`EarlyProbeRefinementStages`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now owns the full typed midpoint-stage, outer-stage, and terminal dispatch path directly, so the old `prepare_outer_samples()` bounce is gone
  - [`EarlyProbeRefinementPipeline`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now delegates raw `start` / `midpoint` / `end` probe inputs directly into `EarlyProbeRefinementStages`
  - the temporary `EarlyProbeRefinementSource` carrier and its one-use `stage_source()` view are gone
  - [`EarlyProbeRefinementStages`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) no longer rebuilds a local `[start, midpoint, end]` array; the midpoint stage now resolves those raw probe inputs through the same shared stage-source entry as the array-backed outer stage
  - the old inline midpoint-only `match index { 0, 1, 2 }` resolver in `EarlyProbeRefinementStages` is gone
  - [`EarlyProbeTripletSource`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now carries the typed midpoint-stage `start` / `midpoint` / `end` source boundary
  - [`EarlyProbeStageSource`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now carries the shared typed stage-source boundary for both triplet-backed midpoint inputs and array-backed outer-stage inputs
  - [`EarlyProbeRefinementTerminal`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now owns the typed interval-aware segment handoff plus the `None => Some(false)` terminal behavior
- The interval-aware refinement handoff remains typed and Rust-owned:
  - [`PreparedIntervalAwareRefinementSideLayout`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) carries coarse/outer/inner segment layouts
  - [`PreparedIntervalAwareRefinementSideLayouts`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) chooses the stronger coarse side and the winning outer-vs-inner segment before handing off to the shared refinement path
  - [`RefinementSegment`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) still owns score-based creation, stronger-segment choice, local-window checks, and the adaptive stronger-half chase
- The exercised non-solid offset shell fixture stays green on the Rust-first path.
- The exercised closed offset solid fixture stays green, including the direct shell-local parity assertion in [`ported_brep_uses_rust_owned_volume_for_offset_solids()`](rust/lean_occt/tests/brep_workflows.rs).

## Remaining Blocker

`offset_shell_bbox()` still ends at shell-local OCCT bbox for shells that fail all current validated Rust candidates. In the early unsupported-edge probe entry, the remaining structural duplication is narrower again:

- the early stage itself is now source-resolver-backed instead of split across array and triplet sample mapping
- the midpoint-stage and outer-stage request/sample mapping now live in the same typed stage layout
- the stage layout now returns typed `Result<[NormalizedEdgeSample; N], bool>` directly through one shared `refinement_result_from_source()` entry
- the top-level entry now delegates through [`EARLY_PROBE_REFINEMENT_PIPELINE`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs)
- the pipeline now passes raw `start` / `midpoint` / `end` inputs directly into [`EarlyProbeRefinementStages`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs), and the terminal handoff remains typed through [`EarlyProbeRefinementTerminal`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs)
- the midpoint-stage, outer-stage, and terminal dispatch now already live in [`EarlyProbeRefinementStages`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs)
- the midpoint-stage source triplet is now resolved inside [`EarlyProbeStageLayout`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs), so `EarlyProbeRefinementStages` no longer rebuilds a one-use midpoint-stage source array
- but the outer stage in [`EarlyProbeRefinementStages`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) still reaches that same shared stage-source entry through the array-only `EarlyProbeStageLayout::refinement_result()` bounce

The next blocker is to collapse that remaining array-only wrapper so the full early probe path stays on one typed `EarlyProbeStageSource` boundary end to end instead of carrying a one-off outer-stage array bounce at the stage layout boundary.

## Focus

1. Keep the non-solid offset bbox win in place.
2. Keep the now-green direct shell parity check for the exercised closed offset solid.
3. Stay on loader-owned shell-local inventories; do not reintroduce fresh raw `subshapes_occt()` traversal.
4. Keep the shell-boundary candidate on the public Rust edge/vertex path.
5. Keep validating every accepted shell candidate against shell-local OCCT bbox.
6. Prefer structural Rust-side refinement improvements over adding another copied probe tier or another isolated chooser.
7. Keep the verification bar unchanged:
   - `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
   - `cargo check --manifest-path rust/lean_occt/Cargo.toml`
   - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
   - `cargo test --manifest-path rust/lean_occt/Cargo.toml`

## Why This Is Next

This turn finished collapsing the last midpoint-source closure bounce: `EarlyProbeRefinementStages` now hands the midpoint stage into the shared source-resolver path through typed `EarlyProbeStageSource::Triplet(...)`, and the shared stage layout owns direct typed source lookup instead of a one-use adapter closure. That leaves the next real seam one layer smaller again: the array-only `refinement_result()` wrapper that still adapts the outer stage into the same typed stage-source path.
