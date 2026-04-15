# Next Task

Collapse the remaining local matched-wire collection state in `PreparedFaceTopology::collect_matched_face_wires()` into a small builder in `face_snapshot.rs`.

## Focus

- Reevaluate whether `used_root_wire_indices`, `face_wire_indices`, `face_wire_orientations`, `face_wire_areas`, and `used_edges` should move behind a dedicated builder or accumulator type local to this path.
- Keep `PreparedFaceTopology` as the final owner of matched-wire assembly and preserve the direct snapshot accumulator handoff.
- Preserve the shared planar-face validation rule, per-wire root-wire matching behavior, planar wire area computation, wire-role classification, face range offsets, edge-face ordering, and packed snapshot output unchanged.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With the per-wire matched-wire writes isolated, the next concentrated block in this path is the remaining parallel local collection state inside `collect_matched_face_wires()`. Moving that behind one small builder keeps the cleanup bounded while simplifying the final face-topology assembly flow.
