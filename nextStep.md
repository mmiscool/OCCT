# Next Task

Collapse the now-thin root snapshot loader layer so `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/root_topology.rs` is no longer just a one-function pass-through module.

## Focus

- Decide whether `load_root_topology_snapshot()` belongs in `topology.rs`, `snapshot_build.rs`, or a renamed root snapshot module, and remove the extra shell layer.
- Keep the current root loading, edge loading, wire ordering, and failure handling unchanged.
- Preserve the existing downstream face-snapshot behavior and the final `Context::ported_topology()` / `Context::ported_brep()` results.
- Preserve the downstream `Context::ported_topology()` / `Context::ported_brep()` behavior and existing topology snapshot parity.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With the root-edge and root-wire helpers moved out, `root_topology.rs` now mostly exists to host a single loader entry point. The next cleanup is removing or renaming that shell so the root snapshot stage has a clearer ownership boundary.
