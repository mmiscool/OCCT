# Next Task

Extract swept extrusion/revolution surface builders from `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_surface.rs`.

## Focus

- Split the remaining `SurfaceKind::Extrusion` and `SurfaceKind::Revolution` construction bodies in `ported_swept_face_surface_from_topology()` into dedicated helpers.
- Keep the new shared swept-basis selection helper in place, and let each new helper own only its payload fetch plus `PortedSweptSurface` assembly.
- Leave the surrounding topology builder and face-surface descriptor routing unchanged.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the extraction.

## Why This Is Next

The branch-local basis selection duplication is gone, so the next small cleanup is isolating the two payload-specific swept-surface constructors. That keeps `ported_swept_face_surface_from_topology()` moving toward a thin dispatcher without changing any geometry behavior.
