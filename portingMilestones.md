# Rust Port Milestones

This file is the control plane for the Codex loop. The goal is to move tested, user-visible capability from OCCT-backed paths to Rust-owned paths in bounded slices.

## Working Rules

- A turn only counts as progress if it does at least one of these: deletes or narrows an OCCT fallback, expands a Rust-owned capability, or adds regression coverage for newly Rust-owned behavior.
- Treat analysis-only, probe-only, formatting-only, and helper-only turns as failed porting turns unless they are paired with a real Rust-owned behavior move in the same turn.
- Bias toward decisive, coherent cuts: replace an exercised fallback branch or capability family end-to-end instead of making the smallest local edit around it.
- Multi-file changes are expected when the port requires them. Carry a Rust-owned path through data structures, call sites, C ABI glue, integration tests, examples, and docs rather than stopping at the first seam.
- If a prerequisite refactor is needed, perform it only as part of the same turn that removes or strictly narrows an OCCT fallback or lands new tested Rust behavior.
- Use compiler and test failures as the work queue for finishing the chosen porting cut; do not retreat to a tiny safe change solely because the larger Rust replacement touches several modules.
- Stay on the exercised kernel slice first: `ported_geometry`, `brep`, `document`, `pipeline`, and their integration tests. Do not drift into placeholder `occt_port/DataExchange` files just because they are easy to touch.
- If a bounded cut stalls for one turn, record the blocker in `nextStep.md` and move to the next task inside the same milestone.

## Turn Status

- Completed evidence: `rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs` now makes `PortedCurve::from_context_with_ported_payloads()` return only Rust-derived line, circle, and ellipse payloads, with `None` for unsupported extraction instead of rescuing through `edge_line_payload_occt()`, `edge_circle_payload_occt()`, or `edge_ellipse_payload_occt()`. `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/brep_materialize.rs` and `face_topology.rs` now propagate public-route Rust curve extraction errors instead of retrying `PortedCurve::from_context_with_geometry()`. `rust/lean_occt/tests/brep_workflows.rs` now asserts exercised line, circle, and ellipse BRep edges populate `BrepEdge::ported_curve`, derive edge lengths from those ported curves, match the public Rust curve path, and stay close to OCCT samples/lengths. `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_exact_curve_bounding_boxes`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` all passed.
- Active milestone: `M5. BRep Curve Payload Fallback Cleanup`.
- Next bounded cut: continue `M5` by removing the remaining raw rescue inside `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_snapshot.rs`, where planar wire reconstruction still catches `PortedCurve::from_context_with_ported_payloads()` errors and retries `PortedCurve::from_context_with_geometry()`. Carry the same Rust-owned curve payload behavior through `append_root_edge_sample_points()` in `swept_face.rs` if needed, then strengthen `brep_workflows` around holed planar faces or swept/offset face roots that exercise root-edge sampling without the OCCT payload rescue.
- Verification:
  - `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
  - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`
  - `cargo check --manifest-path rust/lean_occt/Cargo.toml`
  - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows`
  - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows`
  - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
  - `cargo test --manifest-path rust/lean_occt/Cargo.toml`

## M1. Rust-Owned Offset Shell Bounding Boxes

Outcome: exercised offset shells and offset solids stop depending on the final shell-local OCCT bbox fallback in `offset_shell_bbox()` in `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs`.

Status: complete on 2026-04-16. The exercised `brep_workflows` offset-solid fixture now records `OffsetShellBboxSource::Brep`, and the unconditional `or(Some(shell_occt_bbox))` tier has been replaced by an explicit unsupported-shell guard that only leaves OCCT as an escape hatch when no prepared shell topology inventory exists.

Definition of done: the exercised offset fixtures in `brep_workflows` stay green, the winning shell bbox path is Rust-owned for those fixtures, and the final `or(Some(shell_occt_bbox))` tier is either removed for the exercised path or isolated behind an explicit unsupported-case guard.

Bounded tasks: make the shell-bbox winner observable, promote the nearest Rust-owned winner (`validated_shell_boundary_bbox`, `validated_shell_mesh_bbox`, or `validated_shell_brep_bbox`), then delete or strictly narrow the final fallback.

Verification: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`.

## M2. Whole-Shape Summary Fallback Reduction

Outcome: `ported_shape_summary()` stops using `fallback_summary()` for bbox and volume on the supported analytic, swept, and offset families already exercised in `brep_workflows`.

