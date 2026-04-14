use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

use crate::{
    BoxParams, ConeParams, CylinderParams, EdgeSelector, EllipseEdgeParams, Error, FaceSelector,
    FeatureSpec, HelixParams, ModelDocument, OffsetParams, PrismParams, RevolutionParams,
    ShapeReport, ShapeSummary, SphereParams, ThroughHoleCut, TorusParams,
};

static PIPELINE_INSTANCE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FeatureId(String);

impl FeatureId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for FeatureId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FeaturePersistentData {
    pub output_shape_name: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeatureBuildSource {
    Rebuilt,
    CachedImport,
}

#[derive(Clone, Debug, Default)]
pub struct FeatureRuntimeState {
    pub last_output_summary: Option<ShapeSummary>,
    pub last_error: Option<String>,
    pub last_build_source: Option<FeatureBuildSource>,
    pub cache_step_path: Option<PathBuf>,
    pub is_dirty: bool,
    pub last_build_revision: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FeatureOperation {
    AddBox {
        params: BoxParams,
    },
    AddCylinder {
        params: CylinderParams,
    },
    AddCone {
        params: ConeParams,
    },
    AddSphere {
        params: SphereParams,
    },
    AddTorus {
        params: TorusParams,
    },
    AddEllipseEdge {
        params: EllipseEdgeParams,
    },
    AddHelix {
        params: HelixParams,
    },
    BoxWithThroughHole {
        spec: ThroughHoleCut,
    },
    Cut {
        lhs: FeatureId,
        rhs: FeatureId,
    },
    Fuse {
        lhs: FeatureId,
        rhs: FeatureId,
    },
    Common {
        lhs: FeatureId,
        rhs: FeatureId,
    },
    Fillet {
        input: FeatureId,
        selector: EdgeSelector,
        radius: f64,
    },
    CylindricalHole {
        input: FeatureId,
        selector: FaceSelector,
        radius: f64,
    },
    Offset {
        input: FeatureId,
        params: OffsetParams,
    },
    Prism {
        input: FeatureId,
        params: PrismParams,
    },
    Revolution {
        input: FeatureId,
        params: RevolutionParams,
    },
    ImportStep {
        path: PathBuf,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeatureRecord {
    pub id: FeatureId,
    pub name: String,
    pub operation: FeatureOperation,
    #[serde(default)]
    pub persistent: FeaturePersistentData,
    #[serde(skip, default)]
    pub runtime: FeatureRuntimeState,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SerializableFeaturePipeline {
    next_id: u64,
    features: Vec<FeatureRecord>,
}

pub struct FeaturePipeline {
    features: Vec<FeatureRecord>,
    next_id: u64,
    build_revision: u64,
    dirty_from: Option<usize>,
    instance_id: u64,
}

impl FeaturePipeline {
    pub fn new() -> Self {
        Self {
            features: Vec::new(),
            next_id: 0,
            build_revision: 0,
            dirty_from: None,
            instance_id: next_pipeline_instance_id(),
        }
    }

    pub fn features(&self) -> &[FeatureRecord] {
        &self.features
    }

    pub fn feature(&self, id: &FeatureId) -> Result<&FeatureRecord, Error> {
        self.features
            .iter()
            .find(|feature| feature.id == *id)
            .ok_or_else(|| Error::new(format!("unknown feature '{id}'")))
    }

    pub fn dirty_feature_ids(&self) -> Vec<&FeatureId> {
        self.features
            .iter()
            .filter(|feature| feature.runtime.is_dirty)
            .map(|feature| &feature.id)
            .collect()
    }

    pub fn dirty_start_index(&self) -> Option<usize> {
        self.dirty_from
    }

    pub fn rename_feature(&mut self, id: &FeatureId, name: impl Into<String>) -> Result<(), Error> {
        let feature = self.feature_mut(id)?;
        feature.name = name.into();
        Ok(())
    }

    pub fn replace_feature_operation(
        &mut self,
        id: &FeatureId,
        operation: FeatureOperation,
    ) -> Result<(), Error> {
        let index = self.feature_index(id)?;
        self.features[index].operation = operation;
        self.mark_dirty_from_index(index);
        Ok(())
    }

    pub fn replace_feature_spec(&mut self, id: &FeatureId, spec: FeatureSpec) -> Result<(), Error> {
        let index = self.feature_index(id)?;
        let operation = spec.to_operation()?;
        if let Some(name) = spec.name {
            self.features[index].name = name;
        }
        self.features[index].operation = operation;
        self.mark_dirty_from_index(index);
        Ok(())
    }

    pub fn mark_feature_dirty(&mut self, id: &FeatureId) -> Result<(), Error> {
        let index = self.feature_index(id)?;
        self.mark_dirty_from_index(index);
        Ok(())
    }

    pub fn to_json_string(&self) -> Result<String, Error> {
        serde_json::to_string(&self.serializable())
            .map_err(|error| Error::new(format!("failed to serialize feature pipeline: {error}")))
    }

    pub fn to_json_string_pretty(&self) -> Result<String, Error> {
        serde_json::to_string_pretty(&self.serializable())
            .map_err(|error| Error::new(format!("failed to serialize feature pipeline: {error}")))
    }

    pub fn save_json(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        fs::write(path, self.to_json_string_pretty()?)
            .map_err(|error| Error::new(format!("failed to write feature pipeline JSON: {error}")))
    }

    pub fn from_json_str(json: &str) -> Result<Self, Error> {
        let serializable: SerializableFeaturePipeline =
            serde_json::from_str(json).map_err(|error| {
                Error::new(format!("failed to parse feature pipeline JSON: {error}"))
            })?;

        let mut pipeline = Self {
            features: serializable.features,
            next_id: serializable.next_id,
            build_revision: 0,
            dirty_from: None,
            instance_id: next_pipeline_instance_id(),
        };
        if !pipeline.features.is_empty() {
            pipeline.mark_dirty_from_index(0);
        }
        Ok(pipeline)
    }

    pub fn load_json(path: impl AsRef<Path>) -> Result<Self, Error> {
        let json = fs::read_to_string(path).map_err(|error| {
            Error::new(format!("failed to read feature pipeline JSON: {error}"))
        })?;
        Self::from_json_str(&json)
    }

    pub fn add_box(&mut self, name: impl Into<String>, params: BoxParams) -> FeatureId {
        self.push_feature(name, FeatureOperation::AddBox { params })
    }

    pub fn add_feature_spec(&mut self, spec: FeatureSpec) -> Result<FeatureId, Error> {
        let name = spec.resolved_name();
        let operation = spec.to_operation()?;
        Ok(self.push_feature(name, operation))
    }

    pub fn add_cylinder(&mut self, name: impl Into<String>, params: CylinderParams) -> FeatureId {
        self.push_feature(name, FeatureOperation::AddCylinder { params })
    }

    pub fn add_cone(&mut self, name: impl Into<String>, params: ConeParams) -> FeatureId {
        self.push_feature(name, FeatureOperation::AddCone { params })
    }

    pub fn add_sphere(&mut self, name: impl Into<String>, params: SphereParams) -> FeatureId {
        self.push_feature(name, FeatureOperation::AddSphere { params })
    }

    pub fn add_torus(&mut self, name: impl Into<String>, params: TorusParams) -> FeatureId {
        self.push_feature(name, FeatureOperation::AddTorus { params })
    }

    pub fn add_ellipse_edge(
        &mut self,
        name: impl Into<String>,
        params: EllipseEdgeParams,
    ) -> FeatureId {
        self.push_feature(name, FeatureOperation::AddEllipseEdge { params })
    }

    pub fn add_helix(&mut self, name: impl Into<String>, params: HelixParams) -> FeatureId {
        self.push_feature(name, FeatureOperation::AddHelix { params })
    }

    pub fn add_box_with_through_hole(
        &mut self,
        name: impl Into<String>,
        spec: ThroughHoleCut,
    ) -> FeatureId {
        self.push_feature(name, FeatureOperation::BoxWithThroughHole { spec })
    }

    pub fn add_cut(
        &mut self,
        name: impl Into<String>,
        lhs: &FeatureId,
        rhs: &FeatureId,
    ) -> FeatureId {
        self.push_feature(
            name,
            FeatureOperation::Cut {
                lhs: lhs.clone(),
                rhs: rhs.clone(),
            },
        )
    }

    pub fn add_fuse(
        &mut self,
        name: impl Into<String>,
        lhs: &FeatureId,
        rhs: &FeatureId,
    ) -> FeatureId {
        self.push_feature(
            name,
            FeatureOperation::Fuse {
                lhs: lhs.clone(),
                rhs: rhs.clone(),
            },
        )
    }

    pub fn add_common(
        &mut self,
        name: impl Into<String>,
        lhs: &FeatureId,
        rhs: &FeatureId,
    ) -> FeatureId {
        self.push_feature(
            name,
            FeatureOperation::Common {
                lhs: lhs.clone(),
                rhs: rhs.clone(),
            },
        )
    }

    pub fn add_fillet(
        &mut self,
        name: impl Into<String>,
        input: &FeatureId,
        selector: EdgeSelector,
        radius: f64,
    ) -> FeatureId {
        self.push_feature(
            name,
            FeatureOperation::Fillet {
                input: input.clone(),
                selector,
                radius,
            },
        )
    }

    pub fn add_cylindrical_hole(
        &mut self,
        name: impl Into<String>,
        input: &FeatureId,
        selector: FaceSelector,
        radius: f64,
    ) -> FeatureId {
        self.push_feature(
            name,
            FeatureOperation::CylindricalHole {
                input: input.clone(),
                selector,
                radius,
            },
        )
    }

    pub fn add_offset(
        &mut self,
        name: impl Into<String>,
        input: &FeatureId,
        params: OffsetParams,
    ) -> FeatureId {
        self.push_feature(
            name,
            FeatureOperation::Offset {
                input: input.clone(),
                params,
            },
        )
    }

    pub fn add_prism(
        &mut self,
        name: impl Into<String>,
        input: &FeatureId,
        params: PrismParams,
    ) -> FeatureId {
        self.push_feature(
            name,
            FeatureOperation::Prism {
                input: input.clone(),
                params,
            },
        )
    }

    pub fn add_revolution(
        &mut self,
        name: impl Into<String>,
        input: &FeatureId,
        params: RevolutionParams,
    ) -> FeatureId {
        self.push_feature(
            name,
            FeatureOperation::Revolution {
                input: input.clone(),
                params,
            },
        )
    }

    pub fn import_step(&mut self, name: impl Into<String>, path: impl AsRef<Path>) -> FeatureId {
        self.push_feature(
            name,
            FeatureOperation::ImportStep {
                path: path.as_ref().to_path_buf(),
            },
        )
    }

    pub fn rebuild(&mut self) -> Result<FeaturePipelineBuild, Error> {
        self.build_revision += 1;
        let mut document = ModelDocument::new()?;
        let mut output_shape_names = BTreeMap::new();
        let mut rebuild_from = self.initial_rebuild_index();

        for index in 0..self.features.len() {
            let feature = &mut self.features[index];
            let output_name = feature_shape_name(&feature.id);

            let used_cache = if index < rebuild_from {
                feature
                    .runtime
                    .cache_step_path
                    .as_ref()
                    .filter(|path| path.is_file())
                    .and_then(|path| document.import_step(&output_name, path).ok())
                    .is_some()
            } else {
                false
            };

            if !used_cache {
                rebuild_from = rebuild_from.min(index);
                if let Err(error) = run_feature_operation(
                    &mut document,
                    &output_shape_names,
                    &output_name,
                    &feature.operation,
                ) {
                    feature.runtime.last_error = Some(error.to_string());
                    feature.runtime.last_build_source = None;
                    feature.runtime.last_build_revision = None;
                    return Err(error);
                }

                let cache_path = cache_path_for_feature(self.instance_id, &feature.id)?;
                write_document_shape_step(&document, &output_name, &cache_path)?;
                feature.runtime.cache_step_path = Some(cache_path);
                feature.runtime.last_build_source = Some(FeatureBuildSource::Rebuilt);
            } else {
                feature.runtime.last_build_source = Some(FeatureBuildSource::CachedImport);
            }

            let summary = document.summary(&output_name)?;
            feature.persistent.output_shape_name = Some(output_name.clone());
            feature.runtime.last_output_summary = Some(summary);
            feature.runtime.last_error = None;
            feature.runtime.is_dirty = false;
            feature.runtime.last_build_revision = Some(self.build_revision);
            output_shape_names.insert(feature.id.clone(), output_name);
        }

        self.dirty_from = None;

        Ok(FeaturePipelineBuild {
            document,
            output_shape_names,
        })
    }

    fn serializable(&self) -> SerializableFeaturePipeline {
        SerializableFeaturePipeline {
            next_id: self.next_id,
            features: self.features.clone(),
        }
    }

    fn push_feature(&mut self, name: impl Into<String>, operation: FeatureOperation) -> FeatureId {
        self.next_id += 1;
        let id = FeatureId(format!("feature_{:04}", self.next_id));
        self.features.push(FeatureRecord {
            id: id.clone(),
            name: name.into(),
            operation,
            persistent: FeaturePersistentData::default(),
            runtime: FeatureRuntimeState {
                is_dirty: true,
                ..FeatureRuntimeState::default()
            },
        });
        self.mark_dirty_from_index(self.features.len() - 1);
        id
    }

    fn feature_index(&self, id: &FeatureId) -> Result<usize, Error> {
        self.features
            .iter()
            .position(|feature| feature.id == *id)
            .ok_or_else(|| Error::new(format!("unknown feature '{id}'")))
    }

    fn feature_mut(&mut self, id: &FeatureId) -> Result<&mut FeatureRecord, Error> {
        let index = self.feature_index(id)?;
        Ok(&mut self.features[index])
    }

    fn mark_dirty_from_index(&mut self, index: usize) {
        if index >= self.features.len() {
            return;
        }
        self.dirty_from = Some(self.dirty_from.map_or(index, |current| current.min(index)));
        for feature in &mut self.features[index..] {
            feature.runtime.is_dirty = true;
            feature.runtime.last_build_source = None;
            feature.runtime.last_build_revision = None;
        }
    }

    fn initial_rebuild_index(&self) -> usize {
        let mut rebuild_from = self.dirty_from.unwrap_or(self.features.len());
        for (index, feature) in self.features.iter().enumerate().take(rebuild_from) {
            if !feature
                .runtime
                .cache_step_path
                .as_ref()
                .is_some_and(|path| path.is_file())
            {
                rebuild_from = index;
                break;
            }
        }
        rebuild_from
    }
}

pub struct FeaturePipelineBuild {
    document: ModelDocument,
    output_shape_names: BTreeMap<FeatureId, String>,
}

impl FeaturePipelineBuild {
    pub fn document(&self) -> &ModelDocument {
        &self.document
    }

    pub fn document_mut(&mut self) -> &mut ModelDocument {
        &mut self.document
    }

    pub fn shape_name(&self, feature_id: &FeatureId) -> Result<&str, Error> {
        self.output_shape_names
            .get(feature_id)
            .map(|name| name.as_str())
            .ok_or_else(|| Error::new(format!("feature '{feature_id}' has no built output")))
    }

    pub fn summary(&self, feature_id: &FeatureId) -> Result<ShapeSummary, Error> {
        self.document.summary(self.shape_name(feature_id)?)
    }

    pub fn report(&self, feature_id: &FeatureId) -> Result<ShapeReport, Error> {
        self.document.report(self.shape_name(feature_id)?)
    }

    pub fn export_step(
        &mut self,
        feature_id: &FeatureId,
        path: impl AsRef<Path>,
    ) -> Result<(), Error> {
        let shape_name = self.shape_name(feature_id)?.to_owned();
        self.document.export_step(shape_name, path)
    }
}

fn next_pipeline_instance_id() -> u64 {
    PIPELINE_INSTANCE_COUNTER.fetch_add(1, Ordering::Relaxed) + 1
}

fn feature_shape_name(id: &FeatureId) -> String {
    format!("feature__{}", id.as_str())
}

fn resolve_output_name<'a>(
    outputs: &'a BTreeMap<FeatureId, String>,
    feature_id: &FeatureId,
) -> Result<&'a str, Error> {
    outputs
        .get(feature_id)
        .map(|name| name.as_str())
        .ok_or_else(|| Error::new(format!("feature '{feature_id}' has no upstream output")))
}

fn cache_path_for_feature(instance_id: u64, feature_id: &FeatureId) -> Result<PathBuf, Error> {
    let cache_dir = std::env::temp_dir()
        .join("lean_occt-pipeline-cache")
        .join(format!("pipeline_{instance_id:016x}"));
    fs::create_dir_all(&cache_dir).map_err(|error| {
        Error::new(format!(
            "failed to create pipeline cache directory: {error}"
        ))
    })?;
    Ok(cache_dir.join(format!("{}.step", feature_id.as_str())))
}

fn write_document_shape_step(
    document: &ModelDocument,
    shape_name: &str,
    path: &Path,
) -> Result<(), Error> {
    let shape = document.shape(shape_name)?;
    document.kernel().write_step(shape, path)
}

fn run_feature_operation(
    document: &mut ModelDocument,
    output_shape_names: &BTreeMap<FeatureId, String>,
    output_name: &str,
    operation: &FeatureOperation,
) -> Result<(), Error> {
    match operation {
        FeatureOperation::AddBox { params } => document.insert_box(output_name, *params),
        FeatureOperation::AddCylinder { params } => document.insert_cylinder(output_name, *params),
        FeatureOperation::AddCone { params } => document.insert_cone(output_name, *params),
        FeatureOperation::AddSphere { params } => document.insert_sphere(output_name, *params),
        FeatureOperation::AddTorus { params } => document.insert_torus(output_name, *params),
        FeatureOperation::AddEllipseEdge { params } => {
            document.insert_ellipse_edge(output_name, *params)
        }
        FeatureOperation::AddHelix { params } => document.insert_helix(output_name, *params),
        FeatureOperation::BoxWithThroughHole { spec } => {
            document.box_with_through_hole(output_name, *spec)
        }
        FeatureOperation::Cut { lhs, rhs } => document.cut(
            output_name,
            resolve_output_name(output_shape_names, lhs)?,
            resolve_output_name(output_shape_names, rhs)?,
        ),
        FeatureOperation::Fuse { lhs, rhs } => document.fuse(
            output_name,
            resolve_output_name(output_shape_names, lhs)?,
            resolve_output_name(output_shape_names, rhs)?,
        ),
        FeatureOperation::Common { lhs, rhs } => document.common(
            output_name,
            resolve_output_name(output_shape_names, lhs)?,
            resolve_output_name(output_shape_names, rhs)?,
        ),
        FeatureOperation::Fillet {
            input,
            selector,
            radius,
        } => document
            .fillet_selected_edge(
                output_name,
                resolve_output_name(output_shape_names, input)?,
                *selector,
                *radius,
            )
            .map(|_| ()),
        FeatureOperation::CylindricalHole {
            input,
            selector,
            radius,
        } => document
            .cylindrical_hole_on_selected_face(
                output_name,
                resolve_output_name(output_shape_names, input)?,
                *selector,
                *radius,
            )
            .map(|_| ()),
        FeatureOperation::Offset { input, params } => document.offset(
            output_name,
            resolve_output_name(output_shape_names, input)?,
            *params,
        ),
        FeatureOperation::Prism { input, params } => document.prism(
            output_name,
            resolve_output_name(output_shape_names, input)?,
            *params,
        ),
        FeatureOperation::Revolution { input, params } => document.revolution(
            output_name,
            resolve_output_name(output_shape_names, input)?,
            *params,
        ),
        FeatureOperation::ImportStep { path } => document.import_step(output_name, path),
    }
}
