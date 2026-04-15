# Next Task

Collapse the thin `PreparedFaceTopologyBuilder::new()` constructor in `face_snapshot.rs`.

## Focus

- Reevaluate whether the builder initialization in `PreparedFaceTopologyBuilder::build()` should construct the state inline or via a more local helper without changing capacity sizing or ownership boundaries.
- Keep `PreparedFaceTopology` as the final assembled result and preserve the direct snapshot accumulator handoff.
- Preserve the shared planar-face validation rule, per-wire root-wire matching behavior, planar wire area computation, face range offsets, edge-face ordering, and packed snapshot output unchanged.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With the per-wire match now inlined into `build()`, the next tiny indirection in this builder flow is the separate `PreparedFaceTopologyBuilder::new()` constructor used only at that entry point. Collapsing that constructor is the next bounded cleanup toward a tighter builder-owned collection flow.
