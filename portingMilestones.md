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

- Completed evidence: `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/brep_materialize.rs::ported_brep_edges()` now routes every BRep edge through `ported_brep_edge_geometry_and_curve()`, which calls the public/Rust-owned `Context::edge_geometry()` path and requires a `PortedCurve` for supported `Line`, `Circle`, and `Ellipse` edges instead of swallowing failures into `edge_geometry_occt()`. `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_topology.rs::single_face_edge_with_route()` now uses the same strict helper for both public and raw single-face routes, and `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_prepare.rs` routes raw BRep face geometry through `Context::face_geometry()` instead of direct `face_geometry_occt()`. `rust/lean_occt/tests/brep_workflows.rs`, `document_workflows.rs`, and `selector_workflows.rs` now assert exercised supported BRep edges carry Rust-owned `BrepEdge::ported_curve` data.
- Active milestone: `M10. Rust-Owned Topology Construction Geometry`.
- Next bounded cut: replace the topology-construction raw edge geometry/endpoints reads in `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/edge_topology.rs::root_edge_topology()` and `brep/wire_topology.rs::wire_occurrence()` with a Rust-owned helper that uses public `edge_geometry()` and `edge_endpoints()` for supported line/circle/ellipse edges, while preserving explicit unsupported-kind OCCT oracle access only where topology matching truly cannot be ported yet.
- Verification:
  - `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
  - `cargo check --manifest-path rust/lean_occt/Cargo.toml`
  - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology`
  - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_exact_curve_bounding_boxes`
  - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces`
  - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows document_supports_query_driven_features`
  - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows selectors_choose_expected_faces_and_edges`
  - `cargo test --manifest-path rust/lean_occt/Cargo.toml`
  - `! rg -n 'Err\\(_\\) => context\\.edge_geometry_occt|FaceSurfaceRoute::Public => match context\\.edge_geometry|FaceSurfaceRoute::Raw => context\\.edge_geometry_occt|FaceSurfaceRoute::Raw => context\\.face_geometry_occt|raw_brep_edge_geometry_and_curve' rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval rust/lean_occt/src/lib.rs`
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

Status: complete on 2026-04-26. `ModelDocument::edges()`, `ModelDocument::faces()`, `select_edge()`, `select_face()`, and high-level inspection consume `BrepShape`, and the remaining public `Context::subshape()`/`Context::subshapes()` bridge through `shape_queries.rs` now requires `ported_topology()` before it will validate counts and return the loaded Rust topology handles. `Context::subshape_count()` also uses Rust topology for supported face, wire, edge, vertex, and shell counts, with direct OCCT count fallback kept only for unsupported shape kinds or explicit raw oracle APIs.

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

Status: complete on 2026-04-26. The analytic public face payload family is narrowed: `face_plane_payload()`, `face_cylinder_payload()`, `face_cone_payload()`, `face_sphere_payload()`, and `face_torus_payload()` no longer call direct `face_*_payload_occt()` helpers when Rust descriptor extraction returns `None`; they require Rust `PortedSurface` extraction for supported analytic kinds and fail explicitly if it cannot produce the payload. The swept public face payload family is narrowed: `face_revolution_payload()` and `face_extrusion_payload()` now require Rust `PortedFaceSurface::Swept` extraction for supported swept kinds and no longer rescue through direct swept OCCT payload helpers. The top-level public offset payload is narrowed: `face_offset_payload()` now requires Rust `PortedFaceSurface::Offset` extraction for supported offset faces and no longer rescues through direct `face_offset_payload_occt()`. The offset-basis face-surface family is narrowed: `face_offset_basis_geometry()` and the analytic/swept basis surface payload wrappers now require `ported_offset_face_surface_payload()` and no longer rescue through direct `face_offset_basis_*_occt()` helpers. The offset-basis curve family is narrowed: public offset-basis curve geometry and line/circle/ellipse payload wrappers now require `ported_offset_face_surface_payload()` and no longer rescue through direct `face_offset_basis_curve_*_occt()` helpers. The public edge payload family is narrowed: `edge_line_payload()`, `edge_circle_payload()`, and `edge_ellipse_payload()` now require `PortedCurve::from_context_with_ported_payloads()` and no longer rescue through direct `edge_*_payload_occt()` helpers. A full payload-fallback scan found no remaining `None => self.*payload_occt(shape)` branches in `rust/lean_occt/src/lib.rs`; tests use explicit OCCT helpers only as parity oracles.

Definition of done: for supported analytic, swept, and offset descriptors, a Rust extraction failure or kind mismatch produces an explicit Rust error/`None` distinction instead of silently returning an OCCT payload; unsupported shapes may still use explicit raw OCCT APIs only when the caller asks for them through `*_payload_occt()`.

Bounded tasks: complete. The public analytic face, swept face, offset face, offset-basis face, offset-basis curve, and edge payload families no longer silently rescue through direct OCCT payload helpers after Rust descriptor extraction returns `None`.

