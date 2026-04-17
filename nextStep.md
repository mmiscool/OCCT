# Next Task

Current milestone: `M2. Whole-Shape Summary Fallback Reduction` from `portingMilestones.md`.

## Completed Evidence

- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs` now makes the remaining multi-face `face_bboxes_occt()` path explicit: `offset_faces_bbox(...)` only keeps that branch when `offset_faces_require_occt_face_union(...)` finds at least one face that still cannot validate through the Rust face path, instead of letting every multi-face offset shell fall through to the OCCT face union implicitly.
- The same file now strengthens `validated_face_brep_bbox(...)` by unioning face boundary geometry into the face summary and mesh candidates before validation and reusing the axis-aware offset expansion there, which keeps more face-level candidates on the Rust side before the explicit guard is consulted.
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces -- --exact`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_volume_for_offset_solids -- --exact`, the full `brep_workflows` suite, `cargo check`, and the full `cargo test` suite all passed after the guard was narrowed.

## Target

Remove the remaining OCCT-described face bbox union inside the exercised multi-face `OffsetFaceUnion` shell-summary path, while preserving the Rust-owned offset-solid root bbox and volume paths already locked in by `brep_workflows`.

## Next Bounded Cut

1. Promote a dedicated Rust-owned bbox candidate for the exercised mesh-backed plane cap inside `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs`, so that face no longer forces `offset_faces_require_occt_face_union(...)` to keep the OCCT face-union branch alive.
2. Delete the multi-face `face_bboxes_occt()` branch once that plane-cap candidate validates the exercised shell union against the shell OCCT bbox, while leaving the single-face offset path untouched unless it also gains a Rust-owned replacement.
3. Keep the existing shell-level regression in `rust/lean_occt/tests/brep_workflows.rs` green and, if needed, add a targeted assertion that the exercised shell family no longer trips the explicit unsupported-face guard once the plane-cap bbox lands.

## Guardrails

- Keep loader-owned `PreparedShellShape` inventories.
- Do not reintroduce raw `subshapes_occt()` shell traversal beyond the existing prepared-shell loading path.
- Keep validating accepted Rust-owned shell bbox candidates against OCCT shell bboxes while the explicit per-face guard remains in place.
- Keep the root bbox probe, root volume probe, and shell probe observable until the remaining whole-shape fallback branches are narrowed behind explicit unsupported-case guards.
- Do not spend the next turn reshuffling summary helpers unless it removes the exercised `face_bboxes_occt()` dependency or lands the plane-cap regression that proves the exercised offset shell is already off the explicit guard.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces -- --exact`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_volume_for_offset_solids -- --exact`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
