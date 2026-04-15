# Next Task

Collapse the remaining standalone planar preflight helper into `PreparedFaceShape` in `face_snapshot.rs`.

## Focus

- Reevaluate whether `multi_wire_face_is_planar()` should move under `PreparedFaceShape` now that the prepared-face type owns both face preload and optional planar-face setup.
- Keep `PreparedFaceTopology` as the owner of per-face setup and preserve the direct accumulator handoff.
- Preserve the shared planar-face validation rule, face-wire matching behavior, planar wire area computation, face range offsets, edge-face ordering, and packed snapshot output unchanged.
- Preserve the downstream `Context::ported_topology()` / `Context::ported_brep()` behavior and existing topology snapshot parity.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

`PreparedFaceShape` now owns the preload constructor and the optional planar-face setup, but the multi-wire planar preflight still lives as a separate free helper. Pulling that rule into the type is the next small cleanup that should make the face snapshot entry read more coherently without changing behavior.
