#include <lean_occt_capi.h>

#include <BRepAdaptor_Curve.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <BRepAlgoAPI_Common.hxx>
#include <BRepAlgoAPI_Cut.hxx>
#include <BRepAlgoAPI_Fuse.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBndLib.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepFeat_MakeCylindricalHole.hxx>
#include <BRepFeat_Status.hxx>
#include <BRepFilletAPI_MakeFillet.hxx>
#include <BRepGProp.hxx>
#include <BRepLib.hxx>
#include <BRepLib_ToolTriangulatedShape.hxx>
#include <BRepLProp_SLProps.hxx>
#include <BRepMesh_IncrementalMesh.hxx>
#include <BRepOffsetAPI_MakeOffsetShape.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCone.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakePrism.hxx>
#include <BRepPrimAPI_MakeRevol.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <BRepPrimAPI_MakeTorus.hxx>
#include <BRepTools.hxx>
#include <BRepTools_WireExplorer.hxx>
#include <BRep_Tool.hxx>
#include <Bnd_Box.hxx>
#include <GProp_GProps.hxx>
#include <GeomAbs_Shape.hxx>
#include <GeomAbs_CurveType.hxx>
#include <GeomAbs_SurfaceType.hxx>
#include <HelixBRep_BuilderHelix.hxx>
#include <IFSelect_ReturnStatus.hxx>
#include <NCollection_Array1.hxx>
#include <NCollection_IndexedDataMap.hxx>
#include <NCollection_IndexedMap.hxx>
#include <NCollection_List.hxx>
#include <Poly_Polygon3D.hxx>
#include <Poly_PolygonOnTriangulation.hxx>
#include <Poly_Triangulation.hxx>
#include <Precision.hxx>
#include <STEPControl_Reader.hxx>
#include <STEPControl_StepModelType.hxx>
#include <STEPControl_Writer.hxx>
#include <Standard_Failure.hxx>
#include <Message.hxx>
#include <Message_Messenger.hxx>
#include <Message_Printer.hxx>
#include <TopAbs_Orientation.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <TopLoc_Location.hxx>
#include <TopTools_ShapeMapHasher.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Shape.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopoDS_Wire.hxx>
#include <gp_Ax1.hxx>
#include <gp_Ax3.hxx>
#include <gp_Ax2.hxx>
#include <gp_Cone.hxx>
#include <gp_Circ.hxx>
#include <gp_Cylinder.hxx>
#include <gp_Dir.hxx>
#include <gp_Elips.hxx>
#include <gp_Lin.hxx>
#include <gp_Pnt.hxx>
#include <gp_Pln.hxx>
#include <gp_Sphere.hxx>
#include <gp_Torus.hxx>
#include <gp_Trsf.hxx>
#include <gp_Vec.hxx>

#include <array>
#include <algorithm>
#include <cmath>
#include <cstddef>
#include <cstdint>
#include <exception>
#include <new>
#include <stdexcept>
#include <string>
#include <type_traits>
#include <utility>
#include <vector>

namespace
{
using EdgeFaceMap = NCollection_IndexedDataMap<TopoDS_Shape,
                                               NCollection_List<TopoDS_Shape>,
                                               TopTools_ShapeMapHasher>;

struct MeshBuffers
{
  std::vector<double>   positions;
  std::vector<double>   normals;
  std::vector<uint32_t> triangleIndices;
  std::vector<double>   edgePositions;
  std::array<double, 3> bboxMin {{0.0, 0.0, 0.0}};
  std::array<double, 3> bboxMax {{0.0, 0.0, 0.0}};
  std::size_t           solidCount = 0;
  std::size_t           faceCount = 0;
};

struct TopologyBuffers
{
  std::vector<double>   vertexPositions;
  std::vector<uint32_t> edgeVertexIndices;
  std::vector<double>   edgeLengths;
  std::vector<uint32_t> edgeFaceRanges;
  std::vector<uint32_t> edgeFaceIndices;
  std::vector<uint32_t> wireRanges;
  std::vector<uint32_t> wireEdgeIndices;
  std::vector<uint8_t>  wireEdgeOrientations;
  std::vector<uint32_t> wireVertexRanges;
  std::vector<uint32_t> wireVertexIndices;
  std::vector<uint32_t> faceRanges;
  std::vector<uint32_t> faceWireIndices;
  std::vector<uint8_t>  faceWireOrientations;
  std::vector<uint8_t>  faceWireRoles;
};

class ScopedMessengerSilencer
{
public:
  ScopedMessengerSilencer()
      : myMessenger(Message::DefaultMessenger())
  {
    if (myMessenger.IsNull())
    {
      return;
    }

    for (NCollection_Sequence<occ::handle<Message_Printer>>::Iterator a_printer_it(
           myMessenger->Printers());
         a_printer_it.More();
         a_printer_it.Next())
    {
      mySavedPrinters.Append(a_printer_it.Value());
    }
    myMessenger->ChangePrinters().Clear();
  }

  ~ScopedMessengerSilencer()
  {
    if (myMessenger.IsNull())
    {
      return;
    }

    myMessenger->ChangePrinters().Clear();
    for (NCollection_Sequence<occ::handle<Message_Printer>>::Iterator a_printer_it(mySavedPrinters);
         a_printer_it.More();
         a_printer_it.Next())
    {
      myMessenger->AddPrinter(a_printer_it.Value());
    }
  }

private:
  occ::handle<Message_Messenger>                   myMessenger;
  NCollection_Sequence<occ::handle<Message_Printer>> mySavedPrinters;
};

static void appendPoint(std::vector<double>& the_values, const gp_Pnt& the_point)
{
  the_values.push_back(the_point.X());
  the_values.push_back(the_point.Y());
  the_values.push_back(the_point.Z());
}

static void appendDir(std::vector<double>& the_values, const gp_Dir& the_dir)
{
  the_values.push_back(the_dir.X());
  the_values.push_back(the_dir.Y());
  the_values.push_back(the_dir.Z());
}

static void writeDir(double* the_xyz3, const gp_Dir& the_dir)
{
  the_xyz3[0] = the_dir.X();
  the_xyz3[1] = the_dir.Y();
  the_xyz3[2] = the_dir.Z();
}

static void fillCurveGeometry(const Adaptor3d_Curve& the_curve, LeanOcctEdgeGeometry& the_geometry);
static void fillFaceGeometry(const Adaptor3d_Surface& the_surface, LeanOcctFaceGeometry& the_geometry);
static void fillLinePayload(const Adaptor3d_Curve& the_curve, LeanOcctLinePayload& the_payload);
static void fillCirclePayload(const Adaptor3d_Curve& the_curve, LeanOcctCirclePayload& the_payload);
static void fillEllipsePayload(const Adaptor3d_Curve& the_curve, LeanOcctEllipsePayload& the_payload);
static void fillPlanePayload(const Adaptor3d_Surface& the_surface, LeanOcctPlanePayload& the_payload);
static void fillCylinderPayload(const Adaptor3d_Surface& the_surface, LeanOcctCylinderPayload& the_payload);
static void fillConePayload(const Adaptor3d_Surface& the_surface, LeanOcctConePayload& the_payload);
static void fillSpherePayload(const Adaptor3d_Surface& the_surface, LeanOcctSpherePayload& the_payload);
static void fillTorusPayload(const Adaptor3d_Surface& the_surface, LeanOcctTorusPayload& the_payload);
static void fillRevolutionSurfacePayload(const Adaptor3d_Surface& the_surface,
                                         LeanOcctRevolutionSurfacePayload& the_payload);
static void fillExtrusionSurfacePayload(const Adaptor3d_Surface& the_surface,
                                        LeanOcctExtrusionSurfacePayload& the_payload);
static occ::handle<Adaptor3d_Surface> offsetBasisSurface(const TopoDS_Face& the_face);
static occ::handle<Adaptor3d_Curve> offsetBasisCurve(const TopoDS_Face& the_face);

} // namespace

struct LeanOcctContext
{
  std::string LastError;
};

struct LeanOcctShape
{
  TopoDS_Shape Shape;
};

struct LeanOcctMesh
{
  MeshBuffers Buffers;
};

struct LeanOcctTopology
{
  TopologyBuffers Buffers;
};

