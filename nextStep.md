# Next Task

Move the remaining per-wire iteration in `PreparedFaceTopology::collect_matched_face_wires()` onto `PreparedFaceTopologyBuilder` in `face_snapshot.rs`.

## Focus

- Reevaluate whether the builder should own the per-wire `match_face_wire`, optional planar area fetch, and append flow for one face wire.
- Keep `PreparedFaceTopology` as the final assembled result and preserve the direct snapshot accumulator handoff.
- Preserve the shared planar-face validation rule, per-wire root-wire matching behavior, planar wire area computation, wire-role classification, face range offsets, edge-face ordering, and packed snapshot output unchanged.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With the parallel local collection state collapsed into `PreparedFaceTopologyBuilder`, the next concentrated block in this path is the remaining per-wire iteration inside `collect_matched_face_wires()`. Moving that onto the builder is the next bounded cleanup toward making the face-topology collector a thin coordinator.
