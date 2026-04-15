# Next Task

Extract the remaining face-shape loading and validation entry out of `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs` so the snapshot entry point becomes a thin coordinator over fully-owned root and face snapshot helpers.

## Focus

- Move the `subshapes_occt(..., ShapeKind::Face)` load plus validation entry path into `brep/face_snapshot.rs`, ideally as a helper that returns the validated face-shape list needed by the face packer.
- Keep the current behavior for multi-wire non-planar rejection, the public `face_geometry()` with explicit OCCT fallback, and the face-shape ordering consumed by `pack_ported_face_snapshot()`.
- Leave `ported_topology_snapshot()` in `topology.rs`, but trim it down to a call into the face snapshot input helper, a call into the root snapshot helper, a call into the face packing helper, and final `TopologySnapshot` assembly.
- Preserve the downstream `Context::ported_topology()` / `Context::ported_brep()` behavior and existing topology snapshot parity.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the extraction.

## Why This Is Next

With the root-loading prelude now owned by `brep/root_topology.rs`, the remaining stage-specific logic in `topology.rs` is the front-door face-shape load and validation call. Pulling that next leaves the snapshot entry point as a thin coordinator over one face input helper, one root input helper, one face packing helper, and final assembly.
