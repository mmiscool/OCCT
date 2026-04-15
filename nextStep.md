# Next Task

Move the multi-wire planar-face setup in `face_snapshot.rs` onto the Rust-first face payload path.

## Focus

- Reevaluate whether `PreparedFaceTopologyBuilder::build()` can use Rust-first face geometry and plane payload reconstruction before falling back to the explicit OCCT helpers.
- Preserve the existing planar wire area behavior, including oriented-edge reconstruction, sampled-point fallback, and magnitude-based outer/inner loop classification.
- Keep root-wire matching, per-face wire preload behavior, and packed snapshot output unchanged.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the change.

## Why This Is Next

The one-use `planar_wire_area_magnitude()` helper is now gone, and the planar wire area path already prefers Rust-owned curve reconstruction for its per-edge segments. The next bounded Rust-port step in this same snapshot flow is to stop seeding multi-wire planar-face setup from raw `face_plane_payload_occt()` and `face_geometry_occt()` when the Rust-owned face payload path can answer first.
