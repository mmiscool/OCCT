# Lean OCCT Reduction Notes

This repository is already the reduced OCCT tree. The retained boundary is focused on:

- authoring topology and geometry
- generalized boolean operations
- feature construction, fillets, offsets, and helix generation
- tessellation for the wasm demo
- direct STEP import/export through `STEPControl`

It deliberately excludes:

- Draw
- OCCT visualization toolkits
- XCAF document/view/material layers
- CAF-based STEP provider layers
- IGES translation
- non-target data exchange formats
- HLR, mesh exchange, and expression toolkits

## Configure

```bash
cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Release
```

For the wasm sample, add the Emscripten toolchain and keep `BUILD_WASM_THREEJS_DEMO=ON`.

## Verify The Boundary

Build the native smoke target that exercises the intended keep set directly:

```bash
cmake --build build --target LeanExchangeSmoke -j 8
ctest --test-dir build --output-on-failure -R LeanExchangeSmoke
```

`LeanExchangeSmoke` covers the current reduction contract:

- primitive authoring with `BRepPrimAPI`
- generalized boolean cut with `BRepAlgoAPI`
- representative authoring operations with `BRepFilletAPI`, `BRepOffsetAPI`, `BRepFeat`, and `HelixBRep`
- tessellation with `BRepMesh_IncrementalMesh`
- direct STEP round-trip with `STEPControl`

## Resulting Keep Set

- `TKernel`
- `TKMath`
- `TKG2d`
- `TKG3d`
- `TKGeomBase`
- `TKBRep`
- `TKGeomAlgo`
- `TKTopAlgo`
- `TKPrim`
- `TKBO`
- `TKBool`
- `TKHelix`
- `TKFillet`
- `TKOffset`
- `TKFeat`
- `TKMesh`
- `TKShHealing`
- `TKDE`
- `TKXSBase`
- `TKSTEPCore`

## Exchange Split

The stock `TKDESTEP` toolkit mixes direct control APIs with CAF/XCAF provider code.
This profile keeps the direct STEP control path and strips the CAF-dependent pieces:

- `TKSTEPCore` removes `STEPCAFControl`, the `DESTEP_Provider` and `DESTEP_ConfigurationNode` glue, and STEP style/material helpers.

This is the reduction boundary to preserve if the next step is a Rust port of the core BREP
authoring / boolean / STEP stack.

## Pruning Workflow

Use this loop for each next reduction pass:

1. Remove one candidate package, helper layer, or toolkit outside the retained modeling/BREP core.
2. Reconfigure and rebuild `LeanExchangeSmoke`.
3. Rebuild the wasm viewer if the cut might affect meshing or topology traversal.
4. Keep the removal only if both still pass.

That keeps the reduction process measurable and prevents non-authoring or non-STEP
layers from creeping back in without hollowing out the intended geometry boundary.
