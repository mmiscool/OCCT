# Next Task

Current milestone: `M2. Whole-Shape Summary Fallback Reduction` from `portingMilestones.md`.

## Completed Evidence

- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep.rs` now exports `OffsetFaceBboxSource` and stores the winning offset-face root bbox source on `BrepShape`, so exercised offset shells can expose whether they still depend on `OcctFaceUnion` or already resolve through a Rust-owned path.
- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs` now threads that source through `ported_shape_summary()` and widens `validated_mesh_bbox(...)` to accept the exercised single-face offset-shell mesh candidate after an axis-by-axis `2 * offset_margin + 1e-4` expansion against the OCCT bbox.
- `rust/lean_occt/tests/brep_workflows.rs` extends `ported_brep_uses_rust_owned_area_for_offset_faces` so the exercised single-face offset shell now proves `brep.offset_face_bbox_source() == Some(OffsetFaceBboxSource::ValidatedMesh)` while keeping the existing root `SummaryBboxSource::OffsetFaceUnion` assertion and OCCT-parity bbox checks.
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces -- --exact`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_volume_for_offset_solids -- --exact`, the full `brep_workflows` suite, `cargo check`, and the full `cargo test` suite all passed after the single-face offset-shell fallback deletion.
- The highest-value remaining blocker inside `M2` is now explicit on the exercised multi-face offset shell: the new public probe still reports `Some(OffsetFaceBboxSource::OcctFaceUnion)` because one plane cap's OCCT face bbox is hundreds of units larger than every current Rust candidate.

## Target

Remove another remaining OCCT-backed whole-shape summary branch inside `M2` without regressing the now-green single-face offset-shell path, the offset-solid root path, or the exact-primitive roots. The highest-value remaining gap is still the exercised multi-face `OffsetFaceUnion` shell-summary path, and the new public source probe narrows the work to the branch that still reports `OcctFaceUnion`.

## Next Bounded Cut

1. Use `BrepShape::offset_face_bbox_source()` on the exercised multi-face offset shell to keep the blocker observable while replacing the remaining `OcctFaceUnion` winner with a Rust-owned branch.
2. Derive a non-recursive Rust bbox candidate for the plane cap inside `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs`; prefer trimmed-surface or topology-backed face extents over broad normalized-corner or loose edge sampling, because the discarded broad candidates underfit the shell by hundreds of units.
3. Feed that candidate into the shell-level `SummaryFaceBrep` validator and delete the multi-face `face_bboxes_occt()` dependency once the exercised shell no longer reports `OcctFaceUnion`.
4. Keep the single-face offset-shell regression on `OffsetFaceBboxSource::ValidatedMesh`, the offset-solid regression, and the exact-primitive regressions green so the next cut cannot silently reintroduce either the single-face or multi-face OCCT face-union branches.

## Guardrails

- Keep loader-owned `PreparedShellShape` inventories.
- Do not reintroduce raw `subshapes_occt()` shell traversal beyond the existing prepared-shell loading path.
- Keep validating accepted Rust-owned shell bbox candidates against OCCT shell bboxes while the explicit per-face guard remains in place.
- Keep the root bbox probe, root volume probe, and shell probe observable until the remaining whole-shape fallback branches are narrowed behind explicit unsupported-case guards.
- Do not spend the next turn reshuffling summary helpers unless it removes the exercised multi-face `face_bboxes_occt()` dependency or lands a regression that proves the exercised multi-face shell is off `OcctFaceUnion`.
- Keep the exercised single-face offset shell on `OffsetFaceBboxSource::ValidatedMesh`; do not let it slide back to `OcctFaceUnion`.
- Keep the rotated torus on `SummaryBboxSource::ExactPrimitive`; do not let it fall back to the generic mesh bbox path just to match OCCT's looser torus envelope.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces -- --exact`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_volume_for_offset_solids -- --exact`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