Verification: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `! rg -n 'None => self.edge_(line|circle|ellipse)_payload_occt\\(shape\\)|edge_(line|circle|ellipse)_payload_occt\\(shape\\)' rust/lean_occt/src/lib.rs`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_analytic_curve_and_surface_payload_queries_match_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_curve_sampling_matches_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`, `git diff --check`.

## M8. Public Geometry and Sampling Fallback Narrowing

Outcome: supported public geometry, sampling, topology, and subshape query methods stop using direct OCCT helpers as the automatic rescue when Rust descriptors or topology can identify the shape; explicit raw OCCT methods remain available only as opt-in parity/oracle APIs or unsupported-shape escape hatches.

Status: complete on 2026-04-26. The public edge geometry/sampling family is narrowed: `edge_geometry()`, `edge_endpoints()`, `edge_sample()`, and `edge_sample_at_parameter()` now require Rust topology/curve extraction for supported line/circle/ellipse edges and only fall back to direct OCCT helpers for non-ported curve kinds. `ported_edge_geometry()` also no longer uses direct line/circle/ellipse payload helpers as a reconstruction rescue. The public face geometry/sampling family is narrowed: `face_geometry()`, `face_sample()`, and `face_sample_normalized()` now require Rust `PortedFaceSurface` extraction for supported analytic, swept, and offset faces and only fall back to direct OCCT helpers for unsupported raw surface kinds. `ported_face_geometry()` validates swept and offset descriptors before reporting ported geometry and retains Rust analytic recovery for raw BSpline/Bezier/unknown faces that sample as supported analytic surfaces. The public vertex/topology/subshape family is narrowed: `vertex_point()` and `topology()` now require Rust topology extraction, while `subshape_count()`, `subshape()`, and `subshapes()` serve `Vertex`, `Edge`, `Wire`, `Face`, and `Shell` from the loaded Rust topology inventory and fail explicitly when that inventory is unavailable.

Definition of done: for supported analytic, swept, offset, and BRep topology descriptors exercised by `ported_geometry_workflows` and `brep_workflows`, public geometry/sampling/topology queries either return Rust-owned results or fail explicitly when Rust extraction cannot cover the supported shape; they do not silently return direct OCCT helper results after descriptor extraction returns `None`.

Bounded tasks: complete. Public edge geometry/sampling, public face geometry/sampling, public vertex/topology, and public supported subshape traversal now require Rust-owned extraction before succeeding; explicit raw OCCT methods remain available as opt-in parity/oracle APIs or unsupported-kind escape hatches.

Verification: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_face_free_shapes -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_simple_multi_face_solids -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_vertex_points_match_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`, `rg -n 'rust_owned_topology_subshape_query_required|unsupported_ported_topology_query_error|ported_subshape_count' rust/lean_occt/src/lib.rs rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep.rs rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/shape_queries.rs`, `git diff --check`.

## M9. BRep Materialization Fallback Narrowing

Outcome: BRep edge and face materialization stop catching Rust-owned public query failures and silently rescuing through direct OCCT geometry helpers for supported curve and surface kinds.

Status: complete on 2026-04-26. `ported_brep_edges()` in `brep_materialize.rs` no longer catches `Context::edge_geometry()` failures and retries `edge_geometry_occt()`. BRep edge materialization now uses `ported_brep_edge_geometry_and_curve()`, which requires public/Rust-owned geometry and a Rust `PortedCurve` for supported line, circle, and ellipse edges. `single_face_edge_with_route()` in `face_topology.rs` uses that same helper for both public and raw routes, so raw single-face BRep edge construction no longer has a separate direct OCCT edge-geometry bypass. `face_prepare.rs` also routes raw BRep face geometry through public `face_geometry()`, preserving explicit unsupported-kind behavior while blocking supported face materialization from bypassing the M8 Rust-owned face query guard.

Definition of done: exercised BRep edge materialization for line, circle, and ellipse edges obtains geometry and ported curves through Rust-owned extraction, unsupported curve kinds remain explicit, and BRep/document/selector workflow tests fail if a supported edge can only be materialized by direct `edge_geometry_occt()` rescue.

Bounded tasks: complete. The public and raw BRep edge materialization routes share the strict Rust-owned edge helper, the old `Err(_) => context.edge_geometry_occt(edge_shape)?` rescues are removed, and raw BRep face preparation now enters through `Context::face_geometry()` instead of direct `face_geometry_occt()`.

Verification: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_exact_curve_bounding_boxes`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows document_supports_query_driven_features`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows selectors_choose_expected_faces_and_edges`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`, `! rg -n 'Err\\(_\\) => context\\.edge_geometry_occt|FaceSurfaceRoute::Public => match context\\.edge_geometry|FaceSurfaceRoute::Raw => context\\.edge_geometry_occt|FaceSurfaceRoute::Raw => context\\.face_geometry_occt|raw_brep_edge_geometry_and_curve' rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval rust/lean_occt/src/lib.rs`, `git diff --check`.

## M10. Rust-Owned Topology Construction Geometry

Outcome: topology construction for exercised BRep roots stops using direct OCCT edge geometry and endpoint reads for supported line, circle, and ellipse edges when building root edge and wire topology.

Status: active. `edge_topology.rs::root_edge_topology()` and `wire_topology.rs::wire_occurrence()` still call `edge_geometry_occt()` and `edge_endpoints_occt()` directly while matching root edges and wire occurrences. Those calls are topology-construction reads rather than public materialization fallbacks, but they can still bypass the Rust-owned supported edge geometry and endpoint gates narrowed under M8 and M9.

Definition of done: exercised root edge topology and wire occurrence matching use public/Rust-owned `edge_geometry()` and `edge_endpoints()` for supported line/circle/ellipse edges; unsupported curve kinds remain explicit; BRep topology and workflow tests fail if supported edge topology can only be reconstructed by direct raw OCCT edge geometry/endpoints.

Bounded tasks: introduce a small topology-edge query helper shared by `root_edge_topology()` and `wire_occurrence()`, route supported edge geometry/endpoints through public Rust-owned queries, keep unsupported raw OCCT reads isolated and named, and strengthen BRep workflow assertions around root edge/wire topology counts and supported edge ported curves.

Verification: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `rg -n 'edge_geometry_occt\\(edge_shape\\)|edge_endpoints_occt\\(edge_shape\\)' rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/edge_topology.rs rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/wire_topology.rs`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`.
