# Next Task

Keep narrowing the remaining OCCT bbox fallback in `ported_shape_summary()`, but start from the new shell-level boundary that now holds for offset solids and compsolids.

## Current State

- [`ported_brep()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep.rs) and [`load_ported_topology()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs) now preserve loaded root `shell_shapes` alongside the existing root `vertex_shapes`, `edge_shapes`, and `face_shapes` when the Rust topology path succeeds.
- [`ported_shape_summary()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now has three offset bbox tiers:
  - non-solid offset shapes use [`offset_shape_bbox_occt()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs), which unions existing root subshape OCCT bboxes across already-loaded vertex, edge, and face inventories
  - offset solids and compsolids now try a narrower shell-level OCCT bbox union before falling back further
  - whole-shape `describe_shape_occt()` is now only the final offset bbox escape hatch instead of the first solid/compsolid branch
- The exercised offset shell and offset solid fixtures in [`brep_workflows.rs`](rust/lean_occt/tests/brep_workflows.rs) both still match OCCT bbox parity after that change.
- The broader Rust-owned bbox path remains in place for:
  - face-free shapes through analytic edges, line segments, or vertex points
  - all-plane / cylinder / cone face sets through analytic boundary edges
  - swept face sets when every boundary edge already has a Rust `PortedCurve`

## Remaining Blocker

Closed offset solids no longer need whole-shape OCCT bbox first, but their current safe path is still a per-shell OCCT bbox union. That is narrower than before, but it is not yet a Rust-owned closed-offset bbox derivation.

## Focus

1. Keep the new shell-level offset-solid bbox win in place.
2. Target the per-shell OCCT bbox union next, not the already-retired whole-shape fallback.
3. Prefer a Rust-owned shell bbox assembled from existing face inventories, offset face descriptors, or shell-local BRep data before using per-shell `describe_shape_occt()`.
4. Keep whole-shape `describe_shape_occt()` as the last escape hatch until the Rust-owned shell path proves parity-safe.
5. Keep the verification bar unchanged:
   - `cargo check --manifest-path rust/lean_occt/Cargo.toml`
   - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
   - `cargo test --manifest-path rust/lean_occt/Cargo.toml`

## Why This Is Next

This turn retired the coarsest offset-solid bbox fallback: the exercised closed offset solid no longer needs whole-shape OCCT bbox. The next aggressive Rust-first step is to replace that remaining shell-level OCCT bbox union with a parity-safe Rust-owned shell bbox path.
