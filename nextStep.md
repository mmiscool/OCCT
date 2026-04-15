# Next Task

Extract the remaining root-loading prelude out of `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs` so the snapshot entry point becomes a thin coordinator over root and face snapshot helpers.

## Focus

- Move the vertex/edge/wire loading and root topology construction block in `ported_topology_snapshot()` into `brep/root_topology.rs` or a tiny sibling helper owned by that root stage.
- Keep the current behavior for vertex-position loading, root edge matching, root wire matching, and packed wire/index layout.
- Leave `ported_topology_snapshot()` in `topology.rs`, but trim it down to face validation, a call into the root snapshot helper, a call into the face snapshot helper, and final `TopologySnapshot` assembly.
- Preserve the downstream `Context::ported_topology()` / `Context::ported_brep()` behavior and existing topology snapshot parity.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the extraction.

## Why This Is Next

With the face validation and face packing both owned by `brep/face_snapshot.rs`, the remaining nontrivial body in `topology.rs` is the root-loading prelude that fetches vertices, builds root edges, builds root wires, and packs the wire ranges. Pulling that next keeps the snapshot entry point converging on pure orchestration over already-split root and face stages.
