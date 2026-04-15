# Next Task

Extract raw face preparation from `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_surface.rs`.

## Focus

- Pull the `face_geometry_occt()`, `PortedSurface::from_context_with_geometry()`, and `ported_face_surface_descriptor_from_surface()` prologue out of `ported_brep_face()` into a small internal helper or prepared-struct boundary.
- Keep the internal/raw route distinct from `prepare_public_face_surface()`, but shape the code so the two paths read as sibling preparation steps instead of one helper plus one open-coded block.
- Leave the lazy mesh fallback, face loops, adjacency wiring, and public `ported_face_surface_descriptor()` / `ported_face_area()` behavior unchanged.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the extraction.

## Why This Is Next

The mesh fallback is now isolated, so the top of `ported_brep_face()` is the largest remaining chunk of setup logic. Extracting that raw preparation step is the next bounded cleanup toward making the internal and public face-materialization flows structurally parallel.
