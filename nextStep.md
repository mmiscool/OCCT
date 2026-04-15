# Next Task

Tighten the duplicated face-wire preload and planar gate in `face_snapshot.rs`.

## Focus

- Reevaluate the repeated `subshapes_occt(..., ShapeKind::Wire)` / wire-count load and planar multi-wire decision that currently happens in both `validate_ported_face_snapshot()` and `PreparedFaceTopology::load()`.
- Keep `PreparedFaceTopology` as the owner of per-face setup and preserve the direct accumulator handoff.
- Preserve the shared planar-face validation rule, face-wire matching behavior, planar wire area computation, face range offsets, edge-face ordering, and packed snapshot output unchanged.
- Preserve the downstream `Context::ported_topology()` / `Context::ported_brep()` behavior and existing topology snapshot parity.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

The temporary `CollectedFaceWires` boundary is gone, but the module still reloads face wires and re-derives the same planar preflight facts in both the snapshot validator and the per-face loader. Tightening that shared entry condition is the next small cleanup that should simplify the face snapshot path without changing behavior.
