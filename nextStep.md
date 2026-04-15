# Next Task

Collapse the remaining face snapshot accumulators into a dedicated local struct.

## Focus

- Replace the parallel `edge_face_lists`, `faces`, `face_wire_indices`, `face_wire_orientations`, and `face_wire_roles` locals in `pack_ported_face_snapshot()` with a dedicated accumulator struct in `face_snapshot.rs`.
- Keep the extracted planar multi-wire setup, face-wire matching helper, face output writeback helper, edge-face flattening helper, planar wire area computation, and loop-role classification unchanged.
- Keep the current face range offsets, edge-face ordering, validation behavior, and packed snapshot output unchanged.
- Preserve the downstream `Context::ported_topology()` / `Context::ported_brep()` behavior and existing topology snapshot parity.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With the per-face writeback now extracted, the main remaining coordination burden in `face_snapshot.rs` is the packer’s parallel mutable vectors. Collapsing them into a local accumulator should make the snapshot stage easier to read without changing any packing behavior.
