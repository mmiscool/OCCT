# Next Task

Split the root wire and edge matching cluster out of `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs` into a dedicated sibling module.

## Focus

- Move `root_edge_topology()`, `ported_wire_occurrences()`, `root_wire_topology()`, and `root_wire_topology_from_snapshot()` out of `brep/topology.rs` into a focused helper module such as `brep/root_topology.rs`.
- Move the supporting matching helpers they depend on as part of the same slice, including `wire_occurrence()`, `order_wire_occurrences()`, `chain_wire_occurrences()`, `match_vertex_index()`, and `approx_points_eq()`.
- Leave `ported_topology_snapshot()` and the face-topology packing flow in `topology.rs`, with the new sibling module providing the reusable root wire/edge matching layer.
- Preserve the current behavior for root-edge/root-wire topology reconstruction and the downstream `Context::ported_topology()` and `Context::ported_brep()` paths.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the extraction.

## Why This Is Next

With the materialization helpers and generic accessors now split out, the largest remaining ownership chunk in `topology.rs` is the root wire/edge matching block that feeds topology snapshot construction. Pulling that cluster into its own sibling module is the next bounded step toward leaving `topology.rs` as a thin orchestrator over topology assembly stages.
