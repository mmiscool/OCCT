# Next Task

Carry the loader-owned root edge and face `Shape` inventories out of the topology loader so `ported_brep()` can stop re-entering raw `subshapes_occt()` after topology construction.

## Focus

- Extend the internal topology-loading boundary so the same root edge and face shape inventories already discovered during [`ported_topology_snapshot()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs) are preserved for higher-level BRep assembly.
- Thread that loader-owned state through the `ported_brep()` path in [`brep.rs`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep.rs) so the successful Rust-topology case does not enumerate root edges and faces a second time just to materialize `BrepEdge` and `BrepFace`.
- Keep the explicit `*_occt()` escape hatches intact for actual raw geometry queries and for the fallback OCCT topology path; only move the root edge/face traversal boundary upward.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the change.

## Why This Is Next

The materializer-local reload is gone: `ported_brep_edges()` and `ported_brep_faces()` now consume caller-owned shape inventories. The next remaining raw traversal is at the `ported_brep()` entry itself, which still calls `subshapes_occt()` for root edges and faces even after the Rust topology loader has already walked and preserved equivalent root-level shape state.
