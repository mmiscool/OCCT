# Next Task

Collapse the one-use `PreparedFaceTopology::new()` constructor in `face_snapshot.rs`.

## Focus

- Reevaluate whether the one-use final constructor `PreparedFaceTopology::new()` should be collapsed directly into `PreparedFaceTopologyBuilder::build()` without changing the assembled carrier or its accumulator handoff.
- Keep `PreparedFaceTopology` as the final assembled result and preserve the direct snapshot accumulator handoff.
- Preserve the shared planar-face validation rule, per-wire root-wire matching behavior, planar wire area computation, face range offsets, edge-face ordering, and packed snapshot output unchanged.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With the one-use planar-face helper now gone, the next tiny indirection in this same builder-owned collection path is `PreparedFaceTopology::new()`, which is only used at the final assembly point in `PreparedFaceTopologyBuilder::build()`. Collapsing that constructor is the next bounded cleanup toward a tighter face snapshot build path.
