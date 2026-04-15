# Next Task

Move the mesh-backed face fallback helper out of `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_surface.rs` and into `brep/summary.rs`.

## Focus

- Extract `LazyMeshFaceFallback` from `face_surface.rs` into `summary.rs`, beside `mesh_face_properties()` and `MeshFaceProperties`.
- Preserve the current eager-load behavior when `ported_face_surface` is missing, and preserve the lazy `load()` path for sample/area fallback resolution.
- Keep the current error messages and `Option`-to-`Error` behavior in `resolve_sample()` and `resolve_area()` unchanged.
- Leave `ported_brep_face()` as a thin caller that asks the shared summary helper for mesh-backed fallback resolution instead of owning that state machine locally.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the extraction.

## Why This Is Next

With `ported_face_area_from_surface()` now living in `brep/face_metrics.rs`, the remaining `LazyMeshFaceFallback` block is the next self-contained helper in `face_surface.rs` whose ownership already aligns with `brep/summary.rs`, the module that provides `mesh_face_properties()` and the mesh-derived fallback payload it wraps.
