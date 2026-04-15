# Next Task

Split the remaining public face query wrappers out of `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_surface.rs` into a dedicated sibling module.

## Focus

- Move `ported_face_surface_descriptor()` and `ported_face_area()` out of `face_surface.rs` into a new focused helper module such as `brep/face_queries.rs`.
- Keep those public wrappers reusing the shared `face_prepare` helpers, `ported_face_area_from_surface()`, and `single_face_topology_with_route()`; this is a code-ownership extraction, not a behavior change.
- Leave `ported_brep_faces()` and `ported_brep_face()` in `face_surface.rs` so that file becomes purely about internal BRep face assembly.
- Preserve the current public routing behavior for `FaceSurfaceRoute::Public`, including the public topology path in `ported_face_area()`.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the extraction.

## Why This Is Next

With the route-aware preparation cluster now living in `brep/face_prepare.rs`, `face_surface.rs` is down to one internal assembly path plus two public face query wrappers. Pulling those wrappers into their own sibling module is the next clean split that leaves `face_surface.rs` focused on BRep face materialization only.
