# Next Task

Keep narrowing the remaining shell-local OCCT bbox fallback in `offset_shell_bbox()`, but stay on the shell-boundary Rust path. The next bounded Rust-first cut is to collapse the duplicated `ControlFlow::Continue/Break` stage-progress match inside `EarlyProbeRefinementStages::needs_refinement()`, so midpoint-stage -> outer-stage -> interval-aware progression stays on one explicit Rust-owned runner path without repeating the same typed stage unwrap twice.

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
  - the old `EarlyProbeSampleRole::Source(usize)` slot mapping is gone
  - [`MidpointEdgeProbePairRequestLayout`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now owns the typed `source spans -> probe pair outcome` handoff and direct probe execution for early probe stages through explicit `first` / `second` [`MidpointEdgeProbeSpanLayout`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) values, so the early stage path no longer carries a raw `[usize; 4]` request-source array or bounces through a second request carrier
  - midpoint-stage and outer-stage sample reuse now goes directly through shared [`EarlyProbeSampleRole`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) arrays over shared [`EarlyProbeSourcePosition`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) slots, so the stage constants no longer hard-code `Source(usize)` entries and no longer bounce through separate stage-local sample-layout carriers
  - the old `EarlyProbeSourceSampleLayout`, `EarlyProbeStageSampleLayout`, and `EarlyProbeSampleLayout` generic layers are gone, so the old duplicated midpoint-vs-outer `source_sample()` / `stage_sample()` ladders and the now-dead one-implementation trait boundaries are gone too
  - midpoint-stage and outer-stage now resolve source samples through shared [`EarlyProbeSourcePosition`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs), so the old raw positional `source_index()` remaps are gone too
  - midpoint-stage and outer-stage sample reuse now goes straight through shared [`EarlyProbeSourcePosition`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) source slots, so the old stage-specific source-slot enums, the duplicated `source_ordinal(self) -> self as usize` alias, and the old `EarlyProbeSourcePosition::from_source_ordinal(...)` bridge are gone too
  - [`EarlyProbeRefinementStages`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now owns the shared typed Rust-owned `Result<[NormalizedEdgeSample; N], bool>` stage result path for both early probe stages through the same array-backed source boundary
  - the midpoint-only `EarlyProbeStageLayout<3, _>` specialization is gone
  - the temporary `EarlyProbeStageProgress` enum and the one-use `continue_stage()` bounce are gone
  - [`MIDPOINT_EARLY_PROBE_STAGE_LAYOUT`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) and [`OUTER_EARLY_PROBE_STAGE_LAYOUT`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now hold direct request-source indices plus sample roles
  - [`EarlyProbeRefinementStages`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now owns the full typed midpoint-stage, outer-stage, and terminal dispatch path directly, so the old `prepare_outer_samples()` bounce is gone
  - the old [`EarlyProbeRefinementPipeline`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) bounce is gone
  - the top-level early probe entry now delegates raw `start` / `midpoint` / `end` probe inputs directly into [`EarlyProbeRefinementStages`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs)
  - the temporary `EarlyProbeRefinementSource` carrier and its one-use `stage_source()` view are gone
  - [`EarlyProbeRefinementStages`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now owns midpoint-stage source materialization, outer-stage source materialization, the typed interval-aware segment handoff, and the adaptive-chase count directly, so the old [`EarlyProbeRefinementTerminal`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) bounce is gone too
  - the old inline midpoint-only `match index { 0, 1, 2 }` resolver in `EarlyProbeRefinementStages` is gone
  - the old midpoint-only `EarlyProbeTripletSource` carrier is gone
  - the old midpoint-only `EarlyProbeStageSource::Triplet { start, midpoint, end }` variant is gone
  - the old [`EarlyProbeStageSource`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) wrapper is gone, and both early stages now pass raw `[NormalizedEdgeSample; N]` arrays straight into the shared typed stage runner
  - [`EarlyProbeRefinementStages`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now keeps midpoint-stage -> outer-stage -> interval-aware progression on explicit `ControlFlow<bool, [NormalizedEdgeSample; N]>` stage-progress values from [`EarlyProbeStageLayout`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs), so the old duplicated `refinement_result(...)? -> match Ok/Err` stage-step execution, the one-use local `stage_samples!` macro, the old `EarlyProbeStageOutcome` layer, the nested `and_then(...)` / `finish(...)` closure chain, and the duplicated handwritten `Ok(samples) => samples` / `Err(result) => return Some(result)` split are gone too
- The interval-aware refinement handoff remains typed and Rust-owned:
  - [`PreparedIntervalAwareRefinementSideLayout`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) carries coarse/outer/inner segment layouts
  - [`PreparedIntervalAwareRefinementSideLayouts`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now owns stronger coarse-side choice, winning outer-vs-inner segment selection, and the terminal `needs_refinement(...)` dispatch from the outer-stage closure in one typed helper boundary, with coarse/outer/midpoint candidates all staying on explicit segment outcomes during stronger-segment choice
  - [`PreparedRefinementTripletLayout::refinement_segment()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now returns an explicit `RefinementSegmentOutcome` for coarse and outer interval-aware segment candidates
  - triplet-layout segments and midpoint-probe segments now both go through the same shared `RefinementSegmentOutcome::from_samples(...)` constructor instead of each translating `RefinementSegment::new(...)` locally
  - [`PreparedRefinementSpanLayout::midpoint_segment()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now returns an explicit `RefinementSegmentOutcome` instead of nested `Option<Option<RefinementSegment>>`
  - midpoint span refinement and adaptive stronger-half refinement now both materialize midpoint candidates through the shared `midpoint_refinement_segment(...)` helper instead of each translating `MidpointEdgeProbeOutcome` into `RefinementSegmentOutcome` locally
  - midpoint segment creation and midpoint probe-pair creation now both reuse the shared typed midpoint-probe resolution boundary on `MidpointEdgeProbeOutcome`, instead of reinterpreting `MidpointEdgeProbeOutcome::{NoProbe, Probe(...)}` in separate callers
  - [`MidpointEdgeProbePairOutcome`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now owns the typed `Err(false)` vs staged-sample result translation for early probe stages, so the stage runner no longer matches `MidpointEdgeProbePairOutcome::{NoPair, Pair(...)}` itself
  - [`RefinementSegment::stronger_half()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now stays on that same explicit `RefinementSegmentOutcome` boundary instead of bouncing through a separate `StrongerHalfOutcome`
  - the four unsupported-edge extremum solvers now return an explicit `EdgeSampleExtremumOutcome` instead of nested `Option<Option<EdgeSample>>`
  - [`RefinementSegment`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) still owns score-based creation, stronger-segment choice, local-window checks, and the adaptive stronger-half chase
- The exercised non-solid offset shell fixture stays green on the Rust-first path.
- The exercised closed offset solid fixture stays green, including the direct shell-local parity assertion in [`ported_brep_uses_rust_owned_volume_for_offset_solids()`](rust/lean_occt/tests/brep_workflows.rs).

## Remaining Blocker

`offset_shell_bbox()` still ends at shell-local OCCT bbox for shells that fail all current validated Rust candidates. In the early unsupported-edge probe entry, the remaining structural duplication is narrower again:

- the early stage itself is now array-backed instead of split across array and triplet sample mapping
- the midpoint-stage and outer-stage request/sample mapping now live in the same typed stage layout
  - [`EarlyProbeStageLayout`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now owns direct `probe_pair(...) -> staged sample roles -> ControlFlow<bool, [NormalizedEdgeSample; N]>` execution for each early probe stage
  - the top-level entry now delegates straight into [`EARLY_PROBE_REFINEMENT_STAGES`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs)
  - the stage runner now receives raw `start` / `midpoint` / `end` inputs directly, and midpoint-stage, outer-stage, and interval-aware tail configuration all already live in [`EarlyProbeRefinementStages`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs)
  - the old `EarlyProbeStageLayout::refinement_result()`, `stage_progress()`, and `continue_with_stage()` bounces are gone
  - [`EarlyProbeRefinementStages`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now owns both midpoint-stage and outer-stage source materialization, plus the typed interval-aware tail dispatch
  - the stage runner now shares the stage-step `probe_pair(...)? -> staged samples/result` boundary across midpoint-stage and outer-stage
  - [`PreparedIntervalAwareRefinementSideLayouts`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now already owns the outer-stage closure’s `None => false`, winning-segment selection, and terminal `segment.needs_refinement(...)` dispatch together
  - the interval-aware segment path no longer carries ambiguous nested `Option` state: midpoint, coarse, and outer candidates now all use explicit `RefinementSegmentOutcome`, the early stage pair request uses an explicit probe-pair outcome, and the unsupported-edge extremum solvers use an explicit edge-sample outcome too
  - midpoint segment selection is now shared through `midpoint_refinement_segment(...)`, and the adaptive stronger-half chase now stays on `RefinementSegmentOutcome` instead of a separate half-only enum
  - but `EarlyProbeRefinementStages::needs_refinement()` still matches `ControlFlow::Continue(samples)` vs `ControlFlow::Break(result)` twice by hand, once for midpoint-stage and once for outer-stage, even though per-stage execution already lives on `EarlyProbeStageLayout` and the runner already owns the typed interval-aware tail

The next blocker is to keep that typed `ControlFlow` progress path but stop spelling the same direct `Continue/Break` stage unwrap twice in `EarlyProbeRefinementStages::needs_refinement()`, so the early stage runner stays on one Rust-owned midpoint-stage -> outer-stage -> interval-aware execution path under a smaller shared stage-progress boundary.

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

This turn moved the shared early-stage unwrap onto the typed stage boundary: midpoint-stage and outer-stage now progress through explicit `ControlFlow<bool, [NormalizedEdgeSample; N]>` values from `EarlyProbeStageLayout`, and the one-use local `stage_samples!` macro plus the old `EarlyProbeStageOutcome` layer are gone. That leaves the next real seam smaller again: `EarlyProbeRefinementStages::needs_refinement()` still spells out the same direct `ControlFlow::Continue/Break` match twice, even though the stage execution boundary and the typed interval-aware tail are already in place.
