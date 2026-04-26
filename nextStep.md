# Next Task

Current milestone: `M49. Rust-Owned Face UV Bounds Seed Narrowing` from `portingMilestones.md`.

## Completed Evidence

- M49 direct single-face swept seed cut remains complete for exercised edge-source `make_prism()` and `make_revolution()` faces.
- M49 multi-face swept seed cut remains complete for the exercised face-source `make_revolution()` side-face path. Face-source sweeps retain conservative `MultiFaceSweptResult` inventories from Rust-owned ellipse edge geometry while edge-source sweeps keep `SingleFaceSweptResult`.
- The first constructor-owned analytic UV-seed family is now complete for exercised `make_box()` planar faces. `make_box()` emits a `MultiFaceAnalyticResult` inventory with the three OCCT-local planar UV seed families derived from `BoxParams` sizes.
- The next constructor-owned analytic UV-seed family is now complete for exercised `make_cylinder()` side and cap faces. `make_cylinder()` emits a `MultiFaceAnalyticResult` inventory with a Rust-owned periodic cylindrical side seed and a shared planar cap seed derived from `CylinderParams`; all three generated faces attach validated `AnalyticSurfaceFaceMetadata`.
- The next constructor-owned analytic UV-seed family is now complete for exercised `make_cone()` side and cap faces. `make_cone()` emits a `MultiFaceAnalyticResult` inventory with a Rust-owned periodic conical side seed derived from constructor slant height and positive-radius planar cap seeds derived from `ConeParams`; all three generated faces attach validated `AnalyticSurfaceFaceMetadata`.
- The next constructor-owned analytic UV-seed family is now complete for exercised `make_sphere()` faces. `make_sphere()` emits a `MultiFaceAnalyticResult` inventory with one Rust-owned periodic spherical seed derived from `SphereParams`; the generated face attaches validated `AnalyticSurfaceFaceMetadata`.
- The remaining exercised primitive constructor-owned analytic UV-seed family is now complete for `make_torus()` faces. `make_torus()` emits a `MultiFaceAnalyticResult` inventory with one Rust-owned doubly periodic torus seed derived from `TorusParams`; the generated face attaches validated `AnalyticSurfaceFaceMetadata`.
- Ported topology now propagates analytic inventories through root/shell/solid loading, validates each generated face against candidate analytic seeds, and attaches `AnalyticSurfaceFaceMetadata` only when exactly one seed matches normalized sample positions and oriented normals.
- `Context::ported_face_geometry()` consumes offset metadata, swept metadata, and analytic metadata before `face_uv_bounds_occt(shape)`. Imported, unsupported, metadata-free, and ambiguous faces remain on the existing raw bounds path.
- Regression coverage now includes all six faces of a non-cubic `make_box()` result, asserting Rust analytic metadata, plane geometry parity, descriptor classification, and normalized sample parity against OCCT. The source guard now covers constructor metadata before raw bounds for swept and analytic helpers.
- Regression coverage now also includes all three faces of a non-unit-axis `make_cylinder()` result, asserting Rust analytic metadata, cylinder/cap geometry parity, descriptor classification, and normalized sample parity against OCCT.
- Regression coverage now also includes all three faces of a truncated `make_cone()` result, asserting Rust analytic metadata, cone/cap geometry parity, descriptor classification, and normalized sample parity against OCCT.
- Regression coverage now also includes the generated face of a non-unit-axis `make_sphere()` result, asserting Rust analytic metadata, sphere geometry parity, descriptor classification, and normalized sample parity against OCCT.
- Regression coverage now also includes the generated face of a non-unit-axis `make_torus()` result, asserting Rust analytic metadata, torus geometry parity, descriptor classification, and normalized sample parity against OCCT.

## Target

Keep narrowing the remaining M49 raw bounds fallback beyond swept faces, box planes, cylinder side/cap faces, cone side/cap faces, sphere faces, and torus faces.

`face_uv_bounds_occt(shape)` still seeds metadata-free analytic/swept candidates, imported faces, unsupported faces, and ambiguous faces whose constructor inventories could not be uniquely validated. The next useful cut is to strictly narrow metadata-bearing faces so a present offset, swept, or analytic metadata record cannot silently fall through to raw UV bounds after Rust validation fails.

## Next Bounded Cut

1. Read `portingMilestones.md` and `nextStep.md` before editing.
2. Split `Context::ported_face_geometry()` metadata checks so offset, swept, and analytic metadata-bearing faces return their Rust-owned geometry or an explicit miss instead of continuing to `face_uv_bounds_occt(shape)`.
3. Keep imported, unsupported, metadata-free, and ambiguous faces on the existing raw bounds path.
4. Strengthen source coverage so the raw bounds call remains reachable only after metadata-bearing branches have been excluded, without weakening the swept or analytic metadata guards.
5. Update both control files with completed evidence, active milestone, next bounded cut, and exact verification commands.

## Guardrails

- Do not reintroduce `face_geometry_occt(shape)` inside `Context::ported_face_geometry()`.
- Keep offset metadata first, swept metadata before `face_uv_bounds_occt(shape)`, analytic metadata before the raw bounds fallback, analytic raw-bounds candidates before the generic swept recognizer.
- Do not attach swept or analytic metadata to metadata-free, imported, invalid, unsupported, or ambiguous faces.
- Preserve explicit raw/oracle face geometry, UV bounds, swept payload, offset payload, basis, and sampling APIs.
- Keep direct swept, multi-face swept, box-plane analytic metadata, cylinder analytic metadata, cone analytic metadata, sphere analytic metadata, torus analytic metadata, public payload, swept BRep solid, swept-offset metadata, multi-source swept offset, offset-solid volume, source-guard, and full-suite regressions green.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml ported_torus_faces_use_rust_analytic_seed_metadata -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml ported_sphere_faces_use_rust_analytic_seed_metadata -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml ported_cone_faces_use_rust_analytic_seed_metadata -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml ported_cylinder_faces_use_rust_analytic_seed_metadata -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml ported_box_plane_faces_use_rust_analytic_seed_metadata -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml ported_face_geometry_classifies_constructor_metadata_before_raw_geometry -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml ported_face_surface_descriptors_cover_supported_faces -- --nocapture`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- `cmake --build build --target LeanOcctCAPI`
- `git diff --check`
