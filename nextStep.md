# Next Task

Collapse the remaining `PreparedPlanarFace` wrapper in `face_snapshot.rs`.

## Focus

- Now that unsupported multi-wire faces are rejected at the snapshot entry, reevaluate whether `PreparedPlanarFace` still needs to exist instead of using a simpler `Option<(PlanePayload, FaceGeometry)>` path.
- Keep `PreparedFaceTopology` as the owner of per-face setup and preserve the direct accumulator handoff.
- Preserve the shared planar-face validation rule, face-wire matching behavior, planar wire area computation, face range offsets, edge-face ordering, and packed snapshot output unchanged.
- Preserve the downstream `Context::ported_topology()` / `Context::ported_brep()` behavior and existing topology snapshot parity.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

The duplicated face-wire preflight is now centralized at the snapshot entry, so `PreparedPlanarFace` is down to a thin local wrapper around “not required” versus “loaded plane payload and face geometry.” Collapsing that wrapper is the next small cleanup that should simplify the per-face setup path without changing behavior.
