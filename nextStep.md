# Next Task

Keep narrowing the remaining shell-local OCCT bbox fallback in `offset_shell_bbox()`, but stay on the shell-boundary Rust path. The next bounded Rust-first cut is to keep the typed midpoint stage, typed stage-result continuation tail, typed outer stage, typed interval-aware tail, and the typed stage-result boundary in place while collapsing the now-thin fixed continuation bounce inside `EarlyProbeStageResultTail::needs_refinement(...)`:

`midpoint_stage.stage_samples_or_refinement(...).continue_with_stage_result_tail(self, ...)`

without reintroducing the old kickoff wrapper or stage-chain stack.

## Current State

- `ported_shape_summary()` still keeps the narrowed offset bbox tiers:
  - non-solid offset shapes validate Rust mesh, expanded Rust mesh, Rust face-BRep union, then only later use narrower OCCT bbox fallbacks
  - offset solids and compsolids use `offset_solid_shell_bbox()`, and each shell still tries validated Rust face-BRep, Rust shell-boundary, Rust mesh, expanded Rust mesh, Rust `ported_brep(shell).summary`, expanded Rust `ported_brep(shell).summary`, and only then shell-local OCCT bbox
- `load_ported_topology()` still preserves loader-owned `PreparedShellShape { shell_shape, shell_vertex_shapes, shell_edge_shapes, shell_face_shapes }` on the successful Rust-topology path.
- `shell_boundary_shape_bbox()` is still a mixed Rust/public shell-boundary union:
  - it always starts from loader-owned shell vertices
  - it unions exact public edge bbox results when available
  - unsupported shell edges still stay on the public Rust route through adaptive sampling, recursive interval refinement, tangent-root polish, near-flat tangent-dip probing, local axis-position extremum search, seeded axis-position search, and the shared stronger-half refinement chase before mesh or OCCT fallback tiers
- The early unsupported-edge probe entry is now fully array-backed on the Rust side:
  - `EarlyProbeStageLayout` works directly on `[NormalizedEdgeSample; N]`
  - `MidpointEdgeProbePairRequestLayout` owns the typed `source spans -> probe pair outcome` handoff
  - midpoint-stage and outer-stage sample reuse goes through shared `EarlyProbeSampleRole` and `EarlyProbeSourcePosition`
  - `EarlyProbeStageLayout::stage_samples_or_refinement(...)` owns the per-stage `Result<[NormalizedEdgeSample; N], Option<bool>>` carry
  - `EarlyProbeStageResult` owns midpoint-stage to outer-stage continuation plus the fixed outer-stage + interval-aware tail handoff
  - the fixed early-probe kickoff now stays on the typed continuation tail directly:
    - `MIDPOINT_EARLY_PROBE_STAGE_LAYOUT` still owns the raw 3-sample midpoint-stage entry through `stage_samples_or_refinement(...)`
    - `EarlyProbeStageResultTail` now owns both the fixed midpoint-stage kickoff and the fixed outer-stage + interval-aware continuation from the top-level entry
  - the old `EarlyProbeKickoff`, `EarlyProbeStageChain`, `EarlyProbeRefinementStages`, `EarlyProbeStageSequence`, `EarlyProbeStagePair`, `EarlyProbeOuterStageTail`, `EarlyProbeStageSamplesOrRefinement`, and the top-level `sampled_edge_interval_needs_probe_refinement()` bridge are all gone
  - the old standalone `early_probe_needs_refinement(...)` helper is gone
  - the old midpoint-stage kickoff helper is gone
  - the top-level early probe entry inside `refine_sampled_edge_interval()` now delegates straight to `EARLY_PROBE_STAGE_RESULT_TAIL.needs_refinement(...)`, so the interval-refinement path no longer spells the fixed kickoff composition inline or uses a dedicated kickoff wrapper, a one-off free helper, or a one-use midpoint-stage shim
- The interval-aware refinement handoff remains typed and Rust-owned:
  - `PreparedIntervalAwareRefinementSideLayouts` owns stronger coarse-side choice, winning outer-vs-inner segment selection, and terminal `segment.needs_refinement(...)` dispatch directly on the final 7-sample boundary
  - midpoint, coarse, and outer candidates all stay on explicit `RefinementSegmentOutcome`
  - midpoint span refinement and adaptive stronger-half refinement both go through shared `midpoint_refinement_segment(...)`
  - the unsupported-edge extremum solvers all return explicit `EdgeSampleExtremumOutcome`
- The exercised non-solid offset shell fixture stays green on the Rust-first path.
- The exercised closed offset solid fixture stays green, including the direct shell-local parity assertion in `ported_brep_uses_rust_owned_volume_for_offset_solids()`.

## Remaining Blocker

`offset_shell_bbox()` still ends at shell-local OCCT bbox for shells that fail all current validated Rust candidates. In the early unsupported-edge probe entry, the remaining structural seam is now very small:

- the top-level early probe entry no longer spells the fixed kickoff composition inline
- the midpoint-stage kickoff, outer-stage progression, and interval-aware tail are all already typed
- the old kickoff wrapper and stage-chain wrapper are gone
- but `EarlyProbeStageResultTail::needs_refinement(...)` still bridges kickoff into continuation through `midpoint_stage.stage_samples_or_refinement(...).continue_with_stage_result_tail(self, ...)`

The next blocker is to keep those typed pieces and the typed continuation-tail boundary, but collapse that one-use continuation bounce so the tail owns the full fixed kickoff-and-continuation path directly.

## Focus

1. Keep the non-solid offset bbox win in place.
2. Keep the now-green direct shell parity check for the exercised closed offset solid.
3. Stay on loader-owned shell-local inventories; do not reintroduce fresh raw `subshapes_occt()` traversal.
4. Keep the shell-boundary candidate on the public Rust edge and vertex path.
5. Keep validating every accepted shell candidate against shell-local OCCT bbox.
6. Prefer structural Rust-side refinement improvements over adding another copied probe tier or another isolated chooser.
7. Keep the verification bar unchanged:
   - `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
   - `cargo check --manifest-path rust/lean_occt/Cargo.toml`
   - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
   - `cargo test --manifest-path rust/lean_occt/Cargo.toml`

## Why This Is Next

This turn moved the fixed kickoff composition off the top-level entry and onto `EarlyProbeStageResultTail`. The remaining seam is smaller now: the early-probe entry is down to one typed tail call, and the only kickoff bridge left is the one-use `stage_samples_or_refinement(...).continue_with_stage_result_tail(...)` path inside that tail boundary.
