# Next Task

Extract shared swept-face basis selection from `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_surface.rs`.

## Focus

- Pull the repeated basis-curve candidate lookup and `select_swept_face_basis_curve()` wiring out of the `SurfaceKind::Extrusion` and `SurfaceKind::Revolution` branches in `ported_swept_face_surface_from_topology()`.
- Keep the two payload fetches (`face_extrusion_payload_occt()` and `face_revolution_payload_occt()`) and the two `PortedSweptSurface` constructors explicit, but share the candidate selection/error-shaping path underneath them.
- Leave the surrounding topology builder and face-surface descriptor routing unchanged.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the extraction.

## Why This Is Next

The face-preparation helpers now share one implementation path, so the clearest remaining duplication in this module is the near-identical swept-basis selection logic for extrusion and revolution faces. Extracting that helper is the next small cleanup that reduces drift without changing behavior.
