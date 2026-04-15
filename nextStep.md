# Next Task

Move topology-backed `subshape()` / `subshapes()` onto the Rust-owned path for face, wire, edge, and vertex kinds.

## Focus

- Reevaluate whether `Context::subshape()` and `Context::subshapes()` can mirror the existing Rust-first `subshape_count()` behavior for topology-backed kinds without re-entering the topology builder in the wrong places.
- Preserve explicit `*_occt()` escape hatches and keep internal callers that require raw OCCT traversal on those raw helpers.
- Once the public topology-backed subshape path is Rust-owned, switch `load_ported_face_snapshot()` off raw `subshapes_occt()` for face and wire enumeration.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the change.

## Why This Is Next

The multi-wire planar-face setup in `face_snapshot.rs` now prefers Rust-first face geometry, plane payloads, and curve reconstruction. The next real OCCT traversal boundary in that snapshot path is raw face and wire enumeration, which still goes through `subshapes_occt()` because the public `subshape()` / `subshapes()` APIs are not yet topology-backed.
