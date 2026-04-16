# Next Task

Keep narrowing the whole-shape OCCT summary fallback in `ported_shape_summary()`, but stay on a parity-safe bbox boundary. The next target is the remaining bounded non-exact families whose bbox can be derived from existing Rust-owned face, edge, or mesh data before crossing back to `describe_shape_occt()`.

## Current State

- [`ported_brep()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep.rs) already hands its existing `faces` inventory directly to [`ported_shape_summary()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs); the extra `summary_faces` rebuild is gone.
- The successful Rust-topology path now carries Public faces through summary derivation, including `ported_face_surface == None` faces that resolve sample and area through the mesh fallback already stored in `BrepFace`.
- [`mesh_bbox()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/mesh.rs) now falls back to the stored mesh bounds from `Context::mesh()` when the point/segment collection path cannot produce a bbox, so one Rust-owned bbox fallback is narrower than before.
- [`topological_shape_bbox()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now keeps bbox ownership in Rust for shapes whose faces are entirely plane/cylinder/cone by using analytic boundary edges before falling back to mesh or whole-shape OCCT summary.
- [`ported_shape_summary()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) still keeps a broad `fallback_summary = || context.describe_shape_occt(shape).ok()` escape hatch for:
  - bbox fallback after `exact_primitive_shape_summary(...)` and `ported_shape_bbox(...)`
  - final volume fallback after exact, analytic, and mesh Rust-owned paths decline

## Remaining Blocker

The next coarse OCCT boundary is still the whole-shape `describe_shape_occt()` fallback inside summary derivation. The safe boundary has moved from planes-only to plane/cylinder/cone face sets, but exact torus-style formulas were not parity-safe against OCCT, so the next cut still needs to stay on bounded non-exact families or mesh/BRep-derived bbox logic that preserves public parity.

## Focus

1. Keep narrowing bbox fallthrough before touching the remaining volume fallback.
2. Reuse Rust-owned `vertices`, `edges`, `faces`, face samples, and mesh-backed state before crossing back to whole-shape OCCT summary.
3. Prefer parity-safe families such as bounded swept/offset or mixed non-exact faces, or mesh/BRep-derived bbox improvements, over new exact analytic bbox formulas.
4. Leave the final volume fallback in place unless a clearly bounded Rust-owned replacement falls out naturally from the bbox work.
5. Keep the verification bar unchanged:
   - `cargo check --manifest-path rust/lean_occt/Cargo.toml`
   - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
   - `cargo test --manifest-path rust/lean_occt/Cargo.toml`

## Why This Is Next

The safe progress in this slice is that non-exact plane/cylinder/cone-bounded shapes now stay on Rust-owned analytic edge bbox logic, and the cut-style BRep summary fixture is pinned on that route. The next aggressive Rust-first step is to keep whole-shape bbox derivation on Rust-owned BRep or mesh inventories for the next bounded non-exact family before falling back to `describe_shape_occt()`.
