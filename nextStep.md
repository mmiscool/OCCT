# Next Task

Move the remaining per-face setup helpers under `PreparedFaceTopology`.

## Focus

- Turn the new `PreparedFaceTopology` path in `face_snapshot.rs` into the clear owner of the remaining setup helpers, especially the current matched-wire collection and role-classification pieces.
- Keep the accumulator-owned writeback/finalization flow, extracted planar multi-wire setup, face-wire matching behavior, planar wire area computation, and packed snapshot output unchanged.
- Keep the current face range offsets, edge-face ordering, validation behavior, and packed snapshot output unchanged.
- Preserve the downstream `Context::ported_topology()` / `Context::ported_brep()` behavior and existing topology snapshot parity.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With the per-face setup now extracted, the remaining free helpers in `face_snapshot.rs` are still conceptually owned by `PreparedFaceTopology`. Moving them under that type should tighten the per-face setup boundary without changing snapshot behavior.
