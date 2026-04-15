# Next Task

Split the face-specific single-face topology routing block out of `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs` into a dedicated sibling module.

## Focus

- Move `FaceSurfaceRoute`, `SingleFaceTopology`, `single_face_topology_with_route()`, `single_face_topology_snapshot()`, `single_face_edge_with_route()`, and `single_face_edge()` out of `brep/topology.rs` into a new focused helper module such as `brep/face_topology.rs`.
- Keep `ported_face_area()` and the swept/public face preparation path reusing those helpers through the new module without changing behavior.
- Leave the root topology snapshot and root BRep materialization helpers in `topology.rs` so that file trends back toward shape-level topology ownership only.
- Preserve the current raw/public route split for edge geometry and curve reconstruction in the single-face path.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the extraction.

## Why This Is Next

With the public face query wrappers now isolated in `brep/face_queries.rs`, the biggest remaining face-specific routing cluster still sitting in the shared topology module is the single-face route-aware helper set. Pulling that into its own sibling module is the next bounded split that keeps `topology.rs` focused on root topology snapshotting and BRep materialization.
