#include <BRepAlgoAPI_Cut.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepFeat_MakeCylindricalHole.hxx>
#include <BRepFeat_Status.hxx>
#include <BRepFilletAPI_MakeFillet.hxx>
#include <BRepGProp.hxx>
#include <BRepMesh_IncrementalMesh.hxx>
#include <BRepOffsetAPI_MakeOffsetShape.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRep_Tool.hxx>
#include <GProp_GProps.hxx>
#include <GeomAbs_Shape.hxx>
#include <HelixBRep_BuilderHelix.hxx>
#include <IFSelect_ReturnStatus.hxx>
#include <NCollection_Array1.hxx>
#include <Poly_Triangulation.hxx>
#include <STEPControl_Reader.hxx>
#include <STEPControl_StepModelType.hxx>
#include <STEPControl_Writer.hxx>
#include <Standard_Failure.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <TopExp_Explorer.hxx>
#include <TopLoc_Location.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Shape.hxx>
#include <TopoDS_Wire.hxx>
#include <gp_Ax1.hxx>
#include <gp_Ax2.hxx>
#include <gp_Ax3.hxx>
#include <gp_Dir.hxx>
#include <gp_Pnt.hxx>

#include <cstdio>
#include <filesystem>
#include <exception>
#include <iostream>
#include <stdexcept>
#include <string>

