# Next Task

Make `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs` a pure coordinator by moving its snapshot carrier structs and final constructor into a dedicated sibling module.

## Focus

- Move `TopologySnapshotRootFields`, `TopologySnapshotFaceFields`, and the final `TopologySnapshot` assembly out of `topology.rs` into a dedicated builder-style sibling module.
- Keep the current root loading, face loading, field wiring, ordering, and failure handling unchanged.
- Preserve `ported_topology_snapshot()` as a thin coordinator over the root snapshot helper, the face snapshot helper, and the final snapshot constructor entry.
- Preserve the downstream `Context::ported_topology()` / `Context::ported_brep()` behavior and existing topology snapshot parity.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the extraction.

## Why This Is Next

With both face and root snapshot modules narrowed to a single exported loader, the next smallest cleanup is to remove the remaining helper-type ownership from `topology.rs` itself so it stays as orchestration only.
