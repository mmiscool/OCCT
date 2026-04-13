// Created on: 2026-04-13
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

#define OCC_VERSION_MAJOR 8
#define OCC_VERSION_MINOR 0
#define OCC_VERSION_MAINTENANCE 0

#define OCC_VERSION_DEVELOPMENT "rc5"

#define OCC_VERSION 8.0
#define OCC_VERSION_STRING "8.0"
#define OCC_VERSION_COMPLETE "8.0.0"

#ifdef OCC_VERSION_DEVELOPMENT
  #define OCC_VERSION_STRING_EXT OCC_VERSION_COMPLETE "." OCC_VERSION_DEVELOPMENT
#else
  #define OCC_VERSION_STRING_EXT OCC_VERSION_COMPLETE
#endif

#define OCC_VERSION_HEX (OCC_VERSION_MAJOR << 16 | OCC_VERSION_MINOR << 8 | OCC_VERSION_MAINTENANCE)

#endif /* _Standard_Version_HeaderFile */
