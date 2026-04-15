# Next Task

Move the remaining root-wire carrier and packing block out of `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/root_topology.rs` into the wire-owned module.

## Focus

- Move `RootWireTopology` and `pack_wire_topology()` out of `root_topology.rs` and into `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/wire_topology.rs`.
- Keep the current root loading, edge matching, wire ordering, and failure handling unchanged.
- Preserve `load_root_topology_snapshot()` and the existing downstream `root_wire_topology()` / face-snapshot behavior, with unchanged results.
- Preserve the downstream `Context::ported_topology()` / `Context::ported_brep()` behavior and existing topology snapshot parity.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the extraction.

## Why This Is Next

With the root-edge helper path moved out, the remaining non-loader state in `root_topology.rs` is the root-wire carrier and packing path; moving that into the wire-owned module leaves `root_topology.rs` closer to a pure root snapshot coordinator.
