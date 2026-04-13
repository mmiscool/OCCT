#!/usr/bin/env python3
"""Export the lean OCCT subset as a standalone source tree.

This exporter keeps the full retained modeling / boolean / STEP authoring
boundary while stripping the stock OCCT build scaffolding from the subset.
The generated tree contains only:

- retained package directories for the selected toolkits
- a generated top-level CMake build
- the reduction smoke harness
- the wasm demo
- licensing and subset metadata
"""

from __future__ import annotations

import argparse
import datetime as dt
import json
import re
import shlex
import shutil
import textwrap
from collections import OrderedDict
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
DEFAULT_SUBSET_ROOT = REPO_ROOT / "subsets" / "lean-authoring-step"
REDUCTION_PROFILE = "LeanAuthoringExchange"

TOP_LEVEL_COPY_FILES = (
    "LICENSE_LGPL_21.txt",
    "OCCT_LGPL_EXCEPTION.txt",
)

COPY_IGNORED_NAMES = {
    "__pycache__",
    "node_modules",
    ".DS_Store",
    ".vite",
    ".vite-temp",
}
COPY_IGNORED_SUFFIXES = {
    ".pyc",
    ".pyo",
}

MODULE_TOOLKITS = OrderedDict(
    [
        ("FoundationClasses", ["TKernel", "TKMath"]),
        ("ModelingData", ["TKG2d", "TKG3d", "TKGeomBase", "TKBRep"]),
        (
            "ModelingAlgorithms",
            [
                "TKGeomAlgo",
                "TKTopAlgo",
                "TKPrim",
                "TKBO",
                "TKBool",
                "TKHelix",
                "TKFillet",
                "TKOffset",
                "TKFeat",
                "TKMesh",
                "TKShHealing",
            ],
        ),
        ("DataExchange", ["TKDE", "TKXSBase", "TKSTEPCore"]),
    ]
)

STEP_DONOR_PACKAGES = [
    ("DESTEPCore", "DESTEP", {"DESTEP_ConfigurationNode.cxx", "DESTEP_ConfigurationNode.hxx", "DESTEP_Provider.cxx", "DESTEP_Provider.hxx"}),
    ("StepAP214", "StepAP214", set()),
    ("RWStepAP214", "RWStepAP214", set()),
    ("StepAP203", "StepAP203", set()),
    ("RWStepAP203", "RWStepAP203", set()),
    (
        "STEPConstructCore",
        "STEPConstruct",
        {
            "STEPConstruct_RenderingProperties.cxx",
            "STEPConstruct_RenderingProperties.hxx",
            "STEPConstruct_Styles.cxx",
            "STEPConstruct_Styles.hxx",
        },
    ),
    ("STEPEdit", "STEPEdit", set()),
    ("GeomToStep", "GeomToStep", set()),
    ("StepToGeom", "StepToGeom", set()),
    ("StepToTopoDS", "StepToTopoDS", set()),
    ("TopoDSToStep", "TopoDSToStep", set()),
    ("STEPControl", "STEPControl", set()),
    ("STEPSelections", "STEPSelections", set()),
    ("StepAP209", "StepAP209", set()),
    ("RWStepAP242", "RWStepAP242", set()),
    ("StepAP242", "StepAP242", set()),
    ("StepElement", "StepElement", set()),
    ("StepFEA", "StepFEA", set()),
    ("RWStepElement", "RWStepElement", set()),
    ("RWStepFEA", "RWStepFEA", set()),
    ("StepVisual", "StepVisual", set()),
    ("RWStepVisual", "RWStepVisual", set()),
    ("StepDimTol", "StepDimTol", set()),
    ("RWStepDimTol", "RWStepDimTol", set()),
    ("StepKinematics", "StepKinematics", set()),
    ("RWStepKinematics", "RWStepKinematics", set()),
    ("StepBasic", "StepBasic", set()),
    ("RWStepBasic", "RWStepBasic", set()),
    ("StepRepr", "StepRepr", set()),
    ("RWStepRepr", "RWStepRepr", set()),
    ("StepGeom", "StepGeom", set()),
    ("RWStepGeom", "RWStepGeom", set()),
    ("StepShape", "StepShape", set()),
    ("RWStepShape", "RWStepShape", set()),
    ("StepSelect", "StepSelect", set()),
    ("StepData", "StepData", set()),
    ("StepFile", "StepFile", set()),
    ("RWHeaderSection", "RWHeaderSection", set()),
    ("APIHeaderSection", "APIHeaderSection", set()),
    ("HeaderSection", "HeaderSection", set()),
    ("StepTidy", "StepTidy", set()),
]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Export the lean OCCT subset.")
    parser.add_argument(
        "--destination",
        type=Path,
        default=DEFAULT_SUBSET_ROOT,
        help=f"Subset output directory (default: {DEFAULT_SUBSET_ROOT})",
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="Overwrite the destination directory if it already exists.",
    )
    return parser.parse_args()


