# Next Task

Current milestone: `M40. Rust-Owned Public Edge Query Support Gates` from `portingMilestones.md`.

## Completed Evidence

- `M39. Rust-Owned Strict BRep Root Edge Support Gate` is complete.
- `strict_brep_root_edge_requires_ported_topology()` now calls `ported_root_edge_geometry()` and no longer uses `edge_geometry_occt()` as the support classifier for root edges.
- Supported root line, circle, and ellipse edges are now exercised in `supported_brep_materialization_requires_ported_topology`: each is classified through `ported_edge_geometry()`, materializes BRep topology from `ported_topology()`, and carries the expected ported curve.
- Unsupported helix child edges are covered as root-edge exclusions: `ported_topology()` and `ported_edge_geometry()` return `None`, while `kernel.brep(&helix_edge)` still succeeds through the explicit raw BRep fallback and matches OCCT topology.
- The strict root-edge gate source guard passed: no `edge_geometry_occt(` call remains between `strict_brep_root_edge_requires_ported_topology()` and `strict_brep_face_inventory_requires_ported_topology()`.
- Verification passed with `(cd rust/lean_occt && cargo fmt)`, `cmake --build build --target LeanOcctCAPI`, `(cd rust/lean_occt && cargo check)`, targeted `brep_workflows` tests, full `(cd rust/lean_occt && cargo test)`, and the strict-gate source guard.

## Target

Remove the next public raw edge geometry support classifiers:

`Context::edge_endpoints()`, `edge_sample()`, `edge_sample_at_parameter()`, and `edge_geometry()` in `rust/lean_occt/src/lib.rs` currently call `edge_geometry_occt()` after their ported query path returns `None` only to decide whether a line/circle/ellipse edge should require Rust-owned behavior.

After M39, root edge support classification is available through the Rust root-edge topology/geometry inventory. The public query gates should use that classifier instead of raw edge geometry. The explicit raw `_occt` operations should remain available only as actual unsupported fallback/oracle calls.

## Next Bounded Cut

1. Read `portingMilestones.md` and `nextStep.md` before editing.
2. Add or expose a Rust-owned public edge query support classifier backed by `ported_root_edge_geometry()` or the root topology inventory.
3. Replace the `edge_geometry_occt()` support-classifier calls in `edge_endpoints()`, `edge_sample()`, `edge_sample_at_parameter()`, and `edge_geometry()`.
4. Preserve Rust-required behavior for supported root line/circle/ellipse geometry, endpoints, and samples.
5. Preserve unsupported helix/root-edge public query fallback through explicit raw `_occt` operations.
6. Strengthen regression coverage if the existing root-edge endpoint/sample and unsupported helix tests do not fail on this fallback family.
7. Add a source guard proving the public edge query gate block contains no `edge_geometry_occt(` call before `edge_geometry_occt()` itself.
8. Update both control files with completed evidence, active milestone, next bounded cut, and exact verification commands.

## Guardrails

- Do not reintroduce direct `edge_geometry_occt()` into `ported_edge_geometry()`, root-edge topology bootstrap, or the strict BRep root-edge gate.
- Do not call public `edge_geometry()` from the new public support classifier; that would recurse through the gate being replaced.
- Keep explicit `Context::edge_geometry_occt()`, `edge_endpoints_occt()`, `edge_sample_occt()`, and `edge_sample_at_parameter_occt()` available as raw oracle APIs.
- Keep unsupported root edge kinds out of the Rust-required family unless the same turn also ports their geometry/topology/query behavior.
- Preserve root edge topology, BRep materialization, public geometry, endpoint, payload, sample, and raw-oracle parity behavior.

## Verification

- `(cd rust/lean_occt && cargo fmt)`
- `cmake --build build --target LeanOcctCAPI`
- `(cd rust/lean_occt && cargo check)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows public_root_edge_endpoints_are_topology_backed -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows root_edge_endpoints_and_topology_use_ported_seed -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows ported_curve_sampling_matches_occt -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows unsupported_root_edge_does_not_use_generic_raw_topology_inventory -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test brep_workflows supported_brep_materialization_requires_ported_topology -- --nocapture)`
- `! perl -0ne 'print $1 if /(pub fn edge_endpoints[\s\S]*?)\n    pub fn edge_geometry_occt/' rust/lean_occt/src/lib.rs | rg -n 'edge_geometry_occt\('`
- `(cd rust/lean_occt && cargo test)`
- `git diff --check`
