# Next Task

Keep narrowing the remaining OCCT bbox fallback in `ported_shape_summary()`, but now work from the new validated shape-local mesh and shell-local Rust-first boundaries for offset shapes.

## Current State

- [`ported_shape_summary()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now has these relevant offset bbox tiers:
  - non-solid offset shapes first try [`offset_faces_bbox()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs), which now validates the shape-local Rust mesh bbox first, then tries the validated Rust face-BRep union, and only then falls back to the per-face OCCT bbox union over loaded root `face_shapes`
  - non-solid offset shapes still keep [`offset_shape_bbox_occt()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) behind that as a narrower raw escape hatch
  - offset solids and compsolids now use [`offset_solid_shell_bbox()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs), and each shell now tries a validated shell-local Rust face-BRep union, then a validated shell-local Rust mesh bbox, then a validated `context.ported_brep(shell).summary`, before falling back to shell-local OCCT bbox
  - whole-shape `describe_shape_occt()` remains the last bbox escape hatch
- [`load_ported_topology()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs) still preserves `PreparedShellShape { shell_shape, shell_face_shapes }` on the successful Rust-topology path.
- The exercised non-solid offset shell fixture stays green on the newer Rust-first path.
- The exercised closed offset solid fixture stays green on the validated shell-local Rust-first path.

## Remaining Blocker

Closed offset-solid shells still have hard cases where the shell-root Rust summary underestimates the OCCT bbox by about the offset distance on some axes, so those shells still fall back to shell-local OCCT bbox. The direct shell-root summary itself is not parity-safe yet, even though the validated shell-local path keeps the parent solid green.

## Focus

1. Keep the non-solid offset bbox win in place.
2. Keep the validated shell-local Rust-first path in place for closed offset solids and compsolids.
3. Use `PreparedShellShape::shell_face_shapes` and shell-local loaded data; do not go back to reloading shell faces through fresh raw `subshapes_occt()` calls.
4. Improve shell-root Rust bbox parity so more shells clear validation before the per-shell OCCT fallback.
5. Prefer shell-local face descriptors, shell-local face BReps, shell-local mesh/boundary data, or explicit offset-distance-aware adjustments; do not revisit root-face unions.
6. Keep the verification bar unchanged:
   - `cargo check --manifest-path rust/lean_occt/Cargo.toml`
   - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
   - `cargo test --manifest-path rust/lean_occt/Cargo.toml`

## Why This Is Next

This turn made the offset bbox path more Rust-first without weakening parity:

- non-solid offset shapes now get a validated shape-local Rust mesh bbox chance before raw face bbox union
- closed offset-solid shells now get a validated shell-local Rust mesh bbox chance before raw shell bbox fallback

The next step is to fix the shell-root summary drift itself so those validated shell-local candidates succeed more often.
