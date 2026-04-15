use super::face_snapshot::load_ported_face_snapshot;
use super::root_topology::load_root_topology_snapshot;
use super::snapshot_build::build_ported_topology_snapshot;
use super::*;

pub(super) fn ported_topology_snapshot(
    context: &Context,
    shape: &Shape,
) -> Result<Option<TopologySnapshot>, Error> {
    let Some(root_topology) = load_root_topology_snapshot(context, shape)? else {
        return Ok(None);
    };
    let Some(face_topology) = load_ported_face_snapshot(
        context,
        shape,
        &root_topology.root_wires,
        &root_topology.root_edges,
        &root_topology.edge_shapes,
        &root_topology.vertex_positions,
        root_topology.edges.len(),
    )?
    else {
        return Ok(None);
    };

    Ok(Some(build_ported_topology_snapshot(
        root_topology,
        face_topology,
    )))
}
