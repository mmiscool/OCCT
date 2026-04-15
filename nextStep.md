# Next Task

Extract the remaining route-aware face preparation helpers out of `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_surface.rs` into a dedicated sibling module.

## Focus

- Move `PreparedFaceSurface`, `ported_face_surface_descriptor_from_surface_with_route()`, `prepare_face_surface_with_route()`, `prepare_face_surface_with_geometry()`, and `face_geometry_with_route()` out of `face_surface.rs` into a new focused helper module such as `brep/face_prepare.rs`.
- Preserve the explicit raw/public routing through `FaceSurfaceRoute`; this is a code-ownership extraction, not a behavior change.
- Keep `ported_face_surface_descriptor()`, `ported_brep_face()`, and `ported_face_area()` in `face_surface.rs` as thin callers over the shared preparation helper.
- Do not change the analytic/offset/swept descriptor selection order or the current `PortedSurface::from_context_with_geometry()` entry path.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the extraction.

## Why This Is Next

With the mesh fallback now living in `brep/summary.rs`, `face_surface.rs` is down to face assembly plus one coherent route-aware preparation block. That preparation cluster is the next self-contained unit whose extraction would leave `face_surface.rs` focused on assembly and public entry points.
