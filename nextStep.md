# Next Task

Collapse the thin per-face append wrapper in `face_snapshot.rs`.

## Focus

- Keep `PreparedFaceTopology` as the owner of per-face setup and make the handoff into `FaceSnapshotAccumulator` more direct, likely by letting the accumulator consume `PreparedFaceTopology` itself.
- Preserve the shared planar-face validation rule and the now-internal planar multi-wire setup path.
- Keep the accumulator-owned writeback/finalization flow, face-wire matching behavior, planar wire area computation, face range offsets, edge-face ordering, and packed snapshot output unchanged.
- Preserve the downstream `Context::ported_topology()` / `Context::ported_brep()` behavior and existing topology snapshot parity.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With per-face setup now fully owned by `PreparedFaceTopology`, the remaining glue in `append_ported_face_topology()` is just a handoff into the accumulator. Tightening that boundary should simplify the pack loop without changing behavior.
