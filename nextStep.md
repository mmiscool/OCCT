# Next Task

Collapse the one-use `planar_wire_area_magnitude()` helper in `face_snapshot.rs`.

## Focus

- Reevaluate whether the one-use `planar_wire_area_magnitude()` helper should be inlined directly into `PreparedFaceTopologyBuilder::build()` without changing planar wire area behavior.
- Preserve the exact oriented-edge geometry reconstruction, sampled-point fallback behavior, and area magnitude comparison used for multi-wire planar faces.
- Keep root-wire matching, per-face wire preload behavior, and packed snapshot output unchanged.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With the one-use `match_root_wire_index()` helper now gone, the next remaining one-use helper in this face snapshot path is `planar_wire_area_magnitude()`, which is only used inside `PreparedFaceTopologyBuilder::build()`. Inlining that area calculation is the next bounded cleanup before larger structural changes.
