# Next Task

Collapse the thin `PreparedFaceTopology::load()` wrapper in `face_snapshot.rs`.

## Focus

- Reevaluate whether the early `root_wires.is_empty()` / empty-face guard should live in `collect_matched_face_wires()` instead of a separate wrapper.
- Reevaluate whether `pack_ported_face_snapshot()` should call the matched-wire builder directly once the wrapper is gone.
- Keep `PreparedFaceTopology` as the owner of matched-wire assembly and preserve the direct accumulator handoff.
- Preserve the shared planar-face validation rule, face-wire matching behavior, planar wire area computation, face range offsets, edge-face ordering, and packed snapshot output unchanged.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With the helper-only wire accessors removed, `PreparedFaceTopology::load()` is now mostly a guard plus a direct call into `collect_matched_face_wires()`. Collapsing that wrapper is the next bounded cleanup in the same per-face snapshot path.
