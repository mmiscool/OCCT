# Rust Port Strategy

The port should advance by Rust-owned capabilities, not by class-by-class translation of OCCT.

Most files under `rust/lean_occt/src/occt_port/` are useful as a package map, but many are placeholders. Real progress is measured in the exercised Rust kernel slice: geometry descriptors and evaluators, BRep snapshots, summaries, selectors, documents, pipelines, C ABI glue, and integration tests.

## OCCT Roles

OCCT should be used in three explicit roles:

- Constructor backend for operations not yet implemented natively in Rust.
- Snapshot producer for normalized topology and geometry data that Rust can own downstream.
- Test oracle for parity checks and unsupported/raw escape hatches.

Any automatic OCCT query path outside those roles is porting debt unless it is isolated behind an explicitly named unsupported/imported/raw API.

## Progress Unit

Prefer vertical shape-family ownership over isolated fallback removal. A porting turn should move one authored or exercised family closer to Rust ownership across several of these layers:

- retained Rust construction metadata
- normalized topology/geometry snapshot
- Rust geometry payload descriptors
- Rust BRep materialization
- bbox, area, volume, edge-length, and sampling queries
- selectors, documents, pipeline workflows, examples, and tests

Fallback narrowing still matters, but it should be attached to a user-visible ownership row. Do not spend a turn only deleting a small fallback if the same change does not make a shape family more Rust-owned end to end.

## Priority Families

Work on authored analytic and swept families before imported freeform geometry:

- box and planar faces
- cylinder, cone, sphere, torus, and ellipse/line/circle edges
- prism and revolution faces
- direct and generated offset faces
- simple shells and solids assembled from the above

Keep booleans on the C++ backend until the Rust-owned representation and inspection layer is strong enough. For boolean outputs, prefer importing a normalized snapshot and then owning downstream queries in Rust.

## Snapshot Direction

The high-leverage target is a normalized Rust-owned BRep/geometry snapshot. It should carry enough data for Rust to answer downstream queries without repeated raw OCCT calls:

- root kind, subshape identities, counts, and orientations
- vertices, edges, wires, faces, shells, adjacency, and loop roles
- locations/transforms and tolerances when needed
- curve and surface kind, bounds, payloads, pcurves, and source metadata
- authored-shape metadata tying generated faces back to Rust operations

The C ABI may still produce this snapshot while Rust owns interpretation, validation, and higher-level behavior.

## Guardrails

- Do not translate placeholder OCCT package files just because they are easy to touch.
- Do not bind broad C++ classes directly as the main porting strategy.
- Keep raw OCCT helpers available as explicit oracle and unsupported APIs.
- Keep regression coverage around user-visible behavior, not only source-grep guards.
- If a shape family is declared supported, missing Rust metadata or snapshot data should fail explicitly instead of silently falling back to OCCT queries.
