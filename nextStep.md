# Next Task

Collapse the thin `apply_face_wire()` builder helper in `face_snapshot.rs`.

## Focus

- Reevaluate whether `PreparedFaceTopologyBuilder::collect_face_wire()` should update builder state directly after matching and area resolution, or whether the matched-wire carrier should absorb the last area append without widening its ownership.
- Keep `PreparedFaceTopology` as the final assembled result and preserve the direct snapshot accumulator handoff.
- Preserve the shared planar-face validation rule, per-wire root-wire matching behavior, planar wire area computation, face range offsets, edge-face ordering, and packed snapshot output unchanged.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With the slice-level loop now in `build()`, the next tiny indirection in this path is the separate `apply_face_wire()` helper between per-wire matching and builder mutation. Collapsing that helper is the next bounded cleanup toward a tighter builder-owned collection flow.
