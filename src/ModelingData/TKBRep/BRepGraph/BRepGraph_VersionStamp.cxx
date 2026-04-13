// Copyright (c) 2026 OPEN CASCADE SAS
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

#include <BRepGraph_VersionStamp.hxx>

#include <Standard_UUID.hxx>

#include <cstring>

//=================================================================================================

Standard_GUID BRepGraph_VersionStamp::ToGUID(const Standard_GUID& theGraphGUID) const
{
  // Pack fields into a flat byte buffer to avoid struct padding issues.
  const Standard_UUID aGraphUUID = theGraphGUID.ToUUID();
  const uint8_t       aDomain    = static_cast<uint8_t>(myDomain);

  size_t aCounter = 0;
  int    aKind    = 0;
  if (myDomain == Domain::Entity)
  {
    aCounter = myUID.Counter();
    aKind    = static_cast<int>(myUID.Kind());
  }
  else if (myDomain == Domain::Ref)
  {
    aCounter = myRefUID.Counter();
    aKind    = static_cast<int>(myRefUID.Kind());
  }

  uint8_t aBuffer[sizeof(aGraphUUID) + sizeof(aDomain) + sizeof(aCounter) + sizeof(aKind)
                  + sizeof(myMutationGen) + sizeof(myGeneration)];
  size_t  anOff = 0;
  std::memcpy(aBuffer + anOff, &aGraphUUID, sizeof(aGraphUUID));
  anOff += sizeof(aGraphUUID);
  std::memcpy(aBuffer + anOff, &aDomain, sizeof(aDomain));
  anOff += sizeof(aDomain);
  std::memcpy(aBuffer + anOff, &aCounter, sizeof(aCounter));
  anOff += sizeof(aCounter);
  std::memcpy(aBuffer + anOff, &aKind, sizeof(aKind));
  anOff += sizeof(aKind);
  std::memcpy(aBuffer + anOff, &myMutationGen, sizeof(myMutationGen));
  anOff += sizeof(myMutationGen);
  std::memcpy(aBuffer + anOff, &myGeneration, sizeof(myGeneration));
  anOff += sizeof(myGeneration);

  Standard_UUID aResultUUID;
  const size_t  aHalfOff = sizeof(aGraphUUID);
  if constexpr (sizeof(size_t) >= 8)
  {
    // Two independent 64-bit hashes fill the 128-bit GUID on native 64-bit targets.
    const size_t aHash1 = opencascade::hashBytes(aBuffer, static_cast<int>(anOff));
    const size_t aHash2 =
      opencascade::hashBytes(aBuffer + aHalfOff, static_cast<int>(anOff - aHalfOff));
    std::memcpy(&aResultUUID, &aHash1, 8);
    std::memcpy(reinterpret_cast<uint8_t*>(&aResultUUID) + 8, &aHash2, 8);
  }
  else
  {
    // WebAssembly is currently 32-bit, so pack the UUID from four salted 32-bit hashes instead.
    const size_t aQuarterOff      = anOff / 4;
    const size_t aThreeQuarterOff = (anOff * 3) / 4;
    const auto   hash32 = [&](const uint32_t theSalt, const size_t theOffset) {
      const uint32_t aPrefixHash =
        static_cast<uint32_t>(opencascade::hashBytes(aBuffer, static_cast<int>(anOff)));
      const uint32_t aSpanHash = static_cast<uint32_t>(
        opencascade::hashBytes(aBuffer + theOffset, static_cast<int>(anOff - theOffset)));
      const uint32_t aMix[3] = {theSalt, aPrefixHash, aSpanHash};
      return static_cast<uint32_t>(opencascade::hashBytes(aMix, sizeof(aMix)));
    };

    const uint32_t aHashes[4] = {
      hash32(0x243F6A88u, 0),
      hash32(0x85A308D3u, aHalfOff),
      hash32(0x13198A2Eu, aQuarterOff),
      hash32(0x03707344u, aThreeQuarterOff)
    };
    static_assert(sizeof(aResultUUID) == sizeof(aHashes), "Unexpected UUID size");
    std::memcpy(&aResultUUID, aHashes, sizeof(aHashes));
  }
  return Standard_GUID(aResultUUID);
}
