#include <lean_occt_capi.h>

#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <filesystem>
#include <iostream>
#include <stdexcept>
#include <string>

namespace
{
static void require(bool the_condition, const LeanOcctContext* the_context, const char* the_message)
{
  if (!the_condition)
  {
    const char* an_error = lean_occt_context_last_error(the_context);
    throw std::runtime_error(std::string(the_message) + ": " + (an_error == nullptr ? "<null>" : an_error));
  }
}

static void removeIfPresent(const std::string& the_path)
{
  std::remove(the_path.c_str());
}

static std::string artifactPath(const char* the_file_name)
{
#ifdef LEAN_OCCT_TEST_ARTIFACTS_DIR
  const std::filesystem::path a_base_dir(LEAN_OCCT_TEST_ARTIFACTS_DIR);
#else
  const std::filesystem::path a_base_dir =
    std::filesystem::current_path() / "test-artifacts" / "ctest";
#endif
  std::filesystem::create_directories(a_base_dir);
  return (a_base_dir / the_file_name).string();
}

static double squaredDistance3(const double* the_lhs, const double* the_rhs)
{
  const double a_dx = the_lhs[0] - the_rhs[0];
  const double a_dy = the_lhs[1] - the_rhs[1];
  const double a_dz = the_lhs[2] - the_rhs[2];
  return a_dx * a_dx + a_dy * a_dy + a_dz * a_dz;
}

static double vectorLength3(const double* the_xyz)
{
  return std::sqrt(the_xyz[0] * the_xyz[0] + the_xyz[1] * the_xyz[1] + the_xyz[2] * the_xyz[2]);
}

static double dotProduct3(const double* the_lhs, const double* the_rhs)
{
  return the_lhs[0] * the_rhs[0] + the_lhs[1] * the_rhs[1] + the_lhs[2] * the_rhs[2];
}
} // namespace

