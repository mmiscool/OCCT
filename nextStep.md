# Next Task

Collapse the one-use `FaceSnapshotAccumulator::append_face_topology_outputs()` helper in `face_snapshot.rs`.

## Focus

- Reevaluate whether the one-use `FaceSnapshotAccumulator::append_face_topology_outputs()` helper should be collapsed directly into the `pack_ported_face_snapshot()` loop without changing accumulator-owned face range writes or edge-face ordering.
- Keep `PreparedFaceTopology` as the final assembled result and preserve the direct snapshot accumulator handoff.
- Preserve the shared planar-face validation rule, per-wire root-wire matching behavior, planar wire area computation, face range offsets, edge-face ordering, and packed snapshot output unchanged.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With the one-use `FaceSnapshotAccumulator::flatten_edge_face_lists()` helper now gone, the next tiny indirection in this same face snapshot path is `FaceSnapshotAccumulator::append_face_topology_outputs()`, which is only used inside the `pack_ported_face_snapshot()` loop. Collapsing that helper is the next bounded cleanup toward a tighter snapshot coordinator path.
