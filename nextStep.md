# Next Task

Tighten the remaining split between `CollectedFaceWires` and `PreparedFaceTopology`.

## Focus

- Now that collection is back to local ownership, reevaluate whether `CollectedFaceWires` should keep its current role or hand off more directly into `PreparedFaceTopology`.
- Keep `PreparedFaceTopology` as the owner of per-face setup and preserve the direct accumulator handoff.
- Preserve the shared planar-face validation rule, face-wire matching behavior, planar wire area computation, face range offsets, edge-face ordering, and packed snapshot output unchanged.
- Preserve the downstream `Context::ported_topology()` / `Context::ported_brep()` behavior and existing topology snapshot parity.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

The mutable out-parameter path is gone, but there is still a small two-step transition from `CollectedFaceWires` into `PreparedFaceTopology`. Tightening that boundary is the next small cleanup that should simplify the load path without changing behavior.