def parse_cmake_sets(the_path: Path) -> dict[str, list[str]]:
    text = the_path.read_text(encoding="utf-8")
    matches = re.finditer(
        r"(?ms)^\s*set\s*\(\s*([A-Za-z0-9_]+)\s*(.*?)\)\s*$",
        text,
    )
    values: dict[str, list[str]] = {}
    for a_match in matches:
        a_name = a_match.group(1)
        a_body = a_match.group(2).strip()
        values[a_name] = shlex.split(a_body, comments=False, posix=True) if a_body else []
    return values


def copytree_filtered(the_source: Path, the_target: Path) -> None:
    def _ignore(_: str, names: list[str]) -> set[str]:
        ignored = set()
        for a_name in names:
            if a_name in COPY_IGNORED_NAMES:
                ignored.add(a_name)
                continue
            if any(a_name.endswith(a_suffix) for a_suffix in COPY_IGNORED_SUFFIXES):
                ignored.add(a_name)
        return ignored

    shutil.copytree(the_source, the_target, ignore=_ignore)


def ensure_parent(the_path: Path) -> None:
    the_path.parent.mkdir(parents=True, exist_ok=True)


def copy_file(the_source: Path, the_target: Path) -> None:
    ensure_parent(the_target)
    shutil.copy2(the_source, the_target)


def write_text(the_target: Path, the_content: str) -> None:
    ensure_parent(the_target)
    the_target.write_text(the_content, encoding="utf-8")


def write_cmake_set(the_variable: str, the_values: list[str], the_comment: str | None = None) -> str:
    lines: list[str] = []
    if the_comment:
        lines.append(f"# {the_comment}")
    lines.append(f"set({the_variable}")
    for a_value in the_values:
        lines.append(f"  {a_value}")
    lines.append(")")
    return "\n".join(lines) + "\n"


def read_toolkit_packages(the_module: str, the_toolkit: str) -> list[str]:
    a_sets = parse_cmake_sets(REPO_ROOT / "src" / the_module / the_toolkit / "PACKAGES.cmake")
    return a_sets[f"OCCT_{the_toolkit}_LIST_OF_PACKAGES"]


def read_toolkit_externlibs(the_module: str, the_toolkit: str) -> list[str]:
    a_sets = parse_cmake_sets(REPO_ROOT / "src" / the_module / the_toolkit / "EXTERNLIB.cmake")
    return a_sets[f"OCCT_{the_toolkit}_EXTERNAL_LIBS"]


def read_package_files(the_files_cmake: Path, the_package_name: str) -> list[str]:
    a_sets = parse_cmake_sets(the_files_cmake)
    return a_sets[f"OCCT_{the_package_name}_FILES"]


def copy_direct_toolkit(the_subset_src_root: Path, the_module: str, the_toolkit: str) -> dict[str, object]:
    a_source_toolkit_dir = REPO_ROOT / "src" / the_module / the_toolkit
    a_subset_toolkit_dir = the_subset_src_root / the_module / the_toolkit
    a_subset_toolkit_dir.mkdir(parents=True, exist_ok=True)

    a_packages = read_toolkit_packages(the_module, the_toolkit)
    a_externlibs = read_toolkit_externlibs(the_module, the_toolkit)

    copy_file(a_source_toolkit_dir / "PACKAGES.cmake", a_subset_toolkit_dir / "PACKAGES.cmake")
    copy_file(a_source_toolkit_dir / "EXTERNLIB.cmake", a_subset_toolkit_dir / "EXTERNLIB.cmake")

    if (a_source_toolkit_dir / "FILES.cmake").exists():
        copy_file(a_source_toolkit_dir / "FILES.cmake", a_subset_toolkit_dir / "FILES.cmake")

    for a_package in a_packages:
        copytree_filtered(a_source_toolkit_dir / a_package, a_subset_toolkit_dir / a_package)

    return {
        "module": the_module,
        "toolkit": the_toolkit,
        "packages": a_packages,
        "externlibs": a_externlibs,
    }


