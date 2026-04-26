#ifndef LEAN_OCCT_CAPI_H
#define LEAN_OCCT_CAPI_H

#include <stddef.h>
#include <stdint.h>

#if defined(_WIN32) && !defined(LEAN_OCCT_CAPI_STATIC)
  #if defined(LEAN_OCCT_CAPI_BUILDING_SHARED)
    #define LEAN_OCCT_CAPI_EXPORT __declspec(dllexport)
  #else
    #define LEAN_OCCT_CAPI_EXPORT __declspec(dllimport)
  #endif
#elif defined(__GNUC__) || defined(__clang__)
  #define LEAN_OCCT_CAPI_EXPORT __attribute__((visibility("default")))
#else
  #define LEAN_OCCT_CAPI_EXPORT
#endif

#ifdef __cplusplus
extern "C" {
#endif

typedef struct LeanOcctContext LeanOcctContext;
typedef struct LeanOcctShape LeanOcctShape;
typedef struct LeanOcctMesh LeanOcctMesh;
typedef struct LeanOcctTopology LeanOcctTopology;

typedef enum LeanOcctResult
{
  LEAN_OCCT_RESULT_OK = 0,
  LEAN_OCCT_RESULT_ERROR = 1
} LeanOcctResult;

typedef enum LeanOcctShapeKind
{
  LEAN_OCCT_SHAPE_KIND_UNKNOWN = 0,
  LEAN_OCCT_SHAPE_KIND_COMPOUND = 1,
  LEAN_OCCT_SHAPE_KIND_COMPSOLID = 2,
  LEAN_OCCT_SHAPE_KIND_SOLID = 3,
  LEAN_OCCT_SHAPE_KIND_SHELL = 4,
  LEAN_OCCT_SHAPE_KIND_FACE = 5,
  LEAN_OCCT_SHAPE_KIND_WIRE = 6,
  LEAN_OCCT_SHAPE_KIND_EDGE = 7,
  LEAN_OCCT_SHAPE_KIND_VERTEX = 8,
  LEAN_OCCT_SHAPE_KIND_SHAPE = 9
} LeanOcctShapeKind;

typedef enum LeanOcctOrientation
{
  LEAN_OCCT_ORIENTATION_FORWARD = 0,
  LEAN_OCCT_ORIENTATION_REVERSED = 1,
  LEAN_OCCT_ORIENTATION_INTERNAL = 2,
  LEAN_OCCT_ORIENTATION_EXTERNAL = 3
} LeanOcctOrientation;

typedef enum LeanOcctLoopRole
{
  LEAN_OCCT_LOOP_ROLE_UNKNOWN = 0,
  LEAN_OCCT_LOOP_ROLE_OUTER = 1,
  LEAN_OCCT_LOOP_ROLE_INNER = 2
} LeanOcctLoopRole;

typedef enum LeanOcctCurveKind
{
  LEAN_OCCT_CURVE_KIND_UNKNOWN = 0,
  LEAN_OCCT_CURVE_KIND_LINE = 1,
  LEAN_OCCT_CURVE_KIND_CIRCLE = 2,
  LEAN_OCCT_CURVE_KIND_ELLIPSE = 3,
  LEAN_OCCT_CURVE_KIND_HYPERBOLA = 4,
  LEAN_OCCT_CURVE_KIND_PARABOLA = 5,
  LEAN_OCCT_CURVE_KIND_BEZIER = 6,
  LEAN_OCCT_CURVE_KIND_BSPLINE = 7,
  LEAN_OCCT_CURVE_KIND_OFFSET = 8,
  LEAN_OCCT_CURVE_KIND_OTHER = 9
} LeanOcctCurveKind;

typedef enum LeanOcctSurfaceKind
{
  LEAN_OCCT_SURFACE_KIND_UNKNOWN = 0,
  LEAN_OCCT_SURFACE_KIND_PLANE = 1,
  LEAN_OCCT_SURFACE_KIND_CYLINDER = 2,
  LEAN_OCCT_SURFACE_KIND_CONE = 3,
  LEAN_OCCT_SURFACE_KIND_SPHERE = 4,
  LEAN_OCCT_SURFACE_KIND_TORUS = 5,
  LEAN_OCCT_SURFACE_KIND_BEZIER = 6,
  LEAN_OCCT_SURFACE_KIND_BSPLINE = 7,
  LEAN_OCCT_SURFACE_KIND_REVOLUTION = 8,
  LEAN_OCCT_SURFACE_KIND_EXTRUSION = 9,
  LEAN_OCCT_SURFACE_KIND_OFFSET = 10,
  LEAN_OCCT_SURFACE_KIND_OTHER = 11
} LeanOcctSurfaceKind;

typedef struct LeanOcctBoxParams
{
  double x;
  double y;
  double z;
  double dx;
  double dy;
  double dz;
} LeanOcctBoxParams;

typedef struct LeanOcctCylinderParams
{
  double x;
  double y;
  double z;
  double axis_x;
  double axis_y;
  double axis_z;
  double radius;
  double height;
} LeanOcctCylinderParams;

typedef struct LeanOcctConeParams
{
  double x;
  double y;
  double z;
  double axis_x;
  double axis_y;
  double axis_z;
  double x_dir_x;
  double x_dir_y;
  double x_dir_z;
  double base_radius;
  double top_radius;
  double height;
} LeanOcctConeParams;

typedef struct LeanOcctSphereParams
{
  double x;
  double y;
  double z;
  double axis_x;
  double axis_y;
  double axis_z;
  double x_dir_x;
  double x_dir_y;
  double x_dir_z;
  double radius;
} LeanOcctSphereParams;

typedef struct LeanOcctTorusParams
{
  double x;
  double y;
  double z;
  double axis_x;
  double axis_y;
  double axis_z;
  double x_dir_x;
  double x_dir_y;
  double x_dir_z;
  double major_radius;
  double minor_radius;
} LeanOcctTorusParams;

typedef struct LeanOcctEllipseEdgeParams
{
  double x;
  double y;
  double z;
  double axis_x;
  double axis_y;
  double axis_z;
  double x_dir_x;
  double x_dir_y;
  double x_dir_z;
  double major_radius;
  double minor_radius;
} LeanOcctEllipseEdgeParams;

typedef struct LeanOcctFilletParams
{
  double radius;
  uint32_t edge_index;
} LeanOcctFilletParams;

typedef struct LeanOcctOffsetParams
{
  double offset;
  double tolerance;
} LeanOcctOffsetParams;

typedef struct LeanOcctCylindricalHoleParams
{
  double x;
  double y;
  double z;
  double axis_x;
  double axis_y;
  double axis_z;
  double radius;
} LeanOcctCylindricalHoleParams;

typedef struct LeanOcctHelixParams
{
  double origin_x;
  double origin_y;
  double origin_z;
  double axis_x;
  double axis_y;
  double axis_z;
  double x_dir_x;
  double x_dir_y;
  double x_dir_z;
  double radius;
  double height;
  double pitch;
} LeanOcctHelixParams;

typedef struct LeanOcctPrismParams
{
  double dx;
  double dy;
  double dz;
} LeanOcctPrismParams;

typedef struct LeanOcctRevolutionParams
{
  double x;
  double y;
  double z;
  double axis_x;
  double axis_y;
  double axis_z;
  double angle_radians;
} LeanOcctRevolutionParams;

typedef struct LeanOcctMeshParams
{
  double linear_deflection;
  double angular_deflection;
  uint8_t is_relative;
} LeanOcctMeshParams;

typedef struct LeanOcctShapeSummary
{
  LeanOcctShapeKind root_kind;
  LeanOcctShapeKind primary_kind;
  size_t compound_count;
  size_t compsolid_count;
  size_t solid_count;
  size_t shell_count;
  size_t face_count;
  size_t wire_count;
  size_t edge_count;
  size_t vertex_count;
  double linear_length;
  double surface_area;
  double volume;
  double bbox_min[3];
  double bbox_max[3];
} LeanOcctShapeSummary;

typedef struct LeanOcctEdgeSample
{
  double position[3];
  double tangent[3];
} LeanOcctEdgeSample;

typedef struct LeanOcctBbox
{
  double min[3];
  double max[3];
} LeanOcctBbox;

typedef struct LeanOcctFaceUvBounds
{
  double u_min;
  double u_max;
  double v_min;
  double v_max;
} LeanOcctFaceUvBounds;

typedef struct LeanOcctFaceSample
{
  double position[3];
  double normal[3];
} LeanOcctFaceSample;

typedef struct LeanOcctEdgeGeometry
{
  LeanOcctCurveKind kind;
  double start_parameter;
  double end_parameter;
  uint8_t is_closed;
  uint8_t is_periodic;
  double period;
} LeanOcctEdgeGeometry;

typedef struct LeanOcctFaceGeometry
{
  LeanOcctSurfaceKind kind;
  double u_min;
  double u_max;
  double v_min;
  double v_max;
  uint8_t is_u_closed;
  uint8_t is_v_closed;
  uint8_t is_u_periodic;
  uint8_t is_v_periodic;
  double u_period;
  double v_period;
} LeanOcctFaceGeometry;

typedef struct LeanOcctLinePayload
{
  double origin[3];
  double direction[3];
} LeanOcctLinePayload;

typedef struct LeanOcctCirclePayload
{
  double center[3];
  double normal[3];
  double x_direction[3];
  double y_direction[3];
  double radius;
} LeanOcctCirclePayload;

typedef struct LeanOcctPlanePayload
{
  double origin[3];
  double normal[3];
  double x_direction[3];
  double y_direction[3];
} LeanOcctPlanePayload;

typedef struct LeanOcctCylinderPayload
{
  double origin[3];
  double axis[3];
  double x_direction[3];
  double y_direction[3];
  double radius;
} LeanOcctCylinderPayload;

typedef struct LeanOcctEllipsePayload
{
  double center[3];
  double normal[3];
  double x_direction[3];
  double y_direction[3];
  double major_radius;
  double minor_radius;
} LeanOcctEllipsePayload;

typedef struct LeanOcctConePayload
{
  double origin[3];
  double axis[3];
  double x_direction[3];
  double y_direction[3];
  double reference_radius;
  double semi_angle;
} LeanOcctConePayload;

typedef struct LeanOcctSpherePayload
{
  double center[3];
  double normal[3];
  double x_direction[3];
  double y_direction[3];
  double radius;
} LeanOcctSpherePayload;

typedef struct LeanOcctTorusPayload
{
  double center[3];
  double axis[3];
  double x_direction[3];
  double y_direction[3];
  double major_radius;
  double minor_radius;
} LeanOcctTorusPayload;

typedef struct LeanOcctRevolutionSurfacePayload
{
  double axis_origin[3];
  double axis_direction[3];
  LeanOcctCurveKind basis_curve_kind;
} LeanOcctRevolutionSurfacePayload;

typedef struct LeanOcctExtrusionSurfacePayload
{
  double direction[3];
  LeanOcctCurveKind basis_curve_kind;
} LeanOcctExtrusionSurfacePayload;

typedef struct LeanOcctOffsetSurfacePayload
{
  double offset_value;
  LeanOcctSurfaceKind basis_surface_kind;
} LeanOcctOffsetSurfacePayload;

LEAN_OCCT_CAPI_EXPORT LeanOcctContext* lean_occt_context_create(void);
LEAN_OCCT_CAPI_EXPORT void lean_occt_context_destroy(LeanOcctContext* the_context);
LEAN_OCCT_CAPI_EXPORT const char* lean_occt_context_last_error(const LeanOcctContext* the_context);

LEAN_OCCT_CAPI_EXPORT LeanOcctShape* lean_occt_shape_make_box(LeanOcctContext* the_context,
                                                              const LeanOcctBoxParams* the_params);
LEAN_OCCT_CAPI_EXPORT LeanOcctShape* lean_occt_shape_make_cylinder(
  LeanOcctContext* the_context,
  const LeanOcctCylinderParams* the_params);
LEAN_OCCT_CAPI_EXPORT LeanOcctShape* lean_occt_shape_make_cone(
  LeanOcctContext* the_context,
  const LeanOcctConeParams* the_params);
LEAN_OCCT_CAPI_EXPORT LeanOcctShape* lean_occt_shape_make_sphere(
  LeanOcctContext* the_context,
  const LeanOcctSphereParams* the_params);
LEAN_OCCT_CAPI_EXPORT LeanOcctShape* lean_occt_shape_make_torus(
  LeanOcctContext* the_context,
  const LeanOcctTorusParams* the_params);
LEAN_OCCT_CAPI_EXPORT LeanOcctShape* lean_occt_shape_make_ellipse_edge(
  LeanOcctContext* the_context,
  const LeanOcctEllipseEdgeParams* the_params);
LEAN_OCCT_CAPI_EXPORT LeanOcctShape* lean_occt_shape_make_fillet(
  LeanOcctContext* the_context,
  const LeanOcctShape* the_shape,
  const LeanOcctFilletParams* the_params);
LEAN_OCCT_CAPI_EXPORT LeanOcctShape* lean_occt_shape_make_offset(
  LeanOcctContext* the_context,
  const LeanOcctShape* the_shape,
  const LeanOcctOffsetParams* the_params);
LEAN_OCCT_CAPI_EXPORT LeanOcctShape* lean_occt_shape_make_offset_surface_face(
  LeanOcctContext* the_context,
  const LeanOcctShape* the_shape,
  const LeanOcctOffsetParams* the_params);
LEAN_OCCT_CAPI_EXPORT LeanOcctShape* lean_occt_shape_make_cylindrical_hole(
  LeanOcctContext* the_context,
  const LeanOcctShape* the_shape,
  const LeanOcctCylindricalHoleParams* the_params);
LEAN_OCCT_CAPI_EXPORT LeanOcctShape* lean_occt_shape_make_helix(
  LeanOcctContext* the_context,
  const LeanOcctHelixParams* the_params);
LEAN_OCCT_CAPI_EXPORT LeanOcctShape* lean_occt_shape_make_prism(
  LeanOcctContext* the_context,
  const LeanOcctShape* the_shape,
  const LeanOcctPrismParams* the_params);
LEAN_OCCT_CAPI_EXPORT LeanOcctShape* lean_occt_shape_make_revolution(
  LeanOcctContext* the_context,
  const LeanOcctShape* the_shape,
  const LeanOcctRevolutionParams* the_params);
LEAN_OCCT_CAPI_EXPORT LeanOcctShape* lean_occt_shape_boolean_cut(LeanOcctContext*      the_context,
                                                                 const LeanOcctShape*  the_lhs,
                                                                 const LeanOcctShape*  the_rhs);
LEAN_OCCT_CAPI_EXPORT LeanOcctShape* lean_occt_shape_boolean_fuse(LeanOcctContext*      the_context,
                                                                  const LeanOcctShape*  the_lhs,
                                                                  const LeanOcctShape*  the_rhs);
LEAN_OCCT_CAPI_EXPORT LeanOcctShape* lean_occt_shape_boolean_common(LeanOcctContext*      the_context,
                                                                    const LeanOcctShape*  the_lhs,
                                                                    const LeanOcctShape*  the_rhs);
LEAN_OCCT_CAPI_EXPORT LeanOcctShape* lean_occt_shape_read_step(LeanOcctContext* the_context,
                                                               const char* the_path_utf8);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_write_step(LeanOcctContext*     the_context,
                                                                const LeanOcctShape* the_shape,
                                                                const char* the_path_utf8);
LEAN_OCCT_CAPI_EXPORT LeanOcctShape* lean_occt_shape_clone(LeanOcctContext*     the_context,
                                                            const LeanOcctShape* the_shape);
LEAN_OCCT_CAPI_EXPORT void lean_occt_shape_destroy(LeanOcctShape* the_shape);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_orientation(
  LeanOcctContext*        the_context,
  const LeanOcctShape*    the_shape,
  LeanOcctOrientation*    the_orientation);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_is_same(LeanOcctContext*     the_context,
                                                             const LeanOcctShape* the_lhs,
                                                             const LeanOcctShape* the_rhs,
                                                             uint8_t*             the_is_same);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_vertex_point(LeanOcctContext*     the_context,
                                                                  const LeanOcctShape* the_shape,
                                                                  double*              the_xyz3);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_edge_endpoints(LeanOcctContext*     the_context,
                                                                    const LeanOcctShape* the_shape,
                                                                    double*              the_start_xyz3,
                                                                    double*              the_end_xyz3);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_edge_sample(LeanOcctContext*        the_context,
                                                                 const LeanOcctShape*    the_shape,
                                                                 double                  the_t,
                                                                 LeanOcctEdgeSample*     the_sample);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_edge_sample_at_parameter(
  LeanOcctContext*        the_context,
  const LeanOcctShape*    the_shape,
  double                  the_parameter,
  LeanOcctEdgeSample*     the_sample);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_edge_geometry(LeanOcctContext*        the_context,
                                                                   const LeanOcctShape*    the_shape,
                                                                   LeanOcctEdgeGeometry*   the_geometry);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_edge_curve_bbox(
  LeanOcctContext*        the_context,
  const LeanOcctShape*    the_shape,
  LeanOcctBbox*           the_bbox);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_edge_line_payload(
  LeanOcctContext*         the_context,
  const LeanOcctShape*     the_shape,
  LeanOcctLinePayload*     the_payload);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_edge_circle_payload(
  LeanOcctContext*         the_context,
  const LeanOcctShape*     the_shape,
  LeanOcctCirclePayload*   the_payload);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_edge_ellipse_payload(
  LeanOcctContext*         the_context,
  const LeanOcctShape*     the_shape,
  LeanOcctEllipsePayload*  the_payload);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_uv_bounds(
  LeanOcctContext*          the_context,
  const LeanOcctShape*      the_shape,
  LeanOcctFaceUvBounds*     the_bounds);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_sample(LeanOcctContext*      the_context,
                                                                 const LeanOcctShape*  the_shape,
                                                                 double                the_u,
                                                                 double                the_v,
                                                                 LeanOcctFaceSample*   the_sample);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_sample_normalized(
  LeanOcctContext*      the_context,
  const LeanOcctShape*  the_shape,
  double                the_u_t,
  double                the_v_t,
  LeanOcctFaceSample*   the_sample);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_geometry(LeanOcctContext*        the_context,
                                                                   const LeanOcctShape*    the_shape,
                                                                   LeanOcctFaceGeometry*   the_geometry);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_pcurve_control_polygon_bbox(
  LeanOcctContext*        the_context,
  const LeanOcctShape*    the_shape,
  LeanOcctBbox*           the_bbox);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_surface_bbox(
  LeanOcctContext*        the_context,
  const LeanOcctShape*    the_shape,
  LeanOcctBbox*           the_bbox);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_plane_payload(
  LeanOcctContext*           the_context,
  const LeanOcctShape*       the_shape,
  LeanOcctPlanePayload*      the_payload);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_cylinder_payload(
  LeanOcctContext*              the_context,
  const LeanOcctShape*          the_shape,
  LeanOcctCylinderPayload*      the_payload);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_cone_payload(
  LeanOcctContext*          the_context,
  const LeanOcctShape*      the_shape,
  LeanOcctConePayload*      the_payload);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_sphere_payload(
  LeanOcctContext*            the_context,
  const LeanOcctShape*        the_shape,
  LeanOcctSpherePayload*      the_payload);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_torus_payload(
  LeanOcctContext*           the_context,
  const LeanOcctShape*       the_shape,
  LeanOcctTorusPayload*      the_payload);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_revolution_payload(
  LeanOcctContext*                    the_context,
  const LeanOcctShape*                the_shape,
  LeanOcctRevolutionSurfacePayload*   the_payload);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_extrusion_payload(
  LeanOcctContext*                   the_context,
  const LeanOcctShape*               the_shape,
  LeanOcctExtrusionSurfacePayload*   the_payload);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_offset_payload(
  LeanOcctContext*                the_context,
  const LeanOcctShape*            the_shape,
  LeanOcctOffsetSurfacePayload*   the_payload);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_offset_basis_geometry(
  LeanOcctContext*                the_context,
  const LeanOcctShape*            the_shape,
  LeanOcctFaceGeometry*           the_geometry);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_offset_basis_plane_payload(
  LeanOcctContext*                the_context,
  const LeanOcctShape*            the_shape,
  LeanOcctPlanePayload*           the_payload);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_offset_basis_cylinder_payload(
  LeanOcctContext*                the_context,
  const LeanOcctShape*            the_shape,
  LeanOcctCylinderPayload*        the_payload);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_offset_basis_cone_payload(
  LeanOcctContext*                the_context,
  const LeanOcctShape*            the_shape,
  LeanOcctConePayload*            the_payload);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_offset_basis_sphere_payload(
  LeanOcctContext*                the_context,
  const LeanOcctShape*            the_shape,
  LeanOcctSpherePayload*          the_payload);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_offset_basis_torus_payload(
  LeanOcctContext*                the_context,
  const LeanOcctShape*            the_shape,
  LeanOcctTorusPayload*           the_payload);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_offset_basis_revolution_payload(
  LeanOcctContext*                    the_context,
  const LeanOcctShape*                the_shape,
  LeanOcctRevolutionSurfacePayload*   the_payload);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_offset_basis_extrusion_payload(
  LeanOcctContext*                   the_context,
  const LeanOcctShape*               the_shape,
  LeanOcctExtrusionSurfacePayload*   the_payload);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_offset_basis_curve_geometry(
  LeanOcctContext*                the_context,
  const LeanOcctShape*            the_shape,
  LeanOcctEdgeGeometry*           the_geometry);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_offset_basis_curve_line_payload(
  LeanOcctContext*                the_context,
  const LeanOcctShape*            the_shape,
  LeanOcctLinePayload*            the_payload);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_offset_basis_curve_circle_payload(
  LeanOcctContext*                the_context,
  const LeanOcctShape*            the_shape,
  LeanOcctCirclePayload*          the_payload);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_face_offset_basis_curve_ellipse_payload(
  LeanOcctContext*                the_context,
  const LeanOcctShape*            the_shape,
  LeanOcctEllipsePayload*         the_payload);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_describe(LeanOcctContext*            the_context,
                                                              const LeanOcctShape*        the_shape,
                                                              LeanOcctShapeSummary*       the_summary);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_subshape_count(
  LeanOcctContext*      the_context,
  const LeanOcctShape*  the_shape,
  LeanOcctShapeKind     the_kind,
  size_t*               the_count);
LEAN_OCCT_CAPI_EXPORT LeanOcctShape* lean_occt_shape_subshape(LeanOcctContext*      the_context,
                                                              const LeanOcctShape*  the_shape,
                                                              LeanOcctShapeKind     the_kind,
                                                              size_t                the_index);
LEAN_OCCT_CAPI_EXPORT LeanOcctShape* lean_occt_shape_root_edge_vertex(
  LeanOcctContext*     the_context,
  const LeanOcctShape* the_shape,
  size_t               the_endpoint_index);
LEAN_OCCT_CAPI_EXPORT LeanOcctResult lean_occt_shape_wire_edge_occurrence_count(
  LeanOcctContext*      the_context,
  const LeanOcctShape*  the_shape,
  size_t*               the_count);
LEAN_OCCT_CAPI_EXPORT LeanOcctShape* lean_occt_shape_wire_edge_occurrence(
  LeanOcctContext*      the_context,
  const LeanOcctShape*  the_shape,
  size_t                the_index);
LEAN_OCCT_CAPI_EXPORT size_t lean_occt_shape_edge_count(const LeanOcctShape* the_shape);
LEAN_OCCT_CAPI_EXPORT size_t lean_occt_shape_face_count_raw(const LeanOcctShape* the_shape);
LEAN_OCCT_CAPI_EXPORT size_t lean_occt_shape_solid_count_raw(const LeanOcctShape* the_shape);
LEAN_OCCT_CAPI_EXPORT double lean_occt_shape_linear_length(const LeanOcctShape* the_shape);
LEAN_OCCT_CAPI_EXPORT LeanOcctTopology* lean_occt_shape_topology(LeanOcctContext*     the_context,
                                                                 const LeanOcctShape* the_shape);

LEAN_OCCT_CAPI_EXPORT void lean_occt_topology_destroy(LeanOcctTopology* the_topology);
LEAN_OCCT_CAPI_EXPORT size_t lean_occt_topology_vertex_count(const LeanOcctTopology* the_topology);
LEAN_OCCT_CAPI_EXPORT size_t lean_occt_topology_edge_count(const LeanOcctTopology* the_topology);
LEAN_OCCT_CAPI_EXPORT size_t lean_occt_topology_wire_count(const LeanOcctTopology* the_topology);
LEAN_OCCT_CAPI_EXPORT size_t lean_occt_topology_face_count(const LeanOcctTopology* the_topology);
LEAN_OCCT_CAPI_EXPORT const double* lean_occt_topology_vertex_positions(const LeanOcctTopology* the_topology);
LEAN_OCCT_CAPI_EXPORT const uint32_t* lean_occt_topology_edge_vertex_indices(
  const LeanOcctTopology* the_topology);
LEAN_OCCT_CAPI_EXPORT const double* lean_occt_topology_edge_lengths(const LeanOcctTopology* the_topology);
LEAN_OCCT_CAPI_EXPORT const uint32_t* lean_occt_topology_edge_face_ranges(
  const LeanOcctTopology* the_topology);
LEAN_OCCT_CAPI_EXPORT const uint32_t* lean_occt_topology_edge_face_indices(
  const LeanOcctTopology* the_topology);
LEAN_OCCT_CAPI_EXPORT const uint32_t* lean_occt_topology_wire_ranges(const LeanOcctTopology* the_topology);
LEAN_OCCT_CAPI_EXPORT const uint32_t* lean_occt_topology_wire_edge_indices(
  const LeanOcctTopology* the_topology);
LEAN_OCCT_CAPI_EXPORT const uint8_t* lean_occt_topology_wire_edge_orientations(
  const LeanOcctTopology* the_topology);
LEAN_OCCT_CAPI_EXPORT const uint32_t* lean_occt_topology_wire_vertex_ranges(
  const LeanOcctTopology* the_topology);
LEAN_OCCT_CAPI_EXPORT const uint32_t* lean_occt_topology_wire_vertex_indices(
  const LeanOcctTopology* the_topology);
LEAN_OCCT_CAPI_EXPORT const uint32_t* lean_occt_topology_face_ranges(const LeanOcctTopology* the_topology);
LEAN_OCCT_CAPI_EXPORT const uint32_t* lean_occt_topology_face_wire_indices(
  const LeanOcctTopology* the_topology);
LEAN_OCCT_CAPI_EXPORT const uint8_t* lean_occt_topology_face_wire_orientations(
  const LeanOcctTopology* the_topology);
LEAN_OCCT_CAPI_EXPORT const uint8_t* lean_occt_topology_face_wire_roles(
  const LeanOcctTopology* the_topology);

LEAN_OCCT_CAPI_EXPORT LeanOcctMesh* lean_occt_shape_mesh(LeanOcctContext*          the_context,
                                                         const LeanOcctShape*      the_shape,
                                                         const LeanOcctMeshParams* the_params);
LEAN_OCCT_CAPI_EXPORT void lean_occt_mesh_destroy(LeanOcctMesh* the_mesh);
LEAN_OCCT_CAPI_EXPORT size_t lean_occt_mesh_vertex_count(const LeanOcctMesh* the_mesh);
LEAN_OCCT_CAPI_EXPORT size_t lean_occt_mesh_triangle_count(const LeanOcctMesh* the_mesh);
LEAN_OCCT_CAPI_EXPORT size_t lean_occt_mesh_edge_segment_count(const LeanOcctMesh* the_mesh);
LEAN_OCCT_CAPI_EXPORT size_t lean_occt_mesh_face_count(const LeanOcctMesh* the_mesh);
LEAN_OCCT_CAPI_EXPORT size_t lean_occt_mesh_solid_count(const LeanOcctMesh* the_mesh);
LEAN_OCCT_CAPI_EXPORT const double* lean_occt_mesh_positions(const LeanOcctMesh* the_mesh);
LEAN_OCCT_CAPI_EXPORT const double* lean_occt_mesh_normals(const LeanOcctMesh* the_mesh);
LEAN_OCCT_CAPI_EXPORT const uint32_t* lean_occt_mesh_triangle_indices(const LeanOcctMesh* the_mesh);
LEAN_OCCT_CAPI_EXPORT const double* lean_occt_mesh_edge_positions(const LeanOcctMesh* the_mesh);
LEAN_OCCT_CAPI_EXPORT void lean_occt_mesh_bounds(const LeanOcctMesh* the_mesh,
                                                 double*             the_min_xyz3,
                                                 double*             the_max_xyz3);

#ifdef __cplusplus
}
#endif

#endif
