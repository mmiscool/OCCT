# Next Task

Move the swept-face reconstruction helpers out of `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_surface.rs` and into `brep/swept_face.rs`.

## Focus

- Extract `ported_swept_face_surface_with_route()`, `ported_swept_face_surface_from_topology()`, `ported_extrusion_face_surface()`, `ported_revolution_face_surface()`, and `select_swept_face_basis()` into the swept-face sibling module now that the single-face topology block has been moved out.
- Preserve the current raw/public route behavior by continuing to pass `FaceSurfaceRoute` and `SingleFaceTopology` through unchanged.
- Keep `face_surface.rs` focused on face preparation, descriptor selection, and face area/sample assembly after the extraction.
- Avoid changing the public query entry points or the swept-surface payload/basis selection behavior while doing the move.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the extraction.

## Why This Is Next

With the route-driven single-face topology helpers now living in `brep/topology.rs`, the remaining swept-face reconstruction path in `face_surface.rs` is the next coherent block whose ownership aligns better with the existing `brep/swept_face.rs` module.
