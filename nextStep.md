# Next Task

Keep narrowing the remaining OCCT bbox fallback in `ported_shape_summary()`, but start from the new shell-local inventory boundary that is now preserved on both the Rust-topology and raw fallback paths.

## Current State

- [`ported_shape_summary()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now has these relevant offset bbox tiers:
  - non-solid offset shapes first try [`offset_faces_bbox()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs), which unions the Rust-owned boundary bbox from loaded vertices and edges with a per-face OCCT bbox union over loaded root `face_shapes`
  - non-solid offset shapes still keep [`offset_shape_bbox_occt()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) behind that as a narrower raw escape hatch
  - offset solids and compsolids still use shell-level OCCT bbox union through [`offset_solid_shell_bbox_occt()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs), but that helper now consumes loader-owned [`PreparedShellShape`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs) inventory instead of a separate shell reload
  - whole-shape `describe_shape_occt()` remains the last bbox escape hatch
- [`load_ported_topology()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs) now preserves `PreparedShellShape { shell_shape, shell_face_shapes }` on the successful Rust-topology path.
- [`ported_brep()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep.rs) now threads that same prepared-shell inventory through both the Rust-topology and raw fallback branches into summary derivation.
- The exercised offset shell fixture stays green on the newer Rust-plus-face path.
- The exercised closed offset solid fixture stays green on the restored shell-level OCCT bbox union.

## Remaining Blocker

Closed offset solids still need per-shell `describe_shape_occt()` parity help. Root-face unions were not enough, and the current solid-safe boundary is still OCCT bbox union over root-loaded shell handles.

## Focus

1. Keep the new non-solid offset bbox win in place.
2. Keep the current shell-level OCCT bbox union in place for closed offset solids and compsolids until a parity-safe shell-local Rust-first replacement is proven.
3. Use the new `PreparedShellShape::shell_face_shapes` inventory next; do not go back to reloading shell faces through fresh raw `subshapes_occt()` calls.
4. Prefer a shell-first bbox assembled from shell-local face descriptors, shell-local face BReps, or shell-local mesh/boundary data before `describe_shape_occt(shell)`.
5. Treat the failed root-face union attempt as evidence that shell-local structure matters; do not just reshuffle root-face unions again.
6. Keep the verification bar unchanged:
   - `cargo check --manifest-path rust/lean_occt/Cargo.toml`
   - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
   - `cargo test --manifest-path rust/lean_occt/Cargo.toml`

## Why This Is Next

This turn did two things that now make the next Rust-first offset-solid cut possible without reopening the old traversal boundary:

- restored the parity-safe solid bbox route after the failed root-face experiment
- preserved shell-local face inventories end to end, so the next step can work from loader-owned shell structure instead of another raw shell-face reload