def rewrite_step_package_files(the_target_dir: Path, the_target_package: str, the_donor_package: str, the_excluded: set[str]) -> list[str]:
    a_source_files = read_package_files(REPO_ROOT / "src" / "DataExchange" / "TKDESTEP" / the_donor_package / "FILES.cmake", the_donor_package)
    a_filtered = [a_file for a_file in a_source_files if a_file not in the_excluded]
    a_content = textwrap.dedent(
        f"""\
        # Source files for {the_target_package} package
        set(OCCT_{the_target_package}_FILES_LOCATION "${{CMAKE_CURRENT_LIST_DIR}}")

        set(OCCT_{the_target_package}_FILES
        """
    )
    for a_file in a_filtered:
        a_content += f"  {a_file}\n"
    a_content += ")\n"
    write_text(the_target_dir / "FILES.cmake", a_content)
    return a_filtered


def copy_step_toolkit(the_subset_src_root: Path) -> dict[str, object]:
    a_module = "DataExchange"
    a_toolkit = "TKSTEPCore"
    a_subset_toolkit_dir = the_subset_src_root / a_module / a_toolkit
    a_subset_toolkit_dir.mkdir(parents=True, exist_ok=True)

    a_package_names = [a_target for a_target, _, _ in STEP_DONOR_PACKAGES]
    a_externlibs = read_toolkit_externlibs(a_module, a_toolkit)

    write_text(
        a_subset_toolkit_dir / "PACKAGES.cmake",
        write_cmake_set(
            "OCCT_TKSTEPCore_LIST_OF_PACKAGES",
            a_package_names,
            "Reduced STEP exchange toolkit focused on direct STEPControl import/export.",
        ),
    )
    copy_file(REPO_ROOT / "src" / a_module / a_toolkit / "EXTERNLIB.cmake", a_subset_toolkit_dir / "EXTERNLIB.cmake")
    write_text(
        a_subset_toolkit_dir / "FILES.cmake",
        "# Source files for TKSTEPCore\nset(OCCT_TKSTEPCore_FILES_LOCATION \"${CMAKE_CURRENT_LIST_DIR}\")\n\nset(OCCT_TKSTEPCore_FILES)\n",
    )

    a_package_metadata: list[dict[str, object]] = []
    for a_target_package, a_donor_package, a_excluded in STEP_DONOR_PACKAGES:
        a_source_dir = REPO_ROOT / "src" / a_module / "TKDESTEP" / a_donor_package
        a_target_dir = a_subset_toolkit_dir / a_target_package
        copytree_filtered(a_source_dir, a_target_dir)
        a_filtered_files = rewrite_step_package_files(a_target_dir, a_target_package, a_donor_package, a_excluded)
        a_package_metadata.append(
            {
                "package": a_target_package,
                "donor_package": a_donor_package,
                "excluded_files": sorted(a_excluded),
                "file_count": len(a_filtered_files),
            }
        )

    return {
        "module": a_module,
        "toolkit": a_toolkit,
        "packages": a_package_names,
        "externlibs": a_externlibs,
        "donor_packages": a_package_metadata,
    }


def write_module_files(the_subset_src_root: Path) -> None:
    write_text(
        the_subset_src_root / "MODULES.cmake",
        write_cmake_set("OCCT_MODULES", list(MODULE_TOOLKITS.keys()), "Retained OCCT modules in the lean subset."),
    )

    for a_module, a_toolkits in MODULE_TOOLKITS.items():
        write_text(
            the_subset_src_root / a_module / "TOOLKITS.cmake",
            write_cmake_set(f"{a_module}_TOOLKITS", a_toolkits, f"Retained toolkits for {a_module}."),
        )


def read_version_values() -> dict[str, str]:
    a_sets = parse_cmake_sets(REPO_ROOT / "adm" / "cmake" / "version.cmake")
    return {
        "major": a_sets["OCC_VERSION_MAJOR"][0],
        "minor": a_sets["OCC_VERSION_MINOR"][0],
        "maintenance": a_sets["OCC_VERSION_MAINTENANCE"][0],
        "development": a_sets["OCC_VERSION_DEVELOPMENT"][0],
    }


