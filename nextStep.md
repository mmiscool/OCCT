# Next Task

Collapse the one-use `PreparedFaceShape::multi_wire_face_is_planar()` helper in `face_snapshot.rs`.

## Focus

- Reevaluate whether the one-use `PreparedFaceShape::multi_wire_face_is_planar()` check should be inlined directly into `load_ported_face_snapshot()` without changing planar preflight behavior.
- Keep `PreparedFaceShape` as the per-face preload carrier and preserve the existing Rust-first `face_geometry()` with explicit OCCT fallback behavior used by the planar gate.
- Preserve per-face wire preload behavior, root-wire matching, planar wire area computation, and packed snapshot output unchanged.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With the one-use `PreparedFaceShape::load()` wrapper now gone, the next tiny coordinator indirection in this face snapshot path is `PreparedFaceShape::multi_wire_face_is_planar()`, which is only used by `load_ported_face_snapshot()`. Inlining that check is the next bounded cleanup toward a tighter face snapshot entry path.
