# Next Task

Collapse the temporary `MatchedFaceWire` carrier in `face_snapshot.rs`.

## Focus

- Reevaluate whether the temporary `MatchedFaceWire` struct should be inlined into `PreparedFaceTopologyBuilder::build()` or replaced with a more direct builder-owned state shape without changing matching behavior.
- Keep `PreparedFaceTopology` as the final assembled result and preserve the direct snapshot accumulator handoff.
- Preserve the shared planar-face validation rule, per-wire root-wire matching behavior, planar wire area computation, face range offsets, edge-face ordering, and packed snapshot output unchanged.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With the planar wire-area lookup now inlined into `PreparedFaceTopologyBuilder::build()`, the next tiny indirection in this same collection path is the temporary `MatchedFaceWire` carrier, which now only groups three fields between matching and builder mutation. Collapsing that carrier is the next bounded cleanup toward a tighter builder-owned collection flow.
