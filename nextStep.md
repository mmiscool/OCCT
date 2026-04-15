# Next Task

Collapse the thin `collect_face_wires()` loop wrapper in `face_snapshot.rs`.

## Focus

- Reevaluate whether the slice-level loop in `PreparedFaceTopologyBuilder::collect_face_wires()` should move into `build()` or collapse into a builder-owned iterator-style entry point without changing the per-wire `collect_face_wire()` behavior.
- Keep `PreparedFaceTopology` as the final assembled result and preserve the direct snapshot accumulator handoff.
- Preserve the shared planar-face validation rule, per-wire root-wire matching behavior, planar wire area computation, face range offsets, edge-face ordering, and packed snapshot output unchanged.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With `match_face_wire()` now reading builder-owned state directly, the next tiny indirection in this path is the separate `collect_face_wires()` loop wrapper over `collect_face_wire()`. Collapsing that wrapper is the next bounded cleanup toward a tighter builder-owned collection flow.
