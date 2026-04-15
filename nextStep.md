# Next Task

Collapse the thin `FaceSnapshotAccumulator::append_prepared_face_topology()` wrapper in `face_snapshot.rs`.

## Focus

- Reevaluate whether the face-wire offset calculation and `PreparedFaceTopology` destructuring should live directly in `append_face_topology_outputs()` instead of a separate accumulator wrapper.
- Reevaluate whether the pack loop should hand the `PreparedFaceTopology` fields to the accumulator directly once the wrapper is gone.
- Keep `PreparedFaceTopology` as the owner of matched-wire assembly and preserve the direct accumulator handoff.
- Preserve the shared planar-face validation rule, face-wire matching behavior, planar wire area computation, face range offsets, edge-face ordering, and packed snapshot output unchanged.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With the thin `PreparedFaceTopology::load()` wrapper removed, the next small layer in the same path is `append_prepared_face_topology()`, which is now mostly a face-wire offset calculation plus a direct call into `append_face_topology_outputs()`.
