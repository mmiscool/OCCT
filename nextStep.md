# Next Task

Collapse the remaining helper-only wire access surface between `PreparedFaceShape` and `PreparedFaceTopology` in `face_snapshot.rs`.

## Focus

- Reevaluate whether `PreparedFaceTopology::load()` still needs the separate `is_empty()` guard now that `collect_matched_face_wires()` already takes `PreparedFaceShape` directly.
- Reevaluate whether `collect_matched_face_wires()` should keep peeling `PreparedFaceShape` back to a raw wire slice, or whether the remaining wire iteration should stay on the prepared-face type itself.
- Keep `PreparedFaceTopology` as the owner of matched-wire assembly and preserve the direct accumulator handoff.
- Preserve the shared planar-face validation rule, face-wire matching behavior, planar wire area computation, face range offsets, edge-face ordering, and packed snapshot output unchanged.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

`collect_matched_face_wires()` now takes `PreparedFaceShape` directly, which removed the old split handoff, but the code still reaches back into helper-only `is_empty()` and `wire_shapes()` accessors immediately afterward. Tightening or eliminating that remaining helper surface is the next bounded cleanup in the same path.
