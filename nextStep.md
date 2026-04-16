# Next Task

Keep narrowing the remaining shell-local OCCT bbox fallback in `offset_shell_bbox()`, but stay on the shell-boundary Rust path. The unsupported-edge interval-aware stage now prepares its chosen side once and hands shared scored segments into the later stronger-half chase. The next bounded Rust-first cut is to remove the remaining hand-rolled midpoint and outer probe sampling in the earlier probe-refinement entry stages so they reuse the same probe-preparation boundary instead of open-coding `t` math and `edge_sample()` calls inline.

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
  - [`scored_refinement_segment()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now builds the reusable scored segment carrier used by both interval-aware and stronger-half refinement
  - [`choose_stronger_refinement_segment()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now owns the shared score-based segment choice used by the interval-aware `left/right` and `outer/inner` stage as well as the later stronger-half chase
  - [`choose_interval_aware_refinement_side()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) and [`prepare_interval_aware_refinement_side()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now isolate the last side-specific assembly at the interval-aware entry, including reuse of [`midpoint_edge_probe()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs)
  - [`sampled_edge_interval_needs_stronger_half_refinement()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) still owns the shared stronger-half chase
  - [`choose_stronger_refinement_half()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) and [`midpoint_edge_probe()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now serve the later shoulder/endpoint/terminal narrowing path through one helper with staged coarse checks
  - [`half_refinement_should_continue()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now provides the shared signal/span-driven adaptive stop rule, with the max-step limit kept only as a safety ceiling
- The exercised non-solid offset shell fixture stays green on the Rust-first path.
- The exercised closed offset solid fixture stays green, including the direct shell-local parity assertion in [`ported_brep_uses_rust_owned_volume_for_offset_solids()`](rust/lean_occt/tests/brep_workflows.rs).

## Remaining Blocker

`offset_shell_bbox()` still ends at shell-local OCCT bbox for shells that fail all current validated Rust candidates. The deeper late refinement tail is no longer the structural problem: interval-aware side preparation and later stronger-half refinement now share the same scored segment and midpoint-probe machinery.

The remaining duplication is now the earlier probe-construction entry. [`sampled_edge_interval_needs_probe_refinement()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) and [`sampled_edge_interval_needs_asymmetric_probe_refinement()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) still manually:

- compute midpoint and outer probe `t` values inline instead of reusing [`midpoint_edge_probe()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs)
- call `context.edge_sample(...)` directly for those probes instead of handing prepared probes or prepared segments into the later shared refinement path

The next blocker is to move those first and outer probe constructions onto a reusable helper or carrier so the early entry stages become: prepare midpoint probes once, run the existing local refinement checks, then hand the later interval-aware stage already-prepared probes instead of rebuilding them through ad hoc sampling logic.

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

This turn finished the intended interval-aware side-preparation step: the interval-aware stage now chooses a suspicious side, prepares its `outer` and `inner` candidate segments once, and reuses the shared scored segment chooser before the stronger-half chase.

The next bounded step is to push that same consolidation one stage earlier. If the probe-refinement entry stops rebuilding midpoint and outer probes through ad hoc `t` math and direct `edge_sample()` calls, the unsupported-edge shell-boundary path gets a cleaner Rust-owned probe-preparation boundary without adding another fallback tier.
