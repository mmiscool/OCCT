# Next Task

Current milestone: `M32. Strict Rust-Owned BRep Requirement for Supported Face-Free Assemblies` from `portingMilestones.md`.

## Completed Evidence

- `M31. Rust-Owned Face-Free Compound Assembly Topology Inventory` is complete for supported shell-free root `Compound` assemblies with direct or nested `Face`, face-free `Wire`, `Edge`, and `Vertex` leaves.
- `root_assembly_topology_inventory_required()` no longer rejects every root `Compound` solely because `summary.face_count == 0` or `summary.shell_count == 0`; strict shell/face requirements remain for `CompSolid`.
- `root_assembly_child_topology_supported()` accepts `ShapeKind::Face`, `ShapeKind::Wire`, `ShapeKind::Edge`, and `ShapeKind::Vertex` leaf roots only when the existing root-specific Rust topology guards/loaders succeed.
- `load_root_assembly_topology_snapshot()` no longer returns `None` solely because `face_shapes` or `root_assembly_shell_shapes` is empty, allowing shell-free assemblies to populate face/wire/edge/vertex inventories with empty shell/solid inventories.
- `ported_brep_uses_rust_owned_topology_for_root_compound_faces` exercises `Compound -> [Face, Face]` and verifies direct child kinds, Rust topology, public face handles, child face topology, BRep topology, and summary parity.
- `ported_brep_uses_rust_owned_topology_for_nested_root_compound_face_free_wires` exercises `Compound -> Compound -> [Wire, Wire]` and verifies the zero-face recursive assembly path, public wire handles, child wire topology, BRep topology, and summary parity.
- `ported_brep_uses_rust_owned_topology_for_root_compound_edges_and_vertices` exercises `Compound -> [Edge, Edge]` and `Compound -> [Vertex, Vertex]`, verifying public edge/vertex handles, child topology, BRep topology, and summary parity.
- Source guards prove the old compound-level `face_count == 0 || shell_count == 0` rejection, the loader's empty face/shell inventory exits, and raw root-level `subshapes_occt()`, `vertex_point_occt()`, and `topology_occt()` calls remain absent from `load_root_assembly_topology_snapshot()`.

## Target

Narrow the remaining raw BRep fallback for supported shell-free assemblies:

`ported_brep() -> supported recursive face-free Compound -> must use Rust topology or error`

M31 gives these assemblies Rust-owned topology, but `strict_brep_requires_ported_topology()` still treats every `Compound` with `summary.face_count == 0` as not requiring ported topology. That leaves a raw `topology_occt()`/`subshapes_occt()` fallback path open if the new face-free assembly loader regresses.

## Next Bounded Cut

1. Read `portingMilestones.md` and `nextStep.md` before editing.
2. Expose or reuse the root-assembly support classifier from the topology loader for strict BRep fallback decisions.
3. Update `strict_brep_requires_ported_topology()` so supported direct or nested face-free `Compound` assemblies require Rust topology even when `face_count == 0`.
4. Keep unsupported/imported assemblies out of the strict requirement so explicit raw/oracle paths remain available.
5. Strengthen workflow/source coverage around wire/edge/vertex compound BRep materialization and add a guard proving the strict BRep branch no longer short-circuits all `Compound` roots on `face_count == 0`.
6. Update both control files with completed evidence, active milestone, next bounded cut, and exact verification commands.

## Guardrails

- Do not weaken or reorder the completed M23-M31 root edge, vertex, wire, face, shell, solid, direct-child assembly, nested assembly, or face-free assembly branches.
- Keep explicit `*_occt()` helpers available as oracle APIs for tests and unsupported/imported shapes.
- Do not broaden the generic raw root inventory fallback while narrowing strict BRep fallback behavior.
- If a recursive assembly classifier reports unsupported, keep the raw fallback decision explicit instead of forcing every `Compound` to require ported topology.

## Verification

- `(cd rust/lean_occt && cargo fmt)`
- `cmake --build build --target LeanOcctCAPI`
- `(cd rust/lean_occt && cargo check)`
- Focused strict-BRep workflow regressions for supported face-free compound assemblies.
- `(cd rust/lean_occt && cargo test)`
- Source guards for strict face-free compound BRep fallback narrowing and unsupported assembly preservation.
- `git diff --check`
