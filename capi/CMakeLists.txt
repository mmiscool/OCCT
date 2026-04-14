project(LeanOcctCAPI LANGUAGES C CXX)

add_library(LeanOcctCAPI
  include/lean_occt_capi.h
  src/lean_occt_capi.cxx
)

target_include_directories(LeanOcctCAPI
  PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/include
    ${LEAN_OCCT_ALL_PACKAGE_DIRS}
)

target_link_libraries(LeanOcctCAPI
  PUBLIC
    TKSTEPCore
    TKDE
    TKXSBase
    TKFeat
    TKOffset
    TKFillet
    TKHelix
    TKMesh
    TKBool
    TKBO
    TKPrim
    TKShHealing
    TKTopAlgo
    TKGeomAlgo
    TKBRep
    TKGeomBase
    TKG3d
    TKG2d
    TKMath
    TKernel
)

target_compile_features(LeanOcctCAPI PUBLIC cxx_std_17)

if(BUILD_SHARED_LIBS)
  target_compile_definitions(LeanOcctCAPI PRIVATE LEAN_OCCT_CAPI_BUILDING_SHARED)
else()
  target_compile_definitions(LeanOcctCAPI PUBLIC LEAN_OCCT_CAPI_STATIC)
endif()

set_target_properties(LeanOcctCAPI PROPERTIES
  FOLDER "Interop"
  VERSION "${LEAN_OCCT_VERSION_COMPLETE}"
  SOVERSION "${LEAN_OCCT_VERSION_SOVERSION}"
)

add_executable(LeanOcctCapiSmoke
  tests/LeanOcctCapiSmoke.cxx
)

target_include_directories(LeanOcctCapiSmoke PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_link_libraries(LeanOcctCapiSmoke PRIVATE LeanOcctCAPI)
target_compile_features(LeanOcctCapiSmoke PRIVATE cxx_std_17)
target_compile_definitions(LeanOcctCapiSmoke PRIVATE
  LEAN_OCCT_TEST_ARTIFACTS_DIR="${CMAKE_SOURCE_DIR}/test-artifacts/ctest"
)

set_target_properties(LeanOcctCapiSmoke PROPERTIES
  FOLDER "Interop"
)

add_test(NAME LeanOcctCapiSmoke COMMAND LeanOcctCapiSmoke)
