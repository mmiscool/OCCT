# Next Task

Collapse the thin builder mutation helpers in `face_snapshot.rs`.

## Focus

- Reevaluate whether `PreparedFaceTopologyBuilder::append_matched_face_wire()` and `push_face_wire_area()` should stay as separate mutators or fold into a single per-wire application helper.
- Keep `PreparedFaceTopology` as the final assembled result and preserve the direct snapshot accumulator handoff.
- Preserve the shared planar-face validation rule, per-wire root-wire matching behavior, planar wire area computation, face range offsets, edge-face ordering, and packed snapshot output unchanged.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With final output assembly now explicit on `PreparedFaceTopology`, the remaining tiny ownership split in this path is the pair of builder mutation helpers used from `collect_face_wire()`. Collapsing those mutators is the next bounded cleanup toward a tighter single-step per-wire collection flow.
