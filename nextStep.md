# Next Task

Deduplicate single-face edge builders in `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_surface.rs`.

## Focus

- Pull the shared `BrepEdge` assembly out of `single_face_edge_raw()` and `single_face_edge_public()` into one helper boundary.
- Keep the raw/public distinction explicit by leaving the geometry and curve acquisition paths separate: raw stays on `edge_geometry_occt()` plus `from_context_with_geometry()`, public keeps its Rust-first fallbacks.
- Leave `single_face_topology_with_edges()` and the surrounding face-surface descriptor routing unchanged.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the extraction.

## Why This Is Next

The swept-surface dispatcher is now thin, so the clearest remaining duplication in this module is the paired raw/public single-face edge builders. Collapsing their shared `BrepEdge` assembly is the next bounded cleanup that reduces drift without changing behavior.
