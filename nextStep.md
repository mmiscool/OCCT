# Next Task

Move `PreparedFaceTopology::match_face_wire()` under the builder-owned face-wire collection path in `face_snapshot.rs`.

## Focus

- Reevaluate whether the builder should own the face-wire topology match directly or whether that logic should move onto `MatchedFaceWire` as a constructor-style helper.
- Keep `PreparedFaceTopology` as the final assembled result and preserve the direct snapshot accumulator handoff.
- Preserve the shared planar-face validation rule, per-wire root-wire matching behavior, planar wire area computation, wire-role classification, face range offsets, edge-face ordering, and packed snapshot output unchanged.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With builder setup, per-wire collection, and finalization folded into one entry point, the remaining non-builder helper in this path is `match_face_wire()`. Moving that logic under the builder-owned face-wire collection path is the next bounded cleanup toward a single-owner implementation.
