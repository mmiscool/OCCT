# Next Task

Keep narrowing the remaining OCCT bbox fallback in `ported_shape_summary()`, but now work from the new validated shell-local Rust-first boundary for closed offset solids.

## Current State

- [`ported_shape_summary()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now has these relevant offset bbox tiers:
  - non-solid offset shapes first try [`offset_faces_bbox()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs), which unions the Rust-owned boundary bbox from loaded vertices and edges with a per-face OCCT bbox union over loaded root `face_shapes`
  - non-solid offset shapes still keep [`offset_shape_bbox_occt()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) behind that as a narrower raw escape hatch
  - offset solids and compsolids now use [`offset_solid_shell_bbox()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs), which tries `context.ported_brep(shell).summary` first per prepared shell and keeps the shell-local OCCT bbox as validation fallback when the Rust shell summary drifts
  - whole-shape `describe_shape_occt()` remains the last bbox escape hatch
- [`load_ported_topology()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs) still preserves `PreparedShellShape { shell_shape, shell_face_shapes }` on the successful Rust-topology path.
- The exercised offset shell fixture stays green on the newer Rust-plus-face path.
- The exercised closed offset solid fixture stays green on the new validated shell-local Rust-first path.

## Remaining Blocker

Closed offset solids still fall back to shell-local `describe_shape_occt(shell)` in the hard cases. The current shell-local Rust summary is not yet parity-safe for those shells, so the new win is only for shells whose Rust summary already matches.

## Focus

1. Keep the non-solid offset bbox win in place.
2. Keep the new validated shell-local Rust-first path in place for closed offset solids and compsolids.
3. Use `PreparedShellShape::shell_face_shapes` next; do not go back to reloading shell faces through fresh raw `subshapes_occt()` calls.
4. Improve shell-local Rust bbox parity before removing the per-shell OCCT validation fallback.
5. Prefer shell-local face descriptors, shell-local face BReps, or shell-local mesh/boundary data; do not revisit root-face unions.
6. Keep the verification bar unchanged:
   - `cargo check --manifest-path rust/lean_occt/Cargo.toml`
   - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
   - `cargo test --manifest-path rust/lean_occt/Cargo.toml`

## Why This Is Next

This turn established the safe shell-first boundary for closed offset solids:

- Rust-owned shell summaries are now attempted first at the shell level
- shell-local OCCT bbox fallback still preserves parity when those Rust summaries are not ready

The next step is to make more prepared shells pass that validation, not to widen the fallback again.
