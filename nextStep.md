# Next Task

Deduplicate face-surface preparation in `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_surface.rs`.

## Focus

- Extract the common `PortedSurface::from_context_with_geometry()` and face-surface descriptor wiring shared by `prepare_raw_face_surface()` and `prepare_public_face_surface()` into one helper boundary.
- Keep the raw/public distinction explicit by leaving geometry acquisition (`face_geometry_occt()` vs `face_geometry()`) and descriptor selection (`ported_face_surface_descriptor_from_surface()` vs `_public`) at the call sites or behind a small mode parameter.
- Avoid re-expanding `ported_brep_face()` or the public `ported_face_surface_descriptor()` / `ported_face_area()` paths while doing the cleanup.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the extraction.

## Why This Is Next

The raw prep is now isolated, which makes the duplicated body between the raw and public face-preparation helpers the clearest remaining cleanup in this module. Collapsing that shared preparation step is the next bounded way to reduce drift while preserving the intentional raw/public split.