def write_standard_version_header(the_subset_src_root: Path, the_version: dict[str, str]) -> None:
    a_target = the_subset_src_root / "FoundationClasses" / "TKernel" / "Standard" / "Standard_Version.hxx"
    a_date = dt.date.today().isoformat()
    if the_version["development"]:
        a_development_define = f'#define OCC_VERSION_DEVELOPMENT "{the_version["development"]}"'
    else:
        a_development_define = "/* #undef OCC_VERSION_DEVELOPMENT */"

    a_content = textwrap.dedent(
        f"""\
        // Created on: {a_date}
        // Copyright (c) 2002-2025 OPEN CASCADE SAS
        //
        // This file is part of Open CASCADE Technology software library.
        //
        // This library is free software; you can redistribute it and/or modify it under
        // the terms of the GNU Lesser General Public License version 2.1 as published
        // by the Free Software Foundation, with special exception defined in the file
        // OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
        // distribution for complete text of the license and disclaimer of any warranty.
        //
        // Alternatively, this file may be used under the terms of Open CASCADE
        // commercial license or contractual agreement.

        #ifndef _Standard_Version_HeaderFile
        #define _Standard_Version_HeaderFile

        #define OCC_VERSION_MAJOR {the_version["major"]}
        #define OCC_VERSION_MINOR {the_version["minor"]}
        #define OCC_VERSION_MAINTENANCE {the_version["maintenance"]}

        {a_development_define}

        #define OCC_VERSION {the_version["major"]}.{the_version["minor"]}
        #define OCC_VERSION_STRING "{the_version["major"]}.{the_version["minor"]}"
        #define OCC_VERSION_COMPLETE "{the_version["major"]}.{the_version["minor"]}.{the_version["maintenance"]}"

        #ifdef OCC_VERSION_DEVELOPMENT
          #define OCC_VERSION_STRING_EXT OCC_VERSION_COMPLETE "." OCC_VERSION_DEVELOPMENT
        #else
          #define OCC_VERSION_STRING_EXT OCC_VERSION_COMPLETE
        #endif

        #define OCC_VERSION_HEX (OCC_VERSION_MAJOR << 16 | OCC_VERSION_MINOR << 8 | OCC_VERSION_MAINTENANCE)

        #endif /* _Standard_Version_HeaderFile */
        """
    )
    write_text(a_target, a_content)


