# Next Task

Deduplicate raw/public face-surface descriptor routing in `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_surface.rs`.

## Focus

- Collapse `ported_face_surface_descriptor_from_surface()` and `ported_face_surface_descriptor_from_surface_public()` behind a small selector boundary instead of keeping two nearly identical wrappers.
- Keep the raw/public distinction explicit by preserving the different topology builders (`single_face_topology` vs `single_face_topology_public`) and by leaving the public `ported_face_surface_descriptor()` / `ported_face_area()` entry points unchanged.
- Leave the swept-surface helpers and single-face edge builders untouched while doing the cleanup.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the extraction.

## Why This Is Next

The single-face edge builders now share one `BrepEdge` assembly path, so the clearest remaining duplication in this module is the paired raw/public face-surface descriptor wrappers. Collapsing that selector boundary is the next bounded cleanup that reduces drift without changing behavior.
