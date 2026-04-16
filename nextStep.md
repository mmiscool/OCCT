# Next Task

Keep narrowing the remaining shell-local OCCT bbox fallback in `offset_shell_bbox()`, using the new validated Rust-first shell candidates that are now green on the exercised closed offset solid.

## Current State

- [`ported_shape_summary()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now has these relevant offset bbox tiers:
  - non-solid offset shapes first try [`offset_faces_bbox()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs), which now validates the shape-local Rust mesh bbox first, then the validated Rust face-BRep union, then the per-face OCCT bbox union over loaded root `face_shapes`
  - non-solid offset shapes keep [`offset_shape_bbox_occt()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) behind that as a narrower raw escape hatch, and only accept the shape-local Rust mesh bbox when it validates against OCCT
  - offset solids and compsolids now use [`offset_solid_shell_bbox()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs), and each shell now tries:
    - a validated shell-local Rust face-BRep union built from [`validated_face_brep_bbox()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs)
    - a validated shell-local Rust mesh bbox
    - a validated shell-local Rust `ported_brep(shell).summary`, including an offset-distance expansion candidate through [`offset_expanded_brep_bbox()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs)
    - only then the shell-local OCCT bbox
- [`load_ported_topology()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs) still preserves `PreparedShellShape { shell_shape, shell_face_shapes }` on the successful Rust-topology path.
- The exercised non-solid offset shell fixture stays green on the newer Rust-first path.
- The exercised closed offset solid fixture now also stays green with a direct per-shell parity assertion in [`ported_brep_uses_rust_owned_volume_for_offset_solids()`](rust/lean_occt/tests/brep_workflows.rs).

## Remaining Blocker

`offset_shell_bbox()` still ends at the raw shell-local OCCT bbox for shells that fail all current validated Rust candidates. The exercised offset solid is green now, but there are still unproven shell shapes where the remaining shell-local OCCT fallback may be hit.

## Focus

1. Keep the non-solid offset bbox win in place.
2. Keep the now-green direct shell parity check for the exercised closed offset solid.
3. Stay on `PreparedShellShape::shell_face_shapes` and shell-local loaded data; do not reintroduce fresh raw `subshapes_occt()` traversal.
4. Add more validated Rust-first shell candidates inside `offset_shell_bbox()` before the final raw shell bbox.
5. Prefer offset-distance-aware shell-local mesh or boundary adjustments, or other shell-local Rust-owned candidates, but validate every new candidate against the shell-local OCCT bbox before accepting it.
6. Keep the verification bar unchanged:
   - `cargo check --manifest-path rust/lean_occt/Cargo.toml`
   - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
   - `cargo test --manifest-path rust/lean_occt/Cargo.toml`

## Why This Is Next

This turn moved more of the shell-level offset bbox path onto Rust-owned data without weakening parity:

- shell-local face unions now reuse the same validated Rust-first face bbox path as the non-solid offset route
- shell-local `ported_brep(shell).summary` now gets an offset-aware expansion candidate before the raw shell bbox fallback
- the exercised closed offset solid now passes a direct per-shell Rust-vs-OCCT bbox parity assertion

The next step is to shrink the remaining shell-local OCCT fallback itself, not to widen fallback elsewhere.