namespace
{
static const char* nullContextError()
{
  return "LeanOcctContext was null.";
}

static const char* nullShapeError()
{
  return "LeanOcctShape was null.";
}

static const char* nullMeshError()
{
  return "LeanOcctMesh was null.";
}

static const char* nullTopologyError()
{
  return "LeanOcctTopology was null.";
}

static void clearError(LeanOcctContext* the_context)
{
  if (the_context != nullptr)
  {
    the_context->LastError.clear();
  }
}

static void setError(LeanOcctContext* the_context, const char* the_message)
{
  if (the_context != nullptr)
  {
    the_context->LastError = (the_message == nullptr) ? "Lean OCCT error." : the_message;
  }
}

static void setError(LeanOcctContext* the_context, const std::string& the_message)
{
  if (the_context != nullptr)
  {
    the_context->LastError = the_message;
  }
}

static const TopoDS_Shape& requireShape(const LeanOcctShape* the_shape);
static LeanOcctShape* makeOwnedShape(const TopoDS_Shape& the_shape);

static void requireContext(const LeanOcctContext* the_context)
{
  if (the_context == nullptr)
  {
    throw std::invalid_argument(nullContextError());
  }
}

template <typename T>
static T* requirePointer(T* the_value, const char* the_message)
{
  if (the_value == nullptr)
  {
    throw std::invalid_argument(the_message);
  }
  return the_value;
}

template <typename T>
static const T* requirePointer(const T* the_value, const char* the_message)
{
  if (the_value == nullptr)
  {
    throw std::invalid_argument(the_message);
  }
  return the_value;
}

static const char* requireNonEmptyUtf8Path(const char* the_path_utf8)
{
  const char* a_path_utf8 = requirePointer(the_path_utf8, "STEP path was empty.");
  if (a_path_utf8[0] == '\0')
  {
    throw std::invalid_argument("STEP path was empty.");
  }
  return a_path_utf8;
}

template <typename TFunctor>
static auto guardCall(LeanOcctContext* the_context, TFunctor&& the_functor) -> decltype(the_functor())
{
  clearError(the_context);
  try
  {
    return the_functor();
  }
  catch (const Standard_Failure& the_failure)
  {
    setError(the_context, the_failure.what());
  }
  catch (const std::exception& the_error)
  {
    setError(the_context, the_error.what());
  }
  catch (...)
  {
    setError(the_context, "Unknown Lean OCCT error.");
  }

  using TResult = decltype(the_functor());
  if constexpr (std::is_pointer<TResult>::value)
  {
    return nullptr;
  }
  else if constexpr (std::is_same<TResult, LeanOcctResult>::value)
  {
    return LEAN_OCCT_RESULT_ERROR;
  }
  else
  {
    return TResult();
  }
}

template <typename TFunctor>
static LeanOcctShape* guardShapeCall(LeanOcctContext* the_context, TFunctor&& the_functor)
{
  return guardCall(the_context, [&]() -> LeanOcctShape* {
    requireContext(the_context);
    return makeOwnedShape(the_functor());
  });
}

template <typename TFunctor>
static LeanOcctResult guardResultCall(LeanOcctContext* the_context, TFunctor&& the_functor)
{
  return guardCall(the_context, [&]() -> LeanOcctResult {
    requireContext(the_context);
    the_functor();
    return LEAN_OCCT_RESULT_OK;
  });
}

template <typename TParams, typename TFunctor>
static LeanOcctShape* createShapeFromParams(LeanOcctContext*       the_context,
                                            const TParams*         the_params,
                                            const char*            the_null_params_message,
                                            TFunctor&&             the_functor)
{
  return guardShapeCall(the_context, [&]() -> TopoDS_Shape {
    const TParams* a_params = requirePointer(the_params, the_null_params_message);
    return the_functor(*a_params);
  });
}

template <typename TParams, typename TFunctor>
static LeanOcctShape* createShapeFromInputShape(LeanOcctContext*       the_context,
                                                const LeanOcctShape*   the_shape,
                                                const TParams*         the_params,
                                                const char*            the_null_params_message,
                                                TFunctor&&             the_functor)
{
  return guardShapeCall(the_context, [&]() -> TopoDS_Shape {
    const TParams* a_params = requirePointer(the_params, the_null_params_message);
    return the_functor(requireShape(the_shape), *a_params);
  });
}

template <typename TOutput, typename TFunctor>
static LeanOcctResult writeOutput(LeanOcctContext* the_context,
                                  TOutput*         the_output,
                                  const char*      the_null_output_message,
                                  TFunctor&&       the_functor)
{
  return guardResultCall(the_context, [&]() {
    TOutput* an_output = requirePointer(the_output, the_null_output_message);
    the_functor(*an_output);
  });
}

template <typename TOutput, typename TRequireShape, typename TFillOutput>
static LeanOcctResult fillShapeOutput(LeanOcctContext*      the_context,
                                      const LeanOcctShape*  the_shape,
                                      TOutput*              the_output,
                                      const char*           the_null_output_message,
                                      TRequireShape&&       the_require_shape,
                                      TFillOutput&&         the_fill_output)
{
  return writeOutput(the_context, the_output, the_null_output_message, [&](TOutput& the_result) {
    the_fill_output(the_require_shape(the_shape), the_result);
  });
}

static const TopoDS_Shape& requireShape(const LeanOcctShape* the_shape)
{
  if (the_shape == nullptr)
  {
    throw std::invalid_argument(nullShapeError());
  }
  if (the_shape->Shape.IsNull())
  {
    throw std::invalid_argument("LeanOcctShape contained a null OCCT shape.");
  }
  return the_shape->Shape;
}

static const LeanOcctMesh& requireMesh(const LeanOcctMesh* the_mesh)
{
  if (the_mesh == nullptr)
  {
    throw std::invalid_argument(nullMeshError());
  }
  return *the_mesh;
}

static const LeanOcctTopology& requireTopology(const LeanOcctTopology* the_topology)
{
  if (the_topology == nullptr)
  {
    throw std::invalid_argument(nullTopologyError());
  }
  return *the_topology;
}

static LeanOcctShape* makeOwnedShape(const TopoDS_Shape& the_shape)
{
  if (the_shape.IsNull())
  {
    throw std::runtime_error("OCCT operation produced a null shape.");
  }

  LeanOcctShape* a_shape = new LeanOcctShape();
  a_shape->Shape = the_shape;
  return a_shape;
}

static TopoDS_Vertex requireVertexShape(const LeanOcctShape* the_shape)
{
  const TopoDS_Shape& a_shape = requireShape(the_shape);
  if (a_shape.ShapeType() != TopAbs_VERTEX)
  {
    throw std::invalid_argument("LeanOcctShape was not a vertex.");
  }
  return TopoDS::Vertex(a_shape);
}

static TopoDS_Edge requireEdgeShape(const LeanOcctShape* the_shape)
{
  const TopoDS_Shape& a_shape = requireShape(the_shape);
  if (a_shape.ShapeType() != TopAbs_EDGE)
  {
    throw std::invalid_argument("LeanOcctShape was not an edge.");
  }
  return TopoDS::Edge(a_shape);
}

static TopoDS_Face requireFaceShape(const LeanOcctShape* the_shape)
{
  const TopoDS_Shape& a_shape = requireShape(the_shape);
  if (a_shape.ShapeType() != TopAbs_FACE)
  {
    throw std::invalid_argument("LeanOcctShape was not a face.");
  }
  return TopoDS::Face(a_shape);
}

static LeanOcctShape* makeBooleanResult(const TopoDS_Shape& the_lhs,
                                        const TopoDS_Shape& the_rhs,
                                        const int           the_kind)
{
  if (the_kind == 0)
  {
    BRepAlgoAPI_Cut an_op(the_lhs, the_rhs);
    an_op.Build();
    if (!an_op.IsDone())
    {
      throw std::runtime_error("Boolean cut failed.");
    }
    return makeOwnedShape(an_op.Shape());
  }

  if (the_kind == 1)
  {
    BRepAlgoAPI_Fuse an_op(the_lhs, the_rhs);
    an_op.Build();
    if (!an_op.IsDone())
    {
      throw std::runtime_error("Boolean fuse failed.");
    }
    return makeOwnedShape(an_op.Shape());
  }

  BRepAlgoAPI_Common an_op(the_lhs, the_rhs);
  an_op.Build();
  if (!an_op.IsDone())
  {
    throw std::runtime_error("Boolean common failed.");
  }
  return makeOwnedShape(an_op.Shape());
}

static TopoDS_Edge requireIndexedEdge(const TopoDS_Shape& the_shape, uint32_t the_edge_index)
{
  uint32_t a_current_index = 0;
  for (TopExp_Explorer an_edge_exp(the_shape, TopAbs_EDGE); an_edge_exp.More();
       an_edge_exp.Next(), ++a_current_index)
  {
    if (a_current_index == the_edge_index)
    {
      return TopoDS::Edge(an_edge_exp.Current());
    }
  }

  throw std::out_of_range("Requested edge index was out of range.");
}

static std::size_t countSubshapes(const TopoDS_Shape& the_shape, TopAbs_ShapeEnum the_kind)
{
  std::size_t a_count = 0;
  for (TopExp_Explorer an_exp(the_shape, the_kind); an_exp.More(); an_exp.Next())
  {
    ++a_count;
  }
  return a_count;
}

static std::size_t countSubshapesInclusive(const TopoDS_Shape& the_shape, TopAbs_ShapeEnum the_kind)
{
  return countSubshapes(the_shape, the_kind);
}

static TopAbs_ShapeEnum toTopAbsShapeEnum(LeanOcctShapeKind the_kind)
{
  switch (the_kind)
  {
    case LEAN_OCCT_SHAPE_KIND_COMPOUND:
      return TopAbs_COMPOUND;
    case LEAN_OCCT_SHAPE_KIND_COMPSOLID:
      return TopAbs_COMPSOLID;
    case LEAN_OCCT_SHAPE_KIND_SOLID:
      return TopAbs_SOLID;
    case LEAN_OCCT_SHAPE_KIND_SHELL:
      return TopAbs_SHELL;
    case LEAN_OCCT_SHAPE_KIND_FACE:
      return TopAbs_FACE;
    case LEAN_OCCT_SHAPE_KIND_WIRE:
      return TopAbs_WIRE;
    case LEAN_OCCT_SHAPE_KIND_EDGE:
      return TopAbs_EDGE;
    case LEAN_OCCT_SHAPE_KIND_VERTEX:
      return TopAbs_VERTEX;
    case LEAN_OCCT_SHAPE_KIND_UNKNOWN:
    case LEAN_OCCT_SHAPE_KIND_SHAPE:
      break;
  }

  throw std::invalid_argument("Requested shape kind is not traversable.");
}

static std::size_t countIndexedSubshapes(const TopoDS_Shape& the_shape, LeanOcctShapeKind the_kind)
{
  NCollection_IndexedMap<TopoDS_Shape, TopTools_ShapeMapHasher> a_map;
  TopExp::MapShapes(the_shape, toTopAbsShapeEnum(the_kind), a_map);
  return static_cast<std::size_t>(a_map.Extent());
}

static TopoDS_Shape indexedSubshape(const TopoDS_Shape& the_shape,
                                    LeanOcctShapeKind   the_kind,
                                    std::size_t         the_index)
{
  NCollection_IndexedMap<TopoDS_Shape, TopTools_ShapeMapHasher> a_map;
  TopExp::MapShapes(the_shape, toTopAbsShapeEnum(the_kind), a_map);
  if (the_index >= static_cast<std::size_t>(a_map.Extent()))
  {
    throw std::out_of_range("Requested subshape index was out of range.");
  }

  return a_map.FindKey(static_cast<int>(the_index + 1));
}

static uint32_t toUInt32(std::size_t the_value)
{
  if (the_value > static_cast<std::size_t>(UINT32_MAX))
  {
    throw std::overflow_error("Topology snapshot exceeded 32-bit index limits.");
  }
  return static_cast<uint32_t>(the_value);
}

static uint32_t toOptionalIndex(int the_index)
{
  return the_index > 0 ? toUInt32(static_cast<std::size_t>(the_index - 1)) : UINT32_MAX;
}

static LeanOcctOrientation toLeanOcctOrientation(TopAbs_Orientation the_orientation)
{
  switch (the_orientation)
  {
    case TopAbs_FORWARD:
      return LEAN_OCCT_ORIENTATION_FORWARD;
    case TopAbs_REVERSED:
      return LEAN_OCCT_ORIENTATION_REVERSED;
    case TopAbs_INTERNAL:
      return LEAN_OCCT_ORIENTATION_INTERNAL;
    case TopAbs_EXTERNAL:
      return LEAN_OCCT_ORIENTATION_EXTERNAL;
  }

  return LEAN_OCCT_ORIENTATION_FORWARD;
}

static LeanOcctLoopRole toLeanOcctLoopRole(const TopoDS_Wire& the_wire, const TopoDS_Wire& the_outer_wire)
{
  if (the_outer_wire.IsNull())
  {
    return LEAN_OCCT_LOOP_ROLE_UNKNOWN;
  }
  return the_wire.IsSame(the_outer_wire) ? LEAN_OCCT_LOOP_ROLE_OUTER : LEAN_OCCT_LOOP_ROLE_INNER;
}

static LeanOcctShapeKind toLeanOcctShapeKind(TopAbs_ShapeEnum the_kind)
{
  switch (the_kind)
  {
    case TopAbs_COMPOUND:
      return LEAN_OCCT_SHAPE_KIND_COMPOUND;
    case TopAbs_COMPSOLID:
      return LEAN_OCCT_SHAPE_KIND_COMPSOLID;
    case TopAbs_SOLID:
      return LEAN_OCCT_SHAPE_KIND_SOLID;
    case TopAbs_SHELL:
      return LEAN_OCCT_SHAPE_KIND_SHELL;
    case TopAbs_FACE:
      return LEAN_OCCT_SHAPE_KIND_FACE;
    case TopAbs_WIRE:
      return LEAN_OCCT_SHAPE_KIND_WIRE;
    case TopAbs_EDGE:
      return LEAN_OCCT_SHAPE_KIND_EDGE;
    case TopAbs_VERTEX:
      return LEAN_OCCT_SHAPE_KIND_VERTEX;
    case TopAbs_SHAPE:
      return LEAN_OCCT_SHAPE_KIND_SHAPE;
  }

  return LEAN_OCCT_SHAPE_KIND_UNKNOWN;
}

static LeanOcctCurveKind toLeanOcctCurveKind(GeomAbs_CurveType the_kind)
{
  switch (the_kind)
  {
    case GeomAbs_Line:
      return LEAN_OCCT_CURVE_KIND_LINE;
    case GeomAbs_Circle:
      return LEAN_OCCT_CURVE_KIND_CIRCLE;
    case GeomAbs_Ellipse:
      return LEAN_OCCT_CURVE_KIND_ELLIPSE;
    case GeomAbs_Hyperbola:
      return LEAN_OCCT_CURVE_KIND_HYPERBOLA;
    case GeomAbs_Parabola:
      return LEAN_OCCT_CURVE_KIND_PARABOLA;
    case GeomAbs_BezierCurve:
      return LEAN_OCCT_CURVE_KIND_BEZIER;
    case GeomAbs_BSplineCurve:
      return LEAN_OCCT_CURVE_KIND_BSPLINE;
    case GeomAbs_OffsetCurve:
      return LEAN_OCCT_CURVE_KIND_OFFSET;
    case GeomAbs_OtherCurve:
      return LEAN_OCCT_CURVE_KIND_OTHER;
  }

  return LEAN_OCCT_CURVE_KIND_UNKNOWN;
}

static LeanOcctSurfaceKind toLeanOcctSurfaceKind(GeomAbs_SurfaceType the_kind)
{
  switch (the_kind)
  {
    case GeomAbs_Plane:
      return LEAN_OCCT_SURFACE_KIND_PLANE;
    case GeomAbs_Cylinder:
      return LEAN_OCCT_SURFACE_KIND_CYLINDER;
    case GeomAbs_Cone:
      return LEAN_OCCT_SURFACE_KIND_CONE;
    case GeomAbs_Sphere:
      return LEAN_OCCT_SURFACE_KIND_SPHERE;
    case GeomAbs_Torus:
      return LEAN_OCCT_SURFACE_KIND_TORUS;
    case GeomAbs_BezierSurface:
      return LEAN_OCCT_SURFACE_KIND_BEZIER;
    case GeomAbs_BSplineSurface:
      return LEAN_OCCT_SURFACE_KIND_BSPLINE;
    case GeomAbs_SurfaceOfRevolution:
      return LEAN_OCCT_SURFACE_KIND_REVOLUTION;
    case GeomAbs_SurfaceOfExtrusion:
      return LEAN_OCCT_SURFACE_KIND_EXTRUSION;
    case GeomAbs_OffsetSurface:
      return LEAN_OCCT_SURFACE_KIND_OFFSET;
    case GeomAbs_OtherSurface:
      return LEAN_OCCT_SURFACE_KIND_OTHER;
  }

  return LEAN_OCCT_SURFACE_KIND_UNKNOWN;
}

static LeanOcctShapeKind detectPrimaryKind(const LeanOcctShapeSummary& the_summary)
{
  if (the_summary.solid_count > 0)
  {
    return LEAN_OCCT_SHAPE_KIND_SOLID;
  }
  if (the_summary.shell_count > 0)
  {
    return LEAN_OCCT_SHAPE_KIND_SHELL;
  }
  if (the_summary.face_count > 0)
  {
    return LEAN_OCCT_SHAPE_KIND_FACE;
  }
  if (the_summary.wire_count > 0)
  {
    return LEAN_OCCT_SHAPE_KIND_WIRE;
  }
  if (the_summary.edge_count > 0)
  {
    return LEAN_OCCT_SHAPE_KIND_EDGE;
  }
  if (the_summary.vertex_count > 0)
  {
    return LEAN_OCCT_SHAPE_KIND_VERTEX;
  }
  if (the_summary.compsolid_count > 0)
  {
    return LEAN_OCCT_SHAPE_KIND_COMPSOLID;
  }
  if (the_summary.compound_count > 0)
  {
    return LEAN_OCCT_SHAPE_KIND_COMPOUND;
  }
  return the_summary.root_kind;
}

static void fillShapeSummary(const TopoDS_Shape& the_shape, LeanOcctShapeSummary& the_summary)
{
  the_summary.root_kind = toLeanOcctShapeKind(the_shape.ShapeType());
  the_summary.primary_kind = LEAN_OCCT_SHAPE_KIND_UNKNOWN;
  the_summary.compound_count = countSubshapesInclusive(the_shape, TopAbs_COMPOUND);
  the_summary.compsolid_count = countSubshapesInclusive(the_shape, TopAbs_COMPSOLID);
  the_summary.solid_count = countSubshapesInclusive(the_shape, TopAbs_SOLID);
  the_summary.shell_count = countSubshapesInclusive(the_shape, TopAbs_SHELL);
  the_summary.face_count = countSubshapesInclusive(the_shape, TopAbs_FACE);
  the_summary.wire_count = countSubshapesInclusive(the_shape, TopAbs_WIRE);
  the_summary.edge_count = countSubshapesInclusive(the_shape, TopAbs_EDGE);
  the_summary.vertex_count = countSubshapesInclusive(the_shape, TopAbs_VERTEX);
  the_summary.linear_length = 0.0;
  the_summary.surface_area = 0.0;
  the_summary.volume = 0.0;
  the_summary.bbox_min[0] = 0.0;
  the_summary.bbox_min[1] = 0.0;
  the_summary.bbox_min[2] = 0.0;
  the_summary.bbox_max[0] = 0.0;
  the_summary.bbox_max[1] = 0.0;
  the_summary.bbox_max[2] = 0.0;

  if (the_summary.edge_count > 0)
  {
    GProp_GProps a_linear_props;
    BRepGProp::LinearProperties(the_shape, a_linear_props);
    the_summary.linear_length = a_linear_props.Mass();
  }

  if (the_summary.face_count > 0)
  {
    GProp_GProps a_surface_props;
    BRepGProp::SurfaceProperties(the_shape, a_surface_props);
    the_summary.surface_area = a_surface_props.Mass();
  }

  if (the_summary.solid_count > 0 || the_summary.compsolid_count > 0)
  {
    GProp_GProps a_volume_props;
    BRepGProp::VolumeProperties(the_shape, a_volume_props);
    the_summary.volume = a_volume_props.Mass();
  }

  Bnd_Box a_bounds;
  BRepBndLib::Add(the_shape, a_bounds, false);
  if (!a_bounds.IsVoid())
  {
    a_bounds.Get(the_summary.bbox_min[0],
                 the_summary.bbox_min[1],
                 the_summary.bbox_min[2],
                 the_summary.bbox_max[0],
                 the_summary.bbox_max[1],
                 the_summary.bbox_max[2]);
  }

  the_summary.primary_kind = detectPrimaryKind(the_summary);
}

static void writePoint(double* the_xyz3, const gp_Pnt& the_point)
{
  the_xyz3[0] = the_point.X();
  the_xyz3[1] = the_point.Y();
  the_xyz3[2] = the_point.Z();
}

static gp_Dir requireDirection(const gp_Vec& the_vector, const char* the_message)
{
  if (the_vector.SquareMagnitude() <= Precision::Confusion() * Precision::Confusion())
  {
    throw std::runtime_error(the_message);
  }
  return gp_Dir(the_vector);
}

static gp_Dir makeDirection(const double the_x,
                            const double the_y,
                            const double the_z,
                            const char*  the_message)
{
  return requireDirection(gp_Vec(the_x, the_y, the_z), the_message);
}

static gp_Ax1 makeAxis1(const double the_x,
                        const double the_y,
                        const double the_z,
                        const double the_dx,
                        const double the_dy,
                        const double the_dz,
                        const char*  the_message)
{
  return gp_Ax1(gp_Pnt(the_x, the_y, the_z), makeDirection(the_dx, the_dy, the_dz, the_message));
}

static gp_Ax2 makeAxis2(const double the_x,
                        const double the_y,
                        const double the_z,
                        const double the_dx,
                        const double the_dy,
                        const double the_dz,
                        const double the_xx,
                        const double the_xy,
                        const double the_xz,
                        const char*  the_axis_message,
                        const char*  the_xdir_message)
{
  return gp_Ax2(gp_Pnt(the_x, the_y, the_z),
                makeDirection(the_dx, the_dy, the_dz, the_axis_message),
                makeDirection(the_xx, the_xy, the_xz, the_xdir_message));
}

static gp_Ax3 makeAxis3(const double the_x,
                        const double the_y,
                        const double the_z,
                        const double the_dx,
                        const double the_dy,
                        const double the_dz,
                        const double the_xx,
                        const double the_xy,
                        const double the_xz,
                        const char*  the_axis_message,
                        const char*  the_xdir_message)
{
  return gp_Ax3(gp_Pnt(the_x, the_y, the_z),
                makeDirection(the_dx, the_dy, the_dz, the_axis_message),
                makeDirection(the_xx, the_xy, the_xz, the_xdir_message));
}

static bool isReversedEdgeSampling(const TopoDS_Edge& the_edge, const BRepAdaptor_Curve& the_curve)
{
  TopoDS_Vertex a_first_vertex;
  TopoDS_Vertex a_last_vertex;
  TopExp::Vertices(the_edge, a_first_vertex, a_last_vertex);
  if (!a_first_vertex.IsNull() && !a_last_vertex.IsNull())
  {
    const gp_Pnt a_start_point = BRep_Tool::Pnt(a_first_vertex);
    const double a_first_distance =
      a_start_point.SquareDistance(the_curve.Value(the_curve.FirstParameter()));
    const double a_last_distance =
      a_start_point.SquareDistance(the_curve.Value(the_curve.LastParameter()));
    if (a_first_distance < a_last_distance)
    {
      return false;
    }
    if (a_first_distance > a_last_distance)
    {
      return true;
    }
  }

  return the_edge.Orientation() == TopAbs_REVERSED;
}

static void fillEdgeGeometry(const TopoDS_Edge& the_edge, LeanOcctEdgeGeometry& the_geometry)
{
  const BRepAdaptor_Curve a_curve(the_edge);
  const bool              is_reversed = isReversedEdgeSampling(the_edge, a_curve);
  const double            a_first_parameter = a_curve.FirstParameter();
  const double            a_last_parameter = a_curve.LastParameter();

  the_geometry.kind = toLeanOcctCurveKind(a_curve.GetType());
  the_geometry.start_parameter = is_reversed ? a_last_parameter : a_first_parameter;
  the_geometry.end_parameter = is_reversed ? a_first_parameter : a_last_parameter;
  the_geometry.is_closed = a_curve.IsClosed() ? 1U : 0U;
  the_geometry.is_periodic = a_curve.IsPeriodic() ? 1U : 0U;
  the_geometry.period = the_geometry.is_periodic != 0 ? a_curve.Period() : 0.0;
}

static double interpolateRange(const double the_first, const double the_last, const double the_t)
{
  return the_first + (the_last - the_first) * the_t;
}

static bool isParameterInsideRange(const double the_parameter,
                                   const double the_first,
                                   const double the_last)
{
  const double a_min = std::min(the_first, the_last);
  const double a_max = std::max(the_first, the_last);
  const double a_tol = Precision::Confusion();
  return the_parameter >= a_min - a_tol && the_parameter <= a_max + a_tol;
}

static void sampleEdgeAtParameter(const TopoDS_Edge&      the_edge,
                                  const double            the_parameter,
                                  LeanOcctEdgeSample&     the_sample)
{
  const BRepAdaptor_Curve a_curve(the_edge);
  const bool              is_reversed = isReversedEdgeSampling(the_edge, a_curve);

  gp_Pnt a_point;
  gp_Vec a_tangent;
  a_curve.D1(the_parameter, a_point, a_tangent);
  if (is_reversed)
  {
    a_tangent.Reverse();
  }

  writePoint(the_sample.position, a_point);
  writeDir(the_sample.tangent,
           requireDirection(a_tangent, "Edge tangent was undefined at the requested parameter."));
}

static void fillCurveGeometry(const Adaptor3d_Curve& the_curve, LeanOcctEdgeGeometry& the_geometry)
{
  the_geometry.kind = toLeanOcctCurveKind(the_curve.GetType());
  the_geometry.start_parameter = the_curve.FirstParameter();
  the_geometry.end_parameter = the_curve.LastParameter();
  the_geometry.is_closed = the_curve.IsClosed() ? 1U : 0U;
  the_geometry.is_periodic = the_curve.IsPeriodic() ? 1U : 0U;
  the_geometry.period = the_geometry.is_periodic != 0 ? the_curve.Period() : 0.0;
}

static void fillFaceGeometry(const TopoDS_Face& the_face, LeanOcctFaceGeometry& the_geometry)
{
  const BRepAdaptor_Surface a_surface(the_face);
  fillFaceGeometry(a_surface, the_geometry);
}

static void fillFaceGeometry(const Adaptor3d_Surface& the_surface, LeanOcctFaceGeometry& the_geometry)
{
  the_geometry.kind = toLeanOcctSurfaceKind(the_surface.GetType());
  the_geometry.u_min = the_surface.FirstUParameter();
  the_geometry.u_max = the_surface.LastUParameter();
  the_geometry.v_min = the_surface.FirstVParameter();
  the_geometry.v_max = the_surface.LastVParameter();
  the_geometry.is_u_closed = the_surface.IsUClosed() ? 1U : 0U;
  the_geometry.is_v_closed = the_surface.IsVClosed() ? 1U : 0U;
  the_geometry.is_u_periodic = the_surface.IsUPeriodic() ? 1U : 0U;
  the_geometry.is_v_periodic = the_surface.IsVPeriodic() ? 1U : 0U;
  the_geometry.u_period = the_geometry.is_u_periodic != 0 ? the_surface.UPeriod() : 0.0;
  the_geometry.v_period = the_geometry.is_v_periodic != 0 ? the_surface.VPeriod() : 0.0;
}

static void sampleFaceAtUv(const TopoDS_Face&     the_face,
                           const double           the_u,
                           const double           the_v,
                           LeanOcctFaceSample&    the_sample)
{
  const BRepAdaptor_Surface a_surface(the_face);
  const gp_Pnt              a_point = a_surface.Value(the_u, the_v);
  BRepLProp_SLProps         a_props(a_surface, the_u, the_v, 1, Precision::Confusion());
  if (!a_props.IsNormalDefined())
  {
    throw std::runtime_error("Face normal was undefined at the requested UV parameter.");
  }

  gp_Dir a_normal = a_props.Normal();
  if (the_face.Orientation() == TopAbs_REVERSED)
  {
    a_normal.Reverse();
  }

  writePoint(the_sample.position, a_point);
  writeDir(the_sample.normal, a_normal);
}

static double dotProduct(const double* the_lhs_xyz3, const double* the_rhs_xyz3)
{
  return the_lhs_xyz3[0] * the_rhs_xyz3[0] + the_lhs_xyz3[1] * the_rhs_xyz3[1]
         + the_lhs_xyz3[2] * the_rhs_xyz3[2];
}

static void fillLinePayload(const TopoDS_Edge& the_edge, LeanOcctLinePayload& the_payload)
{
  const BRepAdaptor_Curve a_curve(the_edge);
  if (a_curve.GetType() != GeomAbs_Line)
  {
    throw std::invalid_argument("LeanOcctShape edge was not a line.");
  }

  const gp_Lin a_line = a_curve.Line();
  writePoint(the_payload.origin, a_line.Location());
  LeanOcctEdgeGeometry a_geometry = {};
  LeanOcctEdgeSample   a_mid_sample = {};
  fillEdgeGeometry(the_edge, a_geometry);
  sampleEdgeAtParameter(the_edge,
                        interpolateRange(a_geometry.start_parameter, a_geometry.end_parameter, 0.5),
                        a_mid_sample);

  gp_Dir a_direction = a_line.Direction();
  double a_direction_xyz[3];
  writeDir(a_direction_xyz, a_direction);
  if (dotProduct(a_direction_xyz, a_mid_sample.tangent) < 0.0)
  {
    a_direction.Reverse();
  }
  writeDir(the_payload.direction, a_direction);
}

static void fillCirclePayload(const TopoDS_Edge& the_edge, LeanOcctCirclePayload& the_payload)
{
  const BRepAdaptor_Curve a_curve(the_edge);
  if (a_curve.GetType() != GeomAbs_Circle)
  {
    throw std::invalid_argument("LeanOcctShape edge was not a circle.");
  }

  const gp_Circ a_circle = a_curve.Circle();
  writePoint(the_payload.center, a_circle.Location());
  writeDir(the_payload.normal, a_circle.Position().Direction());
  writeDir(the_payload.x_direction, a_circle.Position().XDirection());
  writeDir(the_payload.y_direction, a_circle.Position().YDirection());
  the_payload.radius = a_circle.Radius();
}

static void fillLinePayload(const Adaptor3d_Curve& the_curve, LeanOcctLinePayload& the_payload)
{
  if (the_curve.GetType() != GeomAbs_Line)
  {
    throw std::invalid_argument("LeanOcct basis curve was not a line.");
  }

  const gp_Lin a_line = the_curve.Line();
  writePoint(the_payload.origin, a_line.Location());
  writeDir(the_payload.direction, a_line.Direction());
}

static void fillCirclePayload(const Adaptor3d_Curve& the_curve, LeanOcctCirclePayload& the_payload)
{
  if (the_curve.GetType() != GeomAbs_Circle)
  {
    throw std::invalid_argument("LeanOcct basis curve was not a circle.");
  }

  const gp_Circ a_circle = the_curve.Circle();
  writePoint(the_payload.center, a_circle.Location());
  writeDir(the_payload.normal, a_circle.Position().Direction());
  writeDir(the_payload.x_direction, a_circle.Position().XDirection());
  writeDir(the_payload.y_direction, a_circle.Position().YDirection());
  the_payload.radius = a_circle.Radius();
}

static void fillPlanePayload(const TopoDS_Face& the_face, LeanOcctPlanePayload& the_payload)
{
  const BRepAdaptor_Surface a_surface(the_face);
  fillPlanePayload(a_surface, the_payload);
}

static void fillPlanePayload(const Adaptor3d_Surface& the_surface,
                             LeanOcctPlanePayload&    the_payload)
{
  if (the_surface.GetType() != GeomAbs_Plane)
  {
    throw std::invalid_argument("LeanOcctShape face was not a plane.");
  }

  const gp_Pln a_plane = the_surface.Plane();
  writePoint(the_payload.origin, a_plane.Location());
  writeDir(the_payload.normal, a_plane.Position().Direction());
  writeDir(the_payload.x_direction, a_plane.Position().XDirection());
  writeDir(the_payload.y_direction, a_plane.Position().YDirection());
}

static void fillCylinderPayload(const TopoDS_Face& the_face, LeanOcctCylinderPayload& the_payload)
{
  const BRepAdaptor_Surface a_surface(the_face);
  fillCylinderPayload(a_surface, the_payload);
}

static void fillCylinderPayload(const Adaptor3d_Surface& the_surface,
                                LeanOcctCylinderPayload& the_payload)
{
  if (the_surface.GetType() != GeomAbs_Cylinder)
  {
    throw std::invalid_argument("LeanOcctShape face was not a cylinder.");
  }

  const gp_Cylinder a_cylinder = the_surface.Cylinder();
  writePoint(the_payload.origin, a_cylinder.Location());
  writeDir(the_payload.axis, a_cylinder.Position().Direction());
  writeDir(the_payload.x_direction, a_cylinder.Position().XDirection());
  writeDir(the_payload.y_direction, a_cylinder.Position().YDirection());
  the_payload.radius = a_cylinder.Radius();
}

static void fillEllipsePayload(const TopoDS_Edge& the_edge, LeanOcctEllipsePayload& the_payload)
{
  const BRepAdaptor_Curve a_curve(the_edge);
  if (a_curve.GetType() != GeomAbs_Ellipse)
  {
    throw std::invalid_argument("LeanOcctShape edge was not an ellipse.");
  }

  const gp_Elips an_ellipse = a_curve.Ellipse();
  writePoint(the_payload.center, an_ellipse.Location());
  writeDir(the_payload.normal, an_ellipse.Position().Direction());
  writeDir(the_payload.x_direction, an_ellipse.Position().XDirection());
  writeDir(the_payload.y_direction, an_ellipse.Position().YDirection());
  the_payload.major_radius = an_ellipse.MajorRadius();
  the_payload.minor_radius = an_ellipse.MinorRadius();
}

static void fillEllipsePayload(const Adaptor3d_Curve& the_curve, LeanOcctEllipsePayload& the_payload)
{
  if (the_curve.GetType() != GeomAbs_Ellipse)
  {
    throw std::invalid_argument("LeanOcct basis curve was not an ellipse.");
  }

  const gp_Elips an_ellipse = the_curve.Ellipse();
  writePoint(the_payload.center, an_ellipse.Location());
  writeDir(the_payload.normal, an_ellipse.Position().Direction());
  writeDir(the_payload.x_direction, an_ellipse.Position().XDirection());
  writeDir(the_payload.y_direction, an_ellipse.Position().YDirection());
  the_payload.major_radius = an_ellipse.MajorRadius();
  the_payload.minor_radius = an_ellipse.MinorRadius();
}

static void fillConePayload(const TopoDS_Face& the_face, LeanOcctConePayload& the_payload)
{
  const BRepAdaptor_Surface a_surface(the_face);
  fillConePayload(a_surface, the_payload);
}

static void fillConePayload(const Adaptor3d_Surface& the_surface, LeanOcctConePayload& the_payload)
{
  if (the_surface.GetType() != GeomAbs_Cone)
  {
    throw std::invalid_argument("LeanOcctShape face was not a cone.");
  }

  const gp_Cone a_cone = the_surface.Cone();
  writePoint(the_payload.origin, a_cone.Location());
  writeDir(the_payload.axis, a_cone.Position().Direction());
  writeDir(the_payload.x_direction, a_cone.Position().XDirection());
  writeDir(the_payload.y_direction, a_cone.Position().YDirection());
  the_payload.reference_radius = a_cone.RefRadius();
  the_payload.semi_angle = a_cone.SemiAngle();
}

static void fillSpherePayload(const TopoDS_Face& the_face, LeanOcctSpherePayload& the_payload)
{
  const BRepAdaptor_Surface a_surface(the_face);
  fillSpherePayload(a_surface, the_payload);
}

static void fillSpherePayload(const Adaptor3d_Surface& the_surface,
                              LeanOcctSpherePayload&   the_payload)
{
  if (the_surface.GetType() != GeomAbs_Sphere)
  {
    throw std::invalid_argument("LeanOcctShape face was not a sphere.");
  }

  const gp_Sphere a_sphere = the_surface.Sphere();
  writePoint(the_payload.center, a_sphere.Location());
  writeDir(the_payload.normal, a_sphere.Position().Direction());
  writeDir(the_payload.x_direction, a_sphere.Position().XDirection());
  writeDir(the_payload.y_direction, a_sphere.Position().YDirection());
  the_payload.radius = a_sphere.Radius();
}

static void fillTorusPayload(const TopoDS_Face& the_face, LeanOcctTorusPayload& the_payload)
{
  const BRepAdaptor_Surface a_surface(the_face);
  fillTorusPayload(a_surface, the_payload);
}

static void fillTorusPayload(const Adaptor3d_Surface& the_surface, LeanOcctTorusPayload& the_payload)
{
  if (the_surface.GetType() != GeomAbs_Torus)
  {
    throw std::invalid_argument("LeanOcctShape face was not a torus.");
  }

  const gp_Torus a_torus = the_surface.Torus();
  writePoint(the_payload.center, a_torus.Location());
  writeDir(the_payload.axis, a_torus.Position().Direction());
  writeDir(the_payload.x_direction, a_torus.Position().XDirection());
  writeDir(the_payload.y_direction, a_torus.Position().YDirection());
  the_payload.major_radius = a_torus.MajorRadius();
  the_payload.minor_radius = a_torus.MinorRadius();
}

static void fillRevolutionSurfacePayload(const TopoDS_Face& the_face,
                                         LeanOcctRevolutionSurfacePayload& the_payload)
{
  const BRepAdaptor_Surface a_surface(the_face);
  fillRevolutionSurfacePayload(a_surface, the_payload);
}

static void fillRevolutionSurfacePayload(const Adaptor3d_Surface&         the_surface,
                                         LeanOcctRevolutionSurfacePayload& the_payload)
{
  if (the_surface.GetType() != GeomAbs_SurfaceOfRevolution)
  {
    throw std::invalid_argument("LeanOcctShape face was not a surface of revolution.");
  }

  const gp_Ax1 an_axis = the_surface.AxeOfRevolution();
  writePoint(the_payload.axis_origin, an_axis.Location());
  writeDir(the_payload.axis_direction, an_axis.Direction());
  const occ::handle<Adaptor3d_Curve> a_basis_curve = the_surface.BasisCurve();
  if (a_basis_curve.IsNull())
  {
    throw std::runtime_error("Surface of revolution basis curve was null.");
  }
  the_payload.basis_curve_kind = toLeanOcctCurveKind(a_basis_curve->GetType());
}

static void fillExtrusionSurfacePayload(const TopoDS_Face& the_face,
                                        LeanOcctExtrusionSurfacePayload& the_payload)
{
  const BRepAdaptor_Surface a_surface(the_face);
  fillExtrusionSurfacePayload(a_surface, the_payload);
}

static void fillExtrusionSurfacePayload(const Adaptor3d_Surface&        the_surface,
                                        LeanOcctExtrusionSurfacePayload& the_payload)
{
  if (the_surface.GetType() != GeomAbs_SurfaceOfExtrusion)
  {
    throw std::invalid_argument("LeanOcctShape face was not a surface of extrusion.");
  }

  writeDir(the_payload.direction, the_surface.Direction());
  const occ::handle<Adaptor3d_Curve> a_basis_curve = the_surface.BasisCurve();
  if (a_basis_curve.IsNull())
  {
    throw std::runtime_error("Surface of extrusion basis curve was null.");
  }
  the_payload.basis_curve_kind = toLeanOcctCurveKind(a_basis_curve->GetType());
}

static void fillOffsetSurfacePayload(const TopoDS_Face& the_face,
                                     LeanOcctOffsetSurfacePayload& the_payload)
{
  const BRepAdaptor_Surface a_surface(the_face);
  if (a_surface.GetType() != GeomAbs_OffsetSurface)
  {
    throw std::invalid_argument("LeanOcctShape face was not an offset surface.");
  }

  the_payload.offset_value = a_surface.OffsetValue();
  const occ::handle<Adaptor3d_Surface> a_basis_surface = a_surface.BasisSurface();
  if (a_basis_surface.IsNull())
  {
    throw std::runtime_error("Offset surface basis surface was null.");
  }
  the_payload.basis_surface_kind = toLeanOcctSurfaceKind(a_basis_surface->GetType());
}

static occ::handle<Adaptor3d_Surface> offsetBasisSurface(const TopoDS_Face& the_face)
{
  const BRepAdaptor_Surface a_surface(the_face);
  if (a_surface.GetType() != GeomAbs_OffsetSurface)
  {
    throw std::invalid_argument("LeanOcctShape face was not an offset surface.");
  }

  const occ::handle<Adaptor3d_Surface> a_basis_surface = a_surface.BasisSurface();
  if (a_basis_surface.IsNull())
  {
    throw std::runtime_error("Offset surface basis surface was null.");
  }
  return a_basis_surface;
}

static occ::handle<Adaptor3d_Curve> offsetBasisCurve(const TopoDS_Face& the_face)
{
  const occ::handle<Adaptor3d_Surface> a_basis_surface = offsetBasisSurface(the_face);
  const occ::handle<Adaptor3d_Curve>   a_basis_curve = a_basis_surface->BasisCurve();
  if (a_basis_curve.IsNull())
  {
    throw std::runtime_error("Offset surface basis curve was null.");
  }
  return a_basis_curve;
}

static TopologyBuffers snapshotTopology(const TopoDS_Shape& the_shape)
{
  TopologyBuffers a_buffers;

  NCollection_IndexedMap<TopoDS_Shape, TopTools_ShapeMapHasher> a_vertex_map;
  NCollection_IndexedMap<TopoDS_Shape, TopTools_ShapeMapHasher> an_edge_map;
  NCollection_IndexedMap<TopoDS_Shape, TopTools_ShapeMapHasher> a_wire_map;
  NCollection_IndexedMap<TopoDS_Shape, TopTools_ShapeMapHasher> a_face_map;
  EdgeFaceMap                                                   an_edge_face_map;
  TopExp::MapShapes(the_shape, TopAbs_VERTEX, a_vertex_map);
  TopExp::MapShapes(the_shape, TopAbs_EDGE, an_edge_map);
  TopExp::MapShapes(the_shape, TopAbs_WIRE, a_wire_map);
  TopExp::MapShapes(the_shape, TopAbs_FACE, a_face_map);
  TopExp::MapShapesAndUniqueAncestors(the_shape, TopAbs_EDGE, TopAbs_FACE, an_edge_face_map);

  a_buffers.vertexPositions.reserve(static_cast<std::size_t>(a_vertex_map.Extent()) * 3);
  a_buffers.edgeVertexIndices.reserve(static_cast<std::size_t>(an_edge_map.Extent()) * 2);
  a_buffers.edgeLengths.reserve(static_cast<std::size_t>(an_edge_map.Extent()));
  a_buffers.edgeFaceRanges.reserve(static_cast<std::size_t>(an_edge_map.Extent()) * 2);
  a_buffers.wireRanges.reserve(static_cast<std::size_t>(a_wire_map.Extent()) * 2);
  a_buffers.wireVertexRanges.reserve(static_cast<std::size_t>(a_wire_map.Extent()) * 2);
  a_buffers.faceRanges.reserve(static_cast<std::size_t>(a_face_map.Extent()) * 2);

  for (int a_vertex_index = 1; a_vertex_index <= a_vertex_map.Extent(); ++a_vertex_index)
  {
    appendPoint(a_buffers.vertexPositions, BRep_Tool::Pnt(TopoDS::Vertex(a_vertex_map.FindKey(a_vertex_index))));
  }

  for (int an_edge_index = 1; an_edge_index <= an_edge_map.Extent(); ++an_edge_index)
  {
    const TopoDS_Edge& an_edge = TopoDS::Edge(an_edge_map.FindKey(an_edge_index));
    TopoDS_Vertex      a_first_vertex;
    TopoDS_Vertex      a_last_vertex;
    TopExp::Vertices(an_edge, a_first_vertex, a_last_vertex);
    a_buffers.edgeVertexIndices.push_back(toOptionalIndex(a_vertex_map.FindIndex(a_first_vertex)));
    a_buffers.edgeVertexIndices.push_back(toOptionalIndex(a_vertex_map.FindIndex(a_last_vertex)));

    GProp_GProps a_linear_props;
    BRepGProp::LinearProperties(an_edge, a_linear_props);
    a_buffers.edgeLengths.push_back(a_linear_props.Mass());

    const uint32_t an_edge_face_offset = toUInt32(a_buffers.edgeFaceIndices.size());
    uint32_t       an_edge_face_count = 0;
    if (an_edge_face_map.Contains(an_edge))
    {
      const NCollection_List<TopoDS_Shape>& a_faces = an_edge_face_map.FindFromKey(an_edge);
      for (NCollection_List<TopoDS_Shape>::Iterator a_face_it(a_faces); a_face_it.More(); a_face_it.Next())
      {
        const int a_face_index = a_face_map.FindIndex(a_face_it.Value());
        if (a_face_index <= 0)
        {
          throw std::runtime_error("Edge-face adjacency referenced a face outside the indexed face map.");
        }
        a_buffers.edgeFaceIndices.push_back(toUInt32(static_cast<std::size_t>(a_face_index - 1)));
        ++an_edge_face_count;
      }
    }
    a_buffers.edgeFaceRanges.push_back(an_edge_face_offset);
    a_buffers.edgeFaceRanges.push_back(an_edge_face_count);
  }

  for (int a_wire_index = 1; a_wire_index <= a_wire_map.Extent(); ++a_wire_index)
  {
    const TopoDS_Wire& a_wire = TopoDS::Wire(a_wire_map.FindKey(a_wire_index));
    const uint32_t     an_edge_offset = toUInt32(a_buffers.wireEdgeIndices.size());
    const uint32_t     a_vertex_offset = toUInt32(a_buffers.wireVertexIndices.size());
    uint32_t           an_edge_count = 0;
    uint32_t           a_vertex_count = 0;
    uint32_t           a_previous_vertex = UINT32_MAX;
    for (BRepTools_WireExplorer an_wire_exp(a_wire); an_wire_exp.More(); an_wire_exp.Next())
    {
      const TopoDS_Edge& an_edge = an_wire_exp.Current();
      const int          an_index = an_edge_map.FindIndex(an_edge);
      if (an_index <= 0)
      {
        throw std::runtime_error("Wire traversal encountered an edge outside the indexed edge map.");
      }

      TopoDS_Vertex a_first_vertex;
      TopoDS_Vertex a_last_vertex;
      TopExp::Vertices(an_edge, a_first_vertex, a_last_vertex);
      uint32_t a_start_vertex = toOptionalIndex(a_vertex_map.FindIndex(a_first_vertex));
      uint32_t an_end_vertex = toOptionalIndex(a_vertex_map.FindIndex(a_last_vertex));
      if (an_edge.Orientation() == TopAbs_REVERSED)
      {
        std::swap(a_start_vertex, an_end_vertex);
      }

      a_buffers.wireEdgeIndices.push_back(toUInt32(static_cast<std::size_t>(an_index - 1)));
      a_buffers.wireEdgeOrientations.push_back(static_cast<uint8_t>(toLeanOcctOrientation(an_edge.Orientation())));
      ++an_edge_count;

      if (a_vertex_count == 0 || a_previous_vertex != a_start_vertex)
      {
        a_buffers.wireVertexIndices.push_back(a_start_vertex);
        ++a_vertex_count;
      }
      a_buffers.wireVertexIndices.push_back(an_end_vertex);
      ++a_vertex_count;
      a_previous_vertex = an_end_vertex;
    }

    a_buffers.wireRanges.push_back(an_edge_offset);
    a_buffers.wireRanges.push_back(an_edge_count);
    a_buffers.wireVertexRanges.push_back(a_vertex_offset);
    a_buffers.wireVertexRanges.push_back(a_vertex_count);
  }

  for (int a_face_index = 1; a_face_index <= a_face_map.Extent(); ++a_face_index)
  {
    const TopoDS_Face& a_face = TopoDS::Face(a_face_map.FindKey(a_face_index));
    const TopoDS_Wire  an_outer_wire = BRepTools::OuterWire(a_face);
    const uint32_t     an_offset = toUInt32(a_buffers.faceWireIndices.size());
    uint32_t           a_count = 0;
    for (TopExp_Explorer a_wire_exp(a_face, TopAbs_WIRE); a_wire_exp.More(); a_wire_exp.Next())
    {
      const TopoDS_Wire& a_wire = TopoDS::Wire(a_wire_exp.Current());
      const int          an_index = a_wire_map.FindIndex(a_wire);
      if (an_index <= 0)
      {
        throw std::runtime_error("Face traversal encountered a wire outside the indexed wire map.");
      }

      a_buffers.faceWireIndices.push_back(toUInt32(static_cast<std::size_t>(an_index - 1)));
      a_buffers.faceWireOrientations.push_back(static_cast<uint8_t>(toLeanOcctOrientation(a_wire.Orientation())));
      a_buffers.faceWireRoles.push_back(static_cast<uint8_t>(toLeanOcctLoopRole(a_wire, an_outer_wire)));
      ++a_count;
    }

    a_buffers.faceRanges.push_back(an_offset);
    a_buffers.faceRanges.push_back(a_count);
  }

  return a_buffers;
}

static MeshBuffers tessellateShape(const TopoDS_Shape& the_shape, const LeanOcctMeshParams& the_params)
{
  if (the_shape.IsNull())
  {
    throw std::runtime_error("Cannot tessellate a null shape.");
  }

  BRepCheck_Analyzer an_analyzer(the_shape);
  if (!an_analyzer.IsValid())
  {
    throw std::runtime_error("Shape validity check failed before meshing.");
  }

  BRepMesh_IncrementalMesh(the_shape,
                           the_params.linear_deflection,
                           the_params.is_relative != 0,
                           the_params.angular_deflection,
                           false);
  BRepLib::EnsureNormalConsistency(the_shape, 0.001, true);

  MeshBuffers a_buffers;

  Bnd_Box a_bounds;
  BRepBndLib::Add(the_shape, a_bounds, false);
  if (a_bounds.IsVoid())
  {
    throw std::runtime_error("Shape bounding box is empty.");
  }

  double a_xmin = 0.0;
  double a_ymin = 0.0;
  double a_zmin = 0.0;
  double a_xmax = 0.0;
  double a_ymax = 0.0;
  double a_zmax = 0.0;
  a_bounds.Get(a_xmin, a_ymin, a_zmin, a_xmax, a_ymax, a_zmax);
  a_buffers.bboxMin = {a_xmin, a_ymin, a_zmin};
  a_buffers.bboxMax = {a_xmax, a_ymax, a_zmax};

  for (TopExp_Explorer a_solid_exp(the_shape, TopAbs_SOLID); a_solid_exp.More(); a_solid_exp.Next())
  {
    ++a_buffers.solidCount;
  }

  for (TopExp_Explorer a_face_exp(the_shape, TopAbs_FACE); a_face_exp.More(); a_face_exp.Next())
  {
    ++a_buffers.faceCount;

    const TopoDS_Face& a_face = TopoDS::Face(a_face_exp.Current());
    TopLoc_Location    a_loc;
    const occ::handle<Poly_Triangulation>& a_triangulation = BRep_Tool::Triangulation(a_face, a_loc);
    if (a_triangulation.IsNull() || a_triangulation->NbTriangles() == 0)
    {
      continue;
    }

    BRepLib_ToolTriangulatedShape::ComputeNormals(a_face, a_triangulation);

    const gp_Trsf& a_trsf = a_loc.Transformation();
    const bool     is_mirrored = a_trsf.VectorialPart().Determinant() < 0.0;
    const bool     is_reversed = (a_face.Orientation() == TopAbs_REVERSED);

    for (int a_triangle_index = 1; a_triangle_index <= a_triangulation->NbTriangles(); ++a_triangle_index)
    {
      int a_node_index1 = 0;
      int a_node_index2 = 0;
      int a_node_index3 = 0;
      if (is_reversed)
      {
        a_triangulation->Triangle(a_triangle_index).Get(a_node_index1, a_node_index3, a_node_index2);
      }
      else
      {
        a_triangulation->Triangle(a_triangle_index).Get(a_node_index1, a_node_index2, a_node_index3);
      }

      const int an_indices[] = {a_node_index1, a_node_index2, a_node_index3};
      for (const int a_node_index : an_indices)
      {
        gp_Pnt a_point = a_triangulation->Node(a_node_index);
        gp_Dir a_normal = a_triangulation->Normal(a_node_index);
        if (is_reversed ^ is_mirrored)
        {
          a_normal.Reverse();
        }
        if (!a_loc.IsIdentity())
        {
          a_point.Transform(a_trsf);
          a_normal.Transform(a_trsf);
        }

        appendPoint(a_buffers.positions, a_point);
        appendDir(a_buffers.normals, a_normal);
        a_buffers.triangleIndices.push_back(static_cast<uint32_t>(a_buffers.triangleIndices.size()));
      }
    }
  }

  EdgeFaceMap an_edge_face_map;
  TopExp::MapShapesAndUniqueAncestors(the_shape, TopAbs_EDGE, TopAbs_FACE, an_edge_face_map);
  for (EdgeFaceMap::Iterator an_edge_it(an_edge_face_map); an_edge_it.More(); an_edge_it.Next())
  {
    const TopoDS_Edge& an_edge = TopoDS::Edge(an_edge_it.Key());
    if (BRep_Tool::Degenerated(an_edge))
    {
      continue;
    }

    if (an_edge_it.Value().Extent() == 1)
    {
      const TopoDS_Face& a_only_face = TopoDS::Face(an_edge_it.Value().First());
      if (BRep_Tool::IsClosed(an_edge, a_only_face))
      {
        continue;
      }
    }

    occ::handle<Poly_PolygonOnTriangulation> a_poly_on_triangulation;
    occ::handle<Poly_Triangulation>          a_triangulation;
    TopLoc_Location                          a_poly_loc;
    BRep_Tool::PolygonOnTriangulation(an_edge, a_poly_on_triangulation, a_triangulation, a_poly_loc);
    if (!a_poly_on_triangulation.IsNull() && !a_triangulation.IsNull()
        && a_poly_on_triangulation->NbNodes() >= 2)
    {
      const gp_Trsf& a_trsf = a_poly_loc.Transformation();
      gp_Pnt         a_previous = a_triangulation->Node(a_poly_on_triangulation->Node(1));
      if (!a_poly_loc.IsIdentity())
      {
        a_previous.Transform(a_trsf);
      }

      for (int a_node_index = 2; a_node_index <= a_poly_on_triangulation->NbNodes(); ++a_node_index)
      {
        gp_Pnt a_current = a_triangulation->Node(a_poly_on_triangulation->Node(a_node_index));
        if (!a_poly_loc.IsIdentity())
        {
          a_current.Transform(a_trsf);
        }

        appendPoint(a_buffers.edgePositions, a_previous);
        appendPoint(a_buffers.edgePositions, a_current);
        a_previous = a_current;
      }
      continue;
    }

    TopLoc_Location                    a_polygon_loc;
    const occ::handle<Poly_Polygon3D> a_polygon = BRep_Tool::Polygon3D(an_edge, a_polygon_loc);
    if (a_polygon.IsNull() || a_polygon->NbNodes() < 2)
    {
      continue;
    }

    const gp_Trsf& a_trsf = a_polygon_loc.Transformation();
    gp_Pnt         a_previous = a_polygon->Nodes().Value(1);
    if (!a_polygon_loc.IsIdentity())
    {
      a_previous.Transform(a_trsf);
    }

    for (int a_node_index = 2; a_node_index <= a_polygon->NbNodes(); ++a_node_index)
    {
      gp_Pnt a_current = a_polygon->Nodes().Value(a_node_index);
      if (!a_polygon_loc.IsIdentity())
      {
        a_current.Transform(a_trsf);
      }

      appendPoint(a_buffers.edgePositions, a_previous);
      appendPoint(a_buffers.edgePositions, a_current);
      a_previous = a_current;
    }
  }

  if (a_buffers.faceCount == 0)
  {
    throw std::runtime_error("Meshed shape did not contain any faces.");
  }
  if (a_buffers.triangleIndices.empty())
  {
    throw std::runtime_error("Meshed shape did not produce any triangles.");
  }

  return a_buffers;
}
} // namespace

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctContext* lean_occt_context_create(void)
{
  try
  {
    return new LeanOcctContext();
  }
  catch (...)
  {
    return nullptr;
  }
}

