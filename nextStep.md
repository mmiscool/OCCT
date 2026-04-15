# Next Task

Move the remaining topology-only query logic out of `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep.rs` and into `brep/topology.rs`.

## Focus

- Move the implementation bodies for `Context::ported_vertex_point()` and `Context::ported_edge_endpoints()` into topology-owned helper functions in `brep/topology.rs`.
- Keep the `Context` methods in `brep.rs` as thin delegators, so the parent module only exposes the public entry points.
- Reuse the existing `shape_counts()` plus root-kind classification path from `brep/summary.rs` instead of reintroducing inline topology classification logic.
- Leave behavior unchanged and keep `cargo check --manifest-path rust/lean_occt/Cargo.toml` and `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows` passing after the move.

## Why This Is Next

The summary-only scaffolding is now owned by `brep/summary.rs`, and `brep.rs` no longer contains private structs. The next remaining non-entry-point logic in the parent module is the root-kind guarded topology query code for vertex points and edge endpoints, which belongs with the rest of the topology snapshot helpers in `brep/topology.rs`.
