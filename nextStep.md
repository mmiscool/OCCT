# Next Task

Current milestone: `M38. Close Remaining Public Edge Geometry Raw Classifier` from `portingMilestones.md`.

## Completed Evidence

- `M37. Rust-Owned Public Root Edge Geometry Query` is complete.
- `Context::ported_edge_geometry()` now checks `ShapeKind::Edge` roots before the direct `edge_geometry_occt()` read and returns `brep::ported_root_edge_geometry()` for supported root line/circle/ellipse edges.
- `brep::ported_root_edge_geometry()` reads the single `RootEdgeTopology.geometry` from the M36 root topology inventory and returns `Ok(None)` for unsupported roots, non-root shapes, or malformed inventories.
- Unsupported helix root edges return `Ok(None)` from `ported_edge_geometry()` and still resolve through the explicit public/raw `edge_geometry()` fallback.
- Root line geometry uses endpoint length as its Rust-owned parameter span; closed circle/ellipse roots canonicalize orientation through Rust-owned payload reconstruction; open circle arcs reconstruct signed arc spans from endpoints plus a normalized midpoint sample.
- `PortedCurve::from_context_with_ported_payloads()` has a root-geometry branch guarded to the topology-backed root geometry domain, so raw geometry parity callers still use the generic raw-domain payload path.
- Swept BRep descriptors now prefer Rust sample-based face-domain swept basis reconstruction, which keeps extrusion/revolution BRep behavior green after public edge geometry became root-topology-owned.
- Regression coverage now asserts root public/ported/topology geometry parity, unsupported root exclusion, root sample/payload parity against normalized OCCT oracle samples, and raw-domain oracle parity where explicit raw geometry is requested.
- Source guards prove the root branch in `ported_edge_geometry()`, `ported_root_edge_geometry()`, and `root_edge_topology_bootstrap_seed()` do not call direct raw edge geometry/endpoints or recursive public geometry.
- Full verification passed with `cargo test` and `git diff --check`.

## Target

Remove the last raw edge geometry classifier still inside the ported public edge geometry API:

`Context::ported_edge_geometry() -> edge_geometry_occt()` after the root-edge branch.

After M37, supported root edge geometry is Rust-owned. The remaining post-root branch keeps direct raw geometry classification inside `ported_edge_geometry()` for non-root or invalid inputs even though public `edge_geometry()` already owns the explicit raw/oracle fallback when `ported_edge_geometry()` returns `None`.

## Next Bounded Cut

1. Read `portingMilestones.md` and `nextStep.md` before editing.
2. Remove or strictly narrow the remaining post-root `edge_geometry_occt()` call from `Context::ported_edge_geometry()`.
3. If tests expose an exercised non-root edge input, satisfy it from a Rust-owned inventory/topology-backed branch; otherwise return `Ok(None)` after the root-edge branch and let public `edge_geometry()` perform the explicit raw fallback.
4. Preserve supported root line/circle/ellipse behavior from M37 and unsupported root helix behavior.
5. Strengthen coverage so `ported_edge_geometry()` returning `None` is explicitly tested for unsupported/non-owned edge geometry while public `edge_geometry()` still returns the raw/oracle value.
6. Add a source guard proving the whole `ported_edge_geometry()` body contains no `edge_geometry_occt(` call.
7. Update both control files with completed evidence, active milestone, next bounded cut, and exact verification commands.

## Guardrails

- Do not reintroduce direct `edge_geometry_occt()` or `edge_endpoints_occt()` into root-edge topology bootstrap.
- Do not call public `edge_geometry()` or `ported_edge_geometry()` from root-edge topology bootstrap.
- Keep explicit `Context::edge_geometry_occt()` available as the raw oracle API.
- Keep explicit public `Context::edge_geometry()` fallback behavior for unsupported roots unless a Rust-owned replacement is landed in the same turn.
- Preserve root edge endpoint, topology, payload, sample, public geometry, swept BRep, and raw-oracle parity behavior.

## Verification

- `(cd rust/lean_occt && cargo fmt)`
- `cmake --build build --target LeanOcctCAPI`
- `(cd rust/lean_occt && cargo check)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows unsupported_root_edge_does_not_use_generic_raw_topology_inventory -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows ported_curve_sampling_matches_occt -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test brep_workflows)`
- `! perl -0ne 'print $1 if /(pub fn ported_edge_geometry[\s\S]*?)\n    pub fn ported_face_geometry/' rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs | rg -n 'edge_geometry_occt\('`
- `(cd rust/lean_occt && cargo test)`
- `git diff --check`