extern "C" LEAN_OCCT_CAPI_EXPORT void lean_occt_context_destroy(LeanOcctContext* the_context)
{
  delete the_context;
}

extern "C" LEAN_OCCT_CAPI_EXPORT const char* lean_occt_context_last_error(const LeanOcctContext* the_context)
{
  if (the_context == nullptr)
  {
    return nullContextError();
  }
  return the_context->LastError.c_str();
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctShape* lean_occt_shape_make_box(
  LeanOcctContext*          the_context,
  const LeanOcctBoxParams* the_params)
{
  return createShapeFromParams(the_context,
                               the_params,
                               "LeanOcctBoxParams was null.",
                               [&](const LeanOcctBoxParams& the_box_params) -> TopoDS_Shape {
    const gp_Pnt a_corner(the_box_params.x, the_box_params.y, the_box_params.z);
    return BRepPrimAPI_MakeBox(a_corner, the_box_params.dx, the_box_params.dy, the_box_params.dz).Shape();
  });
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctShape* lean_occt_shape_make_cylinder(
  LeanOcctContext*               the_context,
  const LeanOcctCylinderParams*  the_params)
{
  return createShapeFromParams(the_context,
                               the_params,
                               "LeanOcctCylinderParams was null.",
                               [&](const LeanOcctCylinderParams& the_cylinder_params) -> TopoDS_Shape {
    const gp_Ax2 an_axis(gp_Pnt(the_cylinder_params.x, the_cylinder_params.y, the_cylinder_params.z),
                         gp_Dir(the_cylinder_params.axis_x,
                                the_cylinder_params.axis_y,
                                the_cylinder_params.axis_z));
    return BRepPrimAPI_MakeCylinder(an_axis, the_cylinder_params.radius, the_cylinder_params.height).Shape();
  });
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctShape* lean_occt_shape_make_cone(
  LeanOcctContext*           the_context,
  const LeanOcctConeParams*  the_params)
{
  return createShapeFromParams(the_context,
                               the_params,
                               "LeanOcctConeParams was null.",
                               [&](const LeanOcctConeParams& the_cone_params) -> TopoDS_Shape {
    const gp_Ax2 an_axis = makeAxis2(the_cone_params.x,
                                     the_cone_params.y,
                                     the_cone_params.z,
                                     the_cone_params.axis_x,
                                     the_cone_params.axis_y,
                                     the_cone_params.axis_z,
                                     the_cone_params.x_dir_x,
                                     the_cone_params.x_dir_y,
                                     the_cone_params.x_dir_z,
                                     "Cone axis direction was zero-length.",
                                     "Cone X direction was zero-length.");
    return BRepPrimAPI_MakeCone(
             an_axis, the_cone_params.base_radius, the_cone_params.top_radius, the_cone_params.height)
      .Shape();
  });
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctShape* lean_occt_shape_make_sphere(
  LeanOcctContext*             the_context,
  const LeanOcctSphereParams*  the_params)
{
  return createShapeFromParams(the_context,
                               the_params,
                               "LeanOcctSphereParams was null.",
                               [&](const LeanOcctSphereParams& the_sphere_params) -> TopoDS_Shape {
    const gp_Ax2 an_axis = makeAxis2(the_sphere_params.x,
                                     the_sphere_params.y,
                                     the_sphere_params.z,
                                     the_sphere_params.axis_x,
                                     the_sphere_params.axis_y,
                                     the_sphere_params.axis_z,
                                     the_sphere_params.x_dir_x,
                                     the_sphere_params.x_dir_y,
                                     the_sphere_params.x_dir_z,
                                     "Sphere axis direction was zero-length.",
                                     "Sphere X direction was zero-length.");
    return BRepPrimAPI_MakeSphere(an_axis, the_sphere_params.radius).Shape();
  });
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctShape* lean_occt_shape_make_torus(
  LeanOcctContext*            the_context,
  const LeanOcctTorusParams*  the_params)
{
  return createShapeFromParams(the_context,
                               the_params,
                               "LeanOcctTorusParams was null.",
                               [&](const LeanOcctTorusParams& the_torus_params) -> TopoDS_Shape {
    const gp_Ax2 an_axis = makeAxis2(the_torus_params.x,
                                     the_torus_params.y,
                                     the_torus_params.z,
                                     the_torus_params.axis_x,
                                     the_torus_params.axis_y,
                                     the_torus_params.axis_z,
                                     the_torus_params.x_dir_x,
                                     the_torus_params.x_dir_y,
                                     the_torus_params.x_dir_z,
                                     "Torus axis direction was zero-length.",
                                     "Torus X direction was zero-length.");
    return BRepPrimAPI_MakeTorus(an_axis, the_torus_params.major_radius, the_torus_params.minor_radius)
      .Shape();
  });
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctShape* lean_occt_shape_make_ellipse_edge(
  LeanOcctContext*                  the_context,
  const LeanOcctEllipseEdgeParams*  the_params)
{
  return createShapeFromParams(the_context,
                               the_params,
                               "LeanOcctEllipseEdgeParams was null.",
                               [&](const LeanOcctEllipseEdgeParams& the_ellipse_params) -> TopoDS_Shape {
    const gp_Ax2 an_axis = makeAxis2(the_ellipse_params.x,
                                     the_ellipse_params.y,
                                     the_ellipse_params.z,
                                     the_ellipse_params.axis_x,
                                     the_ellipse_params.axis_y,
                                     the_ellipse_params.axis_z,
                                     the_ellipse_params.x_dir_x,
                                     the_ellipse_params.x_dir_y,
                                     the_ellipse_params.x_dir_z,
                                     "Ellipse axis direction was zero-length.",
                                     "Ellipse X direction was zero-length.");
    BRepBuilderAPI_MakeEdge an_edge_builder(
      gp_Elips(an_axis, the_ellipse_params.major_radius, the_ellipse_params.minor_radius));
    if (!an_edge_builder.IsDone())
    {
      throw std::runtime_error("Ellipse edge creation failed.");
    }
    return an_edge_builder.Edge();
  });
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctShape* lean_occt_shape_make_fillet(
  LeanOcctContext*               the_context,
  const LeanOcctShape*           the_shape,
  const LeanOcctFilletParams*    the_params)
{
  return createShapeFromInputShape(the_context,
                                   the_shape,
                                   the_params,
                                   "LeanOcctFilletParams was null.",
                                   [&](const TopoDS_Shape& a_shape,
                                       const LeanOcctFilletParams& the_fillet_params) -> TopoDS_Shape {
    BRepFilletAPI_MakeFillet a_fillet(a_shape);
    a_fillet.Add(the_fillet_params.radius, requireIndexedEdge(a_shape, the_fillet_params.edge_index));
    const TopoDS_Shape a_result = a_fillet.Shape();
    if (!a_fillet.IsDone() || a_result.IsNull())
    {
      throw std::runtime_error("Fillet operation failed.");
    }
    return a_result;
  });
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctShape* lean_occt_shape_make_offset(
  LeanOcctContext*               the_context,
  const LeanOcctShape*           the_shape,
  const LeanOcctOffsetParams*    the_params)
{
  return createShapeFromInputShape(the_context,
                                   the_shape,
                                   the_params,
                                   "LeanOcctOffsetParams was null.",
                                   [&](const TopoDS_Shape& a_shape,
                                       const LeanOcctOffsetParams& the_offset_params) -> TopoDS_Shape {
    BRepOffsetAPI_MakeOffsetShape an_offset;
    an_offset.PerformByJoin(a_shape, the_offset_params.offset, the_offset_params.tolerance);
    const TopoDS_Shape a_result = an_offset.Shape();
    if (a_result.IsNull())
    {
      throw std::runtime_error("Offset operation produced a null shape.");
    }
    return a_result;
  });
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctShape* lean_occt_shape_make_cylindrical_hole(
  LeanOcctContext*                       the_context,
  const LeanOcctShape*                   the_shape,
  const LeanOcctCylindricalHoleParams*   the_params)
{
  return createShapeFromInputShape(the_context,
                                   the_shape,
                                   the_params,
                                   "LeanOcctCylindricalHoleParams was null.",
                                   [&](const TopoDS_Shape& a_shape,
                                       const LeanOcctCylindricalHoleParams& the_hole_params) -> TopoDS_Shape {
    const gp_Ax1 an_axis(gp_Pnt(the_hole_params.x, the_hole_params.y, the_hole_params.z),
                         gp_Dir(the_hole_params.axis_x, the_hole_params.axis_y, the_hole_params.axis_z));

    BRepFeat_MakeCylindricalHole a_feature;
    a_feature.Init(a_shape, an_axis);
    a_feature.Perform(the_hole_params.radius);
    a_feature.Build();
    if (a_feature.Status() != BRepFeat_NoError)
    {
      throw std::runtime_error("Cylindrical hole feature operation failed.");
    }

    const TopoDS_Shape a_result = a_feature.Shape();
    if (a_result.IsNull())
    {
      throw std::runtime_error("Cylindrical hole feature produced a null shape.");
    }

    return a_result;
  });
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctShape* lean_occt_shape_make_helix(
  LeanOcctContext*               the_context,
  const LeanOcctHelixParams*     the_params)
{
  return createShapeFromParams(the_context,
                               the_params,
                               "LeanOcctHelixParams was null.",
                               [&](const LeanOcctHelixParams& the_helix_params) -> TopoDS_Shape {
    HelixBRep_BuilderHelix a_builder;

    NCollection_Array1<double> a_heights(1, 1);
    a_heights(1) = the_helix_params.height;

    NCollection_Array1<double> a_pitches(1, 1);
    a_pitches(1) = the_helix_params.pitch;

    NCollection_Array1<bool> a_is_pitches(1, 1);
    a_is_pitches(1) = true;

    const gp_Ax3 an_axis(gp_Pnt(the_helix_params.origin_x, the_helix_params.origin_y, the_helix_params.origin_z),
                         gp_Dir(the_helix_params.axis_x, the_helix_params.axis_y, the_helix_params.axis_z),
                         gp_Dir(the_helix_params.x_dir_x, the_helix_params.x_dir_y, the_helix_params.x_dir_z));
    a_builder.SetParameters(an_axis, the_helix_params.radius, a_heights, a_pitches, a_is_pitches);
    a_builder.SetApproxParameters(1.0e-4, 8, GeomAbs_C1);
    a_builder.Perform();

    if (a_builder.ErrorStatus() != 0)
    {
      throw std::runtime_error("Helix builder failed.");
    }

    const TopoDS_Shape a_result = a_builder.Shape();
    if (a_result.IsNull() || a_result.ShapeType() != TopAbs_WIRE)
    {
      throw std::runtime_error("Helix builder did not produce a wire.");
    }

    return a_result;
  });
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctShape* lean_occt_shape_make_prism(
  LeanOcctContext*            the_context,
  const LeanOcctShape*        the_shape,
  const LeanOcctPrismParams*  the_params)
{
  return createShapeFromInputShape(the_context,
                                   the_shape,
                                   the_params,
                                   "LeanOcctPrismParams was null.",
                                   [&](const TopoDS_Shape& a_shape,
                                       const LeanOcctPrismParams& the_prism_params) -> TopoDS_Shape {
    const gp_Vec a_vector(the_prism_params.dx, the_prism_params.dy, the_prism_params.dz);
    if (a_vector.SquareMagnitude() <= Precision::Confusion() * Precision::Confusion())
    {
      throw std::invalid_argument("Prism direction was zero-length.");
    }

    const TopoDS_Shape a_result = BRepPrimAPI_MakePrism(a_shape, a_vector).Shape();
    if (a_result.IsNull())
    {
      throw std::runtime_error("Prism operation produced a null shape.");
    }
    return a_result;
  });
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctShape* lean_occt_shape_make_revolution(
  LeanOcctContext*                 the_context,
  const LeanOcctShape*             the_shape,
  const LeanOcctRevolutionParams*  the_params)
{
  return createShapeFromInputShape(the_context,
                                   the_shape,
                                   the_params,
                                   "LeanOcctRevolutionParams was null.",
                                   [&](const TopoDS_Shape& a_shape,
                                       const LeanOcctRevolutionParams& the_revolution_params) -> TopoDS_Shape {
    const gp_Ax1 an_axis = makeAxis1(the_revolution_params.x,
                                     the_revolution_params.y,
                                     the_revolution_params.z,
                                     the_revolution_params.axis_x,
                                     the_revolution_params.axis_y,
                                     the_revolution_params.axis_z,
                                     "Revolution axis direction was zero-length.");
    const TopoDS_Shape a_result =
      BRepPrimAPI_MakeRevol(a_shape, an_axis, the_revolution_params.angle_radians).Shape();
    if (a_result.IsNull())
    {
      throw std::runtime_error("Revolution operation produced a null shape.");
    }
    return a_result;
  });
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctShape* lean_occt_shape_boolean_cut(
  LeanOcctContext*     the_context,
  const LeanOcctShape* the_lhs,
  const LeanOcctShape* the_rhs)
{
  return guardCall(the_context, [&]() -> LeanOcctShape* {
    if (the_context == nullptr)
    {
      throw std::invalid_argument(nullContextError());
    }
    return makeBooleanResult(requireShape(the_lhs), requireShape(the_rhs), 0);
  });
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctShape* lean_occt_shape_boolean_fuse(
  LeanOcctContext*     the_context,
  const LeanOcctShape* the_lhs,
  const LeanOcctShape* the_rhs)
{
  return guardCall(the_context, [&]() -> LeanOcctShape* {
    if (the_context == nullptr)
    {
      throw std::invalid_argument(nullContextError());
    }
    return makeBooleanResult(requireShape(the_lhs), requireShape(the_rhs), 1);
  });
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctShape* lean_occt_shape_boolean_common(
  LeanOcctContext*     the_context,
  const LeanOcctShape* the_lhs,
  const LeanOcctShape* the_rhs)
{
  return guardCall(the_context, [&]() -> LeanOcctShape* {
    if (the_context == nullptr)
    {
      throw std::invalid_argument(nullContextError());
    }
    return makeBooleanResult(requireShape(the_lhs), requireShape(the_rhs), 2);
  });
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctShape* lean_occt_shape_read_step(
  LeanOcctContext* the_context,
  const char*      the_path_utf8)
{
  return guardShapeCall(the_context, [&]() -> TopoDS_Shape {
    const char* a_path_utf8 = requireNonEmptyUtf8Path(the_path_utf8);
    ScopedMessengerSilencer a_quiet_messages;
    STEPControl_Reader a_reader;
    if (a_reader.ReadFile(a_path_utf8) != IFSelect_RetDone)
    {
      throw std::runtime_error("STEP read failed.");
    }
    if (a_reader.TransferRoots() <= 0)
    {
      throw std::runtime_error("STEP transfer roots failed.");
    }
    return a_reader.OneShape();
  });
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_write_step(
  LeanOcctContext*     the_context,
  const LeanOcctShape* the_shape,
  const char*          the_path_utf8)
{
  return guardResultCall(the_context, [&]() {
    const char* a_path_utf8 = requireNonEmptyUtf8Path(the_path_utf8);
    ScopedMessengerSilencer a_quiet_messages;
    STEPControl_Writer a_writer;
    if (a_writer.Transfer(requireShape(the_shape), STEPControl_AsIs) != IFSelect_RetDone)
    {
      throw std::runtime_error("STEP transfer failed.");
    }
    if (a_writer.Write(a_path_utf8) != IFSelect_RetDone)
    {
      throw std::runtime_error("STEP write failed.");
    }
  });
}

extern "C" LEAN_OCCT_CAPI_EXPORT void lean_occt_shape_destroy(LeanOcctShape* the_shape)
{
  delete the_shape;
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_orientation(
  LeanOcctContext*       the_context,
  const LeanOcctShape*   the_shape,
  LeanOcctOrientation*   the_orientation)
{
  return fillShapeOutput(the_context,
                         the_shape,
                         the_orientation,
                         "LeanOcctOrientation output pointer was null.",
                         requireShape,
                         [](const TopoDS_Shape& the_shape_value, LeanOcctOrientation& the_result) {
                           the_result = toLeanOcctOrientation(the_shape_value.Orientation());
                         });
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_vertex_point(
  LeanOcctContext*     the_context,
  const LeanOcctShape* the_shape,
  double*              the_xyz3)
{
  return guardCall(the_context, [&]() -> LeanOcctResult {
    if (the_context == nullptr)
    {
      throw std::invalid_argument(nullContextError());
    }
    if (the_xyz3 == nullptr)
    {
      throw std::invalid_argument("Vertex point output pointer was null.");
    }

    writePoint(the_xyz3, BRep_Tool::Pnt(requireVertexShape(the_shape)));
    return LEAN_OCCT_RESULT_OK;
  });
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_edge_endpoints(
  LeanOcctContext*     the_context,
  const LeanOcctShape* the_shape,
  double*              the_start_xyz3,
  double*              the_end_xyz3)
{
  return guardCall(the_context, [&]() -> LeanOcctResult {
    if (the_context == nullptr)
    {
      throw std::invalid_argument(nullContextError());
    }
    if (the_start_xyz3 == nullptr || the_end_xyz3 == nullptr)
    {
      throw std::invalid_argument("Edge endpoint output pointer was null.");
    }

    const TopoDS_Edge an_edge = requireEdgeShape(the_shape);
    TopoDS_Vertex     a_first_vertex;
    TopoDS_Vertex     a_last_vertex;
    TopExp::Vertices(an_edge, a_first_vertex, a_last_vertex);
    if (a_first_vertex.IsNull() || a_last_vertex.IsNull())
    {
      throw std::runtime_error("Edge did not contain two endpoint vertices.");
    }

    writePoint(the_start_xyz3, BRep_Tool::Pnt(a_first_vertex));
    writePoint(the_end_xyz3, BRep_Tool::Pnt(a_last_vertex));
    return LEAN_OCCT_RESULT_OK;
  });
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_edge_sample(
  LeanOcctContext*     the_context,
  const LeanOcctShape* the_shape,
  double               the_t,
  LeanOcctEdgeSample*  the_sample)
{
  return guardCall(the_context, [&]() -> LeanOcctResult {
    if (the_context == nullptr)
    {
      throw std::invalid_argument(nullContextError());
    }
    if (the_sample == nullptr)
    {
      throw std::invalid_argument("LeanOcctEdgeSample output pointer was null.");
    }
    if (!(the_t >= 0.0 && the_t <= 1.0))
    {
      throw std::invalid_argument("Edge sample parameter must be within [0, 1].");
    }

    LeanOcctEdgeGeometry a_geometry = {};
    const TopoDS_Edge    an_edge = requireEdgeShape(the_shape);
    fillEdgeGeometry(an_edge, a_geometry);
    sampleEdgeAtParameter(an_edge,
                          interpolateRange(a_geometry.start_parameter, a_geometry.end_parameter, the_t),
                          *the_sample);
    return LEAN_OCCT_RESULT_OK;
  });
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_edge_sample_at_parameter(
  LeanOcctContext*     the_context,
  const LeanOcctShape* the_shape,
  double               the_parameter,
  LeanOcctEdgeSample*  the_sample)
{
  return guardCall(the_context, [&]() -> LeanOcctResult {
    if (the_context == nullptr)
    {
      throw std::invalid_argument(nullContextError());
    }
    if (the_sample == nullptr)
    {
      throw std::invalid_argument("LeanOcctEdgeSample output pointer was null.");
    }

    LeanOcctEdgeGeometry a_geometry = {};
    const TopoDS_Edge    an_edge = requireEdgeShape(the_shape);
    fillEdgeGeometry(an_edge, a_geometry);
    if (!isParameterInsideRange(the_parameter, a_geometry.start_parameter, a_geometry.end_parameter))
    {
      throw std::out_of_range("Requested edge parameter was outside the trimmed edge domain.");
    }

    sampleEdgeAtParameter(an_edge, the_parameter, *the_sample);
    return LEAN_OCCT_RESULT_OK;
  });
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_edge_geometry(
  LeanOcctContext*       the_context,
  const LeanOcctShape*   the_shape,
  LeanOcctEdgeGeometry*  the_geometry)
{
  return fillShapeOutput(the_context,
                         the_shape,
                         the_geometry,
                         "LeanOcctEdgeGeometry output pointer was null.",
                         requireEdgeShape,
                         fillEdgeGeometry);
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_edge_line_payload(
  LeanOcctContext*      the_context,
  const LeanOcctShape*  the_shape,
  LeanOcctLinePayload*  the_payload)
{
  return fillShapeOutput(the_context,
                         the_shape,
                         the_payload,
                         "LeanOcctLinePayload output pointer was null.",
                         requireEdgeShape,
                         static_cast<void (*)(const TopoDS_Edge&, LeanOcctLinePayload&)>(
                           fillLinePayload));
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_edge_circle_payload(
  LeanOcctContext*        the_context,
  const LeanOcctShape*    the_shape,
  LeanOcctCirclePayload*  the_payload)
{
  return fillShapeOutput(the_context,
                         the_shape,
                         the_payload,
                         "LeanOcctCirclePayload output pointer was null.",
                         requireEdgeShape,
                         static_cast<void (*)(const TopoDS_Edge&, LeanOcctCirclePayload&)>(
                           fillCirclePayload));
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_edge_ellipse_payload(
  LeanOcctContext*         the_context,
  const LeanOcctShape*     the_shape,
  LeanOcctEllipsePayload*  the_payload)
{
  return fillShapeOutput(the_context,
                         the_shape,
                         the_payload,
                         "LeanOcctEllipsePayload output pointer was null.",
                         requireEdgeShape,
                         static_cast<void (*)(const TopoDS_Edge&, LeanOcctEllipsePayload&)>(
                           fillEllipsePayload));
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_uv_bounds(
  LeanOcctContext*      the_context,
  const LeanOcctShape*  the_shape,
  LeanOcctFaceUvBounds* the_bounds)
{
  return guardCall(the_context, [&]() -> LeanOcctResult {
    if (the_context == nullptr)
    {
      throw std::invalid_argument(nullContextError());
    }
    if (the_bounds == nullptr)
    {
      throw std::invalid_argument("LeanOcctFaceUvBounds output pointer was null.");
    }

    LeanOcctFaceGeometry a_geometry = {};
    fillFaceGeometry(requireFaceShape(the_shape), a_geometry);
    the_bounds->u_min = a_geometry.u_min;
    the_bounds->u_max = a_geometry.u_max;
    the_bounds->v_min = a_geometry.v_min;
    the_bounds->v_max = a_geometry.v_max;
    return LEAN_OCCT_RESULT_OK;
  });
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_sample(
  LeanOcctContext*      the_context,
  const LeanOcctShape*  the_shape,
  double                the_u,
  double                the_v,
  LeanOcctFaceSample*   the_sample)
{
  return guardCall(the_context, [&]() -> LeanOcctResult {
    if (the_context == nullptr)
    {
      throw std::invalid_argument(nullContextError());
    }
    if (the_sample == nullptr)
    {
      throw std::invalid_argument("LeanOcctFaceSample output pointer was null.");
    }

    LeanOcctFaceGeometry a_geometry = {};
    const TopoDS_Face    a_face = requireFaceShape(the_shape);
    fillFaceGeometry(a_face, a_geometry);
    if (!isParameterInsideRange(the_u, a_geometry.u_min, a_geometry.u_max)
        || !isParameterInsideRange(the_v, a_geometry.v_min, a_geometry.v_max))
    {
      throw std::out_of_range("Requested UV sample was outside the face bounds.");
    }

    sampleFaceAtUv(a_face, the_u, the_v, *the_sample);
    return LEAN_OCCT_RESULT_OK;
  });
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_sample_normalized(
  LeanOcctContext*      the_context,
  const LeanOcctShape*  the_shape,
  double                the_u_t,
  double                the_v_t,
  LeanOcctFaceSample*   the_sample)
{
  return guardCall(the_context, [&]() -> LeanOcctResult {
    if (the_context == nullptr)
    {
      throw std::invalid_argument(nullContextError());
    }
    if (the_sample == nullptr)
    {
      throw std::invalid_argument("LeanOcctFaceSample output pointer was null.");
    }
    if (!(the_u_t >= 0.0 && the_u_t <= 1.0) || !(the_v_t >= 0.0 && the_v_t <= 1.0))
    {
      throw std::invalid_argument("Normalized face sample parameters must be within [0, 1].");
    }

    LeanOcctFaceGeometry a_geometry = {};
    const TopoDS_Face    a_face = requireFaceShape(the_shape);
    fillFaceGeometry(a_face, a_geometry);
    sampleFaceAtUv(a_face,
                   interpolateRange(a_geometry.u_min, a_geometry.u_max, the_u_t),
                   interpolateRange(a_geometry.v_min, a_geometry.v_max, the_v_t),
                   *the_sample);
    return LEAN_OCCT_RESULT_OK;
  });
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_geometry(
  LeanOcctContext*       the_context,
  const LeanOcctShape*   the_shape,
  LeanOcctFaceGeometry*  the_geometry)
{
  return fillShapeOutput(the_context,
                         the_shape,
                         the_geometry,
                         "LeanOcctFaceGeometry output pointer was null.",
                         requireFaceShape,
                         static_cast<void (*)(const TopoDS_Face&, LeanOcctFaceGeometry&)>(
                           fillFaceGeometry));
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_plane_payload(
  LeanOcctContext*       the_context,
  const LeanOcctShape*   the_shape,
  LeanOcctPlanePayload*  the_payload)
{
  return fillShapeOutput(the_context,
                         the_shape,
                         the_payload,
                         "LeanOcctPlanePayload output pointer was null.",
                         requireFaceShape,
                         static_cast<void (*)(const TopoDS_Face&, LeanOcctPlanePayload&)>(
                           fillPlanePayload));
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_cylinder_payload(
  LeanOcctContext*          the_context,
  const LeanOcctShape*      the_shape,
  LeanOcctCylinderPayload*  the_payload)
{
  return fillShapeOutput(the_context,
                         the_shape,
                         the_payload,
                         "LeanOcctCylinderPayload output pointer was null.",
                         requireFaceShape,
                         static_cast<void (*)(const TopoDS_Face&, LeanOcctCylinderPayload&)>(
                           fillCylinderPayload));
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_cone_payload(
  LeanOcctContext*      the_context,
  const LeanOcctShape*  the_shape,
  LeanOcctConePayload*  the_payload)
{
  return fillShapeOutput(the_context,
                         the_shape,
                         the_payload,
                         "LeanOcctConePayload output pointer was null.",
                         requireFaceShape,
                         static_cast<void (*)(const TopoDS_Face&, LeanOcctConePayload&)>(
                           fillConePayload));
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_sphere_payload(
  LeanOcctContext*        the_context,
  const LeanOcctShape*    the_shape,
  LeanOcctSpherePayload*  the_payload)
{
  return fillShapeOutput(the_context,
                         the_shape,
                         the_payload,
                         "LeanOcctSpherePayload output pointer was null.",
                         requireFaceShape,
                         static_cast<void (*)(const TopoDS_Face&, LeanOcctSpherePayload&)>(
                           fillSpherePayload));
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_torus_payload(
  LeanOcctContext*       the_context,
  const LeanOcctShape*   the_shape,
  LeanOcctTorusPayload*  the_payload)
{
  return fillShapeOutput(the_context,
                         the_shape,
                         the_payload,
                         "LeanOcctTorusPayload output pointer was null.",
                         requireFaceShape,
                         static_cast<void (*)(const TopoDS_Face&, LeanOcctTorusPayload&)>(
                           fillTorusPayload));
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_revolution_payload(
  LeanOcctContext*                 the_context,
  const LeanOcctShape*             the_shape,
  LeanOcctRevolutionSurfacePayload* the_payload)
{
  return fillShapeOutput(the_context,
                         the_shape,
                         the_payload,
                         "LeanOcctRevolutionSurfacePayload output pointer was null.",
                         requireFaceShape,
                         static_cast<void (*)(const TopoDS_Face&, LeanOcctRevolutionSurfacePayload&)>(
                           fillRevolutionSurfacePayload));
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_extrusion_payload(
  LeanOcctContext*                the_context,
  const LeanOcctShape*            the_shape,
  LeanOcctExtrusionSurfacePayload* the_payload)
{
  return fillShapeOutput(the_context,
                         the_shape,
                         the_payload,
                         "LeanOcctExtrusionSurfacePayload output pointer was null.",
                         requireFaceShape,
                         static_cast<void (*)(const TopoDS_Face&, LeanOcctExtrusionSurfacePayload&)>(
                           fillExtrusionSurfacePayload));
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_offset_payload(
  LeanOcctContext*             the_context,
  const LeanOcctShape*         the_shape,
  LeanOcctOffsetSurfacePayload* the_payload)
{
  return fillShapeOutput(the_context,
                         the_shape,
                         the_payload,
                         "LeanOcctOffsetSurfacePayload output pointer was null.",
                         requireFaceShape,
                         fillOffsetSurfacePayload);
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_offset_basis_geometry(
  LeanOcctContext*       the_context,
  const LeanOcctShape*   the_shape,
  LeanOcctFaceGeometry*  the_geometry)
{
  return writeOutput(the_context,
                     the_geometry,
                     "LeanOcctFaceGeometry output pointer was null.",
                     [&](LeanOcctFaceGeometry& the_result) {
                       fillFaceGeometry(*offsetBasisSurface(requireFaceShape(the_shape)), the_result);
                     });
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_offset_basis_plane_payload(
  LeanOcctContext*       the_context,
  const LeanOcctShape*   the_shape,
  LeanOcctPlanePayload*  the_payload)
{
  return writeOutput(the_context,
                     the_payload,
                     "LeanOcctPlanePayload output pointer was null.",
                     [&](LeanOcctPlanePayload& the_result) {
                       fillPlanePayload(*offsetBasisSurface(requireFaceShape(the_shape)), the_result);
                     });
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_offset_basis_cylinder_payload(
  LeanOcctContext*          the_context,
  const LeanOcctShape*      the_shape,
  LeanOcctCylinderPayload*  the_payload)
{
  return writeOutput(the_context,
                     the_payload,
                     "LeanOcctCylinderPayload output pointer was null.",
                     [&](LeanOcctCylinderPayload& the_result) {
                       fillCylinderPayload(*offsetBasisSurface(requireFaceShape(the_shape)), the_result);
                     });
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_offset_basis_cone_payload(
  LeanOcctContext*      the_context,
  const LeanOcctShape*  the_shape,
  LeanOcctConePayload*  the_payload)
{
  return writeOutput(the_context,
                     the_payload,
                     "LeanOcctConePayload output pointer was null.",
                     [&](LeanOcctConePayload& the_result) {
                       fillConePayload(*offsetBasisSurface(requireFaceShape(the_shape)), the_result);
                     });
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_offset_basis_sphere_payload(
  LeanOcctContext*        the_context,
  const LeanOcctShape*    the_shape,
  LeanOcctSpherePayload*  the_payload)
{
  return writeOutput(the_context,
                     the_payload,
                     "LeanOcctSpherePayload output pointer was null.",
                     [&](LeanOcctSpherePayload& the_result) {
                       fillSpherePayload(*offsetBasisSurface(requireFaceShape(the_shape)), the_result);
                     });
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_offset_basis_torus_payload(
  LeanOcctContext*       the_context,
  const LeanOcctShape*   the_shape,
  LeanOcctTorusPayload*  the_payload)
{
  return writeOutput(the_context,
                     the_payload,
                     "LeanOcctTorusPayload output pointer was null.",
                     [&](LeanOcctTorusPayload& the_result) {
                       fillTorusPayload(*offsetBasisSurface(requireFaceShape(the_shape)), the_result);
                     });
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_offset_basis_revolution_payload(
  LeanOcctContext*                  the_context,
  const LeanOcctShape*              the_shape,
  LeanOcctRevolutionSurfacePayload* the_payload)
{
  return writeOutput(the_context,
                     the_payload,
                     "LeanOcctRevolutionSurfacePayload output pointer was null.",
                     [&](LeanOcctRevolutionSurfacePayload& the_result) {
                       fillRevolutionSurfacePayload(*offsetBasisSurface(requireFaceShape(the_shape)),
                                                    the_result);
                     });
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_offset_basis_extrusion_payload(
  LeanOcctContext*                 the_context,
  const LeanOcctShape*             the_shape,
  LeanOcctExtrusionSurfacePayload* the_payload)
{
  return writeOutput(the_context,
                     the_payload,
                     "LeanOcctExtrusionSurfacePayload output pointer was null.",
                     [&](LeanOcctExtrusionSurfacePayload& the_result) {
                       fillExtrusionSurfacePayload(*offsetBasisSurface(requireFaceShape(the_shape)),
                                                  the_result);
                     });
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_offset_basis_curve_geometry(
  LeanOcctContext*       the_context,
  const LeanOcctShape*   the_shape,
  LeanOcctEdgeGeometry*  the_geometry)
{
  return writeOutput(the_context,
                     the_geometry,
                     "LeanOcctEdgeGeometry output pointer was null.",
                     [&](LeanOcctEdgeGeometry& the_result) {
                       fillCurveGeometry(*offsetBasisCurve(requireFaceShape(the_shape)), the_result);
                     });
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_offset_basis_curve_line_payload(
  LeanOcctContext*      the_context,
  const LeanOcctShape*  the_shape,
  LeanOcctLinePayload*  the_payload)
{
  return writeOutput(the_context,
                     the_payload,
                     "LeanOcctLinePayload output pointer was null.",
                     [&](LeanOcctLinePayload& the_result) {
                       fillLinePayload(*offsetBasisCurve(requireFaceShape(the_shape)), the_result);
                     });
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_offset_basis_curve_circle_payload(
  LeanOcctContext*        the_context,
  const LeanOcctShape*    the_shape,
  LeanOcctCirclePayload*  the_payload)
{
  return writeOutput(the_context,
                     the_payload,
                     "LeanOcctCirclePayload output pointer was null.",
                     [&](LeanOcctCirclePayload& the_result) {
                       fillCirclePayload(*offsetBasisCurve(requireFaceShape(the_shape)), the_result);
                     });
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_offset_basis_curve_ellipse_payload(
  LeanOcctContext*         the_context,
  const LeanOcctShape*     the_shape,
  LeanOcctEllipsePayload*  the_payload)
{
  return writeOutput(the_context,
                     the_payload,
                     "LeanOcctEllipsePayload output pointer was null.",
                     [&](LeanOcctEllipsePayload& the_result) {
                       fillEllipsePayload(*offsetBasisCurve(requireFaceShape(the_shape)), the_result);
                     });
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_describe(
  LeanOcctContext*      the_context,
  const LeanOcctShape*  the_shape,
  LeanOcctShapeSummary* the_summary)
{
  return fillShapeOutput(the_context,
                         the_shape,
                         the_summary,
                         "LeanOcctShapeSummary was null.",
                         requireShape,
                         fillShapeSummary);
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_subshape_count(
  LeanOcctContext*      the_context,
  const LeanOcctShape*  the_shape,
  LeanOcctShapeKind     the_kind,
  size_t*               the_count)
{
  return writeOutput(the_context, the_count, "Subshape count output pointer was null.", [&](size_t& the_result) {
    the_result = countIndexedSubshapes(requireShape(the_shape), the_kind);
  });
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctShape* lean_occt_shape_subshape(
  LeanOcctContext*     the_context,
  const LeanOcctShape* the_shape,
  LeanOcctShapeKind    the_kind,
  size_t               the_index)
{
  return guardShapeCall(the_context, [&]() -> TopoDS_Shape {
    return indexedSubshape(requireShape(the_shape), the_kind, the_index);
  });
}

extern "C" LEAN_OCCT_CAPI_EXPORT size_t lean_occt_shape_edge_count(const LeanOcctShape* the_shape)
{
  try
  {
    return countSubshapes(requireShape(the_shape), TopAbs_EDGE);
  }
  catch (...)
  {
    return 0;
  }
}

extern "C" LEAN_OCCT_CAPI_EXPORT size_t lean_occt_shape_face_count_raw(const LeanOcctShape* the_shape)
{
  try
  {
    return countSubshapes(requireShape(the_shape), TopAbs_FACE);
  }
  catch (...)
  {
    return 0;
  }
}

extern "C" LEAN_OCCT_CAPI_EXPORT size_t lean_occt_shape_solid_count_raw(const LeanOcctShape* the_shape)
{
  try
  {
    return countSubshapes(requireShape(the_shape), TopAbs_SOLID);
  }
  catch (...)
  {
    return 0;
  }
}

extern "C" LEAN_OCCT_CAPI_EXPORT double lean_occt_shape_linear_length(const LeanOcctShape* the_shape)
{
  try
  {
    GProp_GProps a_props;
    BRepGProp::LinearProperties(requireShape(the_shape), a_props);
    return a_props.Mass();
  }
  catch (...)
  {
    return 0.0;
  }
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctTopology* lean_occt_shape_topology(
  LeanOcctContext*     the_context,
  const LeanOcctShape* the_shape)
{
  return guardCall(the_context, [&]() -> LeanOcctTopology* {
    if (the_context == nullptr)
    {
      throw std::invalid_argument(nullContextError());
    }

    LeanOcctTopology* a_topology = new LeanOcctTopology();
    a_topology->Buffers = snapshotTopology(requireShape(the_shape));
    return a_topology;
  });
}

extern "C" LEAN_OCCT_CAPI_EXPORT void lean_occt_topology_destroy(LeanOcctTopology* the_topology)
{
  delete the_topology;
}

extern "C" LEAN_OCCT_CAPI_EXPORT size_t lean_occt_topology_vertex_count(const LeanOcctTopology* the_topology)
{
  try
  {
    return requireTopology(the_topology).Buffers.vertexPositions.size() / 3;
  }
  catch (...)
  {
    return 0;
  }
}

extern "C" LEAN_OCCT_CAPI_EXPORT size_t lean_occt_topology_edge_count(const LeanOcctTopology* the_topology)
{
  try
  {
    return requireTopology(the_topology).Buffers.edgeLengths.size();
  }
  catch (...)
  {
    return 0;
  }
}

extern "C" LEAN_OCCT_CAPI_EXPORT size_t lean_occt_topology_wire_count(const LeanOcctTopology* the_topology)
{
  try
  {
    return requireTopology(the_topology).Buffers.wireRanges.size() / 2;
  }
  catch (...)
  {
    return 0;
  }
}

extern "C" LEAN_OCCT_CAPI_EXPORT size_t lean_occt_topology_face_count(const LeanOcctTopology* the_topology)
{
  try
  {
    return requireTopology(the_topology).Buffers.faceRanges.size() / 2;
  }
  catch (...)
  {
    return 0;
  }
}

extern "C" LEAN_OCCT_CAPI_EXPORT const double* lean_occt_topology_vertex_positions(
  const LeanOcctTopology* the_topology)
{
  try
  {
    return requireTopology(the_topology).Buffers.vertexPositions.data();
  }
  catch (...)
  {
    return nullptr;
  }
}

extern "C" LEAN_OCCT_CAPI_EXPORT const uint32_t* lean_occt_topology_edge_vertex_indices(
  const LeanOcctTopology* the_topology)
{
  try
  {
    return requireTopology(the_topology).Buffers.edgeVertexIndices.data();
  }
  catch (...)
  {
    return nullptr;
  }
}

extern "C" LEAN_OCCT_CAPI_EXPORT const double* lean_occt_topology_edge_lengths(
  const LeanOcctTopology* the_topology)
{
  try
  {
    return requireTopology(the_topology).Buffers.edgeLengths.data();
  }
  catch (...)
  {
    return nullptr;
  }
}

extern "C" LEAN_OCCT_CAPI_EXPORT const uint32_t* lean_occt_topology_edge_face_ranges(
  const LeanOcctTopology* the_topology)
{
  try
  {
    return requireTopology(the_topology).Buffers.edgeFaceRanges.data();
  }
  catch (...)
  {
    return nullptr;
  }
}

extern "C" LEAN_OCCT_CAPI_EXPORT const uint32_t* lean_occt_topology_edge_face_indices(
  const LeanOcctTopology* the_topology)
{
  try
  {
    return requireTopology(the_topology).Buffers.edgeFaceIndices.data();
  }
  catch (...)
  {
    return nullptr;
  }
}

extern "C" LEAN_OCCT_CAPI_EXPORT const uint32_t* lean_occt_topology_wire_ranges(
  const LeanOcctTopology* the_topology)
{
  try
  {
    return requireTopology(the_topology).Buffers.wireRanges.data();
  }
  catch (...)
  {
    return nullptr;
  }
}

extern "C" LEAN_OCCT_CAPI_EXPORT const uint32_t* lean_occt_topology_wire_edge_indices(
  const LeanOcctTopology* the_topology)
{
  try
  {
    return requireTopology(the_topology).Buffers.wireEdgeIndices.data();
  }
  catch (...)
  {
    return nullptr;
  }
}

extern "C" LEAN_OCCT_CAPI_EXPORT const uint8_t* lean_occt_topology_wire_edge_orientations(
  const LeanOcctTopology* the_topology)
{
  try
  {
    return requireTopology(the_topology).Buffers.wireEdgeOrientations.data();
  }
  catch (...)
  {
    return nullptr;
  }
}

extern "C" LEAN_OCCT_CAPI_EXPORT const uint32_t* lean_occt_topology_wire_vertex_ranges(
  const LeanOcctTopology* the_topology)
{
  try
  {
    return requireTopology(the_topology).Buffers.wireVertexRanges.data();
  }
  catch (...)
  {
    return nullptr;
  }
}

extern "C" LEAN_OCCT_CAPI_EXPORT const uint32_t* lean_occt_topology_wire_vertex_indices(
  const LeanOcctTopology* the_topology)
{
  try
  {
    return requireTopology(the_topology).Buffers.wireVertexIndices.data();
  }
  catch (...)
  {
    return nullptr;
  }
}

extern "C" LEAN_OCCT_CAPI_EXPORT const uint32_t* lean_occt_topology_face_ranges(
  const LeanOcctTopology* the_topology)
{
  try
  {
    return requireTopology(the_topology).Buffers.faceRanges.data();
  }
  catch (...)
  {
    return nullptr;
  }
}

extern "C" LEAN_OCCT_CAPI_EXPORT const uint32_t* lean_occt_topology_face_wire_indices(
  const LeanOcctTopology* the_topology)
{
  try
  {
    return requireTopology(the_topology).Buffers.faceWireIndices.data();
  }
  catch (...)
  {
    return nullptr;
  }
}

extern "C" LEAN_OCCT_CAPI_EXPORT const uint8_t* lean_occt_topology_face_wire_orientations(
  const LeanOcctTopology* the_topology)
{
  try
  {
    return requireTopology(the_topology).Buffers.faceWireOrientations.data();
  }
  catch (...)
  {
    return nullptr;
  }
}

extern "C" LEAN_OCCT_CAPI_EXPORT const uint8_t* lean_occt_topology_face_wire_roles(
  const LeanOcctTopology* the_topology)
{
  try
  {
    return requireTopology(the_topology).Buffers.faceWireRoles.data();
  }
  catch (...)
  {
    return nullptr;
  }
}

extern "C" LEAN_OCCT_CAPI_EXPORT LeanOcctMesh* lean_occt_shape_mesh(
  LeanOcctContext*          the_context,
  const LeanOcctShape*      the_shape,
  const LeanOcctMeshParams* the_params)
{
  return guardCall(the_context, [&]() -> LeanOcctMesh* {
    if (the_context == nullptr)
    {
      throw std::invalid_argument(nullContextError());
    }
    if (the_params == nullptr)
    {
      throw std::invalid_argument("LeanOcctMeshParams was null.");
    }

    LeanOcctMesh* a_mesh = new LeanOcctMesh();
    a_mesh->Buffers = tessellateShape(requireShape(the_shape), *the_params);
    return a_mesh;
  });
}

extern "C" LEAN_OCCT_CAPI_EXPORT void lean_occt_mesh_destroy(LeanOcctMesh* the_mesh)
{
  delete the_mesh;
}

extern "C" LEAN_OCCT_CAPI_EXPORT size_t lean_occt_mesh_vertex_count(const LeanOcctMesh* the_mesh)
{
  try
  {
    return requireMesh(the_mesh).Buffers.positions.size() / 3;
  }
  catch (...)
  {
    return 0;
  }
}

extern "C" LEAN_OCCT_CAPI_EXPORT size_t lean_occt_mesh_triangle_count(const LeanOcctMesh* the_mesh)
{
  try
  {
    return requireMesh(the_mesh).Buffers.triangleIndices.size() / 3;
  }
  catch (...)
  {
    return 0;
  }
}

extern "C" LEAN_OCCT_CAPI_EXPORT size_t lean_occt_mesh_edge_segment_count(const LeanOcctMesh* the_mesh)
{
  try
  {
    return requireMesh(the_mesh).Buffers.edgePositions.size() / 6;
  }
  catch (...)
  {
    return 0;
  }
}

extern "C" LEAN_OCCT_CAPI_EXPORT size_t lean_occt_mesh_face_count(const LeanOcctMesh* the_mesh)
{
  try
  {
    return requireMesh(the_mesh).Buffers.faceCount;
  }
  catch (...)
  {
    return 0;
  }
}

extern "C" LEAN_OCCT_CAPI_EXPORT size_t lean_occt_mesh_solid_count(const LeanOcctMesh* the_mesh)
{
  try
  {
    return requireMesh(the_mesh).Buffers.solidCount;
  }
  catch (...)
  {
    return 0;
  }
}

extern "C" LEAN_OCCT_CAPI_EXPORT const double* lean_occt_mesh_positions(const LeanOcctMesh* the_mesh)
{
  try
  {
    return requireMesh(the_mesh).Buffers.positions.data();
  }
  catch (...)
  {
    return nullptr;
  }
}

extern "C" LEAN_OCCT_CAPI_EXPORT const double* lean_occt_mesh_normals(const LeanOcctMesh* the_mesh)
{
  try
  {
    return requireMesh(the_mesh).Buffers.normals.data();
  }
  catch (...)
  {
    return nullptr;
  }
}

extern "C" LEAN_OCCT_CAPI_EXPORT const uint32_t* lean_occt_mesh_triangle_indices(
  const LeanOcctMesh* the_mesh)
{
  try
  {
    return requireMesh(the_mesh).Buffers.triangleIndices.data();
  }
  catch (...)
  {
    return nullptr;
  }
}

extern "C" LEAN_OCCT_CAPI_EXPORT const double* lean_occt_mesh_edge_positions(const LeanOcctMesh* the_mesh)
{
  try
  {
    return requireMesh(the_mesh).Buffers.edgePositions.data();
  }
  catch (...)
  {
    return nullptr;
  }
}

extern "C" LEAN_OCCT_CAPI_EXPORT void lean_occt_mesh_bounds(const LeanOcctMesh* the_mesh,
                                                            double*             the_min_xyz3,
                                                            double*             the_max_xyz3)
{
  try
  {
    const LeanOcctMesh& a_mesh = requireMesh(the_mesh);
    if (the_min_xyz3 != nullptr)
    {
      the_min_xyz3[0] = a_mesh.Buffers.bboxMin[0];
      the_min_xyz3[1] = a_mesh.Buffers.bboxMin[1];
      the_min_xyz3[2] = a_mesh.Buffers.bboxMin[2];
    }
    if (the_max_xyz3 != nullptr)
    {
      the_max_xyz3[0] = a_mesh.Buffers.bboxMax[0];
      the_max_xyz3[1] = a_mesh.Buffers.bboxMax[1];
      the_max_xyz3[2] = a_mesh.Buffers.bboxMax[2];
    }
  }
  catch (...)
  {
  }
}
