# Next Task

Keep narrowing the remaining shell-local OCCT bbox fallback in `offset_shell_bbox()`, but stay on the shell-boundary Rust path. The next bounded Rust-first cut is to collapse the remaining midpoint-vs-outer `source_ordinal()` alias in the early shell-edge refinement path, so stage-specific source-slot enums stop each carrying the same `self as usize` cast before the shared ordinal-to-position boundary.

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
  - [`MidpointEdgeProbePairRequestLayout`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now owns the typed `source spans -> probe pair outcome` handoff and direct probe execution for early probe stages through explicit `first` / `second` [`MidpointEdgeProbeSpanLayout`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) values, so [`EarlyProbeStageLayout::refinement_result()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) no longer carries a raw `[usize; 4]` request-source array or bounces through a second request carrier
  - midpoint-stage and outer-stage sample reuse now goes through typed [`MidpointEarlyProbeSourceSampleLayout`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) / [`OuterEarlyProbeSourceSampleLayout`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) enums plus typed [`MidpointEarlyProbeSampleLayout`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) / [`OuterEarlyProbeSampleLayout`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) descriptors, so the stage constants no longer hard-code `Source(usize)` entries
  - midpoint-stage and outer-stage sample reuse now shares one generic Rust-side `EarlyProbeSampleRole` / `EarlyProbeSampleLayout` path behind the typed source-slot enums, so the old duplicated midpoint-vs-outer `source_sample()` / `stage_sample()` ladders are gone
  - midpoint-stage and outer-stage typed source-slot enums now terminate in one shared [`EarlyProbeSourcePosition`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) boundary, so the old raw positional `source_index()` remaps are gone too
  - midpoint-stage and outer-stage typed source-slot enums now also share one typed [`EarlyProbeSourceSlot`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) -> `EarlyProbeSourcePosition::from_source_ordinal(...)` path, so the old duplicated `source_position()` matches are gone too
  - [`EarlyProbeStageLayout::refinement_result()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now owns the shared typed Rust-owned `Result<[NormalizedEdgeSample; N], bool>` stage result path for both early probe stages through the same array-backed source boundary
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
  - the old [`EarlyProbeStageSource`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) wrapper is gone, and both early stages now pass raw `[NormalizedEdgeSample; N]` arrays straight into [`EarlyProbeStageLayout::refinement_result()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs)
  - [`EarlyProbeRefinementStages`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now routes both midpoint-stage and outer-stage through the shared typed `stage_progress()` / `continue_with_stage()` boundary, so the old duplicated `refinement_result(...)? -> match Ok/Err` stage-step execution is gone too
- The interval-aware refinement handoff remains typed and Rust-owned:
  - [`PreparedIntervalAwareRefinementSideLayout`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) carries coarse/outer/inner segment layouts
  - [`PreparedIntervalAwareRefinementSideLayouts`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now owns stronger coarse-side choice, winning outer-vs-inner segment selection, and the terminal `needs_refinement(...)` dispatch from the outer-stage closure in one typed helper boundary, with coarse/outer/midpoint candidates all staying on explicit segment outcomes during stronger-segment choice
  - [`PreparedRefinementTripletLayout::refinement_segment()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now returns an explicit `RefinementSegmentOutcome` for coarse and outer interval-aware segment candidates
  - triplet-layout segments and midpoint-probe segments now both go through the same shared `RefinementSegmentOutcome::from_samples(...)` constructor instead of each translating `RefinementSegment::new(...)` locally
  - [`PreparedRefinementSpanLayout::midpoint_segment()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now returns an explicit `RefinementSegmentOutcome` instead of nested `Option<Option<RefinementSegment>>`
  - midpoint span refinement and adaptive stronger-half refinement now both materialize midpoint candidates through the shared `midpoint_refinement_segment(...)` helper instead of each translating `MidpointEdgeProbeOutcome` into `RefinementSegmentOutcome` locally
  - midpoint segment creation and midpoint probe-pair creation now both reuse the shared typed midpoint-probe resolution boundary on `MidpointEdgeProbeOutcome`, instead of reinterpreting `MidpointEdgeProbeOutcome::{NoProbe, Probe(...)}` in separate callers
  - [`MidpointEdgeProbePairOutcome`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now owns the typed `Err(false)` vs staged-sample result translation for early probe stages, so [`EarlyProbeStageLayout::refinement_result()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) no longer matches `MidpointEdgeProbePairOutcome::{NoPair, Pair(...)}` itself
  - [`RefinementSegment::stronger_half()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now stays on that same explicit `RefinementSegmentOutcome` boundary instead of bouncing through a separate `StrongerHalfOutcome`
  - the four unsupported-edge extremum solvers now return an explicit `EdgeSampleExtremumOutcome` instead of nested `Option<Option<EdgeSample>>`
  - [`RefinementSegment`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) still owns score-based creation, stronger-segment choice, local-window checks, and the adaptive stronger-half chase
- The exercised non-solid offset shell fixture stays green on the Rust-first path.
- The exercised closed offset solid fixture stays green, including the direct shell-local parity assertion in [`ported_brep_uses_rust_owned_volume_for_offset_solids()`](rust/lean_occt/tests/brep_workflows.rs).

## Remaining Blocker

`offset_shell_bbox()` still ends at shell-local OCCT bbox for shells that fail all current validated Rust candidates. In the early unsupported-edge probe entry, the remaining structural duplication is narrower again:

- the early stage itself is now array-backed instead of split across array and triplet sample mapping
- the midpoint-stage and outer-stage request/sample mapping now live in the same typed stage layout
- the stage layout now returns typed `Result<[NormalizedEdgeSample; N], bool>` directly through one shared `refinement_result()` entry
- the top-level entry now delegates straight into [`EARLY_PROBE_REFINEMENT_STAGES`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs)
- the stage runner now receives raw `start` / `midpoint` / `end` inputs directly, and midpoint-stage, outer-stage, and interval-aware tail configuration all already live in [`EarlyProbeRefinementStages`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs)
- the midpoint-stage and outer-stage now both reach the shared stage layout through the same direct array-backed `refinement_result()` entry
- [`EarlyProbeRefinementStages`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now owns both midpoint-stage and outer-stage source materialization, plus the typed interval-aware tail dispatch
- the stage runner now shares the stage-step `refinement_result(...)? -> ControlFlow` boundary across midpoint-stage and outer-stage
- [`PreparedIntervalAwareRefinementSideLayouts`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now already owns the outer-stage closure’s `None => false`, winning-segment selection, and terminal `segment.needs_refinement(...)` dispatch together
- the interval-aware segment path no longer carries ambiguous nested `Option` state: midpoint, coarse, and outer candidates now all use explicit `RefinementSegmentOutcome`, the early stage pair request uses an explicit probe-pair outcome, and the unsupported-edge extremum solvers use an explicit edge-sample outcome too
- midpoint segment selection is now shared through `midpoint_refinement_segment(...)`, and the adaptive stronger-half chase now stays on `RefinementSegmentOutcome` instead of a separate half-only enum
- but the typed source-slot enums still duplicate the one-line ordinal alias one level smaller: midpoint-stage and outer-stage each still keep their own `source_ordinal(self) -> self as usize` implementation before the shared `EarlyProbeSourcePosition::from_source_ordinal(...)` boundary

The next blocker is to collapse that remaining midpoint-vs-outer `source_ordinal()` alias into a smaller shared typed boundary, so the stage-specific source-slot enums stop each carrying the same ordinal cast before the shared position mapping.

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

This turn finished the shared typed source-position cut: midpoint-stage and outer-stage source-slot enums now go through the same `EarlyProbeSourceSlot` -> `EarlyProbeSourcePosition::from_source_ordinal(...)` boundary, and the duplicated `source_position()` matches are gone. That leaves the next real seam one level smaller again: the stage-specific source-slot enums still duplicate the trivial ordinal alias in their own `source_ordinal()` implementations.
