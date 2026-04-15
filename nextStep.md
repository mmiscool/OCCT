# Next Task

Extract the face-wire matching and accumulation loop from the face snapshot packing path.

## Focus

- Pull the per-wire `root_wire_topology()` / `match_root_wire_index()` / accumulator update loop out of `append_ported_face_topology()` in `face_snapshot.rs`.
- Keep the extracted planar multi-wire setup, planar wire area computation, loop-role classification, and packed snapshot output unchanged.
- Keep the current face validation, root-wire matching behavior, used-edge tracking, face-wire orientation writes, and edge-face accumulation unchanged.
- Preserve the downstream `Context::ported_topology()` / `Context::ported_brep()` behavior and existing topology snapshot parity.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With the planar multi-wire setup now extracted, the main remaining mixed concern inside `append_ported_face_topology()` is the face-wire matching loop itself. Pulling that into its own helper should leave the per-face path as orchestration around setup, role classification, and packed-snapshot writes.
