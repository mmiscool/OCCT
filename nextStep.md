# Next Task

Current milestone: `M49. Rust-Owned Face UV Bounds Seed Narrowing` from `portingMilestones.md`.

## Completed Evidence

- `M48. Rust-Owned Swept Face Geometry Classifier Narrowing` is complete.
- `Context::ported_face_geometry()` no longer calls `face_geometry_occt(shape)` at all. The old post-raw `SurfaceKind::Revolution | SurfaceKind::Extrusion` branch and `PortedFaceSurface::Swept` descriptor dispatch are removed.
- Exercised extrusion and revolution faces now classify through `ported_swept_face_geometry_candidate()` before any raw face-geometry classifier. The helper seeds candidate swept geometry, validates the Rust-owned swept descriptor from samples, and rebuilds closed/periodic flags from the selected swept basis curve and swept family.
- Analytic candidates run before swept candidates, so analytic faces still claim planes, cylinders, cones, spheres, and tori before the generic swept recognizer can treat them as swept-like surfaces.
- `ported_face_geometry_classifies_swept_before_raw_geometry` guards the source and fails if `ported_face_geometry()` regains a raw `face_geometry_occt(shape)` call, a post-raw swept kind branch, or raw swept descriptor dispatch.
- Verification passed with `(cd rust/lean_occt && cargo fmt)`, `cmake --build build --target LeanOcctCAPI`, `(cd rust/lean_occt && cargo check)`, focused swept geometry/public payload/BRep swept/offset regressions, source guards proving the raw face-geometry classifier is absent from `ported_face_geometry()`, full `(cd rust/lean_occt && cargo test)`, and `git diff --check`.

## Target

Narrow the remaining OCCT-backed bounds seed in `Context::ported_face_geometry()`.

After M48, the raw kind classifier is gone, but this universal bounds read remains before analytic and swept candidates:

```rust
let bounds = self.face_uv_bounds_occt(shape)?;
```

`face_uv_bounds_occt(shape)` still uses OCCT adaptor geometry to provide the parameter bounds that seed supported Rust-owned analytic and swept classification.

## Next Bounded Cut

1. Read `portingMilestones.md` and `nextStep.md` before editing.
2. Start with constructor-owned swept faces from `make_prism()` and `make_revolution()`.
3. Carry or derive Rust-owned UV bounds/geometry seeds through swept face enumeration for exercised extrusion and revolution faces.
4. Use those seeds in `ported_swept_face_geometry_candidate()` before falling back to `face_uv_bounds_occt(shape)`.
5. Strictly narrow `face_uv_bounds_occt(shape)` to metadata-missing, imported, or unsupported faces, not the normal exercised swept path.
6. Strengthen source or workflow coverage so constructor-owned swept faces fail if their geometry classification silently needs the raw bounds seed again.
7. Update both control files with completed evidence, active milestone, next bounded cut, and exact verification commands.

## Guardrails

- Do not reintroduce `face_geometry_occt(shape)` inside `Context::ported_face_geometry()`.
- Do not classify metadata-free, imported, invalid, or unsupported swept-like faces as Rust-owned unless a Rust descriptor/topology path proves support.
- Keep analytic-first ordering before the generic swept recognizer to avoid misclassifying planes, cylinders, cones, spheres, or tori as swept.
- Preserve explicit raw/oracle face geometry, UV bounds, swept payload, offset payload, basis, and sampling APIs.
- Keep direct swept, swept BRep solid, swept-offset metadata, multi-source swept offset, and offset-solid volume regressions green.

## Verification

- `(cd rust/lean_occt && cargo fmt)`
- `cmake --build build --target LeanOcctCAPI`
- `(cd rust/lean_occt && cargo check)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows ported_face_geometry_classifies_swept_before_raw_geometry -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows ported_swept_surface_sampling_matches_occt -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows public_swept_and_offset_payload_queries_match_occt -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test brep_workflows ported_brep_summarizes_swept_revolution_solids_in_rust -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test brep_workflows ported_brep_maps_multi_source_swept_offsets_in_rust -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test brep_workflows ported_brep_uses_rust_owned_volume_for_offset_solids -- --nocapture)`
- A source guard proving constructor-owned swept geometry uses a Rust bounds seed before `face_uv_bounds_occt(shape)`.
- `(cd rust/lean_occt && cargo test)`
- `git diff --check`
