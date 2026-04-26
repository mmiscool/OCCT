# Next Task

Current milestone: `M22. Strict Supported BRep Materialization Topology Entry` from `portingMilestones.md`.

## Completed Evidence

- `shape_queries.rs::root_edge_endpoints_from_topology_seed()` was replaced by `root_edge_endpoints_from_raw_endpoint_seed()`, removing the full `context.topology_occt(shape)?` snapshot from root edge endpoint seeding.
- The new seed admits only root `Line`, `Circle`, and `Ellipse` edges after a narrow raw geometry kind check, then uses the existing narrow endpoint helper to provide the endpoint positions needed to break root-edge topology recursion.
- Unsupported root edge kinds now return an explicit unsupported-root-edge state from `ported_edge_endpoints()`, letting public callers reach raw/oracle endpoint APIs without recursively loading ported topology.
- `root_edge_endpoints_and_topology_use_ported_seed` covers root line, circle, and ellipse endpoint parity across public, ported, and OCCT oracle queries, then verifies root edge topology endpoint positions and public root subshape counts.
- Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows root_edge_endpoints_and_topology_use_ported_seed -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_curve_sampling_matches_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_face_free_shapes -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`, `! awk '/fn root_edge_endpoints_from_raw_endpoint_seed/,/^}/' rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/shape_queries.rs | rg -n 'topology_occt'`, `! rg -n 'root_edge_endpoints_from_topology_seed' rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/shape_queries.rs`, and `git diff --check`.

## Target

Replace the supported-shape side of `Context::ported_brep()`'s fallback branch:

`None => { self.topology_occt(shape)?, self.subshapes_occt(...), FaceSurfaceRoute::Raw }`

Supported analytic, swept, offset, and supported face-free roots should not silently materialize a public BRep from a full raw OCCT topology snapshot when `load_ported_topology()` misses.

## Next Bounded Cut

1. Add a supported-BRep-materialization classifier using existing summary, root kind, and geometry/topology signals already exercised by `brep_workflows`.
2. Gate `ported_brep()` so supported roots produce an explicit Rust-owned topology/materialization error if `load_ported_topology()` returns `None`.
3. Keep `FaceSurfaceRoute::Raw` only for explicitly unsupported, imported, or ambiguous shapes that cannot be represented by the current Rust topology loader.
4. Add or strengthen a regression that fails if a supported fixture can materialize through the raw `topology_occt(shape)` fallback.
5. Add a source guard proving the raw topology branch in `ported_brep()` is strictly guarded instead of being the unqualified supported fallback.

## Guardrails

- Read `portingMilestones.md` and `nextStep.md` at the start of the next turn before editing.
- Do not reintroduce `root_edge_endpoints_from_topology_seed()` or `context.topology_occt(shape)?` inside the root endpoint seed.
- Do not reintroduce `root_wire_topology_from_snapshot()` or `context.topology_occt(&prepared_wire_shape.wire_shape)` in the wire topology path.
- Do not reintroduce direct OCCT helper fallbacks into public payload, geometry, sampling, topology, vertex, supported subshape, BRep materialization, or topology-construction wrappers narrowed under M7 through M21.
- Keep explicit `*_occt()` helpers available as oracle APIs for tests and unsupported/imported shapes.
- Keep `SingleFaceOffsetResult`, `MultiFaceOffsetResult`, signed offset matching, deterministic multi-source offset scoring, repeated wire occurrence identity matching, and unsupported root-edge raw endpoint escape behavior intact.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- Add the focused strict-BRep-materialization regression command here once named.
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_simple_multi_face_solids -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_face_free_shapes -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- Add the updated `ported_brep()` source guard once the strict raw-topology fallback shape is known.
- `git diff --check`
