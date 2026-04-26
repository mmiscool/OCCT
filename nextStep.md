# Next Task

Current milestone: `M10. Rust-Owned Topology Construction Geometry` from `portingMilestones.md`.

## Completed Evidence

- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/brep_materialize.rs::ported_brep_edges()` now uses `ported_brep_edge_geometry_and_curve()` instead of catching `Context::edge_geometry()` failures and retrying `edge_geometry_occt()`.
- `ported_brep_edge_geometry_and_curve()` routes BRep edge geometry through the public/Rust-owned `Context::edge_geometry()` path and requires a Rust `PortedCurve` for supported `Line`, `Circle`, and `Ellipse` edges.
- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_topology.rs::single_face_edge_with_route()` now uses that same strict helper for both `FaceSurfaceRoute::Public` and `FaceSurfaceRoute::Raw`; the old raw single-face edge geometry helper was removed.
- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_prepare.rs` now routes raw BRep face geometry through `Context::face_geometry()` instead of direct `face_geometry_occt()`, so supported raw BRep face materialization cannot bypass the public Rust-owned face geometry gate.
- `rust/lean_occt/tests/brep_workflows.rs::assert_brep_edge_geometries_match_public()` now requires supported BRep edges to carry a Rust-owned `BrepEdge::ported_curve` matching the public ported curve variant.
- `rust/lean_occt/tests/document_workflows.rs::document_supports_query_driven_features()` now checks the rounded BRep has supported analytic edges and that each one carries a Rust-owned ported curve.
- `rust/lean_occt/tests/selector_workflows.rs::selectors_choose_expected_faces_and_edges()` now checks supported BRep edges and the selected BRep edge retain Rust-owned ported curves.
- Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_exact_curve_bounding_boxes`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows document_supports_query_driven_features`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows selectors_choose_expected_faces_and_edges`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`, `! rg -n 'Err\\(_\\) => context\\.edge_geometry_occt|FaceSurfaceRoute::Public => match context\\.edge_geometry|FaceSurfaceRoute::Raw => context\\.edge_geometry_occt|FaceSurfaceRoute::Raw => context\\.face_geometry_occt|raw_brep_edge_geometry_and_curve' rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval rust/lean_occt/src/lib.rs`, and `git diff --check`.

## Target

Remove or strictly narrow the remaining topology-construction raw edge geometry and endpoint reads that still bypass the Rust-owned public edge query gates for supported line/circle/ellipse edges.

## Next Bounded Cut

1. Start in `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/edge_topology.rs::root_edge_topology()` and `brep/wire_topology.rs::wire_occurrence()`.
2. Replace direct `context.edge_geometry_occt(edge_shape)?` and `context.edge_endpoints_occt(edge_shape)?` calls with a shared topology-edge query helper.
3. Have the helper use public/Rust-owned `context.edge_geometry(edge_shape)?` and `context.edge_endpoints(edge_shape)?` for supported `Line`, `Circle`, and `Ellipse` edges, returning explicit Rust-owned errors when those supported queries cannot be ported.
4. Keep raw OCCT geometry/endpoints only as a named unsupported-kind escape hatch where topology construction cannot yet identify a Rust-owned edge route.
5. Strengthen `brep_workflows` around root edge/wire topology so exercised supported topology construction cannot pass by using direct raw OCCT edge geometry/endpoints.

## Guardrails

- Read `portingMilestones.md` and `nextStep.md` at the start of the next turn before editing.
- Do not reintroduce direct OCCT helper fallbacks into public payload, geometry, sampling, topology, vertex, supported subshape, or BRep materialization wrappers narrowed under M7, M8, and M9.
- Do not reintroduce `Err(_) => context.edge_geometry_occt(edge_shape)?` in BRep edge materialization.
- Do not reintroduce `FaceSurfaceRoute::Raw => context.edge_geometry_occt(edge_shape)` or `FaceSurfaceRoute::Raw => context.face_geometry_occt(face_shape)` in BRep materialization.
- Do not reintroduce `face_bboxes_occt()`, `OffsetFaceBboxSource::OcctFaceUnion`, `offset_shape_bbox_occt()`, or `SummaryBboxSource::OffsetOcctSubshapeUnion`.
- Do not weaken `unsupported_bbox_summary_fallback_allowed()` or `unsupported_volume_summary_fallback_allowed()`.
- Keep `ModelDocument::edges()`, `ModelDocument::faces()`, `select_edge()`, and `select_face()` centered on `BrepShape`.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `rg -n 'edge_geometry_occt\\(edge_shape\\)|edge_endpoints_occt\\(edge_shape\\)' rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/edge_topology.rs rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/wire_topology.rs`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- `git diff --check`
