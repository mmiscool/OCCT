
cmake_minimum_required(VERSION 3.18)

project(LeanOcctSubset LANGUAGES C CXX)

set(LEAN_OCCT_VERSION_COMPLETE "8.0.0")
set(LEAN_OCCT_VERSION_EXTENDED "8.0.0.rc5")
set(LEAN_OCCT_VERSION_SOVERSION "8.0")

set(BUILD_LIBRARY_TYPE "" CACHE STRING "Select library type: Shared or Static.")
set_property(CACHE BUILD_LIBRARY_TYPE PROPERTY STRINGS Shared Static)
if(NOT BUILD_LIBRARY_TYPE)
  if(EMSCRIPTEN)
    set(BUILD_LIBRARY_TYPE "Static" CACHE STRING "Select library type: Shared or Static." FORCE)
  else()
    set(BUILD_LIBRARY_TYPE "Shared" CACHE STRING "Select library type: Shared or Static." FORCE)
  endif()
endif()

if(BUILD_LIBRARY_TYPE STREQUAL "Shared")
  set(BUILD_SHARED_LIBS ON)
elseif(BUILD_LIBRARY_TYPE STREQUAL "Static")
  set(BUILD_SHARED_LIBS OFF)
else()
  message(FATAL_ERROR "Unsupported BUILD_LIBRARY_TYPE='${BUILD_LIBRARY_TYPE}'. Use Shared or Static.")
endif()

option(OCCT_SUBSET_USE_TBB "Enable TBB-backed parallel primitives." OFF)
option(BUILD_LEAN_EXCHANGE_SMOKE "Build the lean subset smoke harness." ON)
option(BUILD_WASM_THREEJS_DEMO "Build the wasm Three.js viewer demo." ${EMSCRIPTEN})

set(CMAKE_C_STANDARD 99)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

if(NOT WIN32)
  add_compile_definitions(OCC_CONVERT_SIGNALS)
endif()
add_compile_definitions(OCCT_NO_PLUGINS)
if(NOT BUILD_SHARED_LIBS)
  add_compile_definitions(OCCT_STATIC_BUILD)
endif()

if(OCCT_SUBSET_USE_TBB)
  find_package(TBB REQUIRED COMPONENTS tbb)
  add_compile_definitions(HAVE_TBB)
endif()

if(NOT WIN32 AND NOT EMSCRIPTEN)
  find_package(Threads REQUIRED)
endif()

include(CTest)
include("${CMAKE_CURRENT_SOURCE_DIR}/cmake/LeanOcctSubset.cmake")

lean_occt_detect_layout()
lean_occt_configure_output_directories()

include("${CMAKE_CURRENT_SOURCE_DIR}/src/MODULES.cmake")

set(LEAN_OCCT_ALL_PACKAGE_DIRS)
foreach(OCCT_MODULE IN LISTS OCCT_MODULES)
  include("${CMAKE_CURRENT_SOURCE_DIR}/src/${OCCT_MODULE}/TOOLKITS.cmake")
  foreach(OCCT_TOOLKIT IN LISTS ${OCCT_MODULE}_TOOLKITS)
    lean_occt_collect_package_dirs("${OCCT_MODULE}" "${OCCT_TOOLKIT}" _toolkit_package_dirs)
    list(APPEND LEAN_OCCT_ALL_PACKAGE_DIRS ${_toolkit_package_dirs})
  endforeach()
endforeach()
list(REMOVE_DUPLICATES LEAN_OCCT_ALL_PACKAGE_DIRS)

foreach(OCCT_MODULE IN LISTS OCCT_MODULES)
  foreach(OCCT_TOOLKIT IN LISTS ${OCCT_MODULE}_TOOLKITS)
    lean_occt_add_toolkit("${OCCT_MODULE}" "${OCCT_TOOLKIT}")
  endforeach()
endforeach()

if(BUILD_LEAN_EXCHANGE_SMOKE)
  add_subdirectory(tools/reduction)
endif()

if(BUILD_WASM_THREEJS_DEMO)
  if(NOT EMSCRIPTEN)
    message(FATAL_ERROR "BUILD_WASM_THREEJS_DEMO requires the Emscripten toolchain.")
  endif()
  add_subdirectory(samples/wasm/three_viewer)
endif()
