# Next Task

Collapse the thin `PreparedFaceTopologyBuilder::finish()` helper in `face_snapshot.rs`.

## Focus

- Reevaluate whether the final role-classification and `PreparedFaceTopology` assembly in `PreparedFaceTopologyBuilder::finish()` should move into `build()` or into a constructor-style path on the output carrier without changing ownership or validation behavior.
- Keep `PreparedFaceTopology` as the final assembled result and preserve the direct snapshot accumulator handoff.
- Preserve the shared planar-face validation rule, per-wire root-wire matching behavior, planar wire area computation, face range offsets, edge-face ordering, and packed snapshot output unchanged.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With the builder initialization now inlined into `build()`, the next tiny indirection in this builder flow is the separate `PreparedFaceTopologyBuilder::finish()` helper used only at the exit point. Collapsing that helper is the next bounded cleanup toward a tighter builder-owned collection flow.
