# Next Task

Split the remaining root-shape query wrappers out of `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs` into a dedicated sibling module.

## Focus

- Move `ported_vertex_point()` and `ported_edge_endpoints()` out of `brep/topology.rs` into a focused helper module such as `brep/shape_queries.rs`.
- Keep those wrappers reusing the existing `shape_counts()` and `classify_root_kind()` path from `brep/summary.rs`; this is an ownership cleanup, not a behavior change.
- Leave `topology.rs` focused on root topology snapshotting, root BRep materialization, and generic topology accessors only.
- Preserve the current behavior for single-root vertex and edge queries on the public Rust path.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the extraction.

## Why This Is Next

With the face-only helpers now split into `brep/face_topology.rs`, the main remaining public-style wrappers still sitting in `topology.rs` are the root-shape vertex and edge query entry points. Pulling those into their own sibling module is the next bounded step toward making `topology.rs` purely about topology construction and traversal.
