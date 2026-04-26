# Next Task

Current milestone: `M41. Rust-Owned Public Face Query Support Gates` from `portingMilestones.md`.

## Completed Evidence

- `M40. Rust-Owned Public Edge Query Support Gates` is complete.
- `Context::edge_endpoints()`, `edge_sample()`, `edge_sample_at_parameter()`, and `edge_geometry()` now use `rust_owned_edge_query_required_kind()` after a ported query miss instead of calling `edge_geometry_occt()` as a support classifier.
- `rust_owned_edge_query_required_kind()` is backed by `brep::ported_root_edge_geometry()`, so supported root line, circle, and ellipse ownership comes from the Rust-owned root-edge topology/geometry inventory.
- Explicit raw edge APIs remain available for unsupported fallback/oracle use. Public unsupported geometry fallback is routed through `raw_edge_geometry()` after `edge_geometry_occt()` itself, while endpoint/sample fallbacks still call the explicit raw endpoint/sample APIs.
- `unsupported_root_edge_does_not_use_generic_raw_topology_inventory` now verifies unsupported helix/root-edge public geometry, endpoints, normalized sampling, and parameter sampling all match the explicit raw oracle path while ported topology/geometry/endpoints stay `None`.
- The public edge query source guard passed: no `edge_geometry_occt(` call remains between `edge_endpoints()` and `edge_geometry_occt()`.
- Verification passed with `(cd rust/lean_occt && cargo fmt)`, `cmake --build build --target LeanOcctCAPI`, `(cd rust/lean_occt && cargo check)`, the focused M40 public edge/BRep workflow tests, the public edge source guard, full `(cd rust/lean_occt && cargo test)`, and `git diff --check`.

## Target

Remove the next public raw face geometry support classifiers:

`Context::face_sample()`, `face_sample_normalized()`, and `face_geometry()` in `rust/lean_occt/src/lib.rs` currently call `face_geometry_occt()` after their ported query path returns `None` only to decide whether an analytic, swept, or offset face should require Rust-owned behavior.

The public face query gates should use a non-recursive Rust-owned face support classifier backed by the existing face descriptor/topology inventory instead of raw face geometry. The explicit raw `_occt` operations should remain available only as actual unsupported fallback/oracle calls.

## Next Bounded Cut

1. Read `portingMilestones.md` and `nextStep.md` before editing.
2. Add or expose a non-recursive Rust-owned public face query support classifier backed by face descriptor/topology support.
3. Replace the `face_geometry_occt()` support-classifier calls in `face_sample()`, `face_sample_normalized()`, and `face_geometry()`.
4. Preserve Rust-required behavior for supported analytic, swept, and offset face geometry and samples.
5. Preserve unsupported face public query fallback through explicit raw `_occt` operations.
6. Strengthen regression coverage if the existing supported face sample/geometry and unsupported fallback tests do not fail on this fallback family.
7. Add a source guard proving the public face query gate block contains no `face_geometry_occt(` call before `face_geometry_occt()` itself.
8. Update both control files with completed evidence, active milestone, next bounded cut, and exact verification commands.

## Guardrails

- Do not reintroduce direct `edge_geometry_occt()` into public edge query gates, `ported_edge_geometry()`, root-edge topology bootstrap, or the strict BRep root-edge gate.
- Do not call public `face_geometry()` from the new public face support classifier; that would recurse through the gate being replaced.
- Keep explicit `Context::face_geometry_occt()`, `face_sample_occt()`, and `face_sample_normalized_occt()` available as raw oracle APIs.
- Keep unsupported face kinds out of the Rust-required family unless the same turn also ports their geometry/topology/query behavior.
- Preserve face descriptor, BRep materialization, public geometry, sample, payload, and raw-oracle parity behavior.

## Verification

- `(cd rust/lean_occt && cargo fmt)`
- `cmake --build build --target LeanOcctCAPI`
- `(cd rust/lean_occt && cargo check)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows ported_face_surface_descriptors_cover_supported_faces -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows ported_surface_sampling_matches_occt -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows ported_swept_surface_sampling_matches_occt -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows ported_offset_surface_sampling_matches_occt -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test brep_workflows ported_brep_uses_rust_owned_topology_for_simple_single_face_shapes -- --nocapture)`
- `! perl -0ne 'print $1 if /(pub fn face_sample[\s\S]*?)\n    pub fn face_geometry_occt/' rust/lean_occt/src/lib.rs | rg -n 'face_geometry_occt\('`
- `(cd rust/lean_occt && cargo test)`
- `git diff --check`
