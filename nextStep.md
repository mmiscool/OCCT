# Next Task

Collapse the builder setup, face-wire collection, and finalization in `PreparedFaceTopology::collect_matched_face_wires()` into a single builder-owned entry point in `face_snapshot.rs`.

## Focus

- Reevaluate whether `PreparedFaceTopologyBuilder` should expose one `build`-style entry that owns `new(...)`, per-wire collection, and `finish()`.
- Keep `PreparedFaceTopology` as the final assembled result and preserve the direct snapshot accumulator handoff.
- Preserve the shared planar-face validation rule, per-wire root-wire matching behavior, planar wire area computation, wire-role classification, face range offsets, edge-face ordering, and packed snapshot output unchanged.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With the per-wire iteration moved onto `PreparedFaceTopologyBuilder`, the next concentrated block in this path is the remaining builder orchestration in `collect_matched_face_wires()`. Folding that into one builder-owned entry is the next bounded cleanup toward making the face-topology collector a minimal wrapper.
