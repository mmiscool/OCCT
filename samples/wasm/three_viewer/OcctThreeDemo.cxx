#include <BRepAlgoAPI_Cut.hxx>
#include <BRepBndLib.hxx>
#include <BRepLib.hxx>
#include <BRepLib_ToolTriangulatedShape.hxx>
#include <BRepMesh_IncrementalMesh.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRep_Tool.hxx>
#include <Bnd_Box.hxx>
#include <NCollection_IndexedDataMap.hxx>
#include <NCollection_List.hxx>
#include <Poly_Polygon3D.hxx>
#include <Poly_PolygonOnTriangulation.hxx>
#include <Poly_Triangulation.hxx>
#include <Standard_Failure.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopLoc_Location.hxx>
#include <TopTools_ShapeMapHasher.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Shape.hxx>
#include <TopAbs_Orientation.hxx>
#include <gp_Ax2.hxx>
#include <gp_Dir.hxx>
#include <gp_Pnt.hxx>
#include <gp_Trsf.hxx>

#include <emscripten/bind.h>

#include <array>
#include <iomanip>
#include <sstream>
#include <stdexcept>
#include <string>
#include <vector>

namespace
{
struct DemoBuffers
{
  std::vector<float> positions;
  std::vector<float> normals;
  std::vector<float> edgePositions;
  std::array<float, 3> bboxMin {{0.0f, 0.0f, 0.0f}};
  std::array<float, 3> bboxMax {{0.0f, 0.0f, 0.0f}};
};

using EdgeFaceMap = NCollection_IndexedDataMap<TopoDS_Shape,
                                               NCollection_List<TopoDS_Shape>,
                                               TopTools_ShapeMapHasher>;

static void appendPoint(std::vector<float>& theValues, const gp_Pnt& thePoint)
{
  theValues.push_back(static_cast<float>(thePoint.X()));
  theValues.push_back(static_cast<float>(thePoint.Y()));
  theValues.push_back(static_cast<float>(thePoint.Z()));
}

static void appendDir(std::vector<float>& theValues, const gp_Dir& theDir)
{
  theValues.push_back(static_cast<float>(theDir.X()));
  theValues.push_back(static_cast<float>(theDir.Y()));
  theValues.push_back(static_cast<float>(theDir.Z()));
}

static void appendVectorJson(std::ostringstream&        theStream,
                             const std::vector<float>&  theValues)
{
  theStream << "[";
  for (std::size_t aValueIndex = 0; aValueIndex < theValues.size(); ++aValueIndex)
  {
    if (aValueIndex != 0)
    {
      theStream << ",";
    }
    theStream << theValues[aValueIndex];
  }
  theStream << "]";
}

static void appendArrayJson(std::ostringstream&                  theStream,
                            const std::array<float, 3>& theValues)
{
  theStream << "[";
  for (std::size_t aValueIndex = 0; aValueIndex < theValues.size(); ++aValueIndex)
  {
    if (aValueIndex != 0)
    {
      theStream << ",";
    }
    theStream << theValues[aValueIndex];
  }
  theStream << "]";
}

static std::string jsonEscape(const std::string& theText)
{
  std::string aResult;
  aResult.reserve(theText.size());
  for (const char aChar : theText)
  {
    switch (aChar)
    {
      case '\\':
        aResult += "\\\\";
        break;
      case '"':
        aResult += "\\\"";
        break;
      case '\n':
        aResult += "\\n";
        break;
      case '\r':
        aResult += "\\r";
        break;
      case '\t':
        aResult += "\\t";
        break;
      default:
        aResult += aChar;
        break;
    }
  }
  return aResult;
}

static TopoDS_Shape makeDemoShape()
{
  const Standard_Real aBoxSize      = 60.0;
  const Standard_Real aHalfBoxSize  = aBoxSize * 0.5;
  const Standard_Real aHoleRadius   = 12.0;
  const Standard_Real aHoleHeight   = aBoxSize * 1.2;
  const Standard_Real aHoleStartZ   = -aHoleHeight * 0.5;
  const gp_Pnt        aBoxCorner(-aHalfBoxSize, -aHalfBoxSize, -aHalfBoxSize);
  const gp_Ax2        aHoleAxis(gp_Pnt(0.0, 0.0, aHoleStartZ), gp_Dir(0.0, 0.0, 1.0));

  const TopoDS_Shape aBox =
    BRepPrimAPI_MakeBox(aBoxCorner, aBoxSize, aBoxSize, aBoxSize).Shape();
  const TopoDS_Shape aCylinder =
    BRepPrimAPI_MakeCylinder(aHoleAxis, aHoleRadius, aHoleHeight).Shape();

  BRepAlgoAPI_Cut aBooleanCut(aBox, aCylinder);
  aBooleanCut.Build();
  if (!aBooleanCut.IsDone())
  {
    throw std::runtime_error("The boolean cut did not complete.");
  }

  return aBooleanCut.Shape();
}

static DemoBuffers tessellateDemoShape()
{
  DemoBuffers aBuffers;
  const TopoDS_Shape aShape = makeDemoShape();

  BRepMesh_IncrementalMesh(aShape, 0.9, false, 0.35, false);
  BRepLib::EnsureNormalConsistency(aShape, 0.001, true);

  Bnd_Box aBounds;
  BRepBndLib::Add(aShape, aBounds, false);
  if (aBounds.IsVoid())
  {
    throw std::runtime_error("The demo shape has an empty bounding box.");
  }

  Standard_Real aXmin = 0.0;
  Standard_Real aYmin = 0.0;
  Standard_Real aZmin = 0.0;
  Standard_Real aXmax = 0.0;
  Standard_Real aYmax = 0.0;
  Standard_Real aZmax = 0.0;
  aBounds.Get(aXmin, aYmin, aZmin, aXmax, aYmax, aZmax);
  aBuffers.bboxMin = {
    static_cast<float>(aXmin),
    static_cast<float>(aYmin),
    static_cast<float>(aZmin)
  };
  aBuffers.bboxMax = {
    static_cast<float>(aXmax),
    static_cast<float>(aYmax),
    static_cast<float>(aZmax)
  };

  for (TopExp_Explorer aFaceExplorer(aShape, TopAbs_FACE); aFaceExplorer.More(); aFaceExplorer.Next())
  {
    const TopoDS_Face&                     aFace = TopoDS::Face(aFaceExplorer.Current());
    TopLoc_Location                        aLoc;
    const occ::handle<Poly_Triangulation>& aTriangulation = BRep_Tool::Triangulation(aFace, aLoc);
    if (aTriangulation.IsNull() || aTriangulation->NbTriangles() == 0)
    {
      continue;
    }

    BRepLib_ToolTriangulatedShape::ComputeNormals(aFace, aTriangulation);

    const gp_Trsf& aTrsf = aLoc.Transformation();
    const bool     isMirrored = aTrsf.VectorialPart().Determinant() < 0.0;
    const bool     isReversed = (aFace.Orientation() == TopAbs_REVERSED);

    for (int aTriangleIndex = 1; aTriangleIndex <= aTriangulation->NbTriangles(); ++aTriangleIndex)
    {
      int aNodeIndex1 = 0;
      int aNodeIndex2 = 0;
      int aNodeIndex3 = 0;
      if (isReversed)
      {
        aTriangulation->Triangle(aTriangleIndex).Get(aNodeIndex1, aNodeIndex3, aNodeIndex2);
      }
      else
      {
        aTriangulation->Triangle(aTriangleIndex).Get(aNodeIndex1, aNodeIndex2, aNodeIndex3);
      }

      const int anIndices[] = {aNodeIndex1, aNodeIndex2, aNodeIndex3};
      for (const int aNodeIndex : anIndices)
      {
        gp_Pnt aPoint = aTriangulation->Node(aNodeIndex);
        gp_Dir aNormal = aTriangulation->Normal(aNodeIndex);
        if (isReversed ^ isMirrored)
        {
          aNormal.Reverse();
        }
        if (!aLoc.IsIdentity())
        {
          aPoint.Transform(aTrsf);
          aNormal.Transform(aTrsf);
        }

        appendPoint(aBuffers.positions, aPoint);
        appendDir(aBuffers.normals, aNormal);
      }
    }
  }

  EdgeFaceMap anEdgeFaceMap;
  TopExp::MapShapesAndUniqueAncestors(aShape, TopAbs_EDGE, TopAbs_FACE, anEdgeFaceMap);
  for (EdgeFaceMap::Iterator anEdgeIter(anEdgeFaceMap); anEdgeIter.More(); anEdgeIter.Next())
  {
    const TopoDS_Edge& anEdge = TopoDS::Edge(anEdgeIter.Key());
    if (BRep_Tool::Degenerated(anEdge))
    {
      continue;
    }

    if (anEdgeIter.Value().Extent() == 1)
    {
      const TopoDS_Face& aOnlyFace = TopoDS::Face(anEdgeIter.Value().First());
      if (BRep_Tool::IsClosed(anEdge, aOnlyFace))
      {
        continue;
      }
    }

    occ::handle<Poly_PolygonOnTriangulation> aPolyOnTriangulation;
    occ::handle<Poly_Triangulation>          aTriangulation;
    TopLoc_Location                          aPolyLoc;
    BRep_Tool::PolygonOnTriangulation(anEdge, aPolyOnTriangulation, aTriangulation, aPolyLoc);
    if (!aPolyOnTriangulation.IsNull() && !aTriangulation.IsNull() && aPolyOnTriangulation->NbNodes() >= 2)
    {
      const gp_Trsf& aTrsf = aPolyLoc.Transformation();
      gp_Pnt         aPreviousPoint = aTriangulation->Node(aPolyOnTriangulation->Node(1));
      if (!aPolyLoc.IsIdentity())
      {
        aPreviousPoint.Transform(aTrsf);
      }

      for (int aNodeIndex = 2; aNodeIndex <= aPolyOnTriangulation->NbNodes(); ++aNodeIndex)
      {
        gp_Pnt aCurrentPoint = aTriangulation->Node(aPolyOnTriangulation->Node(aNodeIndex));
        if (!aPolyLoc.IsIdentity())
        {
          aCurrentPoint.Transform(aTrsf);
        }

        appendPoint(aBuffers.edgePositions, aPreviousPoint);
        appendPoint(aBuffers.edgePositions, aCurrentPoint);
        aPreviousPoint = aCurrentPoint;
      }
      continue;
    }

    TopLoc_Location                   aPolygonLoc;
    const occ::handle<Poly_Polygon3D> aPolygon3d = BRep_Tool::Polygon3D(anEdge, aPolygonLoc);
    if (aPolygon3d.IsNull() || aPolygon3d->NbNodes() < 2)
    {
      continue;
    }

    const gp_Trsf& aTrsf = aPolygonLoc.Transformation();
    gp_Pnt         aPreviousPoint = aPolygon3d->Nodes().Value(1);
    if (!aPolygonLoc.IsIdentity())
    {
      aPreviousPoint.Transform(aTrsf);
    }

    for (int aNodeIndex = 2; aNodeIndex <= aPolygon3d->NbNodes(); ++aNodeIndex)
    {
      gp_Pnt aCurrentPoint = aPolygon3d->Nodes().Value(aNodeIndex);
      if (!aPolygonLoc.IsIdentity())
      {
        aCurrentPoint.Transform(aTrsf);
      }

      appendPoint(aBuffers.edgePositions, aPreviousPoint);
      appendPoint(aBuffers.edgePositions, aCurrentPoint);
      aPreviousPoint = aCurrentPoint;
    }
  }

  return aBuffers;
}

static std::string buildDemoGeometryJson()
{
  try
  {
    const DemoBuffers aBuffers = tessellateDemoShape();
    std::ostringstream aStream;
    aStream << std::setprecision(9);
    aStream << "{";
    aStream << "\"positions\":";
    appendVectorJson(aStream, aBuffers.positions);
    aStream << ",\"normals\":";
    appendVectorJson(aStream, aBuffers.normals);
    aStream << ",\"edgePositions\":";
    appendVectorJson(aStream, aBuffers.edgePositions);
    aStream << ",\"bboxMin\":";
    appendArrayJson(aStream, aBuffers.bboxMin);
    aStream << ",\"bboxMax\":";
    appendArrayJson(aStream, aBuffers.bboxMax);
    aStream << "}";
    return aStream.str();
  }
  catch (const Standard_Failure& theFailure)
  {
    return std::string("{\"error\":\"")
           + jsonEscape(theFailure.GetMessageString() == nullptr ? "OCCT failure" : theFailure.GetMessageString())
           + "\"}";
  }
  catch (const std::exception& theError)
  {
    return std::string("{\"error\":\"") + jsonEscape(theError.what()) + "\"}";
  }
}
} // namespace

EMSCRIPTEN_BINDINGS(OcctThreeDemo)
{
  emscripten::function("buildDemoGeometryJson", &buildDemoGeometryJson);
}
