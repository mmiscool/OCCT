# Rust Port Milestones

This file is the control plane for the Codex loop. The goal is to move tested, user-visible capability from OCCT-backed paths to Rust-owned paths in bounded slices.

## Working Rules

- A turn only counts as progress if it does at least one of these: deletes or narrows an OCCT fallback, expands a Rust-owned capability, or adds regression coverage for newly Rust-owned behavior.
- Do not spend two consecutive turns only reshaping helpers inside one function unless the second turn removes a fallback or lands new tested behavior.
- Stay on the exercised kernel slice first: `ported_geometry`, `brep`, `document`, `pipeline`, and their integration tests. Do not drift into placeholder `occt_port/DataExchange` files just because they are easy to touch.
- If a bounded cut stalls for one turn, record the blocker in `nextStep.md` and move to the next task inside the same milestone.

## Turn Status

- Completed evidence: `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs` now gives `exact_torus_summary(...)` a Rust-owned analytic bbox, so exercised torus solids resolve their root bbox through `SummaryBboxSource::ExactPrimitive` instead of falling through to the generic whole-shape mesh path. `rust/lean_occt/tests/brep_workflows.rs` extends `ported_brep_uses_exact_primitive_bounding_boxes` with a rotated torus regression that pins the Rust bbox to the analytic torus envelope and keeps the other rotated exact primitives on their OCCT-parity assertions. The focused torus regression, the exercised offset-solid regression, the full `brep_workflows` suite, `cargo check`, and the full `cargo test` suite all passed. The deferred blocker remains the exercised multi-face offset-shell plane cap: the discarded normalized-corner/boundary sampling attempt produced a Rust face union far smaller than the OCCT shell bbox, so that cut was rolled back instead of leaving a half-working face-union replacement in place.
- Active milestone: `M2. Whole-Shape Summary Fallback Reduction`.
- Next bounded cut: reconstruct a real trimmed-surface bbox candidate for the exercised multi-face offset-shell plane cap, validate it against the shell OCCT bbox, and then delete the remaining multi-face `face_bboxes_occt()` branch for that exercised family instead of relying on broad boundary or normalized-corner sampling.
- Verification:
  - `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
  - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_exact_primitive_bounding_boxes -- --exact`
  - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_volume_for_offset_solids -- --exact`
  - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
  - `cargo check --manifest-path rust/lean_occt/Cargo.toml`
  - `cargo test --manifest-path rust/lean_occt/Cargo.toml`

## M1. Rust-Owned Offset Shell Bounding Boxes

Outcome: exercised offset shells and offset solids stop depending on the final shell-local OCCT bbox fallback in `offset_shell_bbox()` in `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs`.

Status: complete on 2026-04-16. The exercised `brep_workflows` offset-solid fixture now records `OffsetShellBboxSource::Brep`, and the unconditional `or(Some(shell_occt_bbox))` tier has been replaced by an explicit unsupported-shell guard that only leaves OCCT as an escape hatch when no prepared shell topology inventory exists.

Definition of done: the exercised offset fixtures in `brep_workflows` stay green, the winning shell bbox path is Rust-owned for those fixtures, and the final `or(Some(shell_occt_bbox))` tier is either removed for the exercised path or isolated behind an explicit unsupported-case guard.

Bounded tasks: make the shell-bbox winner observable, promote the nearest Rust-owned winner (`validated_shell_boundary_bbox`, `validated_shell_mesh_bbox`, or `validated_shell_brep_bbox`), then delete or strictly narrow the final fallback.

Verification: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`.

## M2. Whole-Shape Summary Fallback Reduction

Outcome: `ported_shape_summary()` stops using `fallback_summary()` for bbox and volume on the supported analytic, swept, and offset families already exercised in `brep_workflows`.

Status: active. The exercised offset-solid bbox path resolves through Rust-owned shell-union data tagged as `SummaryBboxSource::OffsetSolidShellUnion`, the exercised offset-solid volume path resolves through `SummaryVolumeSource::FaceContributions` without touching the shared volume fallback, the exercised swept-revolution solid resolves its root volume through `SummaryVolumeSource::FaceContributions`, the exercised exact-primitive roots now include torus solids on `SummaryBboxSource::ExactPrimitive`, the exercised exact-curve and single-face offset roots stay off the generic whole-shape OCCT bbox fallback, and the exercised multi-face offset shells still resolve their root shell summaries through `SummaryBboxSource::OffsetFaceUnion`. The remaining bbox gap inside this milestone is now isolated behind the explicit `offset_faces_require_occt_face_union(...)` guard: one mesh-backed plane cap in the exercised multi-face shell family still needs a real trimmed-surface Rust bbox candidate before the OCCT-described per-face bbox union can be deleted entirely. The shared volume fallback remains only for explicitly unsupported families.

Definition of done: supported solids and shells under current tests resolve bbox and volume through Rust-owned paths plus validation, and any remaining `fallback_summary()` calls are behind explicit unsupported-shape guards instead of being the normal path.

Bounded tasks: identify each surviving `fallback_summary()` branch, replace one branch at a time starting with supported offset and swept cases, and add at least one regression that covers a family which previously fell through to OCCT.

Verification: same as M1.

## M3. Rust-Backed Traversal for Documents and Selectors

Outcome: `ModelDocument`, selectors, and high-level reports use `BrepShape` and `TopologySnapshot` for supported face and edge traversal instead of ad hoc OCCT subshape walks.

Definition of done: `document_workflows`, `selector_workflows`, and `recipe_workflows` stay green while the supported selector/report paths stop relying on raw `subshapes_occt()` for face and edge enumeration.

Bounded tasks: map one traversal-heavy read path at a time, switch it to `ported_brep()` or topology-backed data, then keep parity with the existing behavior through workflow tests.

Verification: `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test recipe_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`.

## M4. Public Query Fallback Cleanup

Outcome: supported public geometry query APIs are served end-to-end from the ported layer for the analytic, swept, and offset kinds already covered by `ported_geometry_workflows`.

Definition of done: the exercised supported query families remain green with added coverage for one newly cleaned-up public path, and OCCT helper fallbacks remain only for explicitly unsupported kinds.

Bounded tasks: remove redundant public fallbacks only after parity tests cover the same kind through the ported descriptor path, then keep unsupported cases explicit instead of implicit.

Verification: `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`.