namespace
{
struct ShapeMetrics
{
  int solids    = 0;
  int faces     = 0;
  int triangles = 0;
};

struct LinearMetrics
{
  int    edges  = 0;
  double length = 0.0;
};

static TopoDS_Shape makeBooleanDemoShape()
{
  const double aBoxSize     = 60.0;
  const double aHalfBoxSize = aBoxSize * 0.5;
  const double aHoleRadius  = 12.0;
  const double aHoleHeight  = aBoxSize * 1.2;
  const gp_Pnt aBoxCorner(-aHalfBoxSize, -aHalfBoxSize, -aHalfBoxSize);
  const gp_Ax2 aHoleAxis(gp_Pnt(0.0, 0.0, -aHoleHeight * 0.5), gp_Dir(0.0, 0.0, 1.0));

  const TopoDS_Shape aBox =
    BRepPrimAPI_MakeBox(aBoxCorner, aBoxSize, aBoxSize, aBoxSize).Shape();
  const TopoDS_Shape aCylinder =
    BRepPrimAPI_MakeCylinder(aHoleAxis, aHoleRadius, aHoleHeight).Shape();

  BRepAlgoAPI_Cut aCut(aBox, aCylinder);
  aCut.Build();
  if (!aCut.IsDone())
  {
    throw std::runtime_error("Boolean cut failed.");
  }

  return aCut.Shape();
}

static TopoDS_Shape makeFilletDemoShape()
{
  const TopoDS_Shape aBox = BRepPrimAPI_MakeBox(40.0, 30.0, 20.0).Shape();

  TopExp_Explorer anEdgeExp(aBox, TopAbs_EDGE);
  if (!anEdgeExp.More())
  {
    throw std::runtime_error("Fillet source shape did not contain any edges.");
  }

  BRepFilletAPI_MakeFillet aFillet(aBox);
  aFillet.Add(3.0, TopoDS::Edge(anEdgeExp.Current()));
  const TopoDS_Shape aResult = aFillet.Shape();
  if (!aFillet.IsDone())
  {
    throw std::runtime_error("Fillet operation failed.");
  }

  return aResult;
}

static TopoDS_Shape makeOffsetDemoShape()
{
  const TopoDS_Shape aBox = BRepPrimAPI_MakeBox(30.0, 30.0, 30.0).Shape();

  BRepOffsetAPI_MakeOffsetShape anOffset;
  anOffset.PerformByJoin(aBox, 2.0, 1.0e-4);
  const TopoDS_Shape aResult = anOffset.Shape();
  if (aResult.IsNull())
  {
    throw std::runtime_error("Offset operation produced a null shape.");
  }

  return aResult;
}

static TopoDS_Shape makeFeatureDemoShape()
{
  const TopoDS_Shape aBox  = BRepPrimAPI_MakeBox(40.0, 40.0, 30.0).Shape();
  const gp_Ax1       aAxis(gp_Pnt(20.0, 20.0, -10.0), gp_Dir(0.0, 0.0, 1.0));

  BRepFeat_MakeCylindricalHole aFeature;
  aFeature.Init(aBox, aAxis);
  aFeature.Perform(6.0);
  aFeature.Build();

  if (aFeature.Status() != BRepFeat_NoError)
  {
    throw std::runtime_error("Cylindrical hole feature operation failed.");
  }

  const TopoDS_Shape aResult = aFeature.Shape();
  if (aResult.IsNull())
  {
    throw std::runtime_error("Cylindrical hole feature produced a null shape.");
  }

  return aResult;
}

static TopoDS_Wire makeHelixDemoShape()
{
  HelixBRep_BuilderHelix aBuilder;

  NCollection_Array1<double> aHeights(1, 1);
  aHeights(1) = 30.0;

  NCollection_Array1<double> aPitches(1, 1);
  aPitches(1) = 10.0;

  NCollection_Array1<bool> aIsPitches(1, 1);
  aIsPitches(1) = true;

  const gp_Ax3 anAxis(gp_Pnt(0.0, 0.0, 0.0), gp_Dir(0.0, 0.0, 1.0), gp_Dir(1.0, 0.0, 0.0));
  aBuilder.SetParameters(anAxis, 20.0, aHeights, aPitches, aIsPitches);
  aBuilder.SetApproxParameters(1.0e-4, 8, GeomAbs_C1);
  aBuilder.Perform();

  if (aBuilder.ErrorStatus() != 0)
  {
    throw std::runtime_error("Helix builder failed.");
  }

  const TopoDS_Shape aResult = aBuilder.Shape();
  if (aResult.IsNull() || aResult.ShapeType() != TopAbs_WIRE)
  {
    throw std::runtime_error("Helix builder did not produce a wire.");
  }

  return TopoDS::Wire(aResult);
}

static ShapeMetrics collectMetrics(const TopoDS_Shape& theShape)
{
  if (theShape.IsNull())
  {
    throw std::runtime_error("Encountered a null shape.");
  }

  BRepCheck_Analyzer anAnalyzer(theShape);
  if (!anAnalyzer.IsValid())
  {
    throw std::runtime_error("Shape validity check failed.");
  }

  BRepMesh_IncrementalMesh(theShape, 0.9, false, 0.35, false);

  ShapeMetrics aMetrics;

  for (TopExp_Explorer aSolidExp(theShape, TopAbs_SOLID); aSolidExp.More(); aSolidExp.Next())
  {
    ++aMetrics.solids;
  }

  for (TopExp_Explorer aFaceExp(theShape, TopAbs_FACE); aFaceExp.More(); aFaceExp.Next())
  {
    ++aMetrics.faces;

    const TopoDS_Face& aFace = TopoDS::Face(aFaceExp.Current());
    TopLoc_Location    aLoc;
    const occ::handle<Poly_Triangulation>& aTriangulation = BRep_Tool::Triangulation(aFace, aLoc);
    if (!aTriangulation.IsNull())
    {
      aMetrics.triangles += aTriangulation->NbTriangles();
    }
  }

  if (aMetrics.solids < 1)
  {
    throw std::runtime_error("Shape did not contain a solid.");
  }
  if (aMetrics.faces < 1)
  {
    throw std::runtime_error("Shape did not contain any faces.");
  }
  if (aMetrics.triangles < 1)
  {
    throw std::runtime_error("Shape did not produce any mesh triangles.");
  }

  return aMetrics;
}

static LinearMetrics collectLinearMetrics(const TopoDS_Shape& theShape)
{
  if (theShape.IsNull())
  {
    throw std::runtime_error("Encountered a null linear shape.");
  }

  BRepCheck_Analyzer anAnalyzer(theShape);
  if (!anAnalyzer.IsValid())
  {
    throw std::runtime_error("Linear shape validity check failed.");
  }

  LinearMetrics aMetrics;
  for (TopExp_Explorer anEdgeExp(theShape, TopAbs_EDGE); anEdgeExp.More(); anEdgeExp.Next())
  {
    ++aMetrics.edges;
  }

  if (aMetrics.edges < 1)
  {
    throw std::runtime_error("Linear shape did not contain any edges.");
  }

  GProp_GProps aLinearProps;
  BRepGProp::LinearProperties(theShape, aLinearProps);
  aMetrics.length = aLinearProps.Mass();
  if (aMetrics.length <= 0.0)
  {
    throw std::runtime_error("Linear shape did not produce any measurable length.");
  }

  return aMetrics;
}

static void writeStep(const TopoDS_Shape& theShape, const std::string& thePath)
{
  STEPControl_Writer aWriter;
  if (aWriter.Transfer(theShape, STEPControl_AsIs) != IFSelect_RetDone)
  {
    throw std::runtime_error("STEP transfer failed.");
  }
  if (aWriter.Write(thePath.c_str()) != IFSelect_RetDone)
  {
    throw std::runtime_error("STEP write failed.");
  }
}

static TopoDS_Shape readStep(const std::string& thePath)
{
  STEPControl_Reader aReader;
  if (aReader.ReadFile(thePath.c_str()) != IFSelect_RetDone)
  {
    throw std::runtime_error("STEP read failed.");
  }
  if (aReader.TransferRoots() <= 0)
  {
    throw std::runtime_error("STEP transfer roots failed.");
  }

  const TopoDS_Shape aShape = aReader.OneShape();
  if (aShape.IsNull())
  {
    throw std::runtime_error("STEP import produced a null shape.");
  }
  return aShape;
}

static void removeIfPresent(const std::string& thePath)
{
  std::remove(thePath.c_str());
}

static std::string artifactPath(const char* theFileName)
{
#ifdef LEAN_OCCT_TEST_ARTIFACTS_DIR
  const std::filesystem::path aBaseDir(LEAN_OCCT_TEST_ARTIFACTS_DIR);
#else
  const std::filesystem::path aBaseDir =
    std::filesystem::current_path() / "test-artifacts" / "ctest";
#endif
  std::filesystem::create_directories(aBaseDir);
  return (aBaseDir / theFileName).string();
}
} // namespace

