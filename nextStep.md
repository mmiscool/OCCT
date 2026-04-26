# Next Task

Current milestone: `M4. Public Query Fallback Cleanup` from `portingMilestones.md`.

## Completed Evidence

- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/shape_queries.rs` now requires `Context::ported_topology(shape)?` before `ported_subshape()`, `ported_subshapes()`, `ported_vertex_point()`, or `ported_edge_endpoints()` claim a Rust-owned result. Unsupported topology returns `Ok(None)` so the public OCCT fallback is explicit at the API boundary instead of being hidden inside `Context::topology()`.
- `rust/lean_occt/src/lib.rs` now makes `Context::subshape_count()` use `ported_topology()` for face, wire, edge, and vertex counts. It calls `subshape_count_occt()` only when no Rust topology snapshot is available.
- `rust/lean_occt/tests/selector_workflows.rs` now pins the selector box to ported topology, asserts public face/edge handle inventories match that topology, and checks `ModelDocument` descriptors/selectors remain aligned with `BrepShape` descriptors carrying ported face surfaces and curves.
- Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test recipe_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml`.
- `M3. Rust-Backed Traversal for Documents and Selectors` is complete. The active work has moved to `M4`.

## Target

Replace or strictly narrow broad public geometry query fallbacks where a Rust descriptor already identifies a supported analytic, swept, or offset kind but the public API can still fall through to an OCCT payload helper.

## Next Bounded Cut

1. In `rust/lean_occt/src/lib.rs`, tighten `edge_line_payload()`, `edge_circle_payload()`, and `edge_ellipse_payload()` so a matching `PortedCurve` returns the Rust-owned payload, a mismatched `Some(PortedCurve::...)` returns an explicit Rust error, and only `None` reaches the OCCT fallback.
2. Apply the same pattern to the primitive face payload accessors backed by `ported_face_surface()`: plane, cylinder, cone, sphere, and torus payloads should not fall through to OCCT after Rust has already identified a different supported surface kind.
3. Strengthen `rust/lean_occt/tests/ported_geometry_workflows.rs` so public payload APIs are asserted against the ported descriptors for supported line/circle/ellipse curves and primitive analytic faces, including at least one mismatched supported-kind request that errors without relying on OCCT.
4. Keep swept and offset payload cleanup bounded for a follow-up if the primitive analytic cut is already large; do not weaken any M2/M3 guards to make public payload tests pass.

## Guardrails

- Read `portingMilestones.md` and `nextStep.md` at the start of the next turn before editing.
- Do not reintroduce `face_bboxes_occt()`, `OffsetFaceBboxSource::OcctFaceUnion`, `offset_shape_bbox_occt()`, or `SummaryBboxSource::OffsetOcctSubshapeUnion`.
- Do not weaken `unsupported_bbox_summary_fallback_allowed()` or `unsupported_volume_summary_fallback_allowed()`.
- Preserve OCCT fallback only for `None`/unsupported descriptor cases in public query APIs. Once a Rust descriptor returns `Some(...)`, mismatched payload requests should fail explicitly in Rust instead of trying another OCCT helper.
- Keep `ModelDocument::edges()`, `ModelDocument::faces()`, `select_edge()`, and `select_face()` centered on `BrepShape`.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
