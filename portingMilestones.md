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

- Completed evidence: `rust/lean_occt/src/lib.rs` now routes public analytic face payload wrappers (`face_plane_payload()`, `face_cylinder_payload()`, `face_cone_payload()`, `face_sphere_payload()`, and `face_torus_payload()`) through `ported_analytic_face_surface_payload()`. That path accepts an existing `ported_face_surface()` descriptor, otherwise uses raw OCCT face geometry only to identify the supported surface kind/bounds and then requires `PortedSurface::from_context_with_geometry()` to produce the Rust-owned payload; the public wrappers no longer call direct `face_*_payload_occt()` helpers after descriptor extraction returns `None`. The explicit `face_*_payload_occt()` methods remain available as opt-in oracle APIs. `rust/lean_occt/tests/ported_geometry_workflows.rs::public_analytic_curve_and_surface_payload_queries_match_occt` now first requires matching Rust descriptors for exercised Plane, Cylinder, Cone, Sphere, and Torus faces, compares each public payload to that descriptor, and only then compares against the explicit OCCT oracle. Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `! rg -n 'None => self.face_(plane|cylinder|cone|sphere|torus)_payload_occt|face_(plane|cylinder|cone|sphere|torus)_payload_occt\\(shape\\)' rust/lean_occt/src/lib.rs`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_analytic_curve_and_surface_payload_queries_match_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_face_surface_descriptors_cover_supported_faces -- --nocapture`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`, and `git diff --check`.
- Active milestone: `M7. Public Payload Fallback Narrowing`.
- Next bounded cut: split the public swept surface payload methods in `rust/lean_occt/src/lib.rs` (`face_revolution_payload()` and `face_extrusion_payload()`) so supported swept faces do not call the direct `face_*_payload_occt()` helper when `ported_face_surface_descriptor()` returns `None`; keep the explicit swept `*_payload_occt()` APIs available as opt-in oracles.
- Verification:
  - `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
  - `cargo check --manifest-path rust/lean_occt/Cargo.toml`
  - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_swept_and_offset_payload_queries_match_occt -- --nocapture`
  - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_analytic_curve_and_surface_payload_queries_match_occt -- --nocapture`
  - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_face_surface_descriptors_cover_supported_faces -- --nocapture`
  - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`
  - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
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

Status: complete on 2026-04-26. The BRep analytic face materialization cut is complete: `face_prepare.rs` now uses `PortedSurface::from_context_with_ported_payloads()`, and `PortedSurface::from_context_with_geometry()` no longer rescues plane, cylinder, cone, sphere, or torus payload extraction through `face_*_payload_occt()`. The analytic offset-basis cut is complete for plane, cylinder, cone, sphere, and torus. The swept offset-basis cut is complete for exercised revolution and extrusion faces: `Context::ported_offset_surface()` now builds swept basis descriptors through Rust-owned reconstructed samples and no longer retries direct `face_offset_basis_revolution_payload_occt()`, `face_offset_basis_extrusion_payload_occt()`, or `face_offset_basis_curve_*_payload_occt()` helpers for those branches. The non-offset swept BRep cut is complete for exercised extrusion and revolution faces: `brep/swept_face.rs` derives swept payloads from topology plus face samples and no longer calls `face_extrusion_payload_occt()` or `face_revolution_payload_occt()`. The planar multi-wire root-face snapshot cut is complete: `brep/face_snapshot.rs` derives the plane with `PortedSurface::from_context_with_ported_payloads()` and no longer accepts `face_plane_payload_occt()` after Rust extraction fails. The analytic face-geometry classification cut is also complete: `ported_geometry.rs::ported_face_geometry()` no longer uses direct analytic `face_*_payload_occt()` helpers as a Rust extraction rescue for plane, cylinder, cone, sphere, or torus.

Definition of done: exercised BRep faces with plane, cylinder, cone, sphere, torus, extrusion, and revolution geometry populate `BrepFace::ported_surface` or `BrepFace::ported_face_surface` through Rust-owned extraction, exercised offset descriptors build analytic and swept basis surfaces without direct `face_offset_basis_*_payload_occt()` helpers, face samples and areas remain stable, and OCCT payload helpers are no longer the normal rescue path for supported BRep face materialization or offset basis descriptor construction.

Bounded tasks: complete. Analytic BRep face materialization, analytic offset-basis descriptors, swept offset-basis descriptors, non-offset swept BRep descriptors, planar multi-wire root-face snapshot reconstruction, and analytic `ported_face_geometry()` classification no longer rescue supported surface-payload extraction through direct OCCT payload helpers.

Verification: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_simple_single_face_shapes -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_face_surface_descriptors_cover_supported_faces -- --nocapture`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cmake --build build --target LeanOcctCAPI`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`, `git diff --check`.

## M7. Public Payload Fallback Narrowing

Outcome: supported public payload query methods stop using direct OCCT payload helpers as the automatic rescue when Rust descriptor extraction returns `None`; the explicit `*_payload_occt()` methods remain available as opt-in parity/oracle APIs.

Status: active. The analytic public face payload family is narrowed: `face_plane_payload()`, `face_cylinder_payload()`, `face_cone_payload()`, `face_sphere_payload()`, and `face_torus_payload()` no longer call direct `face_*_payload_occt()` helpers when Rust descriptor extraction returns `None`; they require Rust `PortedSurface` extraction for supported analytic kinds and fail explicitly if it cannot produce the payload. The remaining direct public face payload helper fallbacks in `rust/lean_occt/src/lib.rs` are the swept wrappers (`face_revolution_payload()`, `face_extrusion_payload()`), top-level offset payload, offset-basis analytic/swept wrappers, and offset-basis curve payload wrappers; tests still use explicit OCCT helpers only as parity oracles.

Definition of done: for supported analytic, swept, and offset descriptors, a Rust extraction failure or kind mismatch produces an explicit Rust error/`None` distinction instead of silently returning an OCCT payload; unsupported shapes may still use explicit raw OCCT APIs only when the caller asks for them through `*_payload_occt()`.

Bounded tasks: next split swept face payload queries in `lib.rs`, require `ported_face_surface_descriptor()`/raw swept geometry to produce a matching Rust `PortedSweptSurface` payload before public `face_revolution_payload()` or `face_extrusion_payload()` can succeed, strengthen `public_swept_and_offset_payload_queries_match_occt` to prove descriptor routing before OCCT oracle comparison, then move to top-level offset and offset-basis public payload methods.

Verification: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_swept_and_offset_payload_queries_match_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_analytic_curve_and_surface_payload_queries_match_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_face_surface_descriptors_cover_supported_faces -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`, `git diff --check`.
