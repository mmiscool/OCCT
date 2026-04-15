# Next Task

Collapse the one-use `FaceSnapshotAccumulator::into_fields()` helper in `face_snapshot.rs`.

## Focus

- Reevaluate whether the one-use `FaceSnapshotAccumulator::into_fields()` helper should be collapsed directly into `pack_ported_face_snapshot()` without changing the final `TopologySnapshotFaceFields` assembly or accumulator-owned writeback.
- Keep `PreparedFaceTopology` as the final assembled result and preserve the direct snapshot accumulator handoff.
- Preserve the shared planar-face validation rule, per-wire root-wire matching behavior, planar wire area computation, face range offsets, edge-face ordering, and packed snapshot output unchanged.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With the one-use `FaceSnapshotAccumulator::new()` constructor now gone, the next tiny indirection in this same face snapshot path is `FaceSnapshotAccumulator::into_fields()`, which is only used at the end of `pack_ported_face_snapshot()`. Collapsing that helper is the next bounded cleanup toward a tighter snapshot coordinator path.
