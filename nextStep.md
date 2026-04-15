# Next Task

Collapse the thin `collect_face_wire()` helper in `face_snapshot.rs`.

## Focus

- Reevaluate whether the remaining per-wire helper in `PreparedFaceTopologyBuilder` should move into the `build()` loop or collapse into a builder-owned collection entry point without changing match, planar area, or mutation behavior.
- Keep `PreparedFaceTopology` as the final assembled result and preserve the direct snapshot accumulator handoff.
- Preserve the shared planar-face validation rule, per-wire root-wire matching behavior, planar wire area computation, face range offsets, edge-face ordering, and packed snapshot output unchanged.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With the final mutation now inlined into the per-wire path, the next tiny indirection in this builder flow is the separate `collect_face_wire()` helper under the `build()` loop. Collapsing that helper is the next bounded cleanup toward a tighter builder-owned collection flow.
