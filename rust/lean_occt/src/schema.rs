use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::pipeline::{FeatureId, FeatureOperation};
use crate::{
    BoxParams, ConeParams, CurveKind, CylinderParams, EdgeSelector, EllipseEdgeParams, Error,
    FaceSelector, HelixParams, OffsetParams, PrismParams, RevolutionParams, SphereParams,
    ThroughHoleCut, TorusParams,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureType {
    AddBox,
    AddCylinder,
    AddCone,
    AddSphere,
    AddTorus,
    AddEllipseEdge,
    AddHelix,
    BoxWithThroughHole,
    Cut,
    Fuse,
    Common,
    Fillet,
    CylindricalHole,
    Offset,
    Prism,
    Revolution,
    ImportStep,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureParamKind {
    Number,
    Vec3,
    EdgeSelector,
    FaceSelector,
    Path,
    BoxParams,
    CylinderParams,
    ConeParams,
    SphereParams,
    TorusParams,
    EllipseEdgeParams,
    HelixParams,
    ThroughHoleCut,
    OffsetParams,
    PrismParams,
    RevolutionParams,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeatureInputDefinition {
    pub name: String,
    pub description: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FeatureParamDefinition {
    pub name: String,
    pub kind: FeatureParamKind,
    pub description: String,
    pub default_value: Value,
    pub required: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FeatureDefinition {
    pub feature_type: FeatureType,
    pub default_name: String,
    pub description: String,
    pub inputs: Vec<FeatureInputDefinition>,
    pub params: Vec<FeatureParamDefinition>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureSpec {
    pub feature_type: FeatureType,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub inputs: Vec<FeatureId>,
    #[serde(default = "empty_params_object")]
    pub params: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ImportStepParams {
    path: PathBuf,
}

impl FeatureType {
    pub fn all() -> &'static [FeatureType] {
        const ALL: &[FeatureType] = &[
            FeatureType::AddBox,
            FeatureType::AddCylinder,
            FeatureType::AddCone,
            FeatureType::AddSphere,
            FeatureType::AddTorus,
            FeatureType::AddEllipseEdge,
            FeatureType::AddHelix,
            FeatureType::BoxWithThroughHole,
            FeatureType::Cut,
            FeatureType::Fuse,
            FeatureType::Common,
            FeatureType::Fillet,
            FeatureType::CylindricalHole,
            FeatureType::Offset,
            FeatureType::Prism,
            FeatureType::Revolution,
            FeatureType::ImportStep,
        ];
        ALL
    }

    pub fn definition(self) -> FeatureDefinition {
        FeatureDefinition {
            feature_type: self,
            default_name: self.default_name().to_owned(),
            description: self.description().to_owned(),
            inputs: self.input_definitions(),
            params: self.param_definitions(),
        }
    }

    pub fn default_name(self) -> &'static str {
        match self {
            FeatureType::AddBox => "Box",
            FeatureType::AddCylinder => "Cylinder",
            FeatureType::AddCone => "Cone",
            FeatureType::AddSphere => "Sphere",
            FeatureType::AddTorus => "Torus",
            FeatureType::AddEllipseEdge => "Ellipse edge",
            FeatureType::AddHelix => "Helix",
            FeatureType::BoxWithThroughHole => "Box with hole",
            FeatureType::Cut => "Cut",
            FeatureType::Fuse => "Fuse",
            FeatureType::Common => "Common",
            FeatureType::Fillet => "Fillet",
            FeatureType::CylindricalHole => "Cylindrical hole",
            FeatureType::Offset => "Offset",
            FeatureType::Prism => "Prism",
            FeatureType::Revolution => "Revolution",
            FeatureType::ImportStep => "Imported STEP",
        }
    }

    fn description(self) -> &'static str {
        match self {
            FeatureType::AddBox => "Create a box primitive.",
            FeatureType::AddCylinder => "Create a cylinder primitive.",
            FeatureType::AddCone => "Create a cone or truncated cone primitive.",
            FeatureType::AddSphere => "Create a sphere primitive.",
            FeatureType::AddTorus => "Create a torus primitive.",
            FeatureType::AddEllipseEdge => "Create an analytic ellipse edge.",
            FeatureType::AddHelix => "Create a helix wire.",
            FeatureType::BoxWithThroughHole => {
                "Create a box and subtract a cylindrical through-hole."
            }
            FeatureType::Cut => "Subtract the right input from the left input.",
            FeatureType::Fuse => "Fuse two upstream shapes.",
            FeatureType::Common => "Intersect two upstream shapes.",
            FeatureType::Fillet => "Apply a fillet to a selected edge on the input shape.",
            FeatureType::CylindricalHole => {
                "Drill a cylindrical hole from a selected planar face on the input shape."
            }
            FeatureType::Offset => "Offset the input shape.",
            FeatureType::Prism => "Extrude the input shape along a direction vector.",
            FeatureType::Revolution => "Revolve the input shape around an axis.",
            FeatureType::ImportStep => "Import a STEP file into the feature history.",
        }
    }

    fn input_definitions(self) -> Vec<FeatureInputDefinition> {
        match self {
            FeatureType::Cut | FeatureType::Fuse | FeatureType::Common => vec![
                input_def("lhs", "Left-hand input shape."),
                input_def("rhs", "Right-hand input shape."),
            ],
            FeatureType::Fillet
            | FeatureType::CylindricalHole
            | FeatureType::Offset
            | FeatureType::Prism
            | FeatureType::Revolution => vec![input_def("input", "Upstream input shape.")],
            _ => Vec::new(),
        }
    }

    fn param_definitions(self) -> Vec<FeatureParamDefinition> {
        match self {
            FeatureType::AddBox => vec![
                param_def(
                    "origin",
                    FeatureParamKind::Vec3,
                    "Box corner origin.",
                    json!([0.0, 0.0, 0.0]),
                ),
                param_def(
                    "size",
                    FeatureParamKind::Vec3,
                    "Box dimensions.",
                    json!([10.0, 10.0, 10.0]),
                ),
            ],
            FeatureType::AddCylinder => vec![
                param_def(
                    "origin",
                    FeatureParamKind::Vec3,
                    "Cylinder base origin.",
                    json!([0.0, 0.0, 0.0]),
                ),
                param_def(
                    "axis",
                    FeatureParamKind::Vec3,
                    "Cylinder axis direction.",
                    json!([0.0, 0.0, 1.0]),
                ),
                param_def(
                    "radius",
                    FeatureParamKind::Number,
                    "Cylinder radius.",
                    json!(5.0),
                ),
                param_def(
                    "height",
                    FeatureParamKind::Number,
                    "Cylinder height.",
                    json!(10.0),
                ),
            ],
            FeatureType::AddCone => vec![
                param_def(
                    "origin",
                    FeatureParamKind::Vec3,
                    "Cone base origin.",
                    json!([0.0, 0.0, 0.0]),
                ),
                param_def(
                    "axis",
                    FeatureParamKind::Vec3,
                    "Cone axis direction.",
                    json!([0.0, 0.0, 1.0]),
                ),
                param_def(
                    "x_direction",
                    FeatureParamKind::Vec3,
                    "Cone local X direction.",
                    json!([1.0, 0.0, 0.0]),
                ),
                param_def(
                    "base_radius",
                    FeatureParamKind::Number,
                    "Cone base radius.",
                    json!(5.0),
                ),
                param_def(
                    "top_radius",
                    FeatureParamKind::Number,
                    "Cone top radius.",
                    json!(2.0),
                ),
                param_def(
                    "height",
                    FeatureParamKind::Number,
                    "Cone height.",
                    json!(12.0),
                ),
            ],
            FeatureType::AddSphere => vec![
                param_def(
                    "origin",
                    FeatureParamKind::Vec3,
                    "Sphere center.",
                    json!([0.0, 0.0, 0.0]),
                ),
                param_def(
                    "axis",
                    FeatureParamKind::Vec3,
                    "Sphere axis direction.",
                    json!([0.0, 0.0, 1.0]),
                ),
                param_def(
                    "x_direction",
                    FeatureParamKind::Vec3,
                    "Sphere local X direction.",
                    json!([1.0, 0.0, 0.0]),
                ),
                param_def(
                    "radius",
                    FeatureParamKind::Number,
                    "Sphere radius.",
                    json!(6.0),
                ),
            ],
            FeatureType::AddTorus => vec![
                param_def(
                    "origin",
                    FeatureParamKind::Vec3,
                    "Torus center.",
                    json!([0.0, 0.0, 0.0]),
                ),
                param_def(
                    "axis",
                    FeatureParamKind::Vec3,
                    "Torus axis direction.",
                    json!([0.0, 0.0, 1.0]),
                ),
                param_def(
                    "x_direction",
                    FeatureParamKind::Vec3,
                    "Torus local X direction.",
                    json!([1.0, 0.0, 0.0]),
                ),
                param_def(
                    "major_radius",
                    FeatureParamKind::Number,
                    "Torus major radius.",
                    json!(12.0),
                ),
                param_def(
                    "minor_radius",
                    FeatureParamKind::Number,
                    "Torus minor radius.",
                    json!(3.0),
                ),
            ],
            FeatureType::AddEllipseEdge => vec![
                param_def(
                    "origin",
                    FeatureParamKind::Vec3,
                    "Ellipse center.",
                    json!([0.0, 0.0, 0.0]),
                ),
                param_def(
                    "axis",
                    FeatureParamKind::Vec3,
                    "Ellipse normal direction.",
                    json!([0.0, 0.0, 1.0]),
                ),
                param_def(
                    "x_direction",
                    FeatureParamKind::Vec3,
                    "Ellipse local X direction.",
                    json!([1.0, 0.0, 0.0]),
                ),
                param_def(
                    "major_radius",
                    FeatureParamKind::Number,
                    "Ellipse major radius.",
                    json!(10.0),
                ),
                param_def(
                    "minor_radius",
                    FeatureParamKind::Number,
                    "Ellipse minor radius.",
                    json!(6.0),
                ),
            ],
            FeatureType::AddHelix => vec![
                param_def(
                    "origin",
                    FeatureParamKind::Vec3,
                    "Helix origin.",
                    json!([0.0, 0.0, 0.0]),
                ),
                param_def(
                    "axis",
                    FeatureParamKind::Vec3,
                    "Helix axis direction.",
                    json!([0.0, 0.0, 1.0]),
                ),
                param_def(
                    "x_direction",
                    FeatureParamKind::Vec3,
                    "Helix local X direction.",
                    json!([1.0, 0.0, 0.0]),
                ),
                param_def(
                    "radius",
                    FeatureParamKind::Number,
                    "Helix radius.",
                    json!(10.0),
                ),
                param_def(
                    "height",
                    FeatureParamKind::Number,
                    "Helix height.",
                    json!(30.0),
                ),
                param_def(
                    "pitch",
                    FeatureParamKind::Number,
                    "Helix pitch.",
                    json!(5.0),
                ),
            ],
            FeatureType::BoxWithThroughHole => vec![
                param_def(
                    "box_params",
                    FeatureParamKind::BoxParams,
                    "Base box parameters.",
                    json!({
                        "origin": [0.0, 0.0, 0.0],
                        "size": [10.0, 10.0, 10.0],
                    }),
                ),
                param_def(
                    "tool_params",
                    FeatureParamKind::CylinderParams,
                    "Cylindrical tool parameters.",
                    json!({
                        "origin": [5.0, 5.0, -5.0],
                        "axis": [0.0, 0.0, 1.0],
                        "radius": 2.0,
                        "height": 20.0,
                    }),
                ),
            ],
            FeatureType::Cut | FeatureType::Fuse | FeatureType::Common => Vec::new(),
            FeatureType::Fillet => vec![
                param_def(
                    "selector",
                    FeatureParamKind::EdgeSelector,
                    "Edge selector for the fillet target.",
                    serde_json::to_value(EdgeSelector::LongestByCurveKind(CurveKind::Line))
                        .expect("edge selector default must serialize"),
                ),
                param_def(
                    "radius",
                    FeatureParamKind::Number,
                    "Fillet radius.",
                    json!(1.0),
                ),
            ],
            FeatureType::CylindricalHole => vec![
                param_def(
                    "selector",
                    FeatureParamKind::FaceSelector,
                    "Face selector for the hole entry face.",
                    serde_json::to_value(FaceSelector::BestAlignedPlane {
                        normal_hint: [0.0, 0.0, 1.0],
                    })
                    .expect("face selector default must serialize"),
                ),
                param_def(
                    "radius",
                    FeatureParamKind::Number,
                    "Hole radius.",
                    json!(2.0),
                ),
            ],
            FeatureType::Offset => vec![
                param_def(
                    "offset",
                    FeatureParamKind::Number,
                    "Offset distance.",
                    json!(1.0),
                ),
                param_def(
                    "tolerance",
                    FeatureParamKind::Number,
                    "Offset tolerance.",
                    json!(1.0e-4),
                ),
            ],
            FeatureType::Prism => vec![param_def(
                "direction",
                FeatureParamKind::Vec3,
                "Extrusion direction vector.",
                json!([0.0, 0.0, 10.0]),
            )],
            FeatureType::Revolution => vec![
                param_def(
                    "origin",
                    FeatureParamKind::Vec3,
                    "Revolution axis origin.",
                    json!([0.0, 0.0, 0.0]),
                ),
                param_def(
                    "axis",
                    FeatureParamKind::Vec3,
                    "Revolution axis direction.",
                    json!([0.0, 0.0, 1.0]),
                ),
                param_def(
                    "angle_radians",
                    FeatureParamKind::Number,
                    "Revolution sweep angle in radians.",
                    json!(std::f64::consts::TAU),
                ),
            ],
            FeatureType::ImportStep => vec![param_def(
                "path",
                FeatureParamKind::Path,
                "STEP file path to import.",
                json!(""),
            )],
        }
    }

    pub fn default_params_json(self) -> Value {
        let mut params = Map::new();
        for param in self.param_definitions() {
            params.insert(param.name, param.default_value);
        }
        Value::Object(params)
    }

    fn validate_input_count(self, inputs: &[FeatureId]) -> Result<(), Error> {
        let expected = self.input_definitions();
        if inputs.len() == expected.len() {
            return Ok(());
        }
        let labels = if expected.is_empty() {
            "no inputs".to_owned()
        } else {
            expected
                .iter()
                .map(|input| input.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        };
        Err(Error::new(format!(
            "{} expects {} input(s) [{}], but received {}.",
            self.default_name(),
            expected.len(),
            labels,
            inputs.len(),
        )))
    }

    fn build_operation(
        self,
        inputs: &[FeatureId],
        params: Value,
    ) -> Result<FeatureOperation, Error> {
        self.validate_input_count(inputs)?;
        match self {
            FeatureType::AddBox => {
                let params = decode_params::<BoxParams>(self, params)?;
                validate_box(&params)?;
                Ok(FeatureOperation::AddBox { params })
            }
            FeatureType::AddCylinder => {
                let params = decode_params::<CylinderParams>(self, params)?;
                validate_cylinder(&params)?;
                Ok(FeatureOperation::AddCylinder { params })
            }
            FeatureType::AddCone => {
                let params = decode_params::<ConeParams>(self, params)?;
                validate_cone(&params)?;
                Ok(FeatureOperation::AddCone { params })
            }
            FeatureType::AddSphere => {
                let params = decode_params::<SphereParams>(self, params)?;
                validate_sphere(&params)?;
                Ok(FeatureOperation::AddSphere { params })
            }
            FeatureType::AddTorus => {
                let params = decode_params::<TorusParams>(self, params)?;
                validate_torus(&params)?;
                Ok(FeatureOperation::AddTorus { params })
            }
            FeatureType::AddEllipseEdge => {
                let params = decode_params::<EllipseEdgeParams>(self, params)?;
                validate_ellipse_edge(&params)?;
                Ok(FeatureOperation::AddEllipseEdge { params })
            }
            FeatureType::AddHelix => {
                let params = decode_params::<HelixParams>(self, params)?;
                validate_helix(&params)?;
                Ok(FeatureOperation::AddHelix { params })
            }
            FeatureType::BoxWithThroughHole => {
                let spec = decode_params::<ThroughHoleCut>(self, params)?;
                validate_box_with_through_hole(&spec)?;
                Ok(FeatureOperation::BoxWithThroughHole { spec })
            }
            FeatureType::Cut => Ok(FeatureOperation::Cut {
                lhs: inputs[0].clone(),
                rhs: inputs[1].clone(),
            }),
            FeatureType::Fuse => Ok(FeatureOperation::Fuse {
                lhs: inputs[0].clone(),
                rhs: inputs[1].clone(),
            }),
            FeatureType::Common => Ok(FeatureOperation::Common {
                lhs: inputs[0].clone(),
                rhs: inputs[1].clone(),
            }),
            FeatureType::Fillet => {
                let params = decode_params::<FilletSpecParams>(self, params)?;
                ensure_positive("radius", params.radius)?;
                Ok(FeatureOperation::Fillet {
                    input: inputs[0].clone(),
                    selector: params.selector,
                    radius: params.radius,
                })
            }
            FeatureType::CylindricalHole => {
                let params = decode_params::<CylindricalHoleSpecParams>(self, params)?;
                ensure_positive("radius", params.radius)?;
                Ok(FeatureOperation::CylindricalHole {
                    input: inputs[0].clone(),
                    selector: params.selector,
                    radius: params.radius,
                })
            }
            FeatureType::Offset => {
                let params = decode_params::<OffsetParams>(self, params)?;
                ensure_positive("tolerance", params.tolerance)?;
                Ok(FeatureOperation::Offset {
                    input: inputs[0].clone(),
                    params,
                })
            }
            FeatureType::Prism => {
                let params = decode_params::<PrismParams>(self, params)?;
                validate_prism(&params)?;
                Ok(FeatureOperation::Prism {
                    input: inputs[0].clone(),
                    params,
                })
            }
            FeatureType::Revolution => {
                let params = decode_params::<RevolutionParams>(self, params)?;
                validate_revolution(&params)?;
                Ok(FeatureOperation::Revolution {
                    input: inputs[0].clone(),
                    params,
                })
            }
            FeatureType::ImportStep => {
                let params = decode_params::<ImportStepParams>(self, params)?;
                if params.path.as_os_str().is_empty() {
                    return Err(Error::new("Imported STEP requires a non-empty path."));
                }
                Ok(FeatureOperation::ImportStep { path: params.path })
            }
        }
    }
}

impl FeatureDefinition {
    pub fn default_spec(&self) -> FeatureSpec {
        FeatureSpec::new(self.feature_type).with_name(self.default_name.clone())
    }
}

impl FeatureSpec {
    pub fn new(feature_type: FeatureType) -> Self {
        Self {
            feature_type,
            name: None,
            inputs: Vec::new(),
            params: empty_params_object(),
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_input(mut self, feature_id: &FeatureId) -> Self {
        self.inputs.push(feature_id.clone());
        self
    }

    pub fn definition(&self) -> FeatureDefinition {
        self.feature_type.definition()
    }

    pub fn resolved_name(&self) -> String {
        self.name
            .clone()
            .unwrap_or_else(|| self.feature_type.default_name().to_owned())
    }

    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    pub fn push_input(&mut self, feature_id: &FeatureId) {
        self.inputs.push(feature_id.clone());
    }

    pub fn set_param<T: Serialize>(&mut self, key: &str, value: T) -> Result<(), Error> {
        let definition = self.definition();
        if !definition.params.iter().any(|param| param.name == key) {
            let allowed = definition
                .params
                .iter()
                .map(|param| param.name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            return Err(Error::new(format!(
                "{} has no parameter named '{key}'. Allowed parameters: [{}].",
                definition.default_name, allowed,
            )));
        }

        let value = serde_json::to_value(value).map_err(|error| {
            Error::new(format!("failed to serialize parameter '{key}': {error}"))
        })?;
        let object = self
            .params
            .as_object_mut()
            .ok_or_else(|| Error::new("feature parameters must be a JSON object."))?;
        object.insert(key.to_owned(), value);
        Ok(())
    }

    pub fn replace_params<T: Serialize>(&mut self, params: T) -> Result<(), Error> {
        let params = serde_json::to_value(params).map_err(|error| {
            Error::new(format!("failed to serialize feature parameters: {error}"))
        })?;
        if !params.is_object() {
            return Err(Error::new(
                "feature parameters must serialize to a JSON object.",
            ));
        }
        self.params = params;
        Ok(())
    }

    pub fn resolved_params(&self) -> Result<Value, Error> {
        let overrides = self
            .params
            .as_object()
            .ok_or_else(|| Error::new("feature parameters must be a JSON object."))?;
        let definition = self.definition();
        for key in overrides.keys() {
            if !definition.params.iter().any(|param| &param.name == key) {
                let allowed = definition
                    .params
                    .iter()
                    .map(|param| param.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(Error::new(format!(
                    "{} has no parameter named '{key}'. Allowed parameters: [{}].",
                    definition.default_name, allowed,
                )));
            }
        }

        let mut resolved = self.feature_type.default_params_json();
        merge_json_objects(&mut resolved, &self.params)?;
        Ok(resolved)
    }

    pub fn validate(&self) -> Result<(), Error> {
        self.to_operation().map(|_| ())
    }

    pub fn to_operation(&self) -> Result<FeatureOperation, Error> {
        self.feature_type
            .build_operation(&self.inputs, self.resolved_params()?)
    }
}

pub fn feature_definitions() -> Vec<FeatureDefinition> {
    FeatureType::all()
        .iter()
        .copied()
        .map(FeatureType::definition)
        .collect()
}

fn empty_params_object() -> Value {
    Value::Object(Map::new())
}

fn input_def(name: &str, description: &str) -> FeatureInputDefinition {
    FeatureInputDefinition {
        name: name.to_owned(),
        description: description.to_owned(),
    }
}

fn param_def(
    name: &str,
    kind: FeatureParamKind,
    description: &str,
    default_value: Value,
) -> FeatureParamDefinition {
    FeatureParamDefinition {
        name: name.to_owned(),
        kind,
        description: description.to_owned(),
        default_value,
        required: true,
    }
}

fn merge_json_objects(base: &mut Value, overrides: &Value) -> Result<(), Error> {
    match (base, overrides) {
        (Value::Object(base_object), Value::Object(override_object)) => {
            for (key, override_value) in override_object {
                match base_object.get_mut(key) {
                    Some(base_value) => merge_json_objects(base_value, override_value)?,
                    None => {
                        base_object.insert(key.clone(), override_value.clone());
                    }
                }
            }
            Ok(())
        }
        (Value::Object(_), Value::Null) => Ok(()),
        (base_slot, override_value) => {
            *base_slot = override_value.clone();
            Ok(())
        }
    }
}

fn decode_params<T>(feature_type: FeatureType, params: Value) -> Result<T, Error>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_value(params).map_err(|error| {
        Error::new(format!(
            "invalid parameters for {}: {error}",
            feature_type.default_name()
        ))
    })
}

fn validate_box(params: &BoxParams) -> Result<(), Error> {
    ensure_positive_vec3("size", params.size)
}

fn validate_cylinder(params: &CylinderParams) -> Result<(), Error> {
    ensure_nonzero_vec3("axis", params.axis)?;
    ensure_positive("radius", params.radius)?;
    ensure_positive("height", params.height)
}

fn validate_cone(params: &ConeParams) -> Result<(), Error> {
    ensure_nonzero_vec3("axis", params.axis)?;
    ensure_nonzero_vec3("x_direction", params.x_direction)?;
    ensure_positive("base_radius", params.base_radius)?;
    ensure_nonnegative("top_radius", params.top_radius)?;
    ensure_positive("height", params.height)
}

fn validate_sphere(params: &SphereParams) -> Result<(), Error> {
    ensure_nonzero_vec3("axis", params.axis)?;
    ensure_nonzero_vec3("x_direction", params.x_direction)?;
    ensure_positive("radius", params.radius)
}

fn validate_torus(params: &TorusParams) -> Result<(), Error> {
    ensure_nonzero_vec3("axis", params.axis)?;
    ensure_nonzero_vec3("x_direction", params.x_direction)?;
    ensure_positive("major_radius", params.major_radius)?;
    ensure_positive("minor_radius", params.minor_radius)?;
    if params.major_radius <= params.minor_radius {
        return Err(Error::new(
            "major_radius must be greater than minor_radius for a torus.",
        ));
    }
    Ok(())
}

fn validate_ellipse_edge(params: &EllipseEdgeParams) -> Result<(), Error> {
    ensure_nonzero_vec3("axis", params.axis)?;
    ensure_nonzero_vec3("x_direction", params.x_direction)?;
    ensure_positive("major_radius", params.major_radius)?;
    ensure_positive("minor_radius", params.minor_radius)
}

fn validate_helix(params: &HelixParams) -> Result<(), Error> {
    ensure_nonzero_vec3("axis", params.axis)?;
    ensure_nonzero_vec3("x_direction", params.x_direction)?;
    ensure_positive("radius", params.radius)?;
    ensure_positive("height", params.height)?;
    ensure_positive("pitch", params.pitch)
}

fn validate_box_with_through_hole(spec: &ThroughHoleCut) -> Result<(), Error> {
    validate_box(&spec.box_params)?;
    validate_cylinder(&spec.tool_params)
}

fn validate_prism(params: &PrismParams) -> Result<(), Error> {
    ensure_nonzero_vec3("direction", params.direction)
}

fn validate_revolution(params: &RevolutionParams) -> Result<(), Error> {
    ensure_nonzero_vec3("axis", params.axis)?;
    ensure_positive("angle_radians", params.angle_radians)
}

fn ensure_positive_vec3(name: &str, value: [f64; 3]) -> Result<(), Error> {
    if value
        .iter()
        .all(|component| component.is_finite() && *component > 0.0)
    {
        Ok(())
    } else {
        Err(Error::new(format!(
            "{name} must contain only positive values."
        )))
    }
}

fn ensure_nonzero_vec3(name: &str, value: [f64; 3]) -> Result<(), Error> {
    if value.iter().all(|component| component.is_finite())
        && value.iter().any(|component| component.abs() > 1.0e-12)
    {
        Ok(())
    } else {
        Err(Error::new(format!(
            "{name} must be a non-zero finite vector."
        )))
    }
}

fn ensure_positive(name: &str, value: f64) -> Result<(), Error> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(Error::new(format!("{name} must be positive.")))
    }
}

fn ensure_nonnegative(name: &str, value: f64) -> Result<(), Error> {
    if value.is_finite() && value >= 0.0 {
        Ok(())
    } else {
        Err(Error::new(format!("{name} must be non-negative.")))
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FilletSpecParams {
    selector: EdgeSelector,
    radius: f64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CylindricalHoleSpecParams {
    selector: FaceSelector,
    radius: f64,
}
