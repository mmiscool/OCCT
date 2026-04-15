# Next Task

Deduplicate raw/public face-preparation entry routing in `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_surface.rs`.

## Focus

- Collapse `prepare_raw_face_surface()` and the remaining public face-preparation entry flow behind one small selector-aware boundary instead of keeping separate raw/public setup paths.
- Route face-geometry acquisition through the same explicit `FaceSurfaceRoute` mode that now drives descriptor selection and single-face topology construction.
- Preserve the current behavior split: the raw route must stay on `face_geometry_occt()`, while the public route must keep the Rust-first `face_geometry()` behavior and still allow callers that already computed geometry to pass it through unchanged.
- Leave the swept-surface helpers, area dispatch, and single-face topology/edge routing untouched while doing the cleanup.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the extraction.

## Why This Is Next

The raw/public route enum now covers descriptor selection and single-face topology construction, so the clearest remaining split in this module is the face-preparation entry path around geometry acquisition. Folding that into the same selector boundary is the next bounded cleanup that reduces drift without changing behavior.
