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

- Completed evidence: `rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs::Context::ported_offset_surface()` now routes plane, cylinder, cone, sphere, and torus offset-basis descriptors through Rust-owned `ported_offset_basis_surface_payload()` instead of direct `face_offset_basis_*_payload_occt()` helpers. `rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry/payloads.rs` now shares sampler-driven analytic payload builders so offset-basis samples are reconstructed from the offset face by subtracting the signed offset along the natural normal. Raw `Geom_OffsetSurface` faces are preserved as `SurfaceKind::Offset` by `ported_face_geometry()` instead of being reclassified by analytic probing. `Context::make_offset_surface_face()` and the matching C ABI fixture create natural trimmed offset-surface faces for plane/cylinder/cone/sphere/torus regression coverage. Natural no-loop BRep faces now compute rectangular analytic areas in Rust. `brep_workflows::ported_brep_uses_rust_owned_area_for_offset_faces`, `ported_geometry_workflows::ported_face_surface_descriptors_cover_supported_faces`, and `ported_geometry_workflows::public_offset_basis_queries_match_occt` now cover analytic offset basis descriptors, sample parity, and area parity. Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cmake --build build --target LeanOcctCAPI`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_face_surface_descriptors_cover_supported_faces`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_offset_basis_queries_match_occt`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`, and `git diff --check`.
- Active milestone: `M6. BRep Surface Payload Fallback Cleanup`.
- Next bounded cut: replace the remaining swept offset-basis branches in `rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs::Context::ported_offset_surface()` for revolution and extrusion. Reconstruct swept basis payloads and curves from offset samples plus basis geometry, then remove or strictly narrow `face_offset_basis_revolution_payload_occt()`, `face_offset_basis_extrusion_payload_occt()`, and `face_offset_basis_curve_*_payload_occt()` for exercised swept offset surfaces.
- Verification:
  - `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
  - `cmake --build build --target LeanOcctCAPI`
  - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces`
  - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_face_surface_descriptors_cover_supported_faces`
  - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_offset_basis_queries_match_occt`
  - `cargo check --manifest-path rust/lean_occt/Cargo.toml`
  - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
  - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`
  - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows`
  - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows`
  - `cargo test --manifest-path rust/lean_occt/Cargo.toml`
  - `git diff --check`

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

Status: complete on 2026-04-26. `PortedCurve::from_context_with_ported_payloads()` no longer calls the OCCT line, circle, or ellipse payload helpers; public BRep materialization paths in `brep_materialize.rs` and `face_topology.rs` no longer swallow extraction errors into `PortedCurve::from_context_with_geometry()`; planar root-wire reconstruction in `face_snapshot.rs` no longer rescues through the raw curve builder; `append_root_edge_sample_points()` in `swept_face.rs` uses Rust-owned ported payload extraction before falling back to OCCT point sampling only for unsupported curves; and the raw `FaceSurfaceRoute::Raw` edge path now uses the same Rust-owned curve payload extraction as the public path. `PortedCurve::from_context_with_geometry()` itself no longer calls `edge_line_payload_occt()`, `edge_circle_payload_occt()`, or `edge_ellipse_payload_occt()`.

Definition of done: exercised BRep edges with line, circle, and ellipse geometry populate `BrepEdge::ported_curve` through Rust-owned extraction, edge lengths and sampled face loops remain stable, and OCCT payload helpers are no longer the normal rescue path for supported public-route BRep edge materialization.

Bounded tasks: complete. The public-route `PortedCurve::from_context_with_ported_payloads()`, `ported_brep_edges()`, `single_face_edge_with_route()`, root-edge `face_snapshot.rs`/`swept_face.rs`, raw `FaceSurfaceRoute::Raw`, and generic `PortedCurve::from_context_with_geometry()` cuts are complete.

Verification: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`.

## M6. BRep Surface Payload Fallback Cleanup

Outcome: BRep face materialization and swept/offset face descriptor paths stop using `PortedSurface::from_context_with_geometry()` as an OCCT-payload rescue after Rust surface-payload extraction has identified an exercised plane, cylinder, cone, sphere, or torus face path.

Status: active. The BRep analytic face materialization cut is complete: `face_prepare.rs` now uses `PortedSurface::from_context_with_ported_payloads()`, and `PortedSurface::from_context_with_geometry()` no longer rescues plane, cylinder, cone, sphere, or torus payload extraction through `face_*_payload_occt()`. The analytic offset-basis cut is also complete for plane, cylinder, cone, sphere, and torus: `Context::ported_offset_surface()` now builds those basis descriptors through Rust-owned reconstructed samples and no longer retries the direct `face_offset_basis_*_payload_occt()` helpers for those analytic branches. The remaining surface descriptor fallback inside this milestone is the swept offset-basis family for revolution and extrusion payloads and basis curves.

Definition of done: exercised BRep faces with plane, cylinder, cone, sphere, and torus geometry populate `BrepFace::ported_surface` and `BrepFace::ported_face_surface` through Rust-owned extraction, exercised offset descriptors build analytic basis surfaces without direct `face_offset_basis_*_payload_occt()` helpers, face samples and areas remain stable, and OCCT payload helpers are no longer the normal rescue path for supported BRep face materialization or offset basis descriptor construction.

Bounded tasks: analytic BRep face materialization and analytic offset-basis descriptors are complete. Next replace the swept offset-basis branches in `Context::ported_offset_surface()` so exercised revolution and extrusion offset basis descriptors are Rust-owned instead of built through `face_offset_basis_revolution_payload_occt()`, `face_offset_basis_extrusion_payload_occt()`, and `face_offset_basis_curve_*_payload_occt()`, then strengthen `ported_geometry_workflows` around swept offset basis payload and curve parity.

Verification: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cmake --build build --target LeanOcctCAPI`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_face_surface_descriptors_cover_supported_faces`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_offset_basis_queries_match_occt`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`, `git diff --check`.
