# Next Task

Extract lazy mesh fallback handling from `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_surface.rs`.

## Focus

- Pull the mesh-fallback state and sample/area fallback logic out of `ported_brep_face()` into a helper boundary.
- Keep the raw/internal face preparation in `ported_brep_face()` and keep the public `ported_face_surface_descriptor()` and `ported_face_area()` paths unchanged.
- Prefer a small helper that owns `mesh_face_properties()` loading and the repeated sample/area error shaping, so the core face builder reads as geometry selection plus final assembly.
- Leave behavior unchanged and keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the extraction.

## Why This Is Next

The per-face raw builder now lives in one function, but the lazy mesh fallback closure still carries most of the local complexity in that path. Isolating that fallback behavior is the next bounded cleanup before any larger restructuring of the internal face materialization flow.
