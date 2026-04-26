# Next Task

Current milestone: `M26. Rust-Owned Root Face Topology Inventory` from `portingMilestones.md`.

## Completed Evidence

- `M25. Rust-Owned Root Vertex Topology Inventory` is complete.
- `load_root_topology_snapshot()` now calls `load_root_vertex_topology_snapshot()` for root vertices after the M23 root-edge branch and before the M24 face-free root-wire branch and generic raw root-inventory sweep.
- The root-vertex branch clones only the root vertex handle, reads the point through the narrow C ABI `lean_occt_shape_root_vertex_point()` wrapped by `root_vertex_point_seed_occt()`, and returns a one-position topology snapshot with empty edge, wire, face, and shell inventories.
- Malformed root vertices return `None` from the root-vertex branch instead of silently falling through to the generic `subshapes_occt(shape, Vertex/Edge/Wire/Face/Shell)` plus `vertex_point_occt()` sweep.
- `ported_vertex_points_match_occt` now checks public topology, public vertex subshape count/handles, public indexed vertex handle, public vertex point, and empty edge/wire/face/shell inventories for exercised root vertices.
- Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cmake --build build --target LeanOcctCAPI -j 8`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_vertex_points_match_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows root_edge_endpoints_and_topology_use_ported_seed -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_face_free_shapes -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`, root-vertex source guards for the narrow seed branch and C ABI symbol, and `git diff --check`.

## Target

Replace the root-face side of `load_root_topology_snapshot()`:

`subshapes_occt(shape, Vertex/Edge/Wire/Shell/Face)` for a supported root `Face`

Root faces should derive their face handle, wire handles, edge inventory, and vertex positions from a root-face-specific branch before the generic raw inventory loader is considered.

## Next Bounded Cut

1. Add a narrow face-wire C ABI seed, exposed through a crate-private Rust wrapper, for ordered root-face wire handles.
2. Add `load_root_face_topology_snapshot()` after the completed root edge, root vertex, and face-free root-wire branches, but before the generic root loader.
3. Guard it to supported `ShapeKind::Face` roots with one face and no shell inventory.
4. Seed the branch with a cloned root face handle and face-local wire handles; do not use the generic root `subshapes_occt(shape, ...)` sweep.
5. Reuse the M20/M24 wire occurrence path plus Rust-owned edge geometry/endpoints/length to build unique edge and vertex inventories.
6. Prepare the single face from those wires so `load_ported_face_snapshot()` can build the face ranges without `topology_occt()`.
7. Strengthen the simple single-face topology/BRep regression so it checks public face, wire, edge, and vertex subshape handles for a root face.
8. Add source guards proving the root-face branch uses the narrow face-wire and wire-occurrence seeds and avoids root-level `subshapes_occt()`, `vertex_point_occt()`, and `topology_occt()`.

## Guardrails

- Read `portingMilestones.md` and `nextStep.md` at the start of the next turn before editing.
- Do not weaken or reorder the completed M23 root-edge, M25 root-vertex, or M24 root-wire branches.
- Keep shell, solid, compound, and ambiguous face-bearing topology on the existing generic path until their own milestones.
- Keep explicit `*_occt()` helpers available as oracle APIs for tests and unsupported/imported shapes.
- If a root face cannot be represented by the narrow branch, return `None` instead of entering the generic raw root inventory path for that root.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cmake --build build --target LeanOcctCAPI -j 8`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_simple_single_face_shapes -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_face_surface_descriptors_cover_supported_faces -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- Add the named root-face topology source guards after implementing the branch.
- `git diff --check`
