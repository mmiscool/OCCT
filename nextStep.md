# Next Task

Extract the remaining face-packing accumulation block out of `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs` so the snapshot entry point becomes a thin coordinator over root loading and face-owned packing helpers.

## Focus

- Move the block that builds `edge_face_lists`, `faces`, `face_wire_indices`, `face_wire_orientations`, `face_wire_roles`, and final `edge_faces` / `edge_face_indices` into `brep/face_snapshot.rs` or a tiny sibling helper owned by the face-packing stage.
- Keep the current behavior for `ported_face_topology()` failure handling, edge-to-face adjacency packing, and face wire range/layout assembly.
- Leave `ported_topology_snapshot()` in `topology.rs`, but trim it down to root-shape loading, a call into the face-packing helper, and final `TopologySnapshot` assembly.
- Preserve the downstream `Context::ported_topology()` / `Context::ported_brep()` behavior and existing topology snapshot parity.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the extraction.

## Why This Is Next

With the preflight validation now owned by the face-snapshot stage, the remaining face-specific logic in `topology.rs` is the accumulation and packing block that turns per-face matches into the final face and edge-face ranges. Pulling that next keeps the snapshot entry point converging on pure orchestration over already-split topology stages.
