# Next Task

Fold the remaining planar multi-wire setup into `PreparedFaceTopology`.

## Focus

- Keep `PreparedFaceTopology` as the clear owner of per-face setup in `face_snapshot.rs` by pulling the remaining planar multi-wire preparation path under that type.
- Preserve the shared planar-face validation rule between snapshot preflight and the per-face setup path.
- Keep the accumulator-owned writeback/finalization flow, face-wire matching behavior, planar wire area computation, face range offsets, edge-face ordering, and packed snapshot output unchanged.
- Preserve the downstream `Context::ported_topology()` / `Context::ported_brep()` behavior and existing topology snapshot parity.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

The matched-wire collection and wire-role classification now live under `PreparedFaceTopology`, so the remaining free setup helper is the planar multi-wire loader. Pulling that under the same owner should finish tightening the per-face setup boundary without changing behavior.
