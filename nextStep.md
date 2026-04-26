# Next Task

Current milestone: `M42. Rust-Owned Strict BRep Face Inventory Support Gate` from `portingMilestones.md`.

## Completed Evidence

- `M41. Rust-Owned Public Face Query Support Gates` is complete.
- `Context::face_sample()`, `face_sample_normalized()`, and `face_geometry()` now use `rust_owned_face_query_required_kind()` after a ported query miss instead of calling `face_geometry_occt()` as a support classifier.
- `rust_owned_face_query_required_kind()` is non-recursive: it calls `Context::ported_face_geometry()` and `brep::ported_face_surface_descriptor()` directly, so supported analytic, swept, and offset face ownership is identified through the Rust face descriptor/topology inventory rather than public `face_geometry()` recursion.
- Explicit raw face sample and geometry APIs remain available for unsupported fallback/oracle use. Public unsupported raw geometry fallback is routed through `raw_face_geometry()` after `face_geometry_occt()` itself, while sample fallbacks still call the explicit raw sample APIs.
- `ported_face_surface_descriptors_cover_supported_faces` now verifies public face geometry matches ported face geometry across the supported analytic, swept, direct-offset, and generated-offset descriptor matrix before sampling checks run.
- The public face query source guard passed: no `face_geometry_occt(` call remains between `face_sample()` and `face_geometry_occt()`.
- Verification passed with `(cd rust/lean_occt && cargo fmt)`, `cmake --build build --target LeanOcctCAPI`, `(cd rust/lean_occt && cargo check)`, the focused M41 public face/BRep workflow tests, the public face source guard, full `(cd rust/lean_occt && cargo test)`, and `git diff --check`.

## Target

Remove the next raw face geometry support classifier in strict BRep materialization:

`strict_brep_face_inventory_requires_ported_topology()` in `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep.rs` currently calls `context.face_geometry_occt(&face_shape)?` only to decide whether a face inventory contains supported analytic, swept, or offset faces that must require Rust-owned topology.

The strict BRep gate should use the Rust-owned face support classifier backed by face geometry descriptors instead of raw face geometry. Unsupported or incomplete inventories should continue through the existing unsupported-shape escape hatch.

## Next Bounded Cut

1. Read `portingMilestones.md` and `nextStep.md` before editing.
2. Reuse or expose the M41 Rust-owned face support classifier for crate-internal strict BRep checks.
3. Replace the `face_geometry_occt()` support-classifier call in `strict_brep_face_inventory_requires_ported_topology()`.
4. Preserve the current face-count validation and unsupported-inventory behavior.
5. Preserve strict Rust-required topology behavior for supported analytic, swept, and offset face inventories.
6. Strengthen strict BRep topology regression coverage if existing tests do not fail on this fallback family.
7. Add a source guard proving the strict inventory function contains no `face_geometry_occt(` call.
8. Update both control files with completed evidence, active milestone, next bounded cut, and exact verification commands.

## Guardrails

- Do not reintroduce direct `face_geometry_occt()` into the public face query gate block, and keep the M41 public source guard clean.
- Do not call public `face_geometry()` from the strict support classifier if that would recurse into support-gate behavior; use the non-recursive descriptor-backed path.
- Keep explicit `Context::face_geometry_occt()`, `face_sample_occt()`, and `face_sample_normalized_occt()` available as raw oracle APIs.
- Keep unsupported face kinds out of the Rust-required strict BRep family unless the same turn also ports their geometry/topology/materialization behavior.
- Preserve existing strict BRep behavior for unsupported or incomplete inventories that legitimately fall through to the unsupported-shape path.

## Verification

- `(cd rust/lean_occt && cargo fmt)`
- `cmake --build build --target LeanOcctCAPI`
- `(cd rust/lean_occt && cargo check)`
- `(cd rust/lean_occt && cargo test --test brep_workflows supported_brep_materialization_requires_ported_topology -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test brep_workflows ported_brep_uses_rust_owned_topology_for_simple_single_face_shapes -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test brep_workflows ported_brep_uses_rust_owned_topology_for_simple_multi_face_solids -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows ported_face_surface_descriptors_cover_supported_faces -- --nocapture)`
- `! awk '/fn strict_brep_face_inventory_requires_ported_topology/,/^}/' rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep.rs | rg -n 'face_geometry_occt\('`
- `(cd rust/lean_occt && cargo test)`
- `git diff --check`
