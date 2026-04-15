# Next Task

Deduplicate the remaining internal/public face-surface descriptor wrappers in `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_surface.rs`.

## Focus

- Replace the repeated control flow in `ported_face_surface_descriptor_from_surface()` and `ported_face_surface_descriptor_from_surface_public()` with a shared helper.
- Do the same for `ported_swept_face_surface()` and `ported_swept_face_surface_public()`, reusing the shared single-face topology path added this turn.
- Preserve the internal/raw vs public/Rust-first edge acquisition split by passing that choice through a narrow helper boundary instead of keeping duplicate wrappers.
- Leave behavior unchanged and keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

The single-face topology duplication is now gone, but `face_surface.rs` still carries paired internal and public wrappers above that shared path. Those functions differ only in which topology/curve acquisition mode they select, so they are the next clean reduction before any deeper porting work in this module.
