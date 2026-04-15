# Next Task

Extract the per-wire root-wire match step from `PreparedFaceTopology::collect_matched_face_wires()` in `face_snapshot.rs`.

## Focus

- Reevaluate whether the root-wire topology build, root-wire index match, used-edge accumulation, and orientation fetch for one face wire should live in a dedicated helper or carrier.
- Keep `PreparedFaceTopology` as the owner of matched-wire assembly and preserve the direct accumulator handoff.
- Preserve the shared planar-face validation rule, face-wire matching behavior, planar wire area computation, face range offsets, edge-face ordering, and packed snapshot output unchanged.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With the planar-face setup boundary made explicit, the next concentrated block in this path is the repeated per-wire match logic inside `collect_matched_face_wires()`. Isolating that step is the next bounded cleanup without changing the surrounding topology assembly flow.
