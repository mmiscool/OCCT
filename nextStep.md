# Next Task

Extract a shared public face-preparation helper in `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_surface.rs`.

## Focus

- Pull the repeated public face setup out of `ported_face_surface_descriptor()` and `ported_face_area()`: face geometry lookup, public single-face topology loading when needed, `PortedSurface` reconstruction, and `PortedFaceSurface` selection.
- Keep the internal/raw `ported_brep_faces()` flow separate, since it still deliberately uses the raw face-geometry boundary and mesh fallback path.
- Prefer a small helper struct or tuple that makes the public face query path explicit instead of recomputing the same state across entry points.
- Leave behavior unchanged and keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the extraction.

## Why This Is Next

The internal/public wrapper duplication is now collapsed, but the public query path still rebuilds the same face state in multiple entry points. That shared preparation is the next bounded extraction before any larger structural change in `face_surface.rs`.
