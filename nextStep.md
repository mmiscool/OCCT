# Next Task

Keep narrowing the remaining shell-local OCCT bbox fallback in `offset_shell_bbox()`, but stay on the shell-boundary Rust path. The next bounded Rust-first cut is to collapse the remaining parallel early-stage role-resolution layer in `sampled_edge_interval_needs_probe_refinement()`, so midpoint-stage and outer-stage request-role/sample-role source mapping stops living behind two adjacent trait families.

## Current State

- [`ported_shape_summary()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) still keeps the narrowed offset bbox tiers:
  - non-solid offset shapes validate Rust mesh, expanded Rust mesh, Rust face-BRep union, then only later use narrower OCCT bbox fallbacks
  - offset solids/compsolids use [`offset_solid_shell_bbox()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs), and each shell now tries validated Rust face-BRep, Rust shell-boundary, Rust mesh, expanded Rust mesh, Rust `ported_brep(shell).summary`, expanded Rust `ported_brep(shell).summary`, and only then shell-local OCCT bbox
- [`load_ported_topology()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs) preserves loader-owned `PreparedShellShape { shell_shape, shell_vertex_shapes, shell_edge_shapes, shell_face_shapes }` on the successful Rust-topology path.
- [`shell_boundary_shape_bbox()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) remains a mixed Rust/public shell-boundary union:
  - it always starts from loader-owned shell vertices
  - it unions exact public edge bbox results when available
  - unsupported shell edges no longer kill the candidate immediately
  - unsupported shell edges still go through adaptive public-edge sampling, recursive interval refinement, tangent-root polish, near-flat tangent-dip probing, local axis-position extremum search, seeded axis-position search, and the shared stronger-half refinement chase before mesh or OCCT fallback tiers
- [`sampled_edge_interval_needs_probe_refinement()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now keeps the early probe entry on one typed Rust-owned stage boundary:
  - [`EarlyProbeStageLayout`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) owns request execution, `NoPair` bailout, stage-local sample materialization, the local sliding-window check, and stage-to-stage handoff through `continue_refinement()`
  - [`MIDPOINT_EARLY_PROBE_STAGE_LAYOUT`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) and [`OUTER_EARLY_PROBE_STAGE_LAYOUT`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now inline their request-layout and sample-layout pairing directly
  - the old `PreparedEarlyProbeStage` enum and `prepare()` bounce are gone
- The remaining stage-local sample ordering is now shared through one generic typed sample-layout boundary:
  - [`EarlyProbeStageSamplesLayout`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now materializes both the 5-sample midpoint chain and the 7-sample outer chain
  - [`MidpointProbeSamplesLayout`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) is gone
  - [`OuterProbeSamplesLayout`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) is gone
  - [`MidpointProbeSampleRole`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) and [`OuterProbeSampleRole`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now just implement the shared [`EarlyProbeStageSampleRole`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) mapping hook
- The interval-aware refinement handoff remains typed and Rust-owned:
  - [`PreparedIntervalAwareRefinementSideLayout`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) carries the coarse/outer/inner segment layouts
  - [`PreparedIntervalAwareRefinementSideLayouts`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) chooses the stronger coarse side and the winning outer-vs-inner segment before handing off to the shared refinement path
  - [`RefinementSegment`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) still owns score-based creation, stronger-segment choice, local-window checks, and the adaptive stronger-half chase
- The exercised non-solid offset shell fixture stays green on the Rust-first path.
- The exercised closed offset solid fixture stays green, including the direct shell-local parity assertion in [`ported_brep_uses_rust_owned_volume_for_offset_solids()`](rust/lean_occt/tests/brep_workflows.rs).

## Remaining Blocker

`offset_shell_bbox()` still ends at shell-local OCCT bbox for shells that fail all current validated Rust candidates. In the early unsupported-edge probe entry, the structural duplication is now much smaller:

- the duplicated midpoint/outer stage-pair wiring is gone
- the duplicated stage-local sample-layout structs are gone
- the early stage now already runs through one typed `EarlyProbeStageLayout` boundary
- but the midpoint-stage and outer-stage source mapping still lives in two adjacent trait families:
  - [`MidpointEdgeProbePairRequestSampleRole`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) resolves request endpoints from a typed source
  - [`EarlyProbeStageSampleRole`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) resolves stage-local samples from the same typed source plus the probe pair

The next blocker is to collapse that remaining parallel role-resolution layer without reintroducing transient midpoint or outer carriers.

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

This turn finished the generic early-stage sample-layout extraction. `sampled_edge_interval_needs_probe_refinement()` now keeps both stages on the same typed `EarlyProbeStageLayout` path, and the stage-local 5-sample and 7-sample orderings now share one generic `EarlyProbeStageSamplesLayout` boundary. That leaves the next real duplication immediately beside it: request-role and sample-role source resolution are still parallel typed mappings over the same midpoint-stage and outer-stage sources. Collapsing that next keeps the shell-boundary Rust path cleaner without widening fallback.
