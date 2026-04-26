# Next Task

Current milestone: `M3. Rust-Backed Traversal for Documents and Selectors` from `portingMilestones.md`.

## Completed Evidence

- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs` now promotes fully loaded analytic, swept, and offset face inventories into the Rust-owned summary requirement set. Supported bbox summaries must resolve through exact primitive, ported topology, offset, or mesh candidates before failing; supported closed-solid volumes must resolve through exact primitive formulas, face contributions, or whole-shape mesh before failing.
- The surviving generic `fallback_summary()` bbox and volume branches are no longer normal continuation paths. They are gated by `unsupported_bbox_summary_fallback_allowed()` and `unsupported_volume_summary_fallback_allowed()`, so only explicitly unsupported or unclassified shapes can still reach `SummaryBboxSource::OcctFallback` or `SummaryVolumeSource::OcctFallback`.
- `rust/lean_occt/tests/brep_workflows.rs` now asserts exact primitive bbox and volume sources are `SummaryBboxSource::ExactPrimitive` and `SummaryVolumeSource::ExactPrimitive`, supported single-face analytic bboxes stay Rust-owned, and supported multi-face analytic solid bbox/volume summaries do not slide to `OcctFallback`. The through-hole boolean volume remains an explicit unsupported fallback case because that shape is not a fully loaded supported face inventory.
- Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cmake --build build --target LeanOcctCAPI`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_exact_primitive_surface_and_volume_formulas -- --exact`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_simple_single_face_shapes -- --exact`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_simple_multi_face_solids -- --exact`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces -- --exact`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_volume_for_offset_solids -- --exact`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml`.
- `M2. Whole-Shape Summary Fallback Reduction` is complete. The remaining active work has moved to `M3`.

## Target

Replace or strictly narrow the remaining OCCT-backed face/edge traversal boundary used by public shape queries, while keeping `ModelDocument`, selectors, and high-level reports backed by `BrepShape`/`TopologySnapshot` for supported shapes.

## Next Bounded Cut

1. In `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/shape_queries.rs`, change `ported_subshape()` and `ported_subshapes()` so the supported path is keyed off `Context::ported_topology(shape)?` rather than `Context::topology(shape)?`; do not let the ported query path silently inherit OCCT topology fallback.
2. Keep returning real `Shape` handles where the public API requires handles, but isolate that OCCT handle materialization behind an explicit supported-topology validation step and return `Ok(None)` for unsupported topology so the outer fallback is visible and bounded.
3. Strengthen `document_workflows` and/or `selector_workflows` around face/edge selector behavior that is already descriptor-backed through `BrepShape`, so regressions toward raw `subshapes_occt()` traversal have a user-visible test.
4. If this tightening exposes a supported selector/report path that still depends on raw OCCT face/edge enumeration, replace that path with `BrepShape` descriptors in the same turn instead of weakening the guard.

## Guardrails

- Read `portingMilestones.md` and `nextStep.md` at the start of the next turn before editing.
- Do not reintroduce `face_bboxes_occt()`, `OffsetFaceBboxSource::OcctFaceUnion`, `offset_shape_bbox_occt()`, or `SummaryBboxSource::OffsetOcctSubshapeUnion`.
- Do not weaken the new `unsupported_bbox_summary_fallback_allowed()` or `unsupported_volume_summary_fallback_allowed()` guards unless the same turn lands a Rust-owned replacement that keeps supported summaries off `OcctFallback`.
- Keep `ModelDocument::edges()`, `ModelDocument::faces()`, `select_edge()`, and `select_face()` centered on `BrepShape`; do not route selectors through `Context::subshapes()` just to get handles.
- Preserve OCCT `Shape` handle materialization only for APIs that must return or consume handles; traversal decisions and counts for supported face/edge families should come from Rust topology.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test recipe_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