int main()
{
  const std::string aStepPath = artifactPath("LeanExchangeSmoke.step");

  try
  {
    removeIfPresent(aStepPath);

    const ShapeMetrics  aSourceMetrics  = collectMetrics(makeBooleanDemoShape());
    const ShapeMetrics  aFilletMetrics  = collectMetrics(makeFilletDemoShape());
    const ShapeMetrics  anOffsetMetrics = collectMetrics(makeOffsetDemoShape());
    const ShapeMetrics  aFeatureMetrics = collectMetrics(makeFeatureDemoShape());
    const LinearMetrics aHelixMetrics   = collectLinearMetrics(makeHelixDemoShape());

    const TopoDS_Shape aStepSourceShape = makeBooleanDemoShape();
    writeStep(aStepSourceShape, aStepPath);
    const ShapeMetrics aStepMetrics = collectMetrics(readStep(aStepPath));

    std::cout << "LeanExchangeSmoke OK\n"
              << "source  solids=" << aSourceMetrics.solids << " faces=" << aSourceMetrics.faces
              << " triangles=" << aSourceMetrics.triangles << "\n"
              << "fillet  solids=" << aFilletMetrics.solids << " faces=" << aFilletMetrics.faces
              << " triangles=" << aFilletMetrics.triangles << "\n"
              << "offset  solids=" << anOffsetMetrics.solids << " faces=" << anOffsetMetrics.faces
              << " triangles=" << anOffsetMetrics.triangles << "\n"
              << "feature solids=" << aFeatureMetrics.solids << " faces=" << aFeatureMetrics.faces
              << " triangles=" << aFeatureMetrics.triangles << "\n"
              << "helix   edges=" << aHelixMetrics.edges << " length=" << aHelixMetrics.length
              << "\n"
              << "step    solids=" << aStepMetrics.solids << " faces=" << aStepMetrics.faces
              << " triangles=" << aStepMetrics.triangles << "\n"
              << "artifact " << aStepPath << "\n";
  }
  catch (const Standard_Failure& theFailure)
  {
    std::cerr << "LeanExchangeSmoke failed: " << theFailure.what() << "\n";
    return 1;
  }
  catch (const std::exception& theError)
  {
    std::cerr << "LeanExchangeSmoke failed: " << theError.what() << "\n";
    return 1;
  }
  return 0;
}
