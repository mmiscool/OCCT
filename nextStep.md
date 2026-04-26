# Next Task

Current milestone: `M31. Rust-Owned Face-Free Compound Assembly Topology Inventory` from `portingMilestones.md`.

## Completed Evidence

- `M30. Rust-Owned Nested Assembly Topology Inventory` is complete for supported nested solid/shell assembly families.
- `lean_occt_shape_make_compound()` and `ModelKernel::make_compound()` add deterministic nested root `Compound` fixtures; `lean_occt_shape_make_compsolid()` and `ModelKernel::make_compsolid()` remain available for nested `CompSolid` fixtures.
- `root_assembly_child_topology_supported()` recursively accepts supported child `Compound` and `CompSolid` nodes down to solid/shell leaves through direct-child wrappers, with `ROOT_ASSEMBLY_MAX_DEPTH` guarding recursion.
- Unsupported nested assembly nodes still return `RootAssemblyTopologyInventory::Unsupported`, and `load_root_topology_snapshot()` returns `Ok(None)` for unsupported assemblies before the generic raw root inventory sweep can run.
- `ported_brep_uses_rust_owned_topology_for_nested_root_compound_solids` exercises `Compound -> Compound -> [Solid, Solid]`, verifies direct child shape kinds, public solid handles, child solid topology, BRep topology, and summary area/volume parity.
- `ported_brep_uses_rust_owned_topology_for_nested_root_compound_compsolid` exercises `Compound -> CompSolid -> [Solid, Solid]`, verifies direct child shape kinds through the `CompSolid` child wrapper, public solid handles, child solid topology, BRep topology, and summary area/volume parity.
- `ported_brep_uses_rust_owned_topology_for_nested_root_compound_shells` exercises `Compound -> Compound -> [Shell, Shell]`, verifies direct child shape kinds, public shell handles, child shell topology, BRep topology, edge geometry/length parity, and summary count/area parity.
- The root-assembly loader still does not call raw root-level `subshapes_occt()`, `vertex_point_occt()`, or `topology_occt()` inside `load_root_assembly_topology_snapshot()`.

## Target

Move the next supported compound assembly shape from the generic raw root inventory sweep into Rust-owned topology:

`Compound direct/nested children -> supported Face/Wire/Edge/Vertex leaves -> root-wide Rust topology`

Face, face-free wire, edge, and vertex roots already have Rust-owned loaders from M23-M26. M31 should let a root `Compound` containing those supported leaves use recursive/direct-child assembly inventory instead of being rejected by the current solid/shell-only assembly guard and shell-required loader path.

## Next Bounded Cut

1. Add deterministic `make_compound()` workflow fixtures for supported face leaves and face-free wire leaves, with direct child kind assertions proving the assembly shape is actually a root `Compound`.
2. Extend the recursive assembly classifier to accept `ShapeKind::Face`, `ShapeKind::Wire`, `ShapeKind::Edge`, and `ShapeKind::Vertex` child roots only when their existing root-specific Rust topology guards pass.
3. Split `load_root_assembly_topology_snapshot()` so shell-free assemblies can populate face/wire/edge/vertex inventories and return empty shell/solid inventories instead of returning `None` when `root_assembly_shell_shapes` is empty.
4. Strengthen workflow coverage for public leaf handles, child topology, BRep topology, summary area/length parity, and the unsupported assembly stop.
5. Add source guards proving face-free compounds enter the root-assembly branch through direct-child/root-assembly wrappers and still avoid root-level generic `subshapes_occt()`, `vertex_point_occt()`, and `topology_occt()`.

## Guardrails

- Read `portingMilestones.md` and `nextStep.md` at the start of the next turn before editing.
- Do not weaken or reorder the completed M23-M30 root edge, vertex, wire, face, shell, solid, direct-child assembly, or nested assembly branches.
- Keep explicit `*_occt()` helpers available as oracle APIs for tests and unsupported/imported shapes.
- Do not broaden the generic raw root inventory fallback while adding face-free compound support.
- If a child `Face`, `Wire`, `Edge`, or `Vertex` root cannot satisfy its existing Rust-owned guard, keep the root assembly unsupported instead of silently falling through to raw topology.

## Verification

- `(cd rust/lean_occt && cargo fmt)`
- `cmake --build build --target LeanOcctCAPI`
- `(cd rust/lean_occt && cargo check)`
- Focused face-free compound workflow regressions.
- `(cd rust/lean_occt && cargo test)`
- Source guards for face-free assembly handling, recursive direct-child classification, empty-shell assembly support, and unsupported assembly stop.
- `git diff --check`
