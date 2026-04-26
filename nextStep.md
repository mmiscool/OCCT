# Next Task

Current milestone: `M25. Rust-Owned Root Vertex Topology Inventory` from `portingMilestones.md`.

## Completed Evidence

- `M24. Rust-Owned Face-Free Root Wire Inventory` is complete.
- `load_root_topology_snapshot()` now calls `load_root_wire_topology_snapshot()` for face-free root wires after the M23 root-edge branch and before the generic raw root-inventory sweep.
- The root-wire branch clones only the root wire handle, reads ordered occurrence edge handles with `wire_edge_occurrences_occt()`, derives occurrence geometry/endpoints through `topology_edge_query()`, computes lengths through `topology_edge_length()`, deduplicates vertex handles/positions from occurrence endpoints, and packs the single root wire through the existing occurrence-ordering path.
- Ambiguous face-free root wires now return `None` from the root-wire branch instead of silently falling through to the generic `subshapes_occt(shape, Vertex/Edge/Wire/Face/Shell)` plus `vertex_point_occt()` sweep.
- `ported_brep_uses_rust_owned_topology_for_face_free_shapes` now strengthens the helix/root-wire case by checking public wire, edge, and vertex handles, empty face/shell inventories, and topology edge lengths.
- Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_face_free_shapes -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows root_edge_endpoints_and_topology_use_ported_seed -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows supported_brep_materialization_requires_ported_topology -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`, root-wire source guards for the narrow occurrence/endpoint branch, and `git diff --check`.
- No C ABI glue changed in M24, so `cmake --build build --target LeanOcctCAPI -j 8` was not required.

## Target

Replace the root-vertex side of `load_root_topology_snapshot()`:

`subshapes_occt(shape, Vertex/Edge/Wire/Shell/Face)` plus `vertex_point_occt()` for a root `Vertex`

Root vertices should derive their single topology vertex position and public vertex handle from a root-vertex-specific branch before the generic raw inventory loader is considered.

## Next Bounded Cut

1. Add `load_root_vertex_topology_snapshot()` before the generic root loader, and keep the M23 root-edge and M24 root-wire branches ahead of the generic path.
2. Guard it to root `ShapeKind::Vertex` values.
3. Seed the branch with a cloned root vertex handle and a narrow vertex point read; do not use the generic root `subshapes_occt(shape, ...)` sweep.
4. Return a one-position topology snapshot with empty edge, wire, face, and shell inventories.
5. Thread the root-vertex inventory through `load_ported_topology()` so `ported_topology()`, public `vertex_point()`, public root-vertex subshape counts, and public root-vertex subshape handles use the Rust-owned branch.
6. Strengthen `ported_vertex_points_match_occt` so it checks public topology, public vertex subshape handle, public vertex point, and empty edge/wire/face/shell inventories for the exercised root vertex.
7. Add source guards proving the root-vertex branch uses the narrow seed and avoids `subshapes_occt()`, `vertex_point_occt()` inside the branch, and `topology_occt()`.

## Guardrails

- Read `portingMilestones.md` and `nextStep.md` at the start of the next turn before editing.
- Do not weaken or reorder the completed M23 root-edge and M24 root-wire branches.
- Keep face-bearing topology on the existing face inventory path until a separate face-inventory milestone.
- Keep explicit `*_occt()` helpers available as oracle APIs for tests and unsupported/imported shapes.
- If a root vertex cannot be represented by the narrow branch, return `None` instead of entering the generic raw root inventory path for that root.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_vertex_points_match_occt -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows root_edge_endpoints_and_topology_use_ported_seed -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_face_free_shapes -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- Add the named root-vertex topology source guards after implementing the branch.
- `git diff --check`