int main()
{
  const std::string a_step_path = artifactPath("LeanOcctCapiSmoke.step");

  LeanOcctContext* a_context = lean_occt_context_create();
  if (a_context == nullptr)
  {
    std::cerr << "LeanOcctCapiSmoke failed: unable to create context\n";
    return 1;
  }

  try
  {
    removeIfPresent(a_step_path);

    const LeanOcctBoxParams a_box_params = {-30.0, -30.0, -30.0, 60.0, 60.0, 60.0};
    const LeanOcctCylinderParams a_cylinder_params = {0.0, 0.0, -36.0, 0.0, 0.0, 1.0, 12.0, 72.0};
    const LeanOcctBoxParams a_fillet_box_params = {0.0, 0.0, 0.0, 40.0, 30.0, 20.0};
    const LeanOcctFilletParams a_fillet_params = {3.0, 0};
    const LeanOcctBoxParams a_offset_box_params = {0.0, 0.0, 0.0, 30.0, 30.0, 30.0};
    const LeanOcctOffsetParams a_offset_params = {2.0, 1.0e-4};
    const LeanOcctBoxParams a_feature_box_params = {0.0, 0.0, 0.0, 40.0, 40.0, 30.0};
    const LeanOcctCylindricalHoleParams a_hole_params = {20.0, 20.0, -10.0, 0.0, 0.0, 1.0, 6.0};
    const LeanOcctHelixParams a_helix_params = {
      0.0, 0.0, 0.0,
      0.0, 0.0, 1.0,
      1.0, 0.0, 0.0,
      20.0, 30.0, 10.0};
    const LeanOcctMeshParams a_mesh_params = {0.9, 0.35, 0};
    LeanOcctShapeSummary a_cut_summary = {};
    LeanOcctShapeSummary a_helix_summary = {};
    LeanOcctShapeSummary a_step_summary = {};
    LeanOcctShapeSummary a_face_summary = {};
    LeanOcctShapeSummary a_wire_summary = {};
    LeanOcctShapeSummary an_edge_summary = {};
    size_t a_cut_edge_count = 0;
    size_t a_cut_vertex_count = 0;
    size_t a_edge_vertex_count = 0;
    size_t a_cut_face_count = 0;
    size_t a_face_wire_count = 0;
    size_t a_wire_edge_count = 0;
    double a_start_xyz[3] = {0.0, 0.0, 0.0};
    double an_end_xyz[3] = {0.0, 0.0, 0.0};
    double a_vertex_xyz[3] = {0.0, 0.0, 0.0};
    LeanOcctEdgeSample a_edge_start_sample = {};
    LeanOcctEdgeSample a_edge_mid_sample = {};
    LeanOcctEdgeSample a_edge_end_sample = {};
    LeanOcctEdgeSample a_edge_mid_parameter_sample = {};
    LeanOcctFaceUvBounds a_face_uv_bounds = {};
    LeanOcctFaceSample a_face_sample = {};
    LeanOcctFaceSample a_face_normalized_sample = {};
    LeanOcctFaceSample a_cylinder_face_sample = {};
    LeanOcctEdgeGeometry a_edge_geometry = {};
    LeanOcctFaceGeometry a_face_geometry = {};
    LeanOcctLinePayload a_line_payload = {};
    LeanOcctPlanePayload a_plane_payload = {};
    LeanOcctCirclePayload a_circle_payload = {};
    LeanOcctCylinderPayload a_cylinder_payload = {};
    bool foundCircleEdge = false;
    bool foundCylinderFace = false;

    LeanOcctShape* a_box = lean_occt_shape_make_box(a_context, &a_box_params);
    require(a_box != nullptr, a_context, "box creation failed");

    LeanOcctShape* a_cylinder = lean_occt_shape_make_cylinder(a_context, &a_cylinder_params);
    require(a_cylinder != nullptr, a_context, "cylinder creation failed");

    LeanOcctShape* a_cut = lean_occt_shape_boolean_cut(a_context, a_box, a_cylinder);
    require(a_cut != nullptr, a_context, "boolean cut failed");

    LeanOcctShape* a_fuse = lean_occt_shape_boolean_fuse(a_context, a_box, a_cylinder);
    require(a_fuse != nullptr, a_context, "boolean fuse failed");

    LeanOcctShape* a_common = lean_occt_shape_boolean_common(a_context, a_box, a_cylinder);
    require(a_common != nullptr, a_context, "boolean common failed");

    LeanOcctShape* a_fillet_source = lean_occt_shape_make_box(a_context, &a_fillet_box_params);
    require(a_fillet_source != nullptr, a_context, "fillet source creation failed");

    LeanOcctShape* a_fillet = lean_occt_shape_make_fillet(a_context, a_fillet_source, &a_fillet_params);
    require(a_fillet != nullptr, a_context, "fillet creation failed");

    LeanOcctShape* a_offset_source = lean_occt_shape_make_box(a_context, &a_offset_box_params);
    require(a_offset_source != nullptr, a_context, "offset source creation failed");

    LeanOcctShape* a_offset = lean_occt_shape_make_offset(a_context, a_offset_source, &a_offset_params);
    require(a_offset != nullptr, a_context, "offset creation failed");

    LeanOcctShape* a_feature_source = lean_occt_shape_make_box(a_context, &a_feature_box_params);
    require(a_feature_source != nullptr, a_context, "feature source creation failed");

    LeanOcctShape* a_hole =
      lean_occt_shape_make_cylindrical_hole(a_context, a_feature_source, &a_hole_params);
    require(a_hole != nullptr, a_context, "cylindrical hole feature creation failed");

    LeanOcctShape* a_helix = lean_occt_shape_make_helix(a_context, &a_helix_params);
    require(a_helix != nullptr, a_context, "helix creation failed");

    LeanOcctMesh* a_cut_mesh = lean_occt_shape_mesh(a_context, a_cut, &a_mesh_params);
    require(a_cut_mesh != nullptr, a_context, "cut meshing failed");

    LeanOcctMesh* a_fillet_mesh = lean_occt_shape_mesh(a_context, a_fillet, &a_mesh_params);
    require(a_fillet_mesh != nullptr, a_context, "fillet meshing failed");

    LeanOcctMesh* a_offset_mesh = lean_occt_shape_mesh(a_context, a_offset, &a_mesh_params);
    require(a_offset_mesh != nullptr, a_context, "offset meshing failed");

    LeanOcctMesh* a_hole_mesh = lean_occt_shape_mesh(a_context, a_hole, &a_mesh_params);
    require(a_hole_mesh != nullptr, a_context, "feature meshing failed");

    require(lean_occt_mesh_solid_count(a_cut_mesh) == 1, a_context, "unexpected solid count");
    require(lean_occt_mesh_face_count(a_cut_mesh) > 0, a_context, "unexpected face count");
    require(lean_occt_mesh_triangle_count(a_cut_mesh) > 0, a_context, "unexpected triangle count");
    require(lean_occt_mesh_edge_segment_count(a_cut_mesh) > 0, a_context, "unexpected edge count");

    require(lean_occt_mesh_solid_count(a_fillet_mesh) == 1, a_context, "unexpected fillet solid count");
    require(lean_occt_mesh_triangle_count(a_fillet_mesh) > 0, a_context, "unexpected fillet triangle count");

    require(lean_occt_mesh_solid_count(a_offset_mesh) == 1, a_context, "unexpected offset solid count");
    require(lean_occt_mesh_triangle_count(a_offset_mesh) > 0, a_context, "unexpected offset triangle count");

    require(lean_occt_mesh_solid_count(a_hole_mesh) == 1, a_context, "unexpected feature solid count");
    require(lean_occt_mesh_triangle_count(a_hole_mesh) > 0, a_context, "unexpected feature triangle count");

    require(lean_occt_shape_edge_count(a_helix) > 0, a_context, "unexpected helix edge count");
    require(lean_occt_shape_linear_length(a_helix) > 0.0, a_context, "unexpected helix length");
    require(lean_occt_shape_describe(a_context, a_cut, &a_cut_summary) == LEAN_OCCT_RESULT_OK,
            a_context,
            "cut summary failed");
    require(lean_occt_shape_describe(a_context, a_helix, &a_helix_summary) == LEAN_OCCT_RESULT_OK,
            a_context,
            "helix summary failed");

    require(a_cut_summary.primary_kind == LEAN_OCCT_SHAPE_KIND_SOLID,
            a_context,
            "unexpected cut primary kind");
    require(a_cut_summary.solid_count == 1, a_context, "unexpected cut summary solid count");
    require(a_cut_summary.face_count == 7, a_context, "unexpected cut summary face count");
    require(a_cut_summary.volume > 0.0, a_context, "unexpected cut summary volume");
    require(a_helix_summary.primary_kind == LEAN_OCCT_SHAPE_KIND_WIRE,
            a_context,
            "unexpected helix primary kind");
    require(a_helix_summary.wire_count == 1, a_context, "unexpected helix wire count");
    require(a_helix_summary.edge_count == 3, a_context, "unexpected helix summary edge count");
    require(a_helix_summary.linear_length > 0.0, a_context, "unexpected helix summary length");
    require(lean_occt_shape_subshape_count(a_context, a_cut, LEAN_OCCT_SHAPE_KIND_FACE, &a_cut_face_count)
              == LEAN_OCCT_RESULT_OK,
            a_context,
            "cut face traversal count failed");
    require(a_cut_face_count == a_cut_summary.face_count, a_context, "cut face traversal count mismatch");
    require(lean_occt_shape_subshape_count(a_context, a_cut, LEAN_OCCT_SHAPE_KIND_EDGE, &a_cut_edge_count)
              == LEAN_OCCT_RESULT_OK,
            a_context,
            "cut edge traversal count failed");
    require(lean_occt_shape_subshape_count(a_context, a_cut, LEAN_OCCT_SHAPE_KIND_VERTEX, &a_cut_vertex_count)
              == LEAN_OCCT_RESULT_OK,
            a_context,
            "cut vertex traversal count failed");

    LeanOcctShape* a_first_face =
      lean_occt_shape_subshape(a_context, a_cut, LEAN_OCCT_SHAPE_KIND_FACE, 0);
    require(a_first_face != nullptr, a_context, "cut first-face traversal failed");
    require(lean_occt_shape_describe(a_context, a_first_face, &a_face_summary) == LEAN_OCCT_RESULT_OK,
            a_context,
            "face summary failed");
    require(a_face_summary.primary_kind == LEAN_OCCT_SHAPE_KIND_FACE, a_context, "unexpected face primary kind");
    require(lean_occt_shape_subshape_count(a_context, a_first_face, LEAN_OCCT_SHAPE_KIND_WIRE, &a_face_wire_count)
              == LEAN_OCCT_RESULT_OK,
            a_context,
            "face wire traversal count failed");
    require(a_face_wire_count >= 1, a_context, "unexpected face wire count");

    LeanOcctShape* a_first_wire =
      lean_occt_shape_subshape(a_context, a_first_face, LEAN_OCCT_SHAPE_KIND_WIRE, 0);
    require(a_first_wire != nullptr, a_context, "face first-wire traversal failed");
    require(lean_occt_shape_describe(a_context, a_first_wire, &a_wire_summary) == LEAN_OCCT_RESULT_OK,
            a_context,
            "wire summary failed");
    require(a_wire_summary.primary_kind == LEAN_OCCT_SHAPE_KIND_WIRE, a_context, "unexpected wire primary kind");
    require(lean_occt_shape_subshape_count(a_context, a_first_wire, LEAN_OCCT_SHAPE_KIND_EDGE, &a_wire_edge_count)
              == LEAN_OCCT_RESULT_OK,
            a_context,
            "wire edge traversal count failed");
    require(a_wire_edge_count >= 1, a_context, "unexpected wire edge count");

    LeanOcctShape* a_first_edge =
      lean_occt_shape_subshape(a_context, a_first_wire, LEAN_OCCT_SHAPE_KIND_EDGE, 0);
    require(a_first_edge != nullptr, a_context, "wire first-edge traversal failed");
    require(lean_occt_shape_describe(a_context, a_first_edge, &an_edge_summary) == LEAN_OCCT_RESULT_OK,
            a_context,
            "edge summary failed");
    require(an_edge_summary.primary_kind == LEAN_OCCT_SHAPE_KIND_EDGE, a_context, "unexpected edge primary kind");
    require(an_edge_summary.linear_length > 0.0, a_context, "unexpected edge linear length");
    require(lean_occt_shape_subshape_count(a_context,
                                           a_first_edge,
                                           LEAN_OCCT_SHAPE_KIND_VERTEX,
                                           &a_edge_vertex_count) == LEAN_OCCT_RESULT_OK,
            a_context,
            "edge vertex traversal count failed");
    require(a_edge_vertex_count == 2, a_context, "unexpected edge vertex count");
    LeanOcctShape* a_first_vertex =
      lean_occt_shape_subshape(a_context, a_first_edge, LEAN_OCCT_SHAPE_KIND_VERTEX, 0);
    require(a_first_vertex != nullptr, a_context, "edge first-vertex traversal failed");
    require(lean_occt_shape_vertex_point(a_context, a_first_vertex, a_vertex_xyz) == LEAN_OCCT_RESULT_OK,
            a_context,
            "vertex point query failed");
    require(lean_occt_shape_edge_endpoints(a_context, a_first_edge, a_start_xyz, an_end_xyz)
              == LEAN_OCCT_RESULT_OK,
            a_context,
            "edge endpoint query failed");
    require(lean_occt_shape_edge_geometry(a_context, a_first_edge, &a_edge_geometry)
              == LEAN_OCCT_RESULT_OK,
            a_context,
            "edge geometry query failed");
    require(lean_occt_shape_edge_line_payload(a_context, a_first_edge, &a_line_payload)
              == LEAN_OCCT_RESULT_OK,
            a_context,
            "edge line payload query failed");
    require(lean_occt_shape_edge_sample(a_context, a_first_edge, 0.0, &a_edge_start_sample)
              == LEAN_OCCT_RESULT_OK,
            a_context,
            "edge start sample query failed");
    require(lean_occt_shape_edge_sample(a_context, a_first_edge, 0.5, &a_edge_mid_sample)
              == LEAN_OCCT_RESULT_OK,
            a_context,
            "edge midpoint sample query failed");
    require(lean_occt_shape_edge_sample(a_context, a_first_edge, 1.0, &a_edge_end_sample)
              == LEAN_OCCT_RESULT_OK,
            a_context,
            "edge end sample query failed");
    require(lean_occt_shape_edge_sample_at_parameter(
              a_context,
              a_first_edge,
              0.5 * (a_edge_geometry.start_parameter + a_edge_geometry.end_parameter),
              &a_edge_mid_parameter_sample) == LEAN_OCCT_RESULT_OK,
            a_context,
            "edge midpoint parameter sample query failed");
    require(a_start_xyz[0] != an_end_xyz[0] || a_start_xyz[1] != an_end_xyz[1]
              || a_start_xyz[2] != an_end_xyz[2],
            a_context,
            "edge endpoints collapsed to the same point");
    require(a_edge_geometry.kind == LEAN_OCCT_CURVE_KIND_LINE,
            a_context,
            "unexpected first-edge curve kind");
    require(a_edge_geometry.is_periodic == 0, a_context, "unexpected first-edge periodicity");
    require(std::abs(vectorLength3(a_line_payload.direction) - 1.0) <= 1.0e-12,
            a_context,
            "line payload direction was not unit length");
    require(squaredDistance3(a_edge_start_sample.position, a_start_xyz) <= 1.0e-18,
            a_context,
            "edge start sample did not match the oriented edge start");
    require(squaredDistance3(a_edge_end_sample.position, an_end_xyz) <= 1.0e-18,
            a_context,
            "edge end sample did not match the oriented edge end");
    require(squaredDistance3(a_edge_mid_parameter_sample.position, a_edge_mid_sample.position) <= 1.0e-18,
            a_context,
            "edge parameter sample did not match normalized midpoint sample");
    require(std::abs(vectorLength3(a_edge_mid_sample.tangent) - 1.0) <= 1.0e-12,
            a_context,
            "edge midpoint tangent was not unit length");
    require(std::abs(std::abs(dotProduct3(a_line_payload.direction, a_edge_mid_sample.tangent)) - 1.0)
              <= 1.0e-12,
            a_context,
            "line payload direction was not aligned with the sampled edge tangent");
    require(lean_occt_shape_face_geometry(a_context, a_first_face, &a_face_geometry)
              == LEAN_OCCT_RESULT_OK,
            a_context,
            "face geometry query failed");
    require(lean_occt_shape_face_plane_payload(a_context, a_first_face, &a_plane_payload)
              == LEAN_OCCT_RESULT_OK,
            a_context,
            "face plane payload query failed");
    require(lean_occt_shape_face_uv_bounds(a_context, a_first_face, &a_face_uv_bounds)
              == LEAN_OCCT_RESULT_OK,
            a_context,
            "face UV bounds query failed");
    require(a_face_uv_bounds.u_min <= a_face_uv_bounds.u_max && a_face_uv_bounds.v_min <= a_face_uv_bounds.v_max,
            a_context,
            "face UV bounds were inverted");
    require(a_face_geometry.kind == LEAN_OCCT_SURFACE_KIND_PLANE,
            a_context,
            "unexpected first-face surface kind");
    const double a_mid_u = 0.5 * (a_face_uv_bounds.u_min + a_face_uv_bounds.u_max);
    const double a_mid_v = 0.5 * (a_face_uv_bounds.v_min + a_face_uv_bounds.v_max);
    require(lean_occt_shape_face_sample(a_context, a_first_face, a_mid_u, a_mid_v, &a_face_sample)
              == LEAN_OCCT_RESULT_OK,
            a_context,
            "face sample query failed");
    require(lean_occt_shape_face_sample_normalized(a_context, a_first_face, 0.5, 0.5, &a_face_normalized_sample)
              == LEAN_OCCT_RESULT_OK,
            a_context,
            "normalized face sample query failed");
    require(std::isfinite(a_face_sample.position[0]) && std::isfinite(a_face_sample.position[1])
              && std::isfinite(a_face_sample.position[2]) && std::isfinite(a_face_sample.normal[0])
              && std::isfinite(a_face_sample.normal[1]) && std::isfinite(a_face_sample.normal[2]),
            a_context,
            "face sample returned non-finite values");
    require(squaredDistance3(a_face_normalized_sample.position, a_face_sample.position) <= 1.0e-18,
            a_context,
            "normalized face sample did not match midpoint UV sample");
    require(std::abs(vectorLength3(a_face_sample.normal) - 1.0) <= 1.0e-12,
            a_context,
            "face sample normal was not unit length");
    require(std::abs(vectorLength3(a_plane_payload.normal) - 1.0) <= 1.0e-12,
            a_context,
            "plane payload normal was not unit length");
    require(std::abs(vectorLength3(a_plane_payload.x_direction) - 1.0) <= 1.0e-12
              && std::abs(vectorLength3(a_plane_payload.y_direction) - 1.0) <= 1.0e-12,
            a_context,
            "plane payload axes were not unit length");
    require(std::abs(std::abs(dotProduct3(a_plane_payload.normal, a_face_sample.normal)) - 1.0)
              <= 1.0e-12,
            a_context,
            "plane payload normal was not aligned with the sampled face normal");

    LeanOcctTopology* a_topology = lean_occt_shape_topology(a_context, a_cut);
    require(a_topology != nullptr, a_context, "topology snapshot failed");
    require(lean_occt_topology_face_count(a_topology) == a_cut_summary.face_count,
            a_context,
            "topology face count mismatch");
    require(lean_occt_topology_edge_count(a_topology) == a_cut_edge_count,
            a_context,
            "topology edge count mismatch");
    require(lean_occt_topology_vertex_count(a_topology) == a_cut_vertex_count,
            a_context,
            "topology vertex count mismatch");

    const uint32_t* a_face_ranges = lean_occt_topology_face_ranges(a_topology);
    const uint32_t* a_face_wire_indices = lean_occt_topology_face_wire_indices(a_topology);
    const uint8_t*  a_face_wire_orientations = lean_occt_topology_face_wire_orientations(a_topology);
    require(a_face_ranges != nullptr && a_face_wire_indices != nullptr && a_face_wire_orientations != nullptr,
            a_context,
            "topology face buffers were null");
    require(a_face_ranges[1] == a_face_wire_count, a_context, "topology first-face wire count mismatch");

    const uint32_t a_first_wire_index = a_face_wire_indices[a_face_ranges[0]];
    const uint32_t* a_wire_ranges = lean_occt_topology_wire_ranges(a_topology);
    const uint32_t* a_wire_edge_indices = lean_occt_topology_wire_edge_indices(a_topology);
    const uint8_t*  a_wire_edge_orientations = lean_occt_topology_wire_edge_orientations(a_topology);
    require(a_wire_ranges != nullptr && a_wire_edge_indices != nullptr && a_wire_edge_orientations != nullptr,
            a_context,
            "topology wire buffers were null");
    require(a_wire_ranges[a_first_wire_index * 2 + 1] == a_wire_edge_count,
            a_context,
            "topology first-wire edge count mismatch");

    const uint32_t a_first_topology_edge_index = a_wire_edge_indices[a_wire_ranges[a_first_wire_index * 2]];
    const uint32_t* a_edge_vertices = lean_occt_topology_edge_vertex_indices(a_topology);
    const double*   a_edge_lengths = lean_occt_topology_edge_lengths(a_topology);
    const double*   a_vertex_positions = lean_occt_topology_vertex_positions(a_topology);
    const uint32_t* a_edge_face_ranges = lean_occt_topology_edge_face_ranges(a_topology);
    const uint32_t* a_edge_face_indices = lean_occt_topology_edge_face_indices(a_topology);
    const uint32_t* a_wire_vertex_ranges = lean_occt_topology_wire_vertex_ranges(a_topology);
    const uint32_t* a_wire_vertex_indices = lean_occt_topology_wire_vertex_indices(a_topology);
    const uint8_t*  a_face_wire_roles = lean_occt_topology_face_wire_roles(a_topology);
    require(a_edge_vertices != nullptr && a_edge_lengths != nullptr && a_vertex_positions != nullptr,
            a_context,
            "topology edge or vertex buffers were null");
    require(a_edge_face_ranges != nullptr && a_edge_face_indices != nullptr && a_wire_vertex_ranges != nullptr
              && a_wire_vertex_indices != nullptr && a_face_wire_roles != nullptr,
            a_context,
            "topology adjacency buffers were null");
    require(a_edge_lengths[a_first_topology_edge_index] > 0.0, a_context, "unexpected topology edge length");
    require(a_wire_edge_orientations[a_wire_ranges[a_first_wire_index * 2]] == LEAN_OCCT_ORIENTATION_FORWARD
              || a_wire_edge_orientations[a_wire_ranges[a_first_wire_index * 2]]
                   == LEAN_OCCT_ORIENTATION_REVERSED,
            a_context,
            "unexpected topology first-wire edge orientation");

    const uint32_t a_first_wire_vertex_offset = a_wire_vertex_ranges[a_first_wire_index * 2];
    const uint32_t a_first_wire_vertex_count = a_wire_vertex_ranges[a_first_wire_index * 2 + 1];
    require(a_first_wire_vertex_count == a_wire_edge_count + 1,
            a_context,
            "topology first-wire vertex count mismatch");
    require(a_wire_vertex_indices[a_first_wire_vertex_offset]
              == a_wire_vertex_indices[a_first_wire_vertex_offset + a_first_wire_vertex_count - 1],
            a_context,
            "topology first-wire was not closed");

    const uint32_t a_first_edge_start = a_edge_vertices[a_first_topology_edge_index * 2];
    const uint32_t a_first_edge_end = a_edge_vertices[a_first_topology_edge_index * 2 + 1];
    if (a_wire_edge_orientations[a_wire_ranges[a_first_wire_index * 2]] == LEAN_OCCT_ORIENTATION_REVERSED)
    {
      require(a_wire_vertex_indices[a_first_wire_vertex_offset] == a_first_edge_end
                && a_wire_vertex_indices[a_first_wire_vertex_offset + 1] == a_first_edge_start,
              a_context,
              "topology first-edge reversed vertex chain mismatch");
    }
    else
    {
      require(a_wire_vertex_indices[a_first_wire_vertex_offset] == a_first_edge_start
                && a_wire_vertex_indices[a_first_wire_vertex_offset + 1] == a_first_edge_end,
              a_context,
              "topology first-edge forward vertex chain mismatch");
    }

    for (size_t a_face_idx = 0; a_face_idx < lean_occt_topology_face_count(a_topology); ++a_face_idx)
    {
      const uint32_t an_offset = a_face_ranges[a_face_idx * 2];
      const uint32_t a_count = a_face_ranges[a_face_idx * 2 + 1];
      uint32_t       an_outer_count = 0;
      for (uint32_t an_item_idx = 0; an_item_idx < a_count; ++an_item_idx)
      {
        if (a_face_wire_roles[an_offset + an_item_idx] == LEAN_OCCT_LOOP_ROLE_OUTER)
        {
          ++an_outer_count;
        }
      }
      require(an_outer_count == 1, a_context, "topology face loop roles were inconsistent");
    }

    bool hasNeighborFace = false;
    for (uint32_t a_face_wire_item = a_face_ranges[0];
         a_face_wire_item < a_face_ranges[0] + a_face_ranges[1];
         ++a_face_wire_item)
    {
      const uint32_t a_wire_index = a_face_wire_indices[a_face_wire_item];
      const uint32_t a_wire_edge_offset = a_wire_ranges[a_wire_index * 2];
      const uint32_t a_wire_edge_count_local = a_wire_ranges[a_wire_index * 2 + 1];
      for (uint32_t a_wire_edge_item = a_wire_edge_offset;
           a_wire_edge_item < a_wire_edge_offset + a_wire_edge_count_local;
           ++a_wire_edge_item)
      {
        const uint32_t a_topology_edge_index = a_wire_edge_indices[a_wire_edge_item];
        const uint32_t an_edge_face_offset = a_edge_face_ranges[a_topology_edge_index * 2];
        const uint32_t an_edge_face_count = a_edge_face_ranges[a_topology_edge_index * 2 + 1];
        for (uint32_t an_edge_face_item = an_edge_face_offset;
             an_edge_face_item < an_edge_face_offset + an_edge_face_count;
             ++an_edge_face_item)
        {
          if (a_edge_face_indices[an_edge_face_item] != 0)
          {
            hasNeighborFace = true;
          }
        }
      }
    }
    require(hasNeighborFace, a_context, "topology face neighbor derivation failed");

    for (size_t an_edge_idx = 0; an_edge_idx < a_cut_edge_count; ++an_edge_idx)
    {
      LeanOcctShape* an_edge_shape =
        lean_occt_shape_subshape(a_context, a_cut, LEAN_OCCT_SHAPE_KIND_EDGE, an_edge_idx);
      require(an_edge_shape != nullptr, a_context, "cut edge traversal for geometry scan failed");
      LeanOcctEdgeGeometry an_edge_geometry = {};
      require(lean_occt_shape_edge_geometry(a_context, an_edge_shape, &an_edge_geometry)
                == LEAN_OCCT_RESULT_OK,
              a_context,
              "cut edge geometry scan failed");
      if (an_edge_geometry.kind == LEAN_OCCT_CURVE_KIND_CIRCLE)
      {
        require(lean_occt_shape_edge_circle_payload(a_context, an_edge_shape, &a_circle_payload)
                  == LEAN_OCCT_RESULT_OK,
                a_context,
                "circular edge payload query failed");
        foundCircleEdge = true;
      }
      lean_occt_shape_destroy(an_edge_shape);
    }
    require(foundCircleEdge, a_context, "cut geometry scan did not find a circular edge");
    require(std::abs(a_circle_payload.radius - 12.0) <= 1.0e-12,
            a_context,
            "unexpected circular edge radius");
    require(std::abs(vectorLength3(a_circle_payload.normal) - 1.0) <= 1.0e-12,
            a_context,
            "circular edge normal was not unit length");

    for (size_t a_face_idx = 0; a_face_idx < a_cut_face_count; ++a_face_idx)
    {
      LeanOcctShape* a_face_shape =
        lean_occt_shape_subshape(a_context, a_cut, LEAN_OCCT_SHAPE_KIND_FACE, a_face_idx);
      require(a_face_shape != nullptr, a_context, "cut face traversal for geometry scan failed");
      LeanOcctFaceGeometry a_face_geometry_scan = {};
      require(lean_occt_shape_face_geometry(a_context, a_face_shape, &a_face_geometry_scan)
                == LEAN_OCCT_RESULT_OK,
              a_context,
              "cut face geometry scan failed");
      if (a_face_geometry_scan.kind == LEAN_OCCT_SURFACE_KIND_CYLINDER)
      {
        require(lean_occt_shape_face_sample_normalized(
                  a_context, a_face_shape, 0.5, 0.5, &a_cylinder_face_sample) == LEAN_OCCT_RESULT_OK,
                a_context,
                "cylindrical face sample query failed");
        require(lean_occt_shape_face_cylinder_payload(a_context, a_face_shape, &a_cylinder_payload)
                  == LEAN_OCCT_RESULT_OK,
                a_context,
                "cylindrical face payload query failed");
        foundCylinderFace = true;
      }
      lean_occt_shape_destroy(a_face_shape);
    }
    require(foundCylinderFace, a_context, "cut geometry scan did not find a cylindrical face");
    require(std::abs(vectorLength3(a_cylinder_face_sample.normal) - 1.0) <= 1.0e-12,
            a_context,
            "cylindrical face normal was not unit length");
    require(std::abs(a_cylinder_payload.radius - 12.0) <= 1.0e-12,
            a_context,
            "unexpected cylindrical face radius");
    require(std::abs(vectorLength3(a_cylinder_payload.axis) - 1.0) <= 1.0e-12,
            a_context,
            "cylindrical face axis was not unit length");
    require(std::abs(dotProduct3(a_cylinder_payload.axis, a_cylinder_face_sample.normal)) <= 1.0e-12,
            a_context,
            "cylindrical face axis was not orthogonal to the sampled face normal");

    {
      const LeanOcctEllipseEdgeParams an_ellipse_params = {
        30.0, 0.0, 0.0,
        0.0, 1.0, 0.0,
        1.0, 0.0, 0.0,
        10.0, 6.0};
      const LeanOcctPrismParams a_prism_params = {0.0, 24.0, 0.0};
      const LeanOcctRevolutionParams a_revolution_params = {
        0.0, 0.0, 0.0,
        0.0, 0.0, 1.0,
        2.0 * 3.14159265358979323846};
      const LeanOcctOffsetParams an_offset_surface_params = {2.5, 1.0e-4};
      const LeanOcctConeParams a_cone_params = {
        0.0, 0.0, 0.0,
        0.0, 0.0, 1.0,
        1.0, 0.0, 0.0,
        15.0, 5.0, 30.0};
      const LeanOcctSphereParams a_sphere_params = {
        0.0, 0.0, 0.0,
        0.0, 0.0, 1.0,
        1.0, 0.0, 0.0,
        14.0};
      const LeanOcctTorusParams a_torus_params = {
        0.0, 0.0, 0.0,
        0.0, 0.0, 1.0,
        1.0, 0.0, 0.0,
        25.0, 6.0};
      const double an_expected_cone_angle = std::atan((15.0 - 5.0) / 30.0);

      LeanOcctShape* an_ellipse_edge = lean_occt_shape_make_ellipse_edge(a_context, &an_ellipse_params);
      require(an_ellipse_edge != nullptr, a_context, "ellipse edge creation failed");
      LeanOcctEdgeGeometry an_ellipse_geometry = {};
      LeanOcctEllipsePayload an_ellipse_payload = {};
      require(lean_occt_shape_edge_geometry(a_context, an_ellipse_edge, &an_ellipse_geometry)
                == LEAN_OCCT_RESULT_OK,
              a_context,
              "ellipse edge geometry query failed");
      require(an_ellipse_geometry.kind == LEAN_OCCT_CURVE_KIND_ELLIPSE,
              a_context,
              "ellipse edge geometry kind mismatch");
      require(lean_occt_shape_edge_ellipse_payload(a_context, an_ellipse_edge, &an_ellipse_payload)
                == LEAN_OCCT_RESULT_OK,
              a_context,
              "ellipse edge payload query failed");
      require(std::abs(an_ellipse_payload.major_radius - 10.0) <= 1.0e-12,
              a_context,
              "ellipse major radius mismatch");
      require(std::abs(an_ellipse_payload.minor_radius - 6.0) <= 1.0e-12,
              a_context,
              "ellipse minor radius mismatch");

      LeanOcctShape* a_prism = lean_occt_shape_make_prism(a_context, an_ellipse_edge, &a_prism_params);
      require(a_prism != nullptr, a_context, "prism creation failed");
      LeanOcctExtrusionSurfacePayload an_extrusion_payload = {};
      bool foundExtrusionFace = false;
      size_t a_prism_face_count = 0;
      require(lean_occt_shape_subshape_count(a_context, a_prism, LEAN_OCCT_SHAPE_KIND_FACE, &a_prism_face_count)
                == LEAN_OCCT_RESULT_OK,
              a_context,
              "prism face traversal count failed");
      for (size_t a_face_idx = 0; a_face_idx < a_prism_face_count; ++a_face_idx)
      {
        LeanOcctShape* a_face_shape =
          lean_occt_shape_subshape(a_context, a_prism, LEAN_OCCT_SHAPE_KIND_FACE, a_face_idx);
        require(a_face_shape != nullptr, a_context, "prism face traversal failed");
        LeanOcctFaceGeometry a_face_geometry_scan = {};
        require(lean_occt_shape_face_geometry(a_context, a_face_shape, &a_face_geometry_scan)
                  == LEAN_OCCT_RESULT_OK,
                a_context,
                "prism face geometry query failed");
        if (a_face_geometry_scan.kind == LEAN_OCCT_SURFACE_KIND_EXTRUSION)
        {
          require(lean_occt_shape_face_extrusion_payload(a_context, a_face_shape, &an_extrusion_payload)
                    == LEAN_OCCT_RESULT_OK,
                  a_context,
                  "extrusion face payload query failed");
          foundExtrusionFace = true;
        }
        lean_occt_shape_destroy(a_face_shape);
        if (foundExtrusionFace)
        {
          break;
        }
      }
      require(foundExtrusionFace, a_context, "expected extrusion face was not found");
      require(an_extrusion_payload.basis_curve_kind == LEAN_OCCT_CURVE_KIND_ELLIPSE,
              a_context,
              "extrusion basis curve kind mismatch");
      require(std::abs(vectorLength3(an_extrusion_payload.direction) - 1.0) <= 1.0e-12,
              a_context,
              "extrusion direction was not unit length");

      LeanOcctShape* a_revolution =
        lean_occt_shape_make_revolution(a_context, an_ellipse_edge, &a_revolution_params);
      require(a_revolution != nullptr, a_context, "revolution creation failed");
      LeanOcctShape* a_revolution_face = nullptr;
      LeanOcctRevolutionSurfacePayload a_revolution_payload = {};
      size_t a_revolution_face_count = 0;
      require(lean_occt_shape_subshape_count(
                a_context, a_revolution, LEAN_OCCT_SHAPE_KIND_FACE, &a_revolution_face_count)
                == LEAN_OCCT_RESULT_OK,
              a_context,
              "revolution face traversal count failed");
      for (size_t a_face_idx = 0; a_face_idx < a_revolution_face_count; ++a_face_idx)
      {
        LeanOcctShape* a_face_shape =
          lean_occt_shape_subshape(a_context, a_revolution, LEAN_OCCT_SHAPE_KIND_FACE, a_face_idx);
        require(a_face_shape != nullptr, a_context, "revolution face traversal failed");
        LeanOcctFaceGeometry a_face_geometry_scan = {};
        require(lean_occt_shape_face_geometry(a_context, a_face_shape, &a_face_geometry_scan)
                  == LEAN_OCCT_RESULT_OK,
                a_context,
                "revolution face geometry query failed");
        if (a_face_geometry_scan.kind == LEAN_OCCT_SURFACE_KIND_REVOLUTION)
        {
          require(lean_occt_shape_face_revolution_payload(a_context, a_face_shape, &a_revolution_payload)
                    == LEAN_OCCT_RESULT_OK,
                  a_context,
                  "revolution face payload query failed");
          a_revolution_face = a_face_shape;
          a_face_shape = nullptr;
        }
        if (a_face_shape != nullptr)
        {
          lean_occt_shape_destroy(a_face_shape);
        }
        if (a_revolution_face != nullptr)
        {
          break;
        }
      }
      require(a_revolution_face != nullptr, a_context, "expected revolution face was not found");
      require(a_revolution_payload.basis_curve_kind == LEAN_OCCT_CURVE_KIND_ELLIPSE,
              a_context,
              "revolution basis curve kind mismatch");
      require(std::abs(vectorLength3(a_revolution_payload.axis_direction) - 1.0) <= 1.0e-12,
              a_context,
              "revolution axis was not unit length");

      LeanOcctShape* an_offset_surface =
        lean_occt_shape_make_offset(a_context, a_revolution_face, &an_offset_surface_params);
      require(an_offset_surface != nullptr, a_context, "offset surface creation failed");
      LeanOcctOffsetSurfacePayload an_offset_surface_payload = {};
      bool foundOffsetFace = false;
      size_t an_offset_face_count = 0;
      require(lean_occt_shape_subshape_count(
                a_context, an_offset_surface, LEAN_OCCT_SHAPE_KIND_FACE, &an_offset_face_count)
                == LEAN_OCCT_RESULT_OK,
              a_context,
              "offset surface face traversal count failed");
      for (size_t a_face_idx = 0; a_face_idx < an_offset_face_count; ++a_face_idx)
      {
        LeanOcctShape* a_face_shape =
          lean_occt_shape_subshape(a_context, an_offset_surface, LEAN_OCCT_SHAPE_KIND_FACE, a_face_idx);
        require(a_face_shape != nullptr, a_context, "offset surface face traversal failed");
        LeanOcctFaceGeometry a_face_geometry_scan = {};
        require(lean_occt_shape_face_geometry(a_context, a_face_shape, &a_face_geometry_scan)
                  == LEAN_OCCT_RESULT_OK,
                a_context,
                "offset surface face geometry query failed");
        if (a_face_geometry_scan.kind == LEAN_OCCT_SURFACE_KIND_OFFSET)
        {
          require(lean_occt_shape_face_offset_payload(
                    a_context, a_face_shape, &an_offset_surface_payload) == LEAN_OCCT_RESULT_OK,
                  a_context,
                  "offset surface payload query failed");
          foundOffsetFace = true;
        }
        lean_occt_shape_destroy(a_face_shape);
        if (foundOffsetFace)
        {
          break;
        }
      }
      require(foundOffsetFace, a_context, "expected offset face was not found");
      require(an_offset_surface_payload.basis_surface_kind == LEAN_OCCT_SURFACE_KIND_REVOLUTION,
              a_context,
              "offset surface basis kind mismatch");
      require(std::abs(std::abs(an_offset_surface_payload.offset_value) - 2.5) <= 1.0e-12,
              a_context,
              "offset surface distance mismatch");

      LeanOcctShape* a_cone = lean_occt_shape_make_cone(a_context, &a_cone_params);
      require(a_cone != nullptr, a_context, "cone creation failed");
      LeanOcctConePayload a_cone_payload = {};
      bool foundConeFace = false;
      size_t a_cone_face_count = 0;
      require(lean_occt_shape_subshape_count(a_context, a_cone, LEAN_OCCT_SHAPE_KIND_FACE, &a_cone_face_count)
                == LEAN_OCCT_RESULT_OK,
              a_context,
              "cone face traversal count failed");
      for (size_t a_face_idx = 0; a_face_idx < a_cone_face_count; ++a_face_idx)
      {
        LeanOcctShape* a_face_shape =
          lean_occt_shape_subshape(a_context, a_cone, LEAN_OCCT_SHAPE_KIND_FACE, a_face_idx);
        require(a_face_shape != nullptr, a_context, "cone face traversal failed");
        LeanOcctFaceGeometry a_face_geometry_scan = {};
        require(lean_occt_shape_face_geometry(a_context, a_face_shape, &a_face_geometry_scan)
                  == LEAN_OCCT_RESULT_OK,
                a_context,
                "cone face geometry query failed");
        if (a_face_geometry_scan.kind == LEAN_OCCT_SURFACE_KIND_CONE)
        {
          require(lean_occt_shape_face_cone_payload(a_context, a_face_shape, &a_cone_payload)
                    == LEAN_OCCT_RESULT_OK,
                  a_context,
                  "cone payload query failed");
          foundConeFace = true;
        }
        lean_occt_shape_destroy(a_face_shape);
        if (foundConeFace)
        {
          break;
        }
      }
      require(foundConeFace, a_context, "expected conical face was not found");
      require(std::abs(a_cone_payload.reference_radius - 15.0) <= 1.0e-12,
              a_context,
              "cone reference radius mismatch");
      require(std::abs(std::abs(a_cone_payload.semi_angle) - an_expected_cone_angle) <= 1.0e-12,
              a_context,
              "cone semi-angle mismatch");

      LeanOcctShape* a_sphere = lean_occt_shape_make_sphere(a_context, &a_sphere_params);
      require(a_sphere != nullptr, a_context, "sphere creation failed");
      LeanOcctSpherePayload a_sphere_payload = {};
      bool foundSphereFace = false;
      size_t a_sphere_face_count = 0;
      require(lean_occt_shape_subshape_count(
                a_context, a_sphere, LEAN_OCCT_SHAPE_KIND_FACE, &a_sphere_face_count)
                == LEAN_OCCT_RESULT_OK,
              a_context,
              "sphere face traversal count failed");
      for (size_t a_face_idx = 0; a_face_idx < a_sphere_face_count; ++a_face_idx)
      {
        LeanOcctShape* a_face_shape =
          lean_occt_shape_subshape(a_context, a_sphere, LEAN_OCCT_SHAPE_KIND_FACE, a_face_idx);
        require(a_face_shape != nullptr, a_context, "sphere face traversal failed");
        LeanOcctFaceGeometry a_face_geometry_scan = {};
        require(lean_occt_shape_face_geometry(a_context, a_face_shape, &a_face_geometry_scan)
                  == LEAN_OCCT_RESULT_OK,
                a_context,
                "sphere face geometry query failed");
        if (a_face_geometry_scan.kind == LEAN_OCCT_SURFACE_KIND_SPHERE)
        {
          require(lean_occt_shape_face_sphere_payload(a_context, a_face_shape, &a_sphere_payload)
                    == LEAN_OCCT_RESULT_OK,
                  a_context,
                  "sphere payload query failed");
          foundSphereFace = true;
        }
        lean_occt_shape_destroy(a_face_shape);
        if (foundSphereFace)
        {
          break;
        }
      }
      require(foundSphereFace, a_context, "expected spherical face was not found");
      require(std::abs(a_sphere_payload.radius - 14.0) <= 1.0e-12,
              a_context,
              "sphere radius mismatch");

      LeanOcctShape* a_torus = lean_occt_shape_make_torus(a_context, &a_torus_params);
      require(a_torus != nullptr, a_context, "torus creation failed");
      LeanOcctTorusPayload a_torus_payload = {};
      bool foundTorusFace = false;
      size_t a_torus_face_count = 0;
      require(lean_occt_shape_subshape_count(a_context, a_torus, LEAN_OCCT_SHAPE_KIND_FACE, &a_torus_face_count)
                == LEAN_OCCT_RESULT_OK,
              a_context,
              "torus face traversal count failed");
      for (size_t a_face_idx = 0; a_face_idx < a_torus_face_count; ++a_face_idx)
      {
        LeanOcctShape* a_face_shape =
          lean_occt_shape_subshape(a_context, a_torus, LEAN_OCCT_SHAPE_KIND_FACE, a_face_idx);
        require(a_face_shape != nullptr, a_context, "torus face traversal failed");
        LeanOcctFaceGeometry a_face_geometry_scan = {};
        require(lean_occt_shape_face_geometry(a_context, a_face_shape, &a_face_geometry_scan)
                  == LEAN_OCCT_RESULT_OK,
                a_context,
                "torus face geometry query failed");
        if (a_face_geometry_scan.kind == LEAN_OCCT_SURFACE_KIND_TORUS)
        {
          require(lean_occt_shape_face_torus_payload(a_context, a_face_shape, &a_torus_payload)
                    == LEAN_OCCT_RESULT_OK,
                  a_context,
                  "torus payload query failed");
          foundTorusFace = true;
        }
        lean_occt_shape_destroy(a_face_shape);
        if (foundTorusFace)
        {
          break;
        }
      }
      require(foundTorusFace, a_context, "expected toroidal face was not found");
      require(std::abs(a_torus_payload.major_radius - 25.0) <= 1.0e-12,
              a_context,
              "torus major radius mismatch");
      require(std::abs(a_torus_payload.minor_radius - 6.0) <= 1.0e-12,
              a_context,
              "torus minor radius mismatch");

      std::cout << "analytic ellipse=(" << an_ellipse_payload.major_radius << ","
                << an_ellipse_payload.minor_radius << ")"
                << " extrusionBasis=" << static_cast<int>(an_extrusion_payload.basis_curve_kind)
                << " revolutionBasis=" << static_cast<int>(a_revolution_payload.basis_curve_kind)
                << " offsetBasis=" << static_cast<int>(an_offset_surface_payload.basis_surface_kind)
                << " cone=(" << a_cone_payload.reference_radius << "," << a_cone_payload.semi_angle
                << ")"
                << " sphere=" << a_sphere_payload.radius
                << " torus=(" << a_torus_payload.major_radius << "," << a_torus_payload.minor_radius
                << ")\n";

      lean_occt_shape_destroy(an_ellipse_edge);
      lean_occt_shape_destroy(a_prism);
      lean_occt_shape_destroy(a_revolution);
      lean_occt_shape_destroy(a_revolution_face);
      lean_occt_shape_destroy(an_offset_surface);
      lean_occt_shape_destroy(a_cone);
      lean_occt_shape_destroy(a_sphere);
      lean_occt_shape_destroy(a_torus);
    }

    require(lean_occt_shape_write_step(a_context, a_cut, a_step_path.c_str()) == LEAN_OCCT_RESULT_OK,
            a_context,
            "STEP write failed");

    LeanOcctShape* a_from_step = lean_occt_shape_read_step(a_context, a_step_path.c_str());
    require(a_from_step != nullptr, a_context, "STEP read failed");

    LeanOcctMesh* a_step_mesh = lean_occt_shape_mesh(a_context, a_from_step, &a_mesh_params);
    require(a_step_mesh != nullptr, a_context, "STEP mesh failed");
    require(lean_occt_shape_describe(a_context, a_from_step, &a_step_summary) == LEAN_OCCT_RESULT_OK,
            a_context,
            "STEP summary failed");
    require(a_step_summary.primary_kind == LEAN_OCCT_SHAPE_KIND_SOLID,
            a_context,
            "unexpected STEP primary kind");
    require(a_step_summary.solid_count == 1, a_context, "unexpected STEP summary solid count");

    std::cout << "LeanOcctCapiSmoke OK\n"
              << "cut    solids=" << lean_occt_mesh_solid_count(a_cut_mesh)
              << " faces=" << lean_occt_mesh_face_count(a_cut_mesh)
              << " triangles=" << lean_occt_mesh_triangle_count(a_cut_mesh)
              << " edgeSegments=" << lean_occt_mesh_edge_segment_count(a_cut_mesh)
              << " rootKind=" << static_cast<int>(a_cut_summary.root_kind)
              << " primaryKind=" << static_cast<int>(a_cut_summary.primary_kind)
              << " volume=" << a_cut_summary.volume
              << " firstFaceWires=" << a_face_wire_count
              << " firstWireEdges=" << a_wire_edge_count
              << " firstEdgeKind=" << static_cast<int>(a_edge_geometry.kind)
              << " firstFaceKind=" << static_cast<int>(a_face_geometry.kind)
              << " lineDir=(" << a_line_payload.direction[0] << "," << a_line_payload.direction[1]
              << "," << a_line_payload.direction[2] << ")"
              << " circleRadius=" << a_circle_payload.radius
              << " firstEdgeStart=(" << a_start_xyz[0] << "," << a_start_xyz[1] << "," << a_start_xyz[2]
              << ")"
              << " midTangent=(" << a_edge_mid_sample.tangent[0] << "," << a_edge_mid_sample.tangent[1]
              << "," << a_edge_mid_sample.tangent[2] << ")"
              << " faceNormal=(" << a_face_sample.normal[0] << "," << a_face_sample.normal[1] << ","
              << a_face_sample.normal[2] << ")"
              << " cylinderRadius=" << a_cylinder_payload.radius
              << " cylinderAxis=(" << a_cylinder_payload.axis[0] << "," << a_cylinder_payload.axis[1]
              << "," << a_cylinder_payload.axis[2] << ")"
              << "\n"
              << "fillet solids=" << lean_occt_mesh_solid_count(a_fillet_mesh)
              << " faces=" << lean_occt_mesh_face_count(a_fillet_mesh)
              << " triangles=" << lean_occt_mesh_triangle_count(a_fillet_mesh)
              << " edgeSegments=" << lean_occt_mesh_edge_segment_count(a_fillet_mesh) << "\n"
              << "offset solids=" << lean_occt_mesh_solid_count(a_offset_mesh)
              << " faces=" << lean_occt_mesh_face_count(a_offset_mesh)
              << " triangles=" << lean_occt_mesh_triangle_count(a_offset_mesh)
              << " edgeSegments=" << lean_occt_mesh_edge_segment_count(a_offset_mesh) << "\n"
              << "feature solids=" << lean_occt_mesh_solid_count(a_hole_mesh)
              << " faces=" << lean_occt_mesh_face_count(a_hole_mesh)
              << " triangles=" << lean_occt_mesh_triangle_count(a_hole_mesh)
              << " edgeSegments=" << lean_occt_mesh_edge_segment_count(a_hole_mesh) << "\n"
              << "helix  kind=" << static_cast<int>(a_helix_summary.root_kind)
              << " primaryKind=" << static_cast<int>(a_helix_summary.primary_kind)
              << " wires=" << a_helix_summary.wire_count
              << " edges=" << a_helix_summary.edge_count
              << " length=" << a_helix_summary.linear_length << "\n"
              << "step   solids=" << lean_occt_mesh_solid_count(a_step_mesh)
              << " faces=" << lean_occt_mesh_face_count(a_step_mesh)
              << " triangles=" << lean_occt_mesh_triangle_count(a_step_mesh)
              << " edgeSegments=" << lean_occt_mesh_edge_segment_count(a_step_mesh)
              << " rootKind=" << static_cast<int>(a_step_summary.root_kind)
              << " primaryKind=" << static_cast<int>(a_step_summary.primary_kind) << "\n"
              << "artifact " << a_step_path << "\n";

    lean_occt_mesh_destroy(a_cut_mesh);
    lean_occt_mesh_destroy(a_fillet_mesh);
    lean_occt_mesh_destroy(a_offset_mesh);
    lean_occt_mesh_destroy(a_hole_mesh);
    lean_occt_mesh_destroy(a_step_mesh);
    lean_occt_topology_destroy(a_topology);
    lean_occt_shape_destroy(a_first_vertex);
    lean_occt_shape_destroy(a_first_edge);
    lean_occt_shape_destroy(a_first_wire);
    lean_occt_shape_destroy(a_first_face);
    lean_occt_shape_destroy(a_helix);
    lean_occt_shape_destroy(a_hole);
    lean_occt_shape_destroy(a_feature_source);
    lean_occt_shape_destroy(a_offset);
    lean_occt_shape_destroy(a_offset_source);
    lean_occt_shape_destroy(a_fillet);
    lean_occt_shape_destroy(a_fillet_source);
    lean_occt_shape_destroy(a_from_step);
    lean_occt_shape_destroy(a_common);
    lean_occt_shape_destroy(a_fuse);
    lean_occt_shape_destroy(a_cut);
    lean_occt_shape_destroy(a_cylinder);
    lean_occt_shape_destroy(a_box);
    lean_occt_context_destroy(a_context);
    return 0;
  }
  catch (const std::exception& the_error)
  {
    std::cerr << "LeanOcctCapiSmoke failed: " << the_error.what() << "\n";
    lean_occt_context_destroy(a_context);
    return 1;
  }
}
