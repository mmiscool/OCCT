# Next Task

Extract the remaining per-face setup path from `append_ported_face_topology()`.

## Focus

- Pull the remaining face-wire load, planar-face selection, and matched-wire classification setup out of `append_ported_face_topology()` in `face_snapshot.rs`.
- Keep the accumulator-owned writeback/finalization flow, extracted planar multi-wire setup, face-wire matching helper, planar wire area computation, and loop-role classification unchanged.
- Keep the current face range offsets, edge-face ordering, validation behavior, and packed snapshot output unchanged.
- Preserve the downstream `Context::ported_topology()` / `Context::ported_brep()` behavior and existing topology snapshot parity.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With the accumulator now owning the packer helpers, the main remaining mixed concern in `face_snapshot.rs` is the per-face setup logic still sitting inline in `append_ported_face_topology()`. Pulling that into its own helper should leave the append path as a thin coordinator over setup, matching, and accumulator writeback.
