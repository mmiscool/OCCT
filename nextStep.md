# Next Task

Keep narrowing the remaining shell-local OCCT bbox fallback in `offset_shell_bbox()`, but stay on the shell-boundary Rust path. The next bounded Rust-first cut is to collapse the remaining raw interval source and final segment handoff inside `EarlyProbeRefinementPipeline::needs_refinement()`, so the early unsupported-edge probe path stops open-coding `[start, midpoint, end]` source assembly and the final `prepare_refinement_segment(...)? -> Some(false)` boundary inside the pipeline body.

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
  - [`EarlyProbeSampleRole`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now only describes stage-local sample ordering over an already-materialized source array
  - [`MIDPOINT_EARLY_PROBE_STAGE_LAYOUT`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) and [`OUTER_EARLY_PROBE_STAGE_LAYOUT`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now hold direct request-source indices plus sample roles
  - [`EarlyProbeRefinementPipeline`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now owns the midpoint stage, outer stage, typed interval-aware side layouts, and the adaptive-chase check count, so [`sampled_edge_interval_needs_probe_refinement()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) is down to a direct pipeline delegate
- The interval-aware refinement handoff remains typed and Rust-owned:
  - [`PreparedIntervalAwareRefinementSideLayout`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) carries coarse/outer/inner segment layouts
  - [`PreparedIntervalAwareRefinementSideLayouts`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) chooses the stronger coarse side and the winning outer-vs-inner segment before handing off to the shared refinement path
  - [`RefinementSegment`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) still owns score-based creation, stronger-segment choice, local-window checks, and the adaptive stronger-half chase
- The exercised non-solid offset shell fixture stays green on the Rust-first path.
- The exercised closed offset solid fixture stays green, including the direct shell-local parity assertion in [`ported_brep_uses_rust_owned_volume_for_offset_solids()`](rust/lean_occt/tests/brep_workflows.rs).

## Remaining Blocker

`offset_shell_bbox()` still ends at shell-local OCCT bbox for shells that fail all current validated Rust candidates. In the early unsupported-edge probe entry, the remaining structural duplication is no longer the two-stage ladder in the top-level function:

- the early stage itself is now array-only
- the midpoint-stage and outer-stage request/sample mapping now live in the same typed stage layout
- the top-level entry now delegates through [`EARLY_PROBE_REFINEMENT_PIPELINE`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs)
- but [`EarlyProbeRefinementPipeline::needs_refinement()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) still:
  - builds the midpoint `[start, midpoint, end]` source array inline
  - performs the final typed interval-aware segment handoff inline after the outer stage
  - carries the `prepare_refinement_segment(...)?` to `Some(false)` conversion inline rather than through a typed terminal boundary

The next blocker is to move that remaining source-plus-terminal glue behind typed Rust-side boundaries without reintroducing transient midpoint or outer carriers.

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

This turn removed the remaining top-level early-probe ladder from `sampled_edge_interval_needs_probe_refinement()`. The midpoint stage, outer stage, typed interval-aware side layouts, and adaptive-chase check count now live under one `EarlyProbeRefinementPipeline`, and the entry point is just a direct delegate. That leaves the next real duplication one level lower: the pipeline still open-codes the initial `[start, midpoint, end]` source assembly and the final typed segment handoff instead of reusing smaller Rust-side source and terminal boundaries.
