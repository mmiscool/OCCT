# Next Task

Collapse the remaining snapshot-builder shell so the top-level topology stage owns its temporary carriers and final assembly directly.

## Focus

- Move `TopologySnapshotRootFields`, `TopologySnapshotFaceFields`, and `build_ported_topology_snapshot()` out of `snapshot_build.rs` into `topology.rs` or another clearly-owned topology-stage module.
- Keep the current root loading, face loading, and final snapshot assembly unchanged.
- Preserve the downstream `Context::ported_topology()` / `Context::ported_brep()` behavior and existing topology snapshot parity.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With the root loader now living in `topology.rs`, the remaining shell around the topology stage is `snapshot_build.rs`, which only carries temporary structs plus the final assembly helper. Folding that into the topology stage removes another thin indirection layer.
