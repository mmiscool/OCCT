# Next Task

Current milestone: `M35. Rust-Owned Public Root Edge Support Classification` from `portingMilestones.md`.

## Completed Evidence

- `M34. Rust-Owned Root Edge Topology Bootstrap Endpoint Seed` is complete.
- `root_edge_topology_bootstrap_seed()` no longer calls `edge_endpoints_occt()`.
- Supported root line/circle/ellipse topology bootstrap now collects endpoint vertex handles with `root_edge_vertex_shape_occt(shape, 0/1)`, resolves their positions through public `Context::vertex_point()`, and carries those positions as the bootstrap `EdgeEndpoints`.
- Closed root edges whose start and end vertex handles are the same still produce one topology vertex with both endpoint indices set to `0`.
- `public_root_edge_endpoints_are_topology_backed` now verifies public root-edge vertex handles resolve to the loaded topology vertex positions before checking public/ported/OCCT endpoint parity.
- Source guards show `edge_endpoints_occt(` is absent from both `brep/topology.rs` and `brep/shape_queries.rs`; the explicit `Context::edge_endpoints_occt()` oracle remains available outside those ported paths.
- Full verification passed with `cargo test`.

## Target

Remove the public-query support classifier's raw geometry read:

`ported_edge_endpoints() -> root_edge_endpoint_topology_support() -> edge_geometry_occt()`

After M34, endpoint data is topology-backed, but `ported_edge_endpoints()` still performs a direct `edge_geometry_occt()` read in `shape_queries.rs` solely to decide whether a root edge is a supported line/circle/ellipse before loading topology. Move that support decision into the root topology loader, and make unsupported root edge kinds stop before the generic raw root inventory loader.

## Next Bounded Cut

1. Read `portingMilestones.md` and `nextStep.md` before editing.
2. Remove `RootEdgeEndpointTopologySupport` and the direct `edge_geometry_occt()` classifier from `shape_queries.rs`.
3. Teach the root topology loader to distinguish not-root-edge, supported-root-edge, and unsupported-root-edge so unsupported root edges return `Ok(None)` instead of falling through to the generic raw inventory sweep.
4. Keep the remaining `edge_geometry_occt()` read isolated to the topology-owned root-edge bootstrap until the following geometry-bootstrap cut.
5. Keep explicit `Context::edge_endpoints_occt()` and `Context::edge_geometry_occt()` available as oracle APIs.
6. Strengthen source guards so the `ported_edge_endpoints()` function body contains no `edge_geometry_occt(`, `edge_endpoints_occt(`, or `RootEdgeEndpointTopologySupport`.
7. Update both control files with completed evidence, active milestone, next bounded cut, and exact verification commands.

## Guardrails

- Do not reintroduce `edge_endpoints_occt()` into `brep/topology.rs` or `brep/shape_queries.rs`.
- Do not broaden the generic raw root topology fallback for unsupported root edge kinds while removing the public classifier.
- Do not route root-edge bootstrap through public `edge_geometry()` yet; it recurses through public endpoints/topology today.
- Preserve line/circle/ellipse root endpoint and topology behavior for public, ported, and OCCT parity tests.

## Verification

- `(cd rust/lean_occt && cargo fmt)`
- `cmake --build build --target LeanOcctCAPI`
- `(cd rust/lean_occt && cargo check)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows public_root_edge_endpoints_are_topology_backed -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows root_edge_endpoints_and_topology_use_ported_seed -- --nocapture)`
- `! perl -0ne 'print $1 if /(pub\(super\) fn ported_edge_endpoints[\s\S]*?)\npub\(super\) fn ported_subshape_count/' rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/shape_queries.rs | rg -n 'edge_geometry_occt\(|edge_endpoints_occt\(|RootEdgeEndpointTopologySupport'`
- `rg -n 'RootEdgeTopologyInventory|UnsupportedRootEdge|load_root_edge_topology_snapshot' rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/shape_queries.rs`
- `(cd rust/lean_occt && cargo test)`
- `git diff --check`
