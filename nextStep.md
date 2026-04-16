# Next Task

Keep narrowing the remaining shell-local OCCT bbox fallback in `offset_shell_bbox()`, but stay on the shell-boundary Rust path. The next bounded Rust-first cut is to collapse the remaining explicit `left` / `right` pair handling inside the new interval-aware side-descriptor pair helper, so that side-pair construction becomes one generic fixed-pair pass before stronger-side choice instead of spelling out dual fields and dual side builds.

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
  - [`midpoint_edge_probe_pair()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) prepares the earlier midpoint and outer probe pairs once for the probe-refinement entry stages
  - [`sampled_edge_sample_windows_need_refinement()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) owns the shared sliding 3-sample window checks used by those early entry stages
  - [`PreparedMidpointProbeChain`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) owns the `start/first_probe/midpoint/second_probe/end` early probe carrier and decides when midpoint-stage evidence is strong enough to advance into outer probes
  - [`PreparedOuterProbeChain`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) owns the `start/left_outer_probe/first_probe/midpoint/second_probe/right_outer_probe/end` carrier and now hands its sample array directly to the typed interval-aware side-descriptor pair
  - [`PreparedIntervalAwareRefinementSideWindow`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) is gone
  - the mirrored `PreparedIntervalAwareRefinementSide::left()` / `right()` remap is gone
  - [`PreparedIntervalAwareRefinementSide`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now builds either side through one generic [`PreparedIntervalAwareRefinementSideDescriptor`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) path
  - [`PreparedIntervalAwareRefinementSideDescriptors`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now owns interval-aware side-pair construction and stronger-side choice before handing the winning segment to the shared refinement path
  - [`RefinementSegment`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) owns score-based creation, stronger-segment choice, the local-window test, and the stronger-half chase
  - [`midpoint_edge_probe()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) serves both the interval-aware inner probe path and the later stronger-half narrowing path
  - [`half_refinement_should_continue()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) provides the shared signal/span-driven adaptive stop rule, with the max-step limit kept only as a safety ceiling
- The exercised non-solid offset shell fixture stays green on the Rust-first path.
- The exercised closed offset solid fixture stays green, including the direct shell-local parity assertion in [`ported_brep_uses_rust_owned_volume_for_offset_solids()`](rust/lean_occt/tests/brep_workflows.rs).

## Remaining Blocker

`offset_shell_bbox()` still ends at shell-local OCCT bbox for shells that fail all current validated Rust candidates. The remaining structural duplication is now inside the interval-aware side-descriptor pair helper:

- [`PreparedIntervalAwareRefinementSideDescriptor`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) is now the generic side constructor input
- the old dual constants and the thin [`PreparedIntervalAwareRefinementSides::from_samples()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) wrapper are gone
- [`PreparedIntervalAwareRefinementSideDescriptors`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) still stores the pair as explicit `left` / `right` fields and still constructs `left` and `right` sides separately before stronger-side choice

The next blocker is to collapse that remaining pair handling behind one smaller generic fixed-pair boundary, so the interval-aware side pair stops spelling out dual fields and dual builds inside the helper and stays more Rust-owned before it hands off to the shared refinement machinery.

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

This turn moved the old dual descriptor constants and the thin `PreparedIntervalAwareRefinementSides::from_samples()` bounce behind one typed `PreparedIntervalAwareRefinementSideDescriptors` boundary. That means the outer-probe path now hands off directly to a descriptor-pair helper instead of spelling out side-pair construction through a separate wrapper.

What remains is the explicit pair handling inside that helper itself: it still stores `left` and `right` separately and still builds both sides through duplicated field access before stronger-side choice. If that final pair logic moves behind one generic fixed-pair boundary, the interval-aware refinement entry will stay cleaner on the Rust-owned side without adding any new fallback tier.
