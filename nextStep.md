# Next Task

Replace the mutable out-parameter collection path in `PreparedFaceTopology`.

## Focus

- Now that `MatchedFaceWires` is gone, reevaluate `PreparedFaceTopology::collect_matched_face_wires()` and remove the current mutable out-parameter shape if there is a clearer local ownership pattern.
- Keep `PreparedFaceTopology` as the owner of per-face setup and preserve the direct accumulator handoff.
- Preserve the shared planar-face validation rule, face-wire matching behavior, planar wire area computation, face range offsets, edge-face ordering, and packed snapshot output unchanged.
- Preserve the downstream `Context::ported_topology()` / `Context::ported_brep()` behavior and existing topology snapshot parity.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

The prepared-face data shape is now tighter, but the collection helper still communicates through several mutable out parameters. Replacing that with a clearer local flow is the next small cleanup that should improve readability without changing behavior.
