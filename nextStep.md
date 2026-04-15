# Next Task

Collapse the one-use `PreparedFaceShape::load_all()` helper in `face_snapshot.rs`.

## Focus

- Reevaluate whether the one-use `PreparedFaceShape::load_all()` wrapper should be inlined directly into `load_ported_face_snapshot()` without changing face preload behavior.
- Keep `PreparedFaceShape` as the prevalidated per-face carrier and preserve the existing `PreparedFaceShape::load()` boundary for per-face setup.
- Preserve the shared planar-face validation rule, per-face wire preload behavior, root-wire matching, planar wire area computation, and packed snapshot output unchanged.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With the one-use `FaceSnapshotAccumulator::append_face_topology_outputs()` helper now gone, the next tiny coordinator indirection in this face snapshot path is `PreparedFaceShape::load_all()`, which is only used by `load_ported_face_snapshot()`. Inlining that preload wrapper is the next bounded cleanup toward a tighter face snapshot entry path.
