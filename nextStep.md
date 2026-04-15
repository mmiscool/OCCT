# Next Task

Move the remaining face-only topology accessors out of `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs` and into the new `brep/face_topology.rs` sibling module.

## Focus

- Move `face_loops()` and `face_adjacent_face_indices()` into `brep/face_topology.rs`, alongside `FaceSurfaceRoute` and `single_face_topology_with_route()`.
- Update `face_surface.rs` and any other face-only callers to import those helpers from `face_topology.rs`.
- Leave `topology.rs` owning root topology snapshotting, root BRep materialization, and generic edge/vertex accessors only.
- Keep behavior unchanged for internal BRep face assembly and the public face query path.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the extraction.

## Why This Is Next

The route-aware single-face builder now lives in `brep/face_topology.rs`, but `topology.rs` still carries the last face-only loop and adjacency accessors that only serve face assembly. Moving those next is the cleanest bounded follow-up and leaves the shared topology module closer to shape-level ownership only.
