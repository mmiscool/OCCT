# Next Task

Tighten the remaining carrier layering inside `PreparedFaceTopology`.

## Focus

- Now that planar-face state is explicit, reevaluate the remaining split between `PreparedFaceTopology` and `MatchedFaceWires` in `face_snapshot.rs`.
- Prefer a clearer ownership boundary for per-face prepared data while keeping the direct accumulator handoff intact.
- Preserve the shared planar-face validation rule, face-wire matching behavior, planar wire area computation, face range offsets, edge-face ordering, and packed snapshot output unchanged.
- Preserve the downstream `Context::ported_topology()` / `Context::ported_brep()` behavior and existing topology snapshot parity.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

The setup and append boundaries are now tight, and planar-face state is explicit. The remaining extra carrier in this path is `MatchedFaceWires`, so tightening that layering is the next small cleanup that should simplify the prepared-face data shape without changing behavior.
