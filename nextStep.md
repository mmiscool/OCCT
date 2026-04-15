# Next Task

Collapse the thin `PreparedFaceTopology::collect_matched_face_wires()` wrapper in `face_snapshot.rs`.

## Focus

- Reevaluate whether callers should invoke the builder directly or whether `PreparedFaceTopology` should gain a constructor-style entry point that replaces the delegating wrapper.
- Keep `PreparedFaceTopology` as the final assembled result and preserve the direct snapshot accumulator handoff.
- Preserve the shared planar-face validation rule, per-wire root-wire matching behavior, planar wire area computation, face range offsets, edge-face ordering, and packed snapshot output unchanged.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With setup, per-wire collection, matching, and role classification now folded into the builder, the remaining extra layer on this path is the delegating `PreparedFaceTopology::collect_matched_face_wires()` wrapper. Collapsing that boundary is the next bounded cleanup toward a single-owner construction path.
