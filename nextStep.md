# Next Task

Current milestone: `M37. Rust-Owned Public Root Edge Geometry Query` from `portingMilestones.md`.

## Completed Evidence

- `M36. Rust-Owned Root Edge Geometry Bootstrap Seed` is complete.
- `root_edge_topology_bootstrap_seed()` no longer calls `edge_geometry_occt()`, public `edge_geometry()`, `ported_edge_geometry()`, `edge_endpoints()`, or `edge_endpoints_occt()`.
- Root-edge topology geometry now comes from `Context::ported_root_edge_topology_bootstrap_geometry()` using the M34 root vertex seed.
- Supported root lines are classified from endpoint/sample collinearity and use endpoint distance for topology length.
- Supported closed circles and ellipses are reconstructed through the ported payload solvers from normalized position samples, and supported open circle arcs are reconstructed from root endpoints plus an interior sample.
- Root-edge topology lengths for circles/ellipses now come from `PortedCurve::length_with_geometry()` instead of the removed `root_edge_topology_bootstrap_length()` raw geometry route.
- Degenerate or unsupported root edges return `RootEdgeTopologyInventory::UnsupportedRootEdge`/`Ok(None)` before the generic raw root inventory sweep; the helix unsupported-root regression remains green.
- `edge_endpoints_occt(` remains absent from `brep/topology.rs` and `brep/shape_queries.rs`.
- Full verification passed with `cargo test`.

## Target

Remove the next public root-edge raw geometry read:

`Context::ported_edge_geometry() -> edge_geometry_occt()`

After M36, topology can seed supported root line/circle/ellipse geometry without direct raw geometry. Public `ported_edge_geometry()` still begins by reading direct OCCT edge geometry, including for root edge shapes that could now return the Rust-owned root topology edge geometry.

## Next Bounded Cut

1. Read `portingMilestones.md` and `nextStep.md` before editing.
2. Add a root-edge branch to `ported_edge_geometry()` that detects `ShapeKind::Edge` root shapes before the direct `edge_geometry_occt()` read.
3. Load Rust topology for that root edge through the M36 bootstrap inventory and return the single `RootEdgeTopology.geometry` when available.
4. Preserve unsupported root edge behavior: unsupported roots should return `Ok(None)` from `ported_edge_geometry()` so public `edge_geometry()` can use the explicit raw/oracle fallback outside the Rust-owned supported path.
5. Do not route the root-edge branch through public `edge_geometry()` or any direct `edge_endpoints_occt()` path.
6. Strengthen root-edge workflow coverage so public geometry for root line/circle/ellipse is proven topology-backed and still matches OCCT oracle geometry where appropriate.
7. Add source guards showing the root-edge branch in `ported_edge_geometry()` contains no `edge_geometry_occt(`, public recursive `edge_geometry(`, or direct endpoint helper.
8. Update both control files with completed evidence, active milestone, next bounded cut, and exact verification commands.

## Guardrails

- Do not reintroduce `edge_endpoints_occt()` into `brep/topology.rs` or `brep/shape_queries.rs`.
- Do not call public `edge_geometry()` or `ported_edge_geometry()` from root-edge topology bootstrap.
- Keep explicit `Context::edge_geometry_occt()` and `Context::edge_endpoints_occt()` available as oracle/unsupported APIs outside the ported root-edge path.
- Preserve line/circle/ellipse root endpoint, topology, payload, sample, and public geometry behavior.

## Verification

- `(cd rust/lean_occt && cargo fmt)`
- `cmake --build build --target LeanOcctCAPI`
- `(cd rust/lean_occt && cargo check)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows root_edge_endpoints_and_topology_use_ported_seed -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows public_root_edge_endpoints_are_topology_backed -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows public_analytic_curve_and_surface_payload_queries_match_occt -- --nocapture)`
- `! perl -0ne 'print $1 if /(pub fn ported_edge_geometry[\s\S]*?)\n    pub fn ported_face_geometry/' rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs | rg -n 'ShapeKind::Edge[\s\S]*edge_geometry_occt\('`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows -- --nocapture)`
- `(cd rust/lean_occt && cargo test)`
- `git diff --check`
