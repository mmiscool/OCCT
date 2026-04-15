# Next Task

Collapse the one-use `match_root_wire_index()` helper in `face_snapshot.rs`.

## Focus

- Reevaluate whether the one-use `match_root_wire_index()` helper should be inlined directly into `PreparedFaceTopologyBuilder::build()` without changing root-wire matching behavior.
- Keep the builder-owned `used_root_wire_indices` tracking intact and preserve the existing equality-based match rule against `RootWireTopology`.
- Preserve planar wire area computation, per-face wire preload behavior, and packed snapshot output unchanged.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With the one-use `PreparedFaceShape::multi_wire_face_is_planar()` helper now gone, the next tiny indirection in this face snapshot path is `match_root_wire_index()`, which is only used inside `PreparedFaceTopologyBuilder::build()`. Inlining that match is the next bounded cleanup before touching the larger planar area helper.
