# Next Task

Deduplicate raw/public single-face topology routing in `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_surface.rs`.

## Focus

- Collapse `single_face_topology()` and `single_face_topology_public()` behind a small selector boundary instead of keeping paired wrappers.
- Fold the route selection for `single_face_edge_raw()` and `single_face_edge_public()` into the same explicit mode so the raw/public distinction stays local to topology construction.
- Preserve the current behavior split: the raw route must stay on `edge_geometry_occt()` plus `PortedCurve::from_context_with_geometry()`, while the public route must keep its Rust-first `edge_geometry()` / `from_context_with_ported_payloads()` fallback behavior.
- Leave `ported_brep_face()`, face-area dispatch, and face-surface preparation unchanged while doing the cleanup.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the extraction.

## Why This Is Next

The face-surface descriptor path now shares one explicit raw/public route selector, so the clearest remaining duplication in this module is the paired single-face topology and edge-builder routing. Collapsing that boundary is the next bounded cleanup that reduces drift without changing behavior.