def generate_helper_cmake() -> str:
    return textwrap.dedent(
        """\
        function(lean_occt_detect_layout)
          math(EXPR _compiler_bitness "32 + 32 * (${CMAKE_SIZEOF_VOID_P} / 8)")

          if(WIN32)
            set(_os_with_bit "win${_compiler_bitness}")
          elseif(APPLE)
            set(_os_with_bit "mac${_compiler_bitness}")
          else()
            set(_os_with_bit "lin${_compiler_bitness}")
          endif()

          if(MSVC)
            set(_compiler "vc14")
          elseif(CMAKE_CXX_COMPILER_ID STREQUAL "GNU")
            set(_compiler "gcc")
          elseif(CMAKE_CXX_COMPILER_ID MATCHES "[Cc][Ll][Aa][Nn][Gg]")
            set(_compiler "clang")
          elseif(CMAKE_CXX_COMPILER_ID MATCHES "[Ii][Nn][Tt][Ee][Ll]")
            set(_compiler "icc")
          else()
            set(_compiler "${CMAKE_CXX_COMPILER_ID}")
            string(REPLACE " " "" _compiler "${_compiler}")
          endif()

          set(COMPILER_BITNESS "${_compiler_bitness}" PARENT_SCOPE)
          set(OS_WITH_BIT "${_os_with_bit}" PARENT_SCOPE)
          set(COMPILER "${_compiler}" PARENT_SCOPE)
          set(INSTALL_DIR_BIN "bin" PARENT_SCOPE)
          set(INSTALL_DIR_LIB "lib" PARENT_SCOPE)
          set(OCCT_INSTALL_BIN_LETTER "" PARENT_SCOPE)
          set(LAYOUT_IS_VCPKG OFF PARENT_SCOPE)
        endfunction()

        function(lean_occt_configure_output_directories)
          set(_binary_dir "${CMAKE_BINARY_DIR}/${OS_WITH_BIT}/${COMPILER}/bin")
          set(_library_dir "${CMAKE_BINARY_DIR}/${OS_WITH_BIT}/${COMPILER}/lib")

          set(CMAKE_RUNTIME_OUTPUT_DIRECTORY "${_binary_dir}" PARENT_SCOPE)
          set(CMAKE_LIBRARY_OUTPUT_DIRECTORY "${_library_dir}" PARENT_SCOPE)
          set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY "${_library_dir}" PARENT_SCOPE)

          foreach(_config DEBUG RELEASE RELWITHDEBINFO MINSIZEREL)
            set("CMAKE_RUNTIME_OUTPUT_DIRECTORY_${_config}" "${_binary_dir}" PARENT_SCOPE)
            set("CMAKE_LIBRARY_OUTPUT_DIRECTORY_${_config}" "${_library_dir}" PARENT_SCOPE)
            set("CMAKE_ARCHIVE_OUTPUT_DIRECTORY_${_config}" "${_library_dir}" PARENT_SCOPE)
          endforeach()
        endfunction()

        function(lean_occt_collect_package_dirs the_module the_toolkit the_out_var)
          include("${CMAKE_CURRENT_SOURCE_DIR}/src/${the_module}/${the_toolkit}/PACKAGES.cmake")
          set(_package_var "OCCT_${the_toolkit}_LIST_OF_PACKAGES")
          set(_package_dirs)
          foreach(_package IN LISTS ${_package_var})
            list(APPEND _package_dirs "${CMAKE_CURRENT_SOURCE_DIR}/src/${the_module}/${the_toolkit}/${_package}")
          endforeach()
          set(${the_out_var} ${_package_dirs} PARENT_SCOPE)
        endfunction()

        function(lean_occt_collect_package_files the_module the_toolkit the_out_sources the_out_headers)
          include("${CMAKE_CURRENT_SOURCE_DIR}/src/${the_module}/${the_toolkit}/PACKAGES.cmake")
          set(_package_var "OCCT_${the_toolkit}_LIST_OF_PACKAGES")

          set(_sources)
          set(_headers)
            foreach(_package IN LISTS ${_package_var})
              set(_package_dir "${CMAKE_CURRENT_SOURCE_DIR}/src/${the_module}/${the_toolkit}/${_package}")
              include("${_package_dir}/FILES.cmake")
              set(_files_var "OCCT_${_package}_FILES")

              foreach(_file IN LISTS ${_files_var})
              set(_abs_file "${_package_dir}/${_file}")
              if(_file MATCHES "\\\\.(c|cc|cpp|cxx|mm)$")
                list(APPEND _sources "${_abs_file}")
              elseif(_file MATCHES "\\\\.(h|hpp|hxx|lxx|gxx|pxx|g)$")
                list(APPEND _headers "${_abs_file}")
              endif()
            endforeach()
          endforeach()

          set(${the_out_sources} ${_sources} PARENT_SCOPE)
          set(${the_out_headers} ${_headers} PARENT_SCOPE)
        endfunction()

        function(lean_occt_resolve_link_items the_out_var)
          set(_resolved)
          foreach(_item IN LISTS ARGN)
            if(TARGET "${_item}")
              list(APPEND _resolved "${_item}")
            elseif(_item STREQUAL "CSF_ThreadLibs")
              if(TARGET Threads::Threads)
                list(APPEND _resolved Threads::Threads)
              endif()
              if(UNIX AND NOT APPLE AND NOT EMSCRIPTEN AND NOT ANDROID)
                list(APPEND _resolved rt)
              endif()
            elseif(_item STREQUAL "CSF_TBB")
              if(OCCT_SUBSET_USE_TBB)
                list(APPEND _resolved TBB::tbb)
              endif()
            elseif(_item STREQUAL "CSF_dl")
              if(CMAKE_DL_LIBS)
                list(APPEND _resolved ${CMAKE_DL_LIBS})
              endif()
            elseif(_item STREQUAL "CSF_advapi32")
              if(WIN32)
                list(APPEND _resolved advapi32)
              endif()
            elseif(_item STREQUAL "CSF_gdi32")
              if(WIN32)
                list(APPEND _resolved gdi32)
              endif()
            elseif(_item STREQUAL "CSF_user32")
              if(WIN32)
                list(APPEND _resolved user32)
              endif()
            elseif(_item STREQUAL "CSF_psapi")
              if(WIN32)
                list(APPEND _resolved psapi)
              endif()
            elseif(_item STREQUAL "CSF_wsock32")
              if(WIN32)
                list(APPEND _resolved wsock32)
              endif()
            elseif(_item STREQUAL "CSF_androidlog")
              if(ANDROID)
                list(APPEND _resolved log)
              endif()
            elseif(_item STREQUAL "CSF_MMGR")
              continue()
            elseif(_item STREQUAL "")
              continue()
            else()
              list(APPEND _resolved "${_item}")
            endif()
          endforeach()

          if(UNIX AND NOT WIN32)
            list(APPEND _resolved m)
          endif()

          list(REMOVE_DUPLICATES _resolved)
          set(${the_out_var} ${_resolved} PARENT_SCOPE)
        endfunction()

        function(lean_occt_add_toolkit the_module the_toolkit)
          lean_occt_collect_package_files("${the_module}" "${the_toolkit}" _sources _headers)
          include("${CMAKE_CURRENT_SOURCE_DIR}/src/${the_module}/${the_toolkit}/EXTERNLIB.cmake")
          set(_deps_var "OCCT_${the_toolkit}_EXTERNAL_LIBS")
          lean_occt_resolve_link_items(_resolved ${${_deps_var}})

          add_library(${the_toolkit} ${_sources} ${_headers})
          target_include_directories(${the_toolkit} PUBLIC ${LEAN_OCCT_ALL_PACKAGE_DIRS})
          target_link_libraries(${the_toolkit} PUBLIC ${_resolved})
          target_compile_features(${the_toolkit} PUBLIC cxx_std_17)

          set_target_properties(${the_toolkit} PROPERTIES
            FOLDER "Modules/${the_module}"
            VERSION "${LEAN_OCCT_VERSION_COMPLETE}"
            SOVERSION "${LEAN_OCCT_VERSION_SOVERSION}"
          )
        endfunction()
        """
    )


