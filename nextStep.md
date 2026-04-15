# Next Task

Tighten the remaining raw wire-slice handoff from `PreparedFaceShape` into `PreparedFaceTopology` in `face_snapshot.rs`.

## Focus

- Reevaluate whether `collect_matched_face_wires()` should take `PreparedFaceShape` more directly now that the prepared-face type owns preload, wire access, and planar-face setup.
- Keep `PreparedFaceTopology` as the owner of per-face setup and preserve the direct accumulator handoff.
- Preserve the shared planar-face validation rule, face-wire matching behavior, planar wire area computation, face range offsets, edge-face ordering, and packed snapshot output unchanged.
- Preserve the downstream `Context::ported_topology()` / `Context::ported_brep()` behavior and existing topology snapshot parity.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

`PreparedFaceShape` now owns the preload constructor, the planar preflight, and the optional planar-face setup, but `PreparedFaceTopology` still immediately peels that back to raw wire slices and a separate planar-face value. Tightening that handoff is the next small cleanup that should make the per-face topology path read more coherently without changing behavior.
