# Next Task

Move the topology-driven BRep materialization prefix out of `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep.rs` and into `brep/topology.rs`.

## Focus

- Extract the vertex, wire, and edge construction block at the start of `Context::ported_brep()` into topology-owned helper functions in `brep/topology.rs`.
- Keep `brep.rs` responsible for the top-level `ported_brep()` orchestration, face assembly, and final summary wiring, but not for the raw topology-to-BRep projection.
- Preserve the current Rust-first edge geometry and curve reconstruction path while moving that code behind the topology helper boundary.
- Leave behavior unchanged and keep `cargo check --manifest-path rust/lean_occt/Cargo.toml` and `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows` passing after the move.

## Why This Is Next

The root-kind topology query helpers now live in `brep/topology.rs`, and the next largest topology-owned slice still sitting in `brep.rs` is the prefix of `ported_brep()` that turns a `TopologySnapshot` plus edge shapes into `BrepVertex`, `BrepWire`, and `BrepEdge` records. Moving that chunk continues shrinking the parent module toward orchestration-only code without mixing it with the face-surface and summary logic that still belongs elsewhere.
