# Next Task

Collapse the thin `MatchedFaceWire::planar_area_magnitude()` helper in `face_snapshot.rs`.

## Focus

- Reevaluate whether the planar wire-area lookup in `MatchedFaceWire::planar_area_magnitude()` should move directly into the builder loop or onto a builder-owned helper without changing matching behavior.
- Keep `PreparedFaceTopology` as the final assembled result and preserve the direct snapshot accumulator handoff.
- Preserve the shared planar-face validation rule, per-wire root-wire matching behavior, planar wire area computation, face range offsets, edge-face ordering, and packed snapshot output unchanged.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With the matched-wire state writes now inlined into `PreparedFaceTopologyBuilder::build()`, the next tiny indirection in this same collection path is `MatchedFaceWire::planar_area_magnitude()`, which is now a one-use wrapper around the planar wire-area lookup. Collapsing that helper is the next bounded cleanup toward a tighter builder-owned collection flow.
