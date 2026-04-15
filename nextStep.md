# Next Task

Tighten the remaining `PreparedFaceShape` to `PreparedFaceTopology::load()` handoff in `face_snapshot.rs`.

## Focus

- Reevaluate whether the remaining face-shape access and planar-face setup logic should be expressed more directly on `PreparedFaceShape` so `PreparedFaceTopology::load()` stays focused on topology matching and role classification.
- Keep `PreparedFaceTopology` as the owner of per-face setup and preserve the direct accumulator handoff.
- Preserve the shared planar-face validation rule, face-wire matching behavior, planar wire area computation, face range offsets, edge-face ordering, and packed snapshot output unchanged.
- Preserve the downstream `Context::ported_topology()` / `Context::ported_brep()` behavior and existing topology snapshot parity.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

The temporary planar wrapper is gone, but `PreparedFaceTopology::load()` still reaches back into `PreparedFaceShape` for raw face-shape access and planar-face loading details. Tightening that boundary is the next small cleanup that should leave the per-face topology path easier to read without changing behavior.
