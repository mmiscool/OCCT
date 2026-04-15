# Next Task

Collapse the one-use `PreparedFaceShape::load()` helper in `face_snapshot.rs`.

## Focus

- Reevaluate whether the one-use `PreparedFaceShape::load()` wrapper should be inlined directly into `load_ported_face_snapshot()` without changing per-face preload behavior.
- Keep `PreparedFaceShape` as the prevalidated per-face carrier and preserve the shared `multi_wire_face_is_planar()` boundary for planar preflight.
- Preserve the per-face wire preload behavior, root-wire matching, planar wire area computation, and packed snapshot output unchanged.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With the one-use `PreparedFaceShape::load_all()` wrapper now gone, the next tiny coordinator indirection in this face snapshot path is `PreparedFaceShape::load()`, which is only used by `load_ported_face_snapshot()`. Inlining that per-face preload wrapper is the next bounded cleanup toward a tighter face snapshot entry path.