Status: complete on 2026-04-26. The exercised offset-solid bbox path resolves through Rust-owned shell-union data tagged as `SummaryBboxSource::OffsetSolidShellUnion`, the exercised offset-solid volume path resolves through `SummaryVolumeSource::FaceContributions` without touching the shared volume fallback, the exercised swept-revolution solid resolves its root volume through `SummaryVolumeSource::FaceContributions`, the exercised exact-primitive roots now include torus solids on `SummaryBboxSource::ExactPrimitive`, the exercised exact-curve roots stay off the generic whole-shape OCCT bbox fallback, the exercised single-face offset shell root is explicitly observed on `OffsetFaceBboxSource::ValidatedMesh`, and the exercised multi-face offset shell root resolves through `OffsetFaceBboxSource::SummaryFaceBrep`. The old whole-face `face_bboxes_occt()`/`OcctFaceUnion` bbox path and the offset-specific `offset_shape_bbox_occt()`/`OffsetOcctSubshapeUnion` path have both been removed. The remaining generic `fallback_summary()` bbox and volume branches are now behind explicit unsupported-shape guards, and fully loaded analytic, swept, or offset face inventories are promoted into Rust-owned bbox/volume requirements when existing Rust candidates cover them.

Definition of done: supported solids and shells under current tests resolve bbox and volume through Rust-owned paths plus validation, and any remaining `fallback_summary()` calls are behind explicit unsupported-shape guards instead of being the normal path.

Bounded tasks: identify each surviving `fallback_summary()` branch, replace one branch at a time starting with supported offset and swept cases, and add at least one regression that covers a family which previously fell through to OCCT.

Verification: same as M1.

## M3. Rust-Backed Traversal for Documents and Selectors

Outcome: `ModelDocument`, selectors, and high-level reports use `BrepShape` and `TopologySnapshot` for supported face and edge traversal instead of ad hoc OCCT subshape walks.

Status: complete on 2026-04-26. `ModelDocument::edges()`, `ModelDocument::faces()`, `select_edge()`, `select_face()`, and high-level inspection consume `BrepShape`, and the remaining public `Context::subshape()`/`Context::subshapes()` bridge through `shape_queries.rs` now requires `ported_topology()` before it will validate counts and materialize handles. `Context::subshape_count()` also uses Rust topology for supported face, wire, edge, and vertex counts, with OCCT count fallback only after the ported topology loader returns `None`.

Definition of done: `document_workflows`, `selector_workflows`, and `recipe_workflows` stay green while the supported selector/report paths stop relying on raw `subshapes_occt()` for face and edge enumeration.

Bounded tasks: map one traversal-heavy read path at a time, switch it to `ported_brep()` or topology-backed data, then keep parity with the existing behavior through workflow tests.

Verification: `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test recipe_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`.

## M4. Public Query Fallback Cleanup

Outcome: supported public geometry query APIs are served end-to-end from the ported layer for the analytic, swept, and offset kinds already covered by `ported_geometry_workflows`.

Status: complete on 2026-04-26. Public line, circle, ellipse, plane, cylinder, cone, sphere, torus, extrusion, revolution, offset payload, offset-basis payload, and swept offset-basis curve geometry/payload APIs now use Rust descriptors for matching supported kinds and return explicit Rust mismatch or unsupported-basis errors for non-matching supported descriptors. The remaining OCCT helper fallbacks in these public query APIs are isolated to `None` from the ported descriptor loaders.

Definition of done: the exercised supported query families remain green with added coverage for one newly cleaned-up public path, and OCCT helper fallbacks remain only for explicitly unsupported kinds.

Bounded tasks: remove redundant public fallbacks only after parity tests cover the same kind through the ported descriptor path, then keep unsupported cases explicit instead of implicit.

Verification: `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`.

## M5. BRep Curve Payload Fallback Cleanup

Outcome: BRep edge materialization and face topology paths stop using `PortedCurve::from_context_with_geometry()` as an OCCT-payload rescue after Rust curve-payload extraction has already identified an exercised line, circle, or ellipse edge path.

Status: active. The first public-route cut is complete: `PortedCurve::from_context_with_ported_payloads()` no longer calls the OCCT line, circle, or ellipse payload helpers, and the public BRep materialization paths in `brep_materialize.rs` and `face_topology.rs` no longer swallow extraction errors into `PortedCurve::from_context_with_geometry()`. Remaining raw/rescue work is bounded to `face_snapshot.rs`, `swept_face.rs`, and the intentionally raw `FaceSurfaceRoute::Raw` branch in `face_topology.rs`.

Definition of done: exercised BRep edges with line, circle, and ellipse geometry populate `BrepEdge::ported_curve` through Rust-owned extraction, edge lengths and sampled face loops remain stable, and OCCT payload helpers are no longer the normal rescue path for supported public-route BRep edge materialization.

Bounded tasks: remove the OCCT payload rescue from one curve materialization path at a time. The public-route `PortedCurve::from_context_with_ported_payloads()`, `ported_brep_edges()`, and `single_face_edge_with_route()` cut is complete; next remove the planar wire/root-edge rescues in `face_snapshot.rs` and `swept_face.rs`, then leave the explicitly raw route for a final bounded cut.

Verification: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`.
