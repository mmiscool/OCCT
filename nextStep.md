# Next Task

Narrow the remaining whole-shape OCCT summary fallback in `ported_shape_summary()`, starting with bounding-box derivation for shapes that already have Rust-owned topology, edges, and faces.

## Current State

- [`ported_brep()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep.rs) now hands its existing `faces` inventory directly to [`ported_shape_summary()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs); the extra `summary_faces` rebuild is gone.
- The successful Rust-topology path now carries Public faces all the way through summary derivation, including `ported_face_surface == None` faces that resolve sample/area through the mesh fallback already stored in `BrepFace`.
- [`ported_shape_summary()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) still keeps a broad `fallback_summary = || context.describe_shape_occt(shape).ok()` escape hatch for:
  - bbox fallback after `exact_primitive_shape_summary(...)` and `ported_shape_bbox(...)`
  - final volume fallback after exact, analytic, and mesh Rust-owned paths decline

## Remaining Blocker

The next coarse OCCT boundary is no longer a face-preparation split; it is the whole-shape `describe_shape_occt()` fallback inside summary derivation. That fallback is wider than necessary now that the Rust path already owns the full BRep/topology inventory.

## Focus

1. Measure and narrow the bbox fallthrough first, because it is lower-risk than the remaining volume fallback.
2. Keep using the existing Rust-owned `vertices`, `edges`, `faces`, and topology data before crossing back to whole-shape OCCT summary.
3. Leave the final volume fallback in place unless a clearly bounded Rust-owned replacement falls out naturally from the bbox work.
4. Keep the verification bar unchanged:
   - `cargo check --manifest-path rust/lean_occt/Cargo.toml`
   - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
   - `cargo test --manifest-path rust/lean_occt/Cargo.toml`

## Why This Is Next

The summary-face duplication is gone. The next aggressive Rust-first step is to replace the remaining coarse whole-shape summary fallback with logic that stays on the Rust-owned BRep/topology path for as long as possible.
