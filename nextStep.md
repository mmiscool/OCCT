# Next Task

Current milestone: `M43. Rust-Owned Public Face Payload Mismatch Gates` from `portingMilestones.md`.

## Completed Evidence

- `M42. Rust-Owned Strict BRep Face Inventory Support Gate` is complete.
- `rust_owned_face_query_required_kind()` is now crate-internal and reused by `strict_brep_face_inventory_requires_ported_topology()`.
- The strict BRep face inventory loop no longer calls `context.face_geometry_occt(&face_shape)?` as a support classifier; each enumerated face is classified through the Rust face descriptor/topology inventory.
- The old duplicated `strict_brep_supported_surface_kind()` matcher was removed.
- Face-count mismatch behavior is unchanged and still returns `false`, preserving the existing unsupported/incomplete inventory escape hatch.
- `strict_brep_face_inventory_gate_uses_rust_face_support` now directly covers analytic multi-face, swept extrusion, direct offset surface, and count-mismatch behavior.
- The strict inventory source guard passed: no `face_geometry_occt(` call remains inside `strict_brep_face_inventory_requires_ported_topology()`.
- Verification passed with `(cd rust/lean_occt && cargo fmt)`, `cmake --build build --target LeanOcctCAPI`, `(cd rust/lean_occt && cargo check)`, the new strict inventory unit test, the focused BRep/ported-geometry workflow tests, the strict inventory source guard, full `(cd rust/lean_occt && cargo test)`, and `git diff --check`.

## Target

Remove the next raw face geometry support/mismatch classifiers in public face payload queries:

`Context::ported_analytic_face_surface_payload()` and `Context::ported_swept_face_surface_payload()` in `rust/lean_occt/src/lib.rs` currently call `self.face_geometry_occt(shape)?` after the Rust descriptor path returns `None`. Those calls decide kind mismatch or support for public plane/cylinder/cone/sphere/torus and revolution/extrusion payload queries.

The public payload helpers should use descriptor-backed Rust support/kind decisions instead of direct raw face geometry. Explicit `*_payload_occt()` methods should remain available as opt-in oracle APIs.

## Next Bounded Cut

1. Read `portingMilestones.md` and `nextStep.md` before editing.
2. Inspect `ported_analytic_face_surface_payload()` and `ported_swept_face_surface_payload()` in `rust/lean_occt/src/lib.rs`.
3. Replace the two `face_geometry_occt()` support/mismatch reads with non-recursive Rust descriptor-backed classification.
4. Preserve explicit mismatch errors for wrong supported face kinds where the Rust descriptor path can identify the actual kind.
5. Preserve unsupported-family behavior without adding new Rust-required families in the same cut.
6. Keep explicit raw `*_payload_occt()` APIs available for tests and oracle use.
7. Strengthen public analytic/swept payload regression coverage if existing workflow assertions do not fail on this fallback family.
8. Add source guards proving both helper bodies contain no `face_geometry_occt(` call.
9. Update both control files with completed evidence, active milestone, next bounded cut, and exact verification commands.

## Guardrails

- Do not reintroduce direct `face_geometry_occt()` into the public face query gate block or strict BRep inventory gate.
- Avoid routing these helpers through public `face_geometry()` if that would recurse into the same support-gate behavior.
- Keep explicit `Context::face_geometry_occt()`, `face_sample_occt()`, `face_sample_normalized_occt()`, and public `*_payload_occt()` APIs available as raw oracle APIs.
- Do not widen strict Rust-required support to unsupported face families unless the same turn ports their geometry and payload behavior.
- Preserve public payload mismatch behavior as explicit Rust errors where supported descriptors expose the actual kind.

## Verification

- `(cd rust/lean_occt && cargo fmt)`
- `cmake --build build --target LeanOcctCAPI`
- `(cd rust/lean_occt && cargo check)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows public_analytic_curve_and_surface_payload_queries_match_occt -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows public_swept_and_offset_payload_queries_match_occt -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows ported_face_surface_descriptors_cover_supported_faces -- --nocapture)`
- `! awk '/fn ported_analytic_face_surface_payload/,/^    }/' rust/lean_occt/src/lib.rs | rg -n 'face_geometry_occt\('`
- `! awk '/fn ported_swept_face_surface_payload/,/^    }/' rust/lean_occt/src/lib.rs | rg -n 'face_geometry_occt\('`
- `(cd rust/lean_occt && cargo test)`
- `git diff --check`
