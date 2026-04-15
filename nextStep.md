# Next Task

Replace the nested planar-face result in `PreparedFaceTopology`.

## Focus

- Keep `PreparedFaceTopology` as the owner of per-face setup, but replace the current nested `Option<Option<(PlanePayload, FaceGeometry)>>` return from the planar-face loader with a clearer carrier or control flow.
- Preserve the shared planar-face validation rule and the current multi-wire planar setup behavior.
- Keep the accumulator-owned writeback/finalization flow, face-wire matching behavior, planar wire area computation, face range offsets, edge-face ordering, and packed snapshot output unchanged.
- Preserve the downstream `Context::ported_topology()` / `Context::ported_brep()` behavior and existing topology snapshot parity.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

The setup and append boundaries are now tight, but `PreparedFaceTopology::load_planar_face()` still communicates three states through nested `Option`s. Replacing that with a clearer representation is the next small cleanup that should improve readability without changing behavior.
