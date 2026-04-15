# Next Task

Tighten the remaining planar-face setup boundary between `PreparedFaceShape` and `PreparedFaceTopology` in `face_snapshot.rs`.

## Focus

- Reevaluate whether `PreparedFaceTopology::collect_matched_face_wires()` should keep pulling an ad hoc `Option<(PlanePayload, FaceGeometry)>` tuple from `PreparedFaceShape::planar_face()`.
- Reevaluate whether the planar multi-wire setup should stay on `PreparedFaceShape` as a more explicit carrier or helper boundary now that the wire-access and wrapper layers are gone.
- Keep `PreparedFaceTopology` as the owner of matched-wire assembly and preserve the direct accumulator handoff.
- Preserve the shared planar-face validation rule, face-wire matching behavior, planar wire area computation, face range offsets, edge-face ordering, and packed snapshot output unchanged.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With the wire-access helpers and thin wrapper layers removed, the remaining cross-type setup handoff in this path is the raw planar-face tuple returned by `PreparedFaceShape::planar_face()`. Tightening that boundary is the next bounded cleanup in the same per-face snapshot logic.
