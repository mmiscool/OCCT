# Next Task

Current milestone: `M2. Whole-Shape Summary Fallback Reduction` from `portingMilestones.md`.

## Completed Evidence

- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs` now gives `exact_torus_summary(...)` a Rust-owned analytic bbox, so supported torus solids no longer need the generic whole-shape mesh bbox branch inside `ported_shape_summary()`.
- `rust/lean_occt/tests/brep_workflows.rs` extends `ported_brep_uses_exact_primitive_bounding_boxes` with a rotated torus regression that pins both `kernel.summarize(...)` and `kernel.brep(...).summary` to the analytic torus envelope while keeping `SummaryBboxSource::ExactPrimitive` observable.
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_exact_primitive_bounding_boxes -- --exact`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_volume_for_offset_solids -- --exact`, the full `brep_workflows` suite, `cargo check`, and the full `cargo test` suite all passed after the torus bbox landed.
- The attempted follow-up on the exercised multi-face offset-shell plane cap was rolled back in the same turn because normalized-corner and sampled-boundary candidates still produced a Rust face union far smaller than the OCCT shell bbox; that blocker remains explicit instead of leaving a broken partial replacement in tree.

## Target

Remove another remaining OCCT-backed whole-shape summary branch inside `M2` without regressing the already green offset-solid and exact-primitive paths. The highest-value remaining gap is still the exercised multi-face `OffsetFaceUnion` shell-summary path, but the next cut needs a tighter face-domain candidate than the rolled-back broad sampling attempt.

## Next Bounded Cut

1. Promote a dedicated Rust-owned bbox candidate for the exercised mesh-backed plane cap inside `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs`, so that face no longer forces `offset_faces_require_occt_face_union(...)` to keep the OCCT face-union branch alive.
2. Build that candidate from the trimmed surface domain or equivalent topology-backed face extents rather than normalized-corner or loose edge sampling; the discarded attempt proved those broader proxies underfit the exercised shell by hundreds of units on one axis.
3. Delete the multi-face `face_bboxes_occt()` branch once that trimmed-face candidate validates the exercised shell union against the shell OCCT bbox, while leaving the single-face offset path untouched unless it also gains a Rust-owned replacement.
4. Keep the exact-primitive torus regression and the existing offset-solid shell regression green so the next cut cannot silently reintroduce the generic mesh bbox path or the shell-local OCCT root fallback.

## Guardrails

- Keep loader-owned `PreparedShellShape` inventories.
- Do not reintroduce raw `subshapes_occt()` shell traversal beyond the existing prepared-shell loading path.
- Keep validating accepted Rust-owned shell bbox candidates against OCCT shell bboxes while the explicit per-face guard remains in place.
- Keep the root bbox probe, root volume probe, and shell probe observable until the remaining whole-shape fallback branches are narrowed behind explicit unsupported-case guards.
- Do not spend the next turn reshuffling summary helpers unless it removes the exercised `face_bboxes_occt()` dependency or lands a regression that proves the exercised offset shell is already off the explicit guard.
- Keep the rotated torus on `SummaryBboxSource::ExactPrimitive`; do not let it fall back to the generic mesh bbox path just to match OCCT's looser torus envelope.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_exact_primitive_bounding_boxes -- --exact`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_volume_for_offset_solids -- --exact`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