def generate_root_cmake(the_version: dict[str, str]) -> str:
    a_development = the_version["development"]
    a_complete = f'{the_version["major"]}.{the_version["minor"]}.{the_version["maintenance"]}'
    a_extended = f"{a_complete}.{a_development}" if a_development else a_complete
    a_soversion = f'{the_version["major"]}.{the_version["minor"]}'
    a_content = """
cmake_minimum_required(VERSION 3.18)

project(LeanOcctSubset LANGUAGES C CXX)

set(LEAN_OCCT_VERSION_COMPLETE "@LEAN_OCCT_VERSION_COMPLETE@")
set(LEAN_OCCT_VERSION_EXTENDED "@LEAN_OCCT_VERSION_EXTENDED@")
set(LEAN_OCCT_VERSION_SOVERSION "@LEAN_OCCT_VERSION_SOVERSION@")

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
"""
    return (
        textwrap.dedent(a_content)
        .replace("@LEAN_OCCT_VERSION_COMPLETE@", a_complete)
        .replace("@LEAN_OCCT_VERSION_EXTENDED@", a_extended)
        .replace("@LEAN_OCCT_VERSION_SOVERSION@", a_soversion)
    )


def generate_readme() -> str:
    return textwrap.dedent(
        """\
        # Lean Authoring + STEP Subset

        This subset keeps the retained OCCT boundary for:

        - full BREP authoring / topology construction
        - full boolean operations
        - retained modeling algorithms used by the authoring stack
        - direct STEP import / export through `STEPControl`
        - the `LeanExchangeSmoke` verification tool
        - the wasm Three.js demo

        It intentionally removes the stock OCCT admin/build scaffolding and excludes:

        - Draw / Tcl / Tk
        - Visualization toolkits
        - Application Framework / XCAF
        - IGES and other non-STEP exchange stacks
        - plugin/provider layers outside the retained direct STEP path

        ## Native build

        ```bash
        cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Release
        cmake --build build --target LeanExchangeSmoke -j 8
        ctest --test-dir build --output-on-failure -R LeanExchangeSmoke
        ```

        ## Wasm demo build

        ```bash
        source /home/user/tools/emsdk/emsdk_env.sh >/dev/null

        cmake -S . -B build-wasm -G Ninja \
          -DCMAKE_BUILD_TYPE=Release \
          -DCMAKE_TOOLCHAIN_FILE=/home/user/tools/emsdk/upstream/emscripten/cmake/Modules/Platform/Emscripten.cmake \
          -DBUILD_LIBRARY_TYPE=Static \
          -DBUILD_LEAN_EXCHANGE_SMOKE=OFF \
          -DBUILD_WASM_THREEJS_DEMO=ON

        cmake --build build-wasm --target OcctThreeDemoWeb -j 8
        ```

        The packaged viewer is emitted under `build-wasm/lin32/clang/bin/web/`.
        """
    )


