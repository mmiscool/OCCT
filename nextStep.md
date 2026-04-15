# Next Task

Extract the face preflight validation loop out of `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs` so the snapshot entry point stays narrowly focused on orchestration.

## Focus

- Move the leading face preflight loop in `ported_topology_snapshot()` into `brep/face_snapshot.rs` or a small sibling helper owned by that face-snapshot stage.
- Keep the current behavior for multi-wire non-planar rejection, including the public `face_geometry()` with explicit OCCT fallback.
- Leave `ported_topology_snapshot()` in `topology.rs`, but trim it down to root-shape loading plus final snapshot assembly over the already-split helpers.
- Preserve the downstream `Context::ported_topology()` / `Context::ported_brep()` behavior and existing topology snapshot parity.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the extraction.

## Why This Is Next

With the root wire packing helper moved out, `topology.rs` is now a single snapshot function whose first chunk is still face-specific validation logic. Pulling that preflight into the face-owned stage is the smallest bounded cleanup that makes the snapshot entry point read as orchestration over root loading, face matching, and final packing.
