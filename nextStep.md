# Next Task

Collapse the remaining final `TopologySnapshot` field assembly in `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs` into a tiny helper so the snapshot entry point becomes a pure coordinator over root load, face load, and snapshot construction.

## Focus

- Move the final `TopologySnapshot { ... }` field assembly behind a dedicated helper or constructor near `brep/topology.rs`.
- Keep the current field mapping between the root snapshot data and the packed face snapshot data unchanged.
- Leave `ported_topology_snapshot()` in `topology.rs`, but trim it down to a call into the root snapshot helper, a call into the face snapshot helper, and a call into the final snapshot-construction helper.
- Preserve the downstream `Context::ported_topology()` / `Context::ported_brep()` behavior and existing topology snapshot parity.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the extraction.

## Why This Is Next

With the face stage now collapsed behind one helper and the root stage already isolated, the only remaining implementation detail in `topology.rs` is the explicit `TopologySnapshot` field assembly. Pulling that behind one tiny helper is the smallest bounded cleanup that leaves the entry point as pure orchestration over the already-split stages.
