# Next Task

Extract the edge-face flattening tail from the face snapshot packing path.

## Focus

- Pull the final `edge_face_lists` to `edge_faces` / `edge_face_indices` flattening out of `pack_ported_face_snapshot()` in `face_snapshot.rs`.
- Keep the extracted planar multi-wire setup, face-wire matching helper, planar wire area computation, loop-role classification, and packed snapshot output unchanged.
- Keep the current edge-face ordering, range offsets, face validation, and per-face accumulation behavior unchanged.
- Preserve the downstream `Context::ported_topology()` / `Context::ported_brep()` behavior and existing topology snapshot parity.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With the per-face wire matching now extracted, the main remaining packing-specific tail in `face_snapshot.rs` is the final edge-face flattening pass. Pulling that into its own helper should leave `pack_ported_face_snapshot()` as orchestration around per-face collection and final snapshot assembly.
