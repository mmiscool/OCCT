# Next Task

Extract the face output writeback tail from the face snapshot packing path.

## Focus

- Pull the final `faces` / `face_wire_indices` / `face_wire_orientations` / `face_wire_roles` writes and `edge_face_lists` updates out of `append_ported_face_topology()` in `face_snapshot.rs`.
- Keep the extracted planar multi-wire setup, face-wire matching helper, edge-face flattening helper, planar wire area computation, and loop-role classification unchanged.
- Keep the current face range offsets, edge-face ordering, validation behavior, and packed snapshot output unchanged.
- Preserve the downstream `Context::ported_topology()` / `Context::ported_brep()` behavior and existing topology snapshot parity.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With the edge-face flattening now extracted, the main remaining mixed writeback block in `face_snapshot.rs` is the final per-face output application inside `append_ported_face_topology()`. Pulling that into its own helper should leave the per-face path as setup, matching, role classification, and a single writeback step.
