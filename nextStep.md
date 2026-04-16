# Next Task

Keep narrowing the remaining shell-local OCCT bbox fallback in `offset_shell_bbox()`, but stay on the shell-boundary Rust path. The mixed-edge shell-boundary union now includes an adaptive sampled public-edge tier; the next task is improving that sampled boundary candidate for the shell edges whose extrema still are not captured by the current midpoint-and-tangent-guided refinement.

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
  - unsupported shell edges now also get a validated adaptive public-edge sampling chance before the later mesh/summary tiers
  - that sampled tier now recursively refines intervals when midpoint samples expand the interval bbox, when sampled tangents indicate an axis turn, or when the midpoint bends materially away from the chord
- The exercised non-solid offset shell fixture stays green on the Rust-first path.
- The exercised closed offset solid fixture stays green, including the direct per-shell parity assertion in [`ported_brep_uses_rust_owned_volume_for_offset_solids()`](rust/lean_occt/tests/brep_workflows.rs).

## Remaining Blocker

`offset_shell_bbox()` still ends at the raw shell-local OCCT bbox for shells that fail all current validated Rust candidates. The new shell-boundary Rust candidate now covers:

- supported boundary edges already determine the shell bbox, or
- every shell edge that matters to the bbox admits a Rust/public exact boundary bbox, or
- the current adaptive public-edge sampling and interval refinement hits the remaining shell-boundary extrema closely enough to validate.

The remaining blocker is shell edges whose decisive bbox extrema still are not captured by the current sampled boundary candidate even after the new adaptive interval refinement. Those shells still skip straight to the later mesh/summary candidates and eventually the raw shell-local OCCT bbox.

## Focus

1. Keep the non-solid offset bbox win in place.
2. Keep the now-green direct shell parity check for the exercised closed offset solid.
3. Stay on loader-owned shell-local inventories; do not reintroduce fresh raw `subshapes_occt()` traversal.
4. Keep the new shell boundary candidate on the public Rust edge/vertex path.
5. Keep the mixed shell-boundary union in place, and make unsupported shell-edge contributions more accurate rather than reverting to all-or-nothing boundary gating.
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
- that shell-boundary path now refines unsupported public-edge samples when midpoint, tangent, or chord-bend checks suggest missed extrema

The next step is to make that shell-local Rust boundary path cover more real offset shells by improving the sampled public-edge contribution for unsupported shell edges beyond the current midpoint/tangent refinement, not by widening fallback elsewhere.