def collect_repo_src_files() -> int:
    return sum(1 for a_path in (REPO_ROOT / "src").rglob("*") if a_path.is_file())


def collect_subset_src_files(the_subset_root: Path) -> int:
    return sum(1 for a_path in (the_subset_root / "src").rglob("*") if a_path.is_file())


def export_subset(the_destination: Path) -> dict[str, object]:
    a_subset_src_root = the_destination / "src"
    a_subset_src_root.mkdir(parents=True, exist_ok=True)

    a_version = read_version_values()
    a_toolkit_metadata: list[dict[str, object]] = []

    for a_file_name in TOP_LEVEL_COPY_FILES:
        copy_file(REPO_ROOT / a_file_name, the_destination / a_file_name)

    write_text(the_destination / "README.md", generate_readme())
    write_text(the_destination / "CMakeLists.txt", generate_root_cmake(a_version))
    write_text(the_destination / "cmake" / "LeanOcctSubset.cmake", generate_helper_cmake())

    for a_module, a_toolkits in MODULE_TOOLKITS.items():
        for a_toolkit in a_toolkits:
            if a_toolkit == "TKSTEPCore":
                a_toolkit_metadata.append(copy_step_toolkit(a_subset_src_root))
            else:
                a_toolkit_metadata.append(copy_direct_toolkit(a_subset_src_root, a_module, a_toolkit))

    write_module_files(a_subset_src_root)
    write_standard_version_header(a_subset_src_root, a_version)

    copytree_filtered(REPO_ROOT / "tools" / "reduction", the_destination / "tools" / "reduction")
    copytree_filtered(REPO_ROOT / "samples" / "wasm" / "three_viewer", the_destination / "samples" / "wasm" / "three_viewer")

    a_subset_src_files = collect_subset_src_files(the_destination)
    a_repo_src_files = collect_repo_src_files()

    a_manifest: dict[str, object] = {
        "subset_name": the_destination.name,
        "reduction_profile": REDUCTION_PROFILE,
        "modules": list(MODULE_TOOLKITS.keys()),
        "toolkits": MODULE_TOOLKITS,
        "toolkit_metadata": a_toolkit_metadata,
        "top_level_files": [
            "CMakeLists.txt",
            "README.md",
            "LICENSE_LGPL_21.txt",
            "OCCT_LGPL_EXCEPTION.txt",
        ],
        "top_level_directories": [
            "cmake",
            "samples/wasm/three_viewer",
            "src",
            "tools/reduction",
        ],
        "stats": {
            "module_count": len(MODULE_TOOLKITS),
            "retained_toolkit_count": sum(len(a_toolkits) for a_toolkits in MODULE_TOOLKITS.values()),
            "subset_src_file_count": a_subset_src_files,
            "repo_src_file_count": a_repo_src_files,
            "subset_src_file_ratio": round(a_subset_src_files / a_repo_src_files, 4),
        },
    }
    write_text(the_destination / "subset-manifest.json", json.dumps(a_manifest, indent=2) + "\n")
    return a_manifest


def main() -> int:
    args = parse_args()
    a_destination = args.destination.resolve()

    if a_destination.exists():
        if not args.force:
            raise SystemExit(f"{a_destination} already exists; use --force to overwrite it.")
        shutil.rmtree(a_destination)

    a_destination.mkdir(parents=True, exist_ok=True)
    a_manifest = export_subset(a_destination)
    print(
        "Exported",
        a_destination,
        f"with {a_manifest['stats']['retained_toolkit_count']} toolkits and {a_manifest['stats']['subset_src_file_count']} src files.",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
