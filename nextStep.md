# Next Task

Current milestone: `M23. Rust-Owned Root Edge Topology Inventory` from `portingMilestones.md`.

## Completed Evidence

- `Context::ported_brep()` now calls `strict_brep_raw_topology_fallback_allowed()` before the raw `self.topology_occt(shape)?` materialization branch.
- The strict classifier requires Rust topology for supported root `Line`, `Circle`, and `Ellipse` edges, face-free wires, and face inventories whose surfaces are supported analytic, swept, or offset kinds.
- Supported roots now return an explicit Rust-owned topology/materialization error when `load_ported_topology()` returns `None` instead of silently switching to `FaceSurfaceRoute::Raw`.
- `supported_brep_materialization_requires_ported_topology` covers representative analytic box, face-free ellipse and helix, swept extrusion and revolution, and offset-surface roots, asserting BRep topology comes from `ported_topology()`.
- Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows supported_brep_materialization_requires_ported_topology -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_simple_multi_face_solids -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_face_free_shapes -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`, `awk '/pub fn ported_brep\(&self, shape: &Shape\)/,/let vertices =/' rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep.rs | rg -n 'strict_brep_raw_topology_fallback_allowed|self\.topology_occt\(shape\)\?|FaceSurfaceRoute::Raw'`, and `git diff --check`.

## Target

Replace the supported root-edge side of `load_root_topology_snapshot()`:

`subshapes_occt(shape, Vertex/Edge/Wire/Shell/Face)` plus `vertex_point_occt()` for a root `Edge`

Root line, circle, and ellipse topology construction should use the Rust-owned endpoint seed and supported edge geometry/length data before the generic raw inventory loader is considered.

## Next Bounded Cut

1. Add a root-edge-specific topology inventory entry before the generic `load_root_topology_snapshot()` path.
2. Construct supported root edge vertex positions from the existing Rust-owned endpoint seed and construct the single topology edge from Rust-owned edge geometry/length data.
3. Thread the root-edge inventory through `load_ported_topology()` so `ported_topology()`, BRep materialization, public root-edge subshape counts, and root-edge topology parity use the Rust-owned branch.
4. Keep unsupported root edges on explicit raw/oracle APIs rather than recursively forcing ported topology.
5. Add or strengthen regression coverage that would fail if supported root edge topology reaches the generic raw `subshapes_occt()`/`vertex_point_occt()` inventory path.
6. Add a source guard proving the supported root-edge topology branch avoids the generic raw inventory calls once the branch is named.

## Guardrails

- Read `portingMilestones.md` and `nextStep.md` at the start of the next turn before editing.
- Do not loosen `strict_brep_raw_topology_fallback_allowed()` or let supported `ported_brep()` roots silently enter `FaceSurfaceRoute::Raw`.
- Do not reintroduce `root_edge_endpoints_from_topology_seed()` or `context.topology_occt(shape)?` inside the root endpoint seed.
- Do not reintroduce `root_wire_topology_from_snapshot()` or `context.topology_occt(&prepared_wire_shape.wire_shape)` in the wire topology path.
- Keep explicit `*_occt()` helpers available as oracle APIs for tests and unsupported/imported shapes.
- Keep `SingleFaceOffsetResult`, `MultiFaceOffsetResult`, signed offset matching, deterministic multi-source offset scoring, repeated wire occurrence identity matching, and unsupported root-edge raw endpoint escape behavior intact.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows root_edge_endpoints_and_topology_use_ported_seed -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_curve_sampling_matches_occt -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_face_free_shapes -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows supported_brep_materialization_requires_ported_topology -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- Add the named root-edge topology source guard after implementing the branch.
- `git diff --check`
