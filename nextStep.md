# Next Task

Current milestone: `M11. Rust-Owned Single-Face Topology Snapshot` from `portingMilestones.md`.

## Completed Evidence

- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/edge_topology.rs::root_edge_topology()` now uses `topology_edge_query()` instead of direct `context.edge_geometry_occt(edge_shape)?` and `context.edge_endpoints_occt(edge_shape)?`.
- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/wire_topology.rs::wire_occurrence()` now uses the same helper, so wire occurrence matching obtains geometry and endpoints through public/Rust-owned `Context::edge_geometry()` and `Context::edge_endpoints()`.
- `wire_topology.rs::root_wire_topology()` now tries Rust occurrence reconstruction before the raw snapshot route; raw `context.topology_occt(&prepared_wire_shape.wire_shape)?` remains only after occurrence reconstruction cannot cover the wire, including duplicate-edge-occurrence wires that the current unique edge inventory cannot represent.
- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/shape_queries.rs::ported_edge_endpoints()` now uses a root-edge topology seed to break root-edge geometry/endpoints recursion without calling the direct endpoint C helper.
- `rust/lean_occt/tests/brep_workflows.rs::assert_topology_edges_match_public_queries()` now checks topology edge endpoints and wire vertex chains against public edge queries, and the exercised topology tests require supported `Line`, `Circle`, or `Ellipse` edge coverage where expected.
- Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_face_free_shapes -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`, `! rg -n 'edge_geometry_occt\\(edge_shape\\)|edge_endpoints_occt\\(edge_shape\\)|edge_geometry_occt\\(local_edge_shape\\)|edge_endpoints_occt\\(local_edge_shape\\)' rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/edge_topology.rs rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/wire_topology.rs`, and `git diff --check`.

## Target

Remove or strictly narrow the active single-face topology snapshot fallback that can still materialize supported single-face BRep topology through raw `context.topology_occt(face_shape)?` after `context.ported_topology(face_shape)?` returns `None`.

## Next Bounded Cut

1. Start in `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_topology.rs::single_face_topology_snapshot()`.
2. Replace the implicit `None => context.topology_occt(face_shape)?` branch with a Rust topology requirement for exercised supported single-face descriptors.
3. Preserve raw OCCT topology only behind an explicit unsupported or ambiguous-topology guard where ported topology cannot yet identify the face route.
4. Strengthen `brep_workflows` for simple single-face and holed planar fixtures so supported single-face topology materialization fails if it can only pass through the raw snapshot fallback.

## Guardrails

- Read `portingMilestones.md` and `nextStep.md` at the start of the next turn before editing.
- Do not reintroduce direct OCCT helper fallbacks into public payload, geometry, sampling, topology, vertex, supported subshape, BRep materialization, or topology-construction wrappers narrowed under M7 through M10.
- Do not reintroduce `context.edge_geometry_occt(edge_shape)?` or `context.edge_endpoints_occt(edge_shape)?` into `edge_topology.rs::root_edge_topology()` or `wire_topology.rs::wire_occurrence()`.
- Keep the remaining raw wire topology snapshot fallback explicitly limited to duplicate-edge-occurrence wires or occurrence reconstruction misses until the wire inventory can represent repeated edge occurrences.
- Do not reintroduce `face_bboxes_occt()`, `OffsetFaceBboxSource::OcctFaceUnion`, `offset_shape_bbox_occt()`, or `SummaryBboxSource::OffsetOcctSubshapeUnion`.
- Do not weaken `unsupported_bbox_summary_fallback_allowed()` or `unsupported_volume_summary_fallback_allowed()`.
- Keep `ModelDocument::edges()`, `ModelDocument::faces()`, `select_edge()`, and `select_face()` centered on `BrepShape`.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_simple_single_face_shapes -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- `! rg -n 'None => context\\.topology_occt\\(face_shape\\)\\?' rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_topology.rs`
- `git diff --check`
