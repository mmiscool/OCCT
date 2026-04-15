# Next Task

Split the face-topology matching and planar loop classification cluster out of `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs` into a dedicated sibling module.

## Focus

- Move `PortedFaceTopology`, `ported_face_topology()`, `match_root_wire_index()`, and `planar_wire_area_magnitude()` out of `brep/topology.rs` into a focused helper module such as `brep/face_snapshot.rs`.
- Keep that new module consuming the already-split `root_topology` helpers instead of reintroducing duplicate ownership.
- Leave `ported_topology_snapshot()` and `pack_wire_topology()` in `topology.rs`, with the new sibling module handling face-level matching and loop-role classification.
- Preserve the current behavior for planar multi-wire face handling and the downstream `Context::ported_topology()` and `Context::ported_brep()` paths.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the extraction.

## Why This Is Next

With the reusable root wire/edge matching block now split out, the remaining heavy ownership in `topology.rs` is the face-specific topology matching and loop-role classification path. Pulling that cluster into its own sibling module is the next bounded cleanup before `topology.rs` becomes mostly orchestration and packing.
