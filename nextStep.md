# Next Task

Move `PreparedFaceTopology::classify_wire_roles()` under the builder-owned finalization path in `face_snapshot.rs`.

## Focus

- Reevaluate whether the role classification helper should become a builder-owned method or a `PreparedFaceTopology` constructor-style helper.
- Keep `PreparedFaceTopology` as the final assembled result and preserve the direct snapshot accumulator handoff.
- Preserve the shared planar-face validation rule, per-wire root-wire matching behavior, planar wire area computation, face range offsets, edge-face ordering, and packed snapshot output unchanged.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With builder setup, per-wire collection, and root-wire matching now folded into one owner, the remaining non-builder helper on this path is `classify_wire_roles()`, which is only used during builder finalization. Moving that role-selection logic under the finalization path is the next bounded cleanup toward a single-owner implementation.
