# Next Task

Deduplicate face-area evaluation in `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_surface.rs`.

## Focus

- Extract the shared analytic/offset/swept area dispatch out of `ported_brep_faces()` and `ported_face_area()`.
- Keep mesh fallback and sample selection local to `ported_brep_faces()`, since the public face-area query intentionally stays on the pure Rust path and returns `None` when no Rust-owned area is available.
- Prefer a helper that takes the already-selected `PortedFaceSurface`, face geometry, and loop/wire/edge context, instead of recomputing match logic in multiple places.
- Leave behavior unchanged and keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the extraction.

## Why This Is Next

The public face-preparation duplication is gone, but the module still repeats the same area-selection match for analytic, offset, and swept faces. That shared computation is the next bounded cleanup before any larger structural move in the face-surface path.
