# Next Task

Keep narrowing the remaining shell-local OCCT bbox fallback in `offset_shell_bbox()`, but stay on the shell-boundary Rust path. The next bounded Rust-first cut is to collapse the remaining mirrored left/right side-chain extraction in `PreparedIntervalAwareRefinementSideChains::from_outer_probe_chain()`, so the interval-aware side-chain pair stops spelling out both side-local outer-probe orderings directly.

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
- The early and late unsupported-edge refinement path is now structurally tighter:
  - [`midpoint_edge_probe_pair()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) prepares the earlier midpoint and outer probe pairs once for the probe-refinement entry stages
  - [`sampled_edge_sample_windows_need_refinement()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) owns the shared sliding 3-sample window checks used by those early entry stages
  - [`PreparedMidpointProbeChain`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) owns the `start/first_probe/midpoint/second_probe/end` early probe carrier and decides when midpoint-stage evidence is strong enough to advance into outer probes
  - [`PreparedOuterProbeChain`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) owns the `start/left_outer_probe/first_probe/midpoint/second_probe/right_outer_probe/end` carrier and now delegates interval-aware side pair preparation to [`PreparedIntervalAwareRefinementSides`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs)
  - [`PreparedIntervalAwareRefinementSides`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now owns left/right side-pair construction and stronger-side choice before handing the winning segment to the shared refinement path
  - [`PreparedIntervalAwareRefinementSideChains`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now owns the smaller prepared side-chain boundary between the outer probe chain and the typed interval-aware side layouts
  - [`PreparedIntervalAwareRefinementSideLayouts`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now owns the dedicated typed layout-pair boundary between the outer probe chain and the final interval-aware side pair
  - [`PreparedIntervalAwareRefinementSide`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now carries typed [`PreparedRefinementTriplet`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) coarse/outer descriptors and a typed [`PreparedRefinementSpan`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) inner descriptor, so the old eight-field positional constructor is gone
  - [`RefinementSegment`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) owns score-based creation, stronger-segment choice, the local-window test, and the stronger-half chase
  - [`midpoint_edge_probe()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) serves both the interval-aware inner probe path and the later stronger-half narrowing path
  - [`half_refinement_should_continue()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) provides the shared signal/span-driven adaptive stop rule, with the max-step limit kept only as a safety ceiling
- The exercised non-solid offset shell fixture stays green on the Rust-first path.
- The exercised closed offset solid fixture stays green, including the direct shell-local parity assertion in [`ported_brep_uses_rust_owned_volume_for_offset_solids()`](rust/lean_occt/tests/brep_workflows.rs).

## Remaining Blocker

`offset_shell_bbox()` still ends at shell-local OCCT bbox for shells that fail all current validated Rust candidates. The deeper late refinement tail is no longer the structural problem: early probe preparation, local probe-window checks, interval-aware side-pair preparation, and the later stronger-half refinement now share the same midpoint-probe and scored-segment machinery.

The remaining duplication is no longer the raw early probe chain, the stage wrappers, the interval-aware staging carrier, the open-coded execution step on the prepared segment, the old side enum, the pair construction in `PreparedOuterProbeChain`, the mirrored `PreparedIntervalAwareRefinementSide::left()` / `right()` remap, the inline triplet/span assembly in `PreparedIntervalAwareRefinementSideLayouts::from_outer_probe_chain()`, or the old eight-field `PreparedIntervalAwareRefinementSide::new(...)` call site. The remaining structural gap is the mirrored side-local chain extraction in [`PreparedIntervalAwareRefinementSideChains::from_outer_probe_chain()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs). That side-chain builder still has to:

- choose whether the side-local outer probe comes before or after the pivot for the left side
- choose the opposite ordering for the right side
- repeat that same mirrored side-local extraction once for the left chain and again for the right chain before layout choice even starts

The next blocker is to move that remaining mirrored side-local chain wiring behind a smaller prepared side-window boundary, so the interval-aware side-chain pair can be built from one typed outer-probe-chain path instead of two inline left/right extractions.

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

This turn carried the interval-aware cleanup one step deeper: the side-local triplet/span assembly is no longer attached to the layout-pair builder itself, and the new `PreparedIntervalAwareRefinementSideChains` boundary now owns the typed handoff from the outer probe chain into the final interval-aware side layouts.

The next bounded step is to remove the remaining mirrored chain remap in `PreparedIntervalAwareRefinementSideChains::from_outer_probe_chain()`. If that last inline left/right side extraction is collapsed behind a smaller prepared side-window boundary, the unsupported-edge shell-boundary path gets a cleaner Rust-owned interval-aware refinement entry without adding another fallback tier.
