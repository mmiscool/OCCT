# Next Task

Current milestone: `M21. Rust-Owned Root Edge Endpoint Seeds` from `portingMilestones.md`.

## Completed Evidence

- `wire_topology.rs::root_wire_topology_from_snapshot()` was deleted, removing the full-wire `context.topology_occt(&prepared_wire_shape.wire_shape)` fallback from supported wire topology construction.
- Root and face wire preparation now records ordered edge occurrence handles through the narrow C ABI `lean_occt_shape_wire_edge_occurrence_count()` and `lean_occt_shape_wire_edge_occurrence()`.
- Rust maps each occurrence handle back to a root edge using exact shape identity from `lean_occt_shape_is_same()` plus Rust edge geometry, length, and vertex compatibility. Occurrence orientation and ordered vertex chains are built in Rust through `root_wire_topology_from_occurrences()`.
- `ported_brep_orders_repeated_wire_edge_occurrences_in_rust` exercises the cylinder seam/repeated-edge wire family, asserts at least one wire repeats a root edge index, matches Rust topology against the OCCT oracle, and verifies BRep materialization keeps the Rust topology.
- Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cmake --build build --target LeanOcctCAPI`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_orders_repeated_wire_edge_occurrences_in_rust -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`, `! rg -n 'root_wire_topology_from_snapshot|topology_occt\(&prepared_wire_shape\.wire_shape\)' rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/wire_topology.rs`, and `git diff --check`.

## Target

Replace `shape_queries.rs::root_edge_endpoints_from_topology_seed()`, which still calls `context.topology_occt(shape)?` to seed endpoint positions for root edge shapes before public/Rust-owned `ported_edge_endpoints()` can complete.

## Next Bounded Cut

1. Add or reuse a narrow root-edge endpoint seed that returns endpoint positions without loading a full `TopologySnapshot`.
2. Route `ported_edge_endpoints()` through that seed for supported root line, circle, and ellipse edges, while keeping unsupported edge kinds explicit.
3. Strengthen public/root edge endpoint and topology-construction regression coverage so line, circle, and ellipse endpoint parity is proven without the raw topology snapshot.
4. Add a guard proving `root_edge_endpoints_from_topology_seed()` no longer calls `context.topology_occt(shape)?`.

## Guardrails

- Read `portingMilestones.md` and `nextStep.md` at the start of the next turn before editing.
- Do not reintroduce `root_wire_topology_from_snapshot()` or `context.topology_occt(&prepared_wire_shape.wire_shape)` in the wire topology path.
- Do not reintroduce direct OCCT helper fallbacks into public payload, geometry, sampling, topology, vertex, supported subshape, BRep materialization, or topology-construction wrappers narrowed under M7 through M20.
- Keep explicit `*_occt()` helpers available as oracle APIs for tests and unsupported/imported shapes.
- Keep `SingleFaceOffsetResult`, `MultiFaceOffsetResult`, signed offset matching, deterministic multi-source offset scoring, and repeated wire occurrence identity matching intact.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- Rebuild C ABI if the endpoint seed adds glue: `cmake --build build --target LeanOcctCAPI`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- Add the focused endpoint regression command here once named.
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_curve_sampling_matches_occt -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_face_free_shapes -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- `! awk '/fn root_edge_endpoints_from_topology_seed/,/^}/' rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/shape_queries.rs | rg -n 'topology_occt'`
- `git diff --check`
