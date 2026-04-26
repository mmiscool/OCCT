# Next Task

Current milestone: `M36. Rust-Owned Root Edge Geometry Bootstrap Seed` from `portingMilestones.md`.

## Completed Evidence

- `M35. Rust-Owned Public Root Edge Support Classification` is complete.
- `RootEdgeEndpointTopologySupport` and `root_edge_endpoint_topology_support()` were removed from `shape_queries.rs`.
- `ported_edge_endpoints()` now loads `LoadedPortedTopology` directly and derives endpoint positions from the loaded topology edge's start/end vertex indices, with no direct `edge_geometry_occt()` or `edge_endpoints_occt()` classifier in the public query path.
- `topology.rs` now owns root-edge support classification through `RootEdgeTopologyInventory::{Supported, UnsupportedRootEdge, NotRootEdge}`.
- `load_root_topology_snapshot()` returns `Ok(None)` for `UnsupportedRootEdge` before the generic raw root inventory sweep can rebuild unsupported root edge topology.
- `unsupported_root_edge_does_not_use_generic_raw_topology_inventory` covers a helix child edge as an unsupported root edge: `ported_topology()` and `ported_edge_endpoints()` return `None`, while public endpoints still match the explicit OCCT oracle fallback.
- Source guards show the `ported_edge_endpoints()` function body contains no `edge_geometry_occt(`, `edge_endpoints_occt(`, or `RootEdgeEndpointTopologySupport`; `edge_endpoints_occt(` remains absent from both `brep/topology.rs` and `brep/shape_queries.rs`.
- Full verification passed with `cargo test`.

## Target

Remove the remaining topology-owned raw geometry seed:

`load_root_edge_topology_inventory() -> root_edge_topology_bootstrap_seed() -> edge_geometry_occt()`

After M35, root-edge support classification is in the topology loader, but `root_edge_topology_bootstrap_seed()` still uses direct OCCT edge geometry to classify supported line/circle/ellipse root edges and seed `RootEdgeTopology`. Replace that bootstrap read with a Rust-owned classifier/geometry seed while preserving unsupported root-edge `Ok(None)` behavior.

## Next Bounded Cut

1. Read `portingMilestones.md` and `nextStep.md` before editing.
2. Build a topology-local bootstrap geometry classifier for root edges that does not call public `edge_geometry()` because that path recurses through public endpoints/topology.
3. Use the M34 vertex seed and existing Rust curve reconstruction helpers where possible to derive supported line/circle/ellipse `EdgeGeometry` for root topology construction.
4. Remove or strictly narrow `root_edge_topology_bootstrap_seed() -> edge_geometry_occt()` in the same turn.
5. Keep unsupported root edge kinds returning `RootEdgeTopologyInventory::UnsupportedRootEdge` and `Ok(None)` from ported topology/endpoint paths.
6. Keep explicit `Context::edge_geometry_occt()` and `Context::edge_endpoints_occt()` available as oracle/unsupported APIs outside the ported root-edge bootstrap.
7. Strengthen source guards so `root_edge_topology_bootstrap_seed()` contains no `edge_geometry_occt(` or `edge_endpoints_occt(`, and `brep/topology.rs`/`brep/shape_queries.rs` still contain no `edge_endpoints_occt(` calls.
8. Update both control files with completed evidence, active milestone, next bounded cut, and exact verification commands.

## Guardrails

- Do not reintroduce `edge_endpoints_occt()` into `brep/topology.rs` or `brep/shape_queries.rs`.
- Do not route the bootstrap through public `edge_geometry()` or `ported_edge_geometry()` until recursion through root-edge topology is broken or bypassed.
- Do not let unsupported root edge kinds fall through to the generic raw root topology inventory.
- Preserve line/circle/ellipse root endpoint and topology behavior for public, ported, and OCCT parity tests.

## Verification

- `(cd rust/lean_occt && cargo fmt)`
- `cmake --build build --target LeanOcctCAPI`
- `(cd rust/lean_occt && cargo check)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows unsupported_root_edge_does_not_use_generic_raw_topology_inventory -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows public_root_edge_endpoints_are_topology_backed -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows root_edge_endpoints_and_topology_use_ported_seed -- --nocapture)`
- `! perl -0ne 'print $1 if /(fn root_edge_topology_bootstrap_seed[\s\S]*?)\nfn root_edge_topology_bootstrap_length/' rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs | rg -n 'edge_geometry_occt\(|edge_endpoints_occt\('`
- `! rg -n 'edge_endpoints_occt\(' rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/shape_queries.rs`
- `(cd rust/lean_occt && cargo test)`
- `git diff --check`
