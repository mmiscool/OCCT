# Next Task

Collapse the remaining two-step face snapshot entry in `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs` into a single face-owned helper so the snapshot entry point is just root load, face load, and final assembly.

## Focus

- Combine the validated face-shape load and the call to `pack_ported_face_snapshot()` behind one face-owned helper in `brep/face_snapshot.rs`.
- Keep the current behavior for face ordering, multi-wire non-planar rejection, and `ported_face_topology()` failure handling.
- Leave `ported_topology_snapshot()` in `topology.rs`, but trim it down to a call into the face snapshot helper, a call into the root snapshot helper, and final `TopologySnapshot` assembly.
- Preserve the downstream `Context::ported_topology()` / `Context::ported_brep()` behavior and existing topology snapshot parity.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the extraction.

## Why This Is Next

With face-shape loading now owned by `brep/face_snapshot.rs`, the remaining face-specific work visible in `topology.rs` is the separate call into `pack_ported_face_snapshot()`. Folding those two face-stage calls together is the smallest cleanup that leaves the snapshot entry point reading as root load, face load, and final `TopologySnapshot` assembly.
