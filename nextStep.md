# Next Task

Extract a per-face materialization helper from `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_surface.rs`.

## Focus

- Pull the raw/internal face setup out of the `ported_brep_faces()` closure: face geometry lookup, `PortedSurface` reconstruction, `PortedFaceSurface` selection, orientation, loop lookup, and lazy mesh fallback setup.
- Keep the public `ported_face_surface_descriptor()` and `ported_face_area()` paths separate; this task is about shrinking the internal BRep face-builder block.
- Prefer a small helper struct or single-face builder function that returns the prepared state needed to compute sample, area, and adjacency without re-deriving local context.
- Leave behavior unchanged and keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the extraction.

## Why This Is Next

The shared public face preparation and area dispatch are now factored out, but `ported_brep_faces()` still carries a large per-face setup block inside its iterator. That internal materialization step is the next bounded cleanup before any broader restructuring of the face-surface path.
