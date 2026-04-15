# Next Task

Extract the remaining root-edge loading and orientation helper block out of `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/root_topology.rs` into a dedicated sibling module.

## Focus

- Move `RootEdgeTopology`, `root_edge_topology()`, and `oriented_edge_geometry()` out of `root_topology.rs` into a sibling module that owns edge loading/orientation.
- Keep the current root loading, edge matching, wire ordering, and failure handling unchanged.
- Preserve `load_root_topology_snapshot()` and the existing downstream `root_wire_topology()` / face-snapshot behavior, with unchanged results.
- Preserve the downstream `Context::ported_topology()` / `Context::ported_brep()` behavior and existing topology snapshot parity.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the extraction.

## Why This Is Next

With the wire reconstruction cluster split out, the next remaining non-loader block in `root_topology.rs` is the root-edge loading/orientation helper path; moving that out leaves the root snapshot module focused on shape enumeration and packaging.
