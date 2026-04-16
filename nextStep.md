# Next Task

Keep narrowing the remaining OCCT bbox fallback in `ported_shape_summary()`, but do it from the new split offset boundary that now holds separately for non-solid and solid offset shapes.

## Current State

- [`ported_shape_summary()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now has four relevant offset bbox tiers:
  - non-solid offset shapes first try [`offset_faces_bbox()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs), which unions the Rust-owned boundary bbox from loaded vertices and edges with a per-face OCCT bbox union over already-loaded root `face_shapes`
  - non-solid offset shapes still keep [`offset_shape_bbox_occt()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) behind that as a narrower raw escape hatch
  - offset solids and compsolids still use the shell-level OCCT bbox union through [`offset_solid_shell_bbox_occt()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs)
  - whole-shape `describe_shape_occt()` remains the last bbox escape hatch
- The exercised offset shell fixture in [`brep_workflows.rs`](rust/lean_occt/tests/brep_workflows.rs) stays green on the new Rust-plus-face route.
- The exercised closed offset solid fixture still needs the shell-level OCCT bbox union for parity.
- The broader Rust-owned bbox path remains in place for:
  - face-free shapes through analytic edges, line segments, or vertex points
  - all-plane / cylinder / cone face sets through analytic boundary edges
  - swept face sets when every boundary edge already has a Rust `PortedCurve`

## Remaining Blocker

Closed offset solids do not match OCCT bbox parity yet on the newer root-face bbox union path. The current parity-safe boundary is still shell-level OCCT bbox union for offset solids and compsolids.

## Focus

1. Keep the new non-solid offset bbox win in place.
2. Keep the shell-level OCCT bbox union in place for closed offset solids and compsolids until a parity-safe Rust-first replacement is proven.
3. Target that shell-level boundary next, not the already-retired whole-shape fallback and not the already-landed non-solid offset path.
4. Prefer a Rust-owned shell bbox assembled from existing shell-local face inventories, offset face descriptors, or shell-local BRep data before using per-shell `describe_shape_occt()`.
5. Treat the failed closed-solid root-face bbox union as evidence that shell-local structure matters; do not just widen or reshuffle root-face OCCT unions again.
6. Keep the verification bar unchanged:
   - `cargo check --manifest-path rust/lean_occt/Cargo.toml`
   - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
   - `cargo test --manifest-path rust/lean_occt/Cargo.toml`

## Why This Is Next

This turn moved non-solid offset bbox derivation further into Rust by combining the Rust-owned boundary bbox with loaded face inventories, and that path held parity. The remaining coarse OCCT boundary is now specifically the closed-offset shell union used by solids and compsolids.
