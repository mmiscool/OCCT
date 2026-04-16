# Next Task

Keep narrowing the remaining shell-local OCCT bbox fallback in `offset_shell_bbox()`, but stay on the shell-boundary Rust path. The next bounded Rust-first cut is to collapse the remaining stage-specific sample-resolution duplication across `MidpointEdgeProbePairRequestLayout<MidpointStageProbeRequestSampleRole>` and `MidpointEdgeProbePairRequestLayout<OuterStageProbeRequestSampleRole>`, so the early probe entry owns one generic typed request-layout materialization path instead of two parallel `request(...)` impls.

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
- The unsupported-edge refinement path is now structurally tighter:
  - [`MidpointEdgeProbePairRequest`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) and the stage request layouts now prepare the earlier midpoint and outer probe pairs for the probe-refinement entry stages
  - [`sampled_edge_sample_windows_need_refinement()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) owns the shared sliding 3-sample window checks used by those early entry stages
- [`PreparedMidpointProbeChain`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now owns a stable five-sample array boundary for the `start/first_probe/midpoint/second_probe/end` early probe carrier, decides when midpoint-stage evidence is strong enough to advance into outer probes, and now prepares the outer-probe chain itself
- the old transient [`PreparedOuterProbeSeed`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) bounce is gone, so the raw `[0] / [1] / [3] / [4]` remap no longer crosses an extra carrier boundary
- [`PreparedMidpointProbeChain::new()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) and [`PreparedOuterProbeChain::from_midpoint_probe_chain()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now own the typed `MidpointEdgeProbePair` to sample-array assembly, so those early carriers no longer spell out the pair application inline
- [`MidpointEdgeProbePairRequest`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now owns the actual midpoint-pair sampling path directly, so the old one-use free `midpoint_edge_probe_pair(...)` helper is gone and the typed request boundary now carries both request data and probe execution
- [`MidpointEdgeProbePairRequestLayout`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now owns the stage-specific request layouts too:
  - `PreparedMidpointProbeChain` no longer carries `midpoint_probe_request()` or `outer_probe_request()` wrapper bounces
  - [`MIDPOINT_STAGE_PROBE_REQUEST_LAYOUT`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now drives midpoint-stage request assembly directly from `prepare()`
  - [`OUTER_STAGE_PROBE_REQUEST_LAYOUT`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now drives outer-stage request assembly directly from `prepare_outer_probe_chain()`
- [`PreparedOuterProbeChain`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now owns a stable seven-sample array boundary, keeps the interval-aware handoff directly in `needs_refinement()`, and receives that prepared sample array straight from the midpoint probe carrier
  - [`PreparedIntervalAwareRefinementSideWindow`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) is gone
  - the mirrored `PreparedIntervalAwareRefinementSide::left()` / `right()` remap is gone
  - [`PreparedIntervalAwareRefinementSide`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) is gone
  - [`PreparedRefinementTriplet`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) and [`PreparedRefinementSpan`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) are gone
  - [`PreparedRefinementTripletLayout`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs), [`PreparedRefinementSpanLayout`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs), and [`PreparedIntervalAwareRefinementSideLayout`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now carry the typed coarse/outer/inner side-layout boundary and materialize interval-aware segments directly from the outer-probe sample array
  - [`PreparedIntervalAwareRefinementSideLayouts`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now owns interval-aware side-pair construction through a named `left` / `right` typed pair, compares the coarse segments on that pair boundary, and keeps both winning-side descriptor materialization and outer-vs-inner winning-segment choice there before handing the winning segment to the shared refinement path
  - [`PreparedIntervalAwareRefinementSideLayout`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) is down to the typed coarse/outer/inner layout data; the old pair-to-layout segment-prep bounce is gone
  - [`RefinementSegment`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) owns score-based creation, stronger-segment choice, the local-window test, and the stronger-half chase
  - [`midpoint_edge_probe()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) serves both the interval-aware inner probe path and the later stronger-half narrowing path
  - [`half_refinement_should_continue()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) provides the shared signal/span-driven adaptive stop rule, with the max-step limit kept only as a safety ceiling
- The exercised non-solid offset shell fixture stays green on the Rust-first path.
- The exercised closed offset solid fixture stays green, including the direct shell-local parity assertion in [`ported_brep_uses_rust_owned_volume_for_offset_solids()`](rust/lean_occt/tests/brep_workflows.rs).

## Remaining Blocker

`offset_shell_bbox()` still ends at shell-local OCCT bbox for shells that fail all current validated Rust candidates. The next structural duplication is smaller and lives inside interval-aware side-pair storage:

- the eager dual-side materialization is gone
- the raw `self.0[0]` / `self.0[1]` stronger-side indexing is gone
- the anonymous two-layout array storage is gone
- the temporary winning [`PreparedIntervalAwareRefinementSide`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) carrier is gone
- the temporary [`PreparedRefinementTriplet`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) and [`PreparedRefinementSpan`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) carriers are gone
- [`PreparedIntervalAwareRefinementSideLayouts`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now compares coarse segments on the layout path and keeps both winning-side descriptor materialization and outer-vs-inner winning-segment choice on that same typed boundary

The next blocker is now the smaller duplicated typed-request materialization inside that stage-layout boundary:

- the pair-to-layout winning-segment bounce is gone
- the one-use [`PreparedOuterProbeChain::prepare_interval_aware_refinement_segment()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) wrapper is gone
- the one-use [`PreparedOuterProbeChain::samples()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) rematerialization is gone
- the one-use [`PreparedMidpointProbeChain::samples()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) rematerialization is gone
- the one-use [`PreparedOuterProbeChain::prepare()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) wrapper is gone
- the raw midpoint-sample index remaps are gone
- the one-use [`PreparedOuterProbeSeed`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) bounce is gone
- the duplicated `MidpointEdgeProbePair` to sample-array assembly is gone
- the repeated inline `midpoint_edge_probe_pair(...)` request call is gone
- the one-use free [`midpoint_edge_probe_pair(...)`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) helper is gone
- the midpoint-stage and outer-stage wrapper bounces are gone
- but [`MidpointEdgeProbePairRequestLayout<MidpointStageProbeRequestSampleRole>`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) and [`MidpointEdgeProbePairRequestLayout<OuterStageProbeRequestSampleRole>`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) still each keep their own stage-specific `request(...)` sample-resolution path

The next blocker is to move that duplicated stage-specific `request(...)` materialization behind one generic typed sample-layout boundary too, so the early probe stage stays cleaner on the Rust-owned side before it advances into interval-aware refinement.

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

This turn finished the typed stage-request-layout cleanup step. `PreparedMidpointProbeChain` now routes both early probe stages directly through named `MidpointEdgeProbePairRequestLayout` constants, and those layouts now own the stage-specific request assembly while `MidpointEdgeProbePairRequest` still owns the actual midpoint-pair sampling logic.

What remains is the next smaller structural duplication inside that request-layout path: the midpoint carrier no longer spells out either stage-specific `MidpointEdgeProbePairRequest::new(...)` mapping, but the two stage role enums still materialize requests through separate `request(...)` impls. If that duplicated sample-resolution step moves behind one generic typed sample-layout boundary too, the early refinement entry will stay cleaner on the Rust-owned side without adding any new fallback tier.
