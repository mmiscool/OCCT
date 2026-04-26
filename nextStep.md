# Next Task

Current milestone: `M24. Rust-Owned Face-Free Root Wire Inventory` from `portingMilestones.md`.

## Completed Evidence

- `M23. Rust-Owned Root Edge Topology Inventory` is complete.
- `load_root_topology_snapshot()` now calls `load_root_edge_topology_snapshot()` before the generic raw root-inventory sweep.
- Supported root `Line`, `Circle`, and `Ellipse` edges build topology from the Rust-owned endpoint seed, public/Rust-owned edge geometry, and ported curve length instead of `subshapes_occt(shape, Vertex/Edge/Wire/Face/Shell)` plus `vertex_point_occt()`.
- The root-edge branch carries only a cloned root edge handle and narrow endpoint vertex handles through the C ABI via `lean_occt_shape_clone()` and `lean_occt_shape_root_edge_vertex()`.
- `root_edge_topology()` and `wire_occurrence()` now share `topology_edge_length()`, so supported edge topology and wire occurrence identity use `PortedCurve::from_context_with_ported_payloads()` lengths. Raw `edge_length(edge_shape)` remains only for unsupported curve kinds.
- `root_edge_endpoints_and_topology_use_ported_seed` now checks root topology length against `ported_edge_length()`, public root-edge edge/vertex subshape handles, public vertex points, and empty wire/face/shell inventories.
- `brep_workflows` now validates topology edge lengths against public ported edge length instead of treating raw OCCT mass-property edge length as the canonical value for supported ellipse topology.
- Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cmake --build build --target LeanOcctCAPI -j 8`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows root_edge_endpoints_and_topology_use_ported_seed -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_curve_sampling_matches_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_face_free_shapes -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows supported_brep_materialization_requires_ported_topology -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_offset_basis_queries_match_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`, source guards for the root-edge branch and shared topology edge length path, and `git diff --check`.

## Target

Replace the face-free root-wire side of `load_root_topology_snapshot()`:

`subshapes_occt(shape, Vertex/Edge/Wire/Shell/Face)` plus `vertex_point_occt()` for a root `Wire`

Face-free root wires should derive their root wire handle, edge occurrence handles, edge inventory, vertex positions, and packed wire topology from narrow wire occurrence handles plus Rust-owned edge geometry/endpoints/length before the generic raw inventory loader is considered.

## Next Bounded Cut

1. Add a root-wire-specific topology inventory entry after the root-edge branch and before the generic `load_root_topology_snapshot()` path.
2. Guard it to root `ShapeKind::Wire` values with no faces, so face-bearing topology keeps the existing path until a separate face-inventory cut.
3. Seed the branch with a cloned root wire handle and `wire_edge_occurrences_occt()` occurrence handles; do not use the generic root `subshapes_occt(shape, ...)` sweep for supported face-free wires.
4. Build root edge shapes from occurrence identity, edge geometry/endpoints from `topology_edge_query()`, and edge lengths from `topology_edge_length()`.
5. Deduplicate vertex positions from occurrence endpoints and construct the single packed root wire with existing occurrence ordering.
6. Thread the root-wire inventory through `load_ported_topology()` so `ported_topology()`, BRep materialization, public root-wire subshape counts, and public root-wire subshape handles use the Rust-owned branch.
7. Strengthen `ported_brep_uses_rust_owned_topology_for_face_free_shapes` around the helix/root-wire case so it checks public edge/wire/vertex handles and topology edge lengths.
8. Add a named source guard proving the root-wire branch avoids `subshapes_occt(shape, Vertex/Edge/Wire/Face/Shell)`, `vertex_point_occt()`, and `topology_occt()`.

## Guardrails

- Read `portingMilestones.md` and `nextStep.md` at the start of the next turn before editing.
- Do not loosen `strict_brep_raw_topology_fallback_allowed()` or let supported `ported_brep()` roots silently enter `FaceSurfaceRoute::Raw`.
- Keep the completed M23 root-edge branch ahead of the generic loader and free of `subshapes_occt()`, `vertex_point_occt()`, and `topology_occt()`.
- Keep unsupported root edges and ambiguous root wires on explicit raw/oracle APIs rather than recursive forced ported topology.
- Do not reintroduce `root_wire_topology_from_snapshot()` or `context.topology_occt(&prepared_wire_shape.wire_shape)` in the wire topology path.
- Keep explicit `*_occt()` helpers available as oracle APIs for tests and unsupported/imported shapes.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cmake --build build --target LeanOcctCAPI -j 8` if C ABI glue changes
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_face_free_shapes -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows root_edge_endpoints_and_topology_use_ported_seed -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows supported_brep_materialization_requires_ported_topology -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- Add the named root-wire topology source guard after implementing the branch.
- `git diff --check`
