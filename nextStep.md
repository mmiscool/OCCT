# Next Task

Current milestone: `M34. Rust-Owned Root Edge Topology Bootstrap Endpoint Seed` from `portingMilestones.md`.

## Completed Evidence

- `M33. Rust-Owned Public Root Edge Endpoint Queries` is complete.
- `ported_edge_endpoints()` no longer calls the raw endpoint seed or `edge_endpoints_occt()`.
- Supported root line/circle/ellipse edge endpoint queries now classify unsupported root edge kinds up front, call `load_ported_topology()`, and derive endpoints from the loaded topology edge's start/end vertex positions.
- Unsupported root edge kinds still return `Ok(None)` from the Rust public endpoint path before generic topology loading, preserving explicit OCCT oracle behavior.
- The remaining direct endpoint read is isolated inside `root_edge_topology_bootstrap_seed()` in `brep/topology.rs`, where root-edge topology construction still needs a bootstrap source.
- `public_root_edge_endpoints_are_topology_backed` covers line, circle, and ellipse root edges and verifies public, ported, and OCCT endpoints against loaded topology vertices.
- Source guards prove the public `ported_edge_endpoints()` function contains no `edge_endpoints_occt(` call and does contain the loaded-topology vertex path.

## Target

Remove the topology-bootstrap direct endpoint seed:

`load_root_edge_topology_snapshot() -> root_edge_topology_bootstrap_seed() -> root edge vertex handles / vertex seed positions`

After M33, public endpoint queries are topology-backed, but root edge topology itself still calls `context.edge_endpoints_occt(shape)?` in `root_edge_topology_bootstrap_seed()`. The next cut should remove that direct OCCT endpoint read from ported topology construction while keeping explicit `Context::edge_endpoints_occt()` available as an oracle API.

## Next Bounded Cut

1. Read `portingMilestones.md` and `nextStep.md` before editing.
2. Change the root-edge bootstrap so it collects start/end vertex handles first with the existing root-edge vertex-handle route.
3. Derive endpoint positions from the root vertex topology/seed path instead of calling `edge_endpoints_occt()`.
4. Preserve closed-edge behavior where start and end vertex handles are the same.
5. Keep unsupported root edge kinds returning `Ok(None)` from the Rust topology path.
6. Strengthen source guards so `edge_endpoints_occt(` is absent from `brep/topology.rs` and `brep/shape_queries.rs`.
7. Update both control files with completed evidence, active milestone, next bounded cut, and exact verification commands.

## Guardrails

- Do not remove explicit `Context::edge_endpoints_occt()`; it remains an oracle and unsupported fallback API.
- Do not reintroduce public endpoint reads from `edge_endpoints_occt()` or the bootstrap helper.
- Keep the M33 public endpoint test and existing root-edge topology test green for line, circle, and ellipse roots.
- Do not broaden the generic raw root topology fallback while replacing the root-edge bootstrap endpoint source.

## Verification

- `(cd rust/lean_occt && cargo fmt)`
- `cmake --build build --target LeanOcctCAPI`
- `(cd rust/lean_occt && cargo check)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows public_root_edge_endpoints_are_topology_backed -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows root_edge_endpoints_and_topology_use_ported_seed -- --nocapture)`
- `! rg -n 'edge_endpoints_occt\(' rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/shape_queries.rs`
- `rg -n 'root_edge_topology_bootstrap_seed|root_edge_vertex_shape_occt|vertex_point' rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs`
- `(cd rust/lean_occt && cargo test)`
- `git diff --check`
