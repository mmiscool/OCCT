# Next Task

Keep narrowing the remaining shell-local OCCT bbox fallback in `offset_shell_bbox()`, but stay on the shell-boundary Rust path. The mixed-edge shell-boundary union is now in place; the next task is adding a validated public-edge sampling candidate for shell edges that still contribute nothing to that boundary bbox.

## Current State

- [`ported_shape_summary()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now has these relevant offset bbox tiers:
  - non-solid offset shapes first try [`offset_faces_bbox()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs), which now validates:
    - the shape-local Rust mesh bbox
    - an offset-distance-expanded shape-local Rust mesh bbox
    - the validated Rust face-BRep union
    - only then the per-face OCCT bbox union over loaded root `face_shapes`
  - non-solid offset shapes keep [`offset_shape_bbox_occt()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) behind that as a narrower raw escape hatch, and only accept the later shape-local Rust mesh tier when it validates against OCCT
  - offset solids and compsolids now use [`offset_solid_shell_bbox()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs), and each shell now tries:
    - a validated shell-local Rust face-BRep union built from [`validated_face_brep_bbox()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs)
    - a validated shell-local Rust boundary bbox built from loader-owned `shell_vertex_shapes` / `shell_edge_shapes`
    - a validated shell-local Rust mesh bbox
    - an offset-distance-expanded shell-local Rust mesh bbox
    - a validated shell-local Rust `ported_brep(shell).summary`
    - an offset-distance-expanded shell-local Rust `ported_brep(shell).summary`
    - only then the shell-local OCCT bbox
- [`load_ported_topology()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs) now preserves `PreparedShellShape { shell_shape, shell_vertex_shapes, shell_edge_shapes, shell_face_shapes }` on the successful Rust-topology path.
- [`shell_boundary_shape_bbox()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) is now a mixed per-edge union:
  - it always starts from the loader-owned shell vertices
  - it unions any edge that already admits a Rust/public exact boundary bbox
  - unsupported shell edges no longer fail the whole shell-boundary candidate immediately
- The exercised non-solid offset shell fixture stays green on the Rust-first path.
- The exercised closed offset solid fixture stays green, including the direct per-shell parity assertion in [`ported_brep_uses_rust_owned_volume_for_offset_solids()`](rust/lean_occt/tests/brep_workflows.rs).

## Remaining Blocker

`offset_shell_bbox()` still ends at the raw shell-local OCCT bbox for shells that fail all current validated Rust candidates. The new shell-boundary Rust candidate only succeeds when shell edges can be evaluated entirely through the current public boundary path:

- supported boundary edges already determine the shell bbox, or
- every shell edge that matters to the bbox admits a Rust/public exact boundary bbox.

Shell edges that still produce no boundary contribution at all are the remaining blocker. Those shells still skip straight to the later mesh/summary candidates and eventually the raw shell-local OCCT bbox.

## Focus

1. Keep the non-solid offset bbox win in place.
2. Keep the now-green direct shell parity check for the exercised closed offset solid.
3. Stay on loader-owned shell-local inventories; do not reintroduce fresh raw `subshapes_occt()` traversal.
4. Keep the new shell boundary candidate on the public Rust edge/vertex path.
5. Keep the mixed shell-boundary union in place, and add new validated shell-edge contributions rather than reverting to all-or-nothing boundary gating.
6. Validate every new shell candidate against the shell-local OCCT bbox before accepting it.
7. Keep the verification bar unchanged:
   - `cargo check --manifest-path rust/lean_occt/Cargo.toml`
   - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
   - `cargo test --manifest-path rust/lean_occt/Cargo.toml`

## Why This Is Next

This turn moved more of the offset bbox path onto Rust-owned data without weakening parity:

- non-solid offset shapes now get an offset-expanded Rust mesh bbox validation chance before the raw face bbox union
- closed offset shells now carry shell-local edge and vertex inventories through `PreparedShellShape`
- closed offset shells now try a validated shell-local Rust boundary bbox before mesh and shell-summary validation

The next step is to make that shell-local Rust boundary path cover more real offset shells by giving currently unsupported shell edges a validated public-edge sampling contribution, not by widening fallback elsewhere.
