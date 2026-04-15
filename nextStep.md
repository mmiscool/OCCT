# Next Task

Move the route-driven single-face topology helpers out of `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_surface.rs` and into `brep/topology.rs`.

## Focus

- Extract `single_face_topology_with_route()`, `single_face_topology_snapshot()`, `single_face_edge_with_route()`, and `single_face_edge()` into the topology sibling module now that they form a self-contained route-based block.
- Preserve the explicit raw/public behavior split for edge acquisition when moving the helpers; the raw route must stay on `edge_geometry_occt()` plus `PortedCurve::from_context_with_geometry()`, and the public route must keep its Rust-first `edge_geometry()` / `from_context_with_ported_payloads()` fallback behavior.
- Keep `face_surface.rs` focused on face preparation, descriptor selection, swept-surface reconstruction, and area/sample assembly after the extraction.
- Avoid changing the public query entry points or the single-face topology data layout while doing the move.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the extraction.

## Why This Is Next

The raw/public selector cleanup now leaves the single-face topology helpers as one contiguous route-driven block inside `face_surface.rs`. Moving that block into `brep/topology.rs` is the next bounded structural cleanup that further matches code ownership without changing behavior.
