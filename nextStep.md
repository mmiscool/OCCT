# Next Task

Split the reusable topology accessor cluster out of `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs` into a dedicated sibling module.

## Focus

- Move `topology_edge()`, `adjacent_face_indices()`, `edge_points()`, and `optional_vertex_position()` out of `brep/topology.rs` into a focused helper module such as `brep/topology_access.rs`.
- Rewire `brep/brep_materialize.rs`, `brep/shape_queries.rs`, and any remaining callers to import those accessors from the new module.
- Leave `topology.rs` focused on topology snapshot construction, root wire/edge matching, and traversal logic instead of mixed access helpers.
- Preserve the current behavior for `Context::ported_brep()`, `ported_vertex_point()`, and `ported_edge_endpoints()`.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the extraction.

## Why This Is Next

With the root BRep materialization helpers now split out, the remaining reusable support cluster in `topology.rs` is the small accessor layer that other sibling modules depend on. Pulling that cluster into its own module is the next bounded ownership cleanup before shrinking `topology.rs` further.
