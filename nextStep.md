# Next Task

Introduce a constructor-style helper for the final `PreparedFaceTopology` assembly in `face_snapshot.rs`.

## Focus

- Reevaluate whether `PreparedFaceTopologyBuilder::finish()` should call a new `PreparedFaceTopology` constructor or whether the final struct assembly should stay builder-owned with a clearer helper boundary.
- Keep `PreparedFaceTopology` as the final assembled result and preserve the direct snapshot accumulator handoff.
- Preserve the shared planar-face validation rule, per-wire root-wire matching behavior, planar wire area computation, face range offsets, edge-face ordering, and packed snapshot output unchanged.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With the delegating wrapper removed, `PreparedFaceTopology` is now a pure output carrier and the remaining direct ownership question is the final struct assembly in `PreparedFaceTopologyBuilder::finish()`. Introducing a constructor-style helper is the next bounded cleanup to make the final boundary explicit without changing the packed snapshot behavior.
