# Next Task

Current milestone: `M33. Rust-Owned Public Root Edge Endpoint Queries` from `portingMilestones.md`.

## Completed Evidence

- `M32. Strict Rust-Owned BRep Requirement for Supported Face-Free Assemblies` is complete.
- `root_assembly_requires_ported_topology()` now reuses the root-assembly topology classifier and reports true only for supported recursive assembly roots.
- `strict_brep_requires_ported_topology()` checks `ShapeKind::Compound` with the assembly classifier before the `summary.face_count == 0` short-circuit, so supported direct, nested, and mixed face-free compounds must load Rust topology before BRep materialization can continue.
- Unsupported and non-assembly zero-face compounds still return `Ok(false)` from the strict gate after classifier rejection, preserving explicit raw/oracle behavior for imported or unsupported assemblies.
- `strict_brep_requires_ported_topology_for_supported_face_free_compounds` covers nested `Compound -> Compound -> [Wire, Wire]`, `Compound -> [Edge, Edge]`, `Compound -> [Vertex, Vertex]`, and mixed `Compound -> [Wire, Edge, Vertex]` roots.
- Existing workflow regressions still cover BRep materialization for `Compound -> [Face, Face]`, nested face-free wire compounds, and direct edge/vertex compounds.
- Source guards prove the strict `Compound` branch calls `root_assembly_requires_ported_topology()` before the zero-face exit and that the helper is wired to `RootAssemblyTopologyInventory::Supported`.

## Target

Remove the public endpoint-query raw seed for supported root edges:

`Context::edge_endpoints() -> ported_edge_endpoints() -> supported root line/circle/ellipse edge -> loaded Rust topology endpoint vertices`

The root-edge topology bootstrap still has a narrow raw endpoint seed, but `ported_edge_endpoints()` currently calls `root_edge_endpoints_from_raw_endpoint_seed()` before trying `context.ported_topology(shape)?`. That leaves the public Rust endpoint query backed by direct `edge_endpoints_occt(shape)?` for supported root line, circle, and ellipse edges even when the completed root-edge topology path can provide endpoint vertices.

## Next Bounded Cut

1. Read `portingMilestones.md` and `nextStep.md` before editing.
2. Split the root-edge bootstrap endpoint seed away from the public `ported_edge_endpoints()` path.
3. Update `ported_edge_endpoints()` so supported root line/circle/ellipse edges load ported topology and derive `EdgeEndpoints` from the single topology edge's start/end vertex positions.
4. Keep unsupported root edge kinds returning `Ok(None)` from the Rust path so explicit OCCT oracle APIs remain available.
5. Preserve non-root-edge behavior and avoid broadening generic raw topology or subshape fallbacks.
6. Strengthen endpoint workflow coverage and add a source guard proving `ported_edge_endpoints()` no longer directly calls `edge_endpoints_occt()`.
7. Update both control files with completed evidence, active milestone, next bounded cut, and exact verification commands.

## Guardrails

- Do not remove the bootstrap seed unless the same turn provides an equivalent Rust-owned root-edge topology construction path; narrowing the public endpoint query is the bounded target.
- Keep explicit `edge_endpoints_occt()` available as an oracle API for tests and unsupported/imported shapes.
- Do not weaken the completed root edge, vertex, wire, face, shell, solid, or assembly topology guards.
- If `load_ported_topology()` returns `None` for an unsupported root edge, keep the public Rust endpoint query explicit about returning `None`.

## Verification

- `(cd rust/lean_occt && cargo fmt)`
- `cmake --build build --target LeanOcctCAPI`
- `(cd rust/lean_occt && cargo check)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows public_root_edge_endpoints_are_topology_backed -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows root_edge_endpoints_and_topology_use_ported_seed -- --nocapture)`
- `! perl -0ne 'print $1 if /(pub\(super\) fn ported_edge_endpoints[\s\S]*?)\nenum RootEdgeEndpointSeed/' rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/shape_queries.rs | rg -n 'edge_endpoints_occt\('`
- `perl -0ne 'print $1 if /(pub\(super\) fn ported_edge_endpoints[\s\S]*?)\nenum RootEdgeEndpointSeed/' rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/shape_queries.rs | rg -n 'load_ported_topology|optional_vertex_position'`
- `(cd rust/lean_occt && cargo test)`
- `git diff --check`
