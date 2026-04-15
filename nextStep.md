# Next Task

Extract the wire-topology reconstruction and matching block out of `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/root_topology.rs` into a dedicated sibling module.

## Focus

- Move the wire occurrence, snapshot-based wire recovery, and ordering helpers out of `root_topology.rs` into a sibling module that owns wire matching.
- Keep the current root loading, edge matching, wire ordering, and failure handling unchanged.
- Preserve `load_root_topology_snapshot()` and `root_wire_topology()` as the public root-stage entry points, with unchanged downstream behavior.
- Preserve the downstream `Context::ported_topology()` / `Context::ported_brep()` behavior and existing topology snapshot parity.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the extraction.

## Why This Is Next

With `topology.rs` reduced to pure orchestration, the next largest self-contained block is the wire reconstruction logic still living inside `root_topology.rs`; moving that cluster out keeps the root loader focused on loading and packaging.
