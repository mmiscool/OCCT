# Next Task

Keep narrowing the whole-shape OCCT bbox fallback in `ported_shape_summary()`, but stay on the parity-safe offset boundary that now exists. The next target is closed offset solids and compsolids: offset shells can already stay off whole-shape OCCT bbox, but offset solids still cannot.

## Current State

- [`ported_brep()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep.rs) now preserves the loaded root `vertex_shapes` inventory alongside the existing `edge_shapes` and `face_shapes` when the Rust topology path succeeds, and passes those inventories into [`ported_shape_summary()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs).
- [`ported_shape_summary()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now has a narrower offset bbox boundary:
  - non-solid offset shapes use [`offset_shape_bbox_occt()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs), which unions existing root subshape OCCT bboxes across the already-loaded vertex, edge, and face inventories before falling back further
  - offset solids and compsolids now skip the Rust mesh bbox path for bbox derivation and go straight back to whole-shape `describe_shape_occt()` for parity
- The exercised offset shell fixture in [`brep_workflows.rs`](rust/lean_occt/tests/brep_workflows.rs) now stays off whole-shape OCCT bbox and still matches OCCT parity.
- The exercised offset solid fixture in [`brep_workflows.rs`](rust/lean_occt/tests/brep_workflows.rs) is still summary-safe only because its bbox path explicitly stays on the whole-shape OCCT fallback.
- The broader Rust-owned bbox path remains in place for:
  - face-free shapes through analytic edges, line segments, or vertex points
  - all-plane / cylinder / cone face sets through analytic boundary edges
  - swept face sets when every boundary edge already has a Rust `PortedCurve`

## Remaining Blocker

Closed offset solids still miss OCCT bbox parity if bbox is derived from the current Rust mesh path or from a simple union of root subshape OCCT bboxes. The gap is not a missing root vertex boundary. It is a closed-volume offset-specific bbox problem, so the next cut needs to target that case directly instead of widening the current shell-safe union.

## Focus

1. Keep the current shell-safe offset bbox win in place.
2. Target closed offset solids and compsolids specifically before revisiting other bounded non-exact families.
3. Prefer a parity-safe closed-offset bbox path built from existing Rust-owned face data, offset basis descriptors, or shell-level inventories before falling back to whole-shape OCCT.
4. Do not reintroduce the mesh bbox path for offset solids unless it is explicitly tightened against OCCT parity.
5. Keep the verification bar unchanged:
   - `cargo check --manifest-path rust/lean_occt/Cargo.toml`
   - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
   - `cargo test --manifest-path rust/lean_occt/Cargo.toml`

## Why This Is Next

This turn established a real Rust-first boundary instead of an experiment: offset shells no longer need whole-shape OCCT bbox, but offset solids still do. The next aggressive step is to retire that remaining closed-offset bbox fallback without regressing the shell case that already holds.
