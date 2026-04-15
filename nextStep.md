# Next Task

Move the remaining face-preload constructor into `impl PreparedFaceShape` in `face_snapshot.rs`.

## Focus

- Reevaluate whether `load_prepared_face_shapes()` should become a `PreparedFaceShape` constructor/loader now that the prepared-face type already owns the related helper surface.
- Keep `PreparedFaceTopology` as the owner of per-face setup and preserve the direct accumulator handoff.
- Preserve the shared planar-face validation rule, face-wire matching behavior, planar wire area computation, face range offsets, edge-face ordering, and packed snapshot output unchanged.
- Preserve the downstream `Context::ported_topology()` / `Context::ported_brep()` behavior and existing topology snapshot parity.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

`PreparedFaceShape` now owns the helper surface used by the per-face topology loader, but the top-level face-preload entry still lives as a separate free function. Moving that constructor path onto the type is the next small cleanup that should make the face snapshot stage read more coherently without changing behavior.
