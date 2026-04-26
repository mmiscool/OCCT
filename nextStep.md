# Next Task

Current milestone: `M9. BRep Materialization Fallback Narrowing` from `portingMilestones.md`.

## Completed Evidence

- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs::LoadedPortedTopology` now carries the root vertex, edge, wire, face, and prepared shell handles captured while loading Rust topology.
- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/shape_queries.rs` serves `ported_subshape_count()`, `ported_subshape()`, and `ported_subshapes()` for `Vertex`, `Edge`, `Wire`, `Face`, and `Shell` from that loaded Rust topology inventory instead of rematerializing the same public inventory through generic OCCT subshape helpers.
- `rust/lean_occt/src/lib.rs` public `vertex_point()` and `topology()` now require Rust `ported_vertex_point()`/`ported_topology()` results and return explicit Rust-owned unsupported errors when the ported topology path cannot cover the shape.
- `rust/lean_occt/src/lib.rs` public `subshape_count()`, `subshape()`, and `subshapes()` now require the Rust topology inventory for `Vertex`, `Edge`, `Wire`, `Face`, and `Shell`; direct OCCT subshape fallbacks remain only for unsupported shape kinds or explicit raw `*_occt()` parity/oracle APIs.
- `rust/lean_occt/tests/brep_workflows.rs` compares public topology and topology-backed public subshape traversal against Rust topology before explicit OCCT oracle checks, including shell materialization for the offset-solid workflow.
- `rust/lean_occt/tests/document_workflows.rs` and `selector_workflows.rs` now exercise public topology/subshape traversal through higher-level document and selector paths and compare it against `ported_topology()`.
- `rust/lean_occt/tests/ported_geometry_workflows.rs::ported_vertex_points_match_occt` now requires public vertex points to match the Rust-owned `ported_vertex_point()` result before comparing with the explicit OCCT oracle.
- Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_face_free_shapes -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_simple_multi_face_solids -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_vertex_points_match_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`, `rg -n 'rust_owned_topology_subshape_query_required|unsupported_ported_topology_query_error|ported_subshape_count' rust/lean_occt/src/lib.rs rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep.rs rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/shape_queries.rs`, and `git diff --check`.

## Target

Remove or strictly narrow the remaining BRep materialization fallback that catches Rust-owned public edge geometry failures and silently calls direct OCCT edge geometry helpers for supported line/circle/ellipse edges.

## Next Bounded Cut

1. Start in `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/brep_materialize.rs::ported_brep_edges()` and `brep/face_topology.rs::single_face_edge_with_route()`.
2. Replace `Err(_) => context.edge_geometry_occt(edge_shape)?` with a Rust-owned supported-edge helper that returns ported edge geometry for line/circle/ellipse edges or an explicit unsupported-kind error, without swallowing a Rust extraction failure into OCCT.
3. Keep explicit raw `edge_geometry_occt()` available as an oracle API and only use unsupported-kind OCCT escape hatches where the BRep route cannot identify a supported ported edge.
4. Strengthen `brep_workflows`, `document_workflows`, and `selector_workflows` so exercised BRep edge materialization proves `BrepEdge::geometry` and `BrepEdge::ported_curve` came through the Rust-owned path before any explicit OCCT oracle comparison.

## Guardrails

- Read `portingMilestones.md` and `nextStep.md` at the start of the next turn before editing.
- Do not reintroduce direct OCCT helper fallbacks into public payload, geometry, sampling, topology, vertex, or supported subshape wrappers narrowed under M7 and M8.
- Do not reintroduce `face_bboxes_occt()`, `OffsetFaceBboxSource::OcctFaceUnion`, `offset_shape_bbox_occt()`, or `SummaryBboxSource::OffsetOcctSubshapeUnion`.
- Do not weaken `unsupported_bbox_summary_fallback_allowed()` or `unsupported_volume_summary_fallback_allowed()`.
- Do not reintroduce OCCT line, circle, or ellipse payload helper rescues into `PortedCurve::from_context_with_geometry()`, `ported_edge_geometry()`, or BRep edge materialization.
- Do not reintroduce OCCT plane, cylinder, cone, sphere, or torus payload helper rescues into `PortedSurface::from_context_with_geometry()`, `ported_face_geometry()`, BRep face materialization, or planar face snapshot reconstruction.
- Do not reintroduce `face_extrusion_payload_occt()` or `face_revolution_payload_occt()` inside `brep/swept_face.rs` or the public swept payload wrappers.
- Do not reintroduce direct `face_offset_payload_occt()`, `face_offset_basis_*_occt()`, or `face_offset_basis_curve_*_occt()` fallbacks inside public wrappers narrowed under M7.
- Keep `ModelDocument::edges()`, `ModelDocument::faces()`, `select_edge()`, and `select_face()` centered on `BrepShape`.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `rg -n 'Err\\(_\\) => context\\.edge_geometry_occt|FaceSurfaceRoute::Raw => context\\.(edge|face)_geometry_occt' rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- `git diff --check`
