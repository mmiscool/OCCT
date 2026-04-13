# OCCT Reduction Profile

`LeanAuthoringExchange` is a first reduction pass for OCCT focused on:

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
cmake -S . -B build-lean \
  -G Ninja \
  -DOCCT_REDUCTION_PROFILE=LeanAuthoringExchange
```

For the wasm sample, add the Emscripten toolchain and keep `BUILD_WASM_THREEJS_DEMO=ON`.

## Verify The Boundary

Build the native smoke target that exercises the intended keep set directly:

```bash
cmake --build build-lean --target LeanExchangeSmoke -j 8
ctest --test-dir build-lean --output-on-failure -R LeanExchangeSmoke
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

1. Remove one candidate toolkit or package outside the retained modeling/BREP core from `adm/cmake/occt_reduction_profile.cmake`.
2. Reconfigure and rebuild `LeanExchangeSmoke`.
3. Rebuild the wasm viewer if the cut might affect meshing or topology traversal.
4. Keep the removal only if both still pass.

That keeps the reduction process measurable and prevents visualization-era or document-layer
dependencies from creeping back in without hollowing out the authoring boundary.

## Export A Standalone Subset

Generate a physically reduced source tree under `subsets/lean-authoring-step`:

```bash
python3 tools/reduction/export_subset.py --force
```

That export keeps the retained authoring/boolean/STEP stack, rewrites `src/MODULES.cmake`
and module `TOOLKITS.cmake` files for the narrower tree, and carries `LeanExchangeSmoke`
forward as the verification target inside the exported subset. The generated
`README.md` and `subset-manifest.json` also record the retained toolkit list,
STEP donor packages, and reduction stats so the subset can be rebuilt and
measured outside the full OCCT tree. Generated caches such as `node_modules`
and `__pycache__` are excluded from the export.
