# Next Task

Extract the optional multi-wire planar-face setup from the face snapshot packing path.

## Focus

- Pull the `face_geometry_occt` / `face_plane_payload_occt` setup for multi-wire planar faces out of `append_ported_face_topology()` in `face_snapshot.rs`.
- Keep the centralized `multi_wire_face_is_planar()` gate, the raw plane payload fetch path, and the planar wire area computation unchanged.
- Keep the current face validation, root-wire matching, planar loop classification, and packed snapshot output unchanged.
- Preserve the downstream `Context::ported_topology()` / `Context::ported_brep()` behavior and existing topology snapshot parity.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With the outer/inner role selection now extracted, the main remaining mixed concern inside `append_ported_face_topology()` is the optional multi-wire planar-face setup. Pulling that into its own helper keeps the per-face path focused on wire matching and accumulator updates.
