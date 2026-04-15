# Next Task

Extract the per-wire matched-wire accumulation step from `PreparedFaceTopology::collect_matched_face_wires()` in `face_snapshot.rs`.

## Focus

- Reevaluate whether the updates to `used_root_wire_indices`, `used_edges`, `face_wire_indices`, and `face_wire_orientations` for one matched face wire should live in a dedicated helper or a small local accumulator type.
- Keep `PreparedFaceTopology` as the owner of matched-wire assembly and preserve the direct accumulator handoff.
- Preserve the shared planar-face validation rule, per-wire root-wire matching behavior, planar wire area computation, wire-role classification, face range offsets, edge-face ordering, and packed snapshot output unchanged.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With the per-wire planar area lookup isolated, the next concentrated block in this path is the repeated matched-wire state update inside `collect_matched_face_wires()`. Extracting that keeps the cleanup bounded while continuing to simplify the per-wire assembly loop without changing snapshot behavior.
