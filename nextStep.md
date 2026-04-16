# Next Task

Keep narrowing the whole-shape OCCT summary fallback in `ported_shape_summary()`, but stay on a parity-safe bbox boundary. The next target is the remaining bounded non-exact families whose bbox still falls through to whole-shape mesh or `describe_shape_occt()` because not every boundary edge is already analytic in Rust.

## Current State

- [`ported_brep()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep.rs) already hands its existing `faces` inventory directly to [`ported_shape_summary()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs); the extra `summary_faces` rebuild is gone.
- [`topological_shape_bbox()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now keeps bbox ownership in Rust for:
  - face-free shapes through analytic edges, line segments, or vertex points
  - all-plane / cylinder / cone face sets through analytic boundary edges
  - swept face sets when every boundary edge already has a Rust `PortedCurve`
- The swept-revolution BRep fixture in [`brep_workflows.rs`](rust/lean_occt/tests/brep_workflows.rs) is now pinned on that wider Rust-owned bbox path against OCCT bbox parity.
- [`mesh_bbox()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/mesh.rs) already falls back to stored mesh bounds from `Context::mesh()` when the point/segment collection path cannot produce a bbox.
- [`ported_shape_summary()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) still keeps a broad `fallback_summary = || context.describe_shape_occt(shape).ok()` escape hatch for:
  - bbox fallback after exact and topological Rust-owned bbox paths decline
  - final volume fallback after exact, analytic, and mesh Rust-owned paths decline

## Remaining Blocker

The next coarse OCCT boundary is still the whole-shape bbox fallback in summary derivation for bounded non-exact families whose faces are already Rust-owned but whose boundary edges are not all reconstructible as analytic `PortedCurve`s. Offset faces are the immediate example: analytic boundary edges alone were not enough to match OCCT bbox parity. The torus-style exact-extrema path is still off the table, so the next cut needs to stay on BRep-owned or mesh-owned bbox logic that matches public OCCT parity.

## Focus

1. Keep narrowing bbox fallthrough before touching the remaining volume fallback.
2. Reuse Rust-owned `vertices`, `edges`, `faces`, face samples, and face-shape inventories before crossing back to whole-shape OCCT summary.
3. Prefer the next parity-safe cut to be per-face mesh or face-local bbox union for bounded non-exact families whose boundary edges are not all analytic, rather than adding new exact analytic formulas.
4. Leave the final volume fallback in place unless a clearly bounded Rust-owned replacement falls out naturally from the bbox work.
5. Keep the verification bar unchanged:
   - `cargo check --manifest-path rust/lean_occt/Cargo.toml`
   - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
   - `cargo test --manifest-path rust/lean_occt/Cargo.toml`

## Why This Is Next

The safe Rust-first bbox boundary now includes the exercised swept family whenever its boundary is already fully analytic in Rust. The next aggressive step is to cover the adjacent bounded non-exact cases, starting with offset faces, by using existing face-local mesh or BRep-owned data before falling back to `describe_shape_occt()`.
