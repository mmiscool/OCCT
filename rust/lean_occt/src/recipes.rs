use crate::{BoxParams, CurveKind, EdgeSelector, Error, FaceSelector, ModelDocument};

#[derive(Clone, Debug)]
pub struct RecipeBuildResult {
    pub final_shape: String,
    pub stages: Vec<String>,
}

impl RecipeBuildResult {
    pub fn final_shape(&self) -> &str {
        &self.final_shape
    }

    pub fn stage_names(&self) -> &[String] {
        &self.stages
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DrilledBlockRecipe {
    pub box_params: BoxParams,
    pub hole_normal_hint: [f64; 3],
    pub hole_radius: f64,
}

impl DrilledBlockRecipe {
    pub fn build(
        &self,
        document: &mut ModelDocument,
        prefix: impl AsRef<str>,
    ) -> Result<RecipeBuildResult, Error> {
        let prefix = prefix.as_ref();
        let base = stage_name(prefix, "base");
        let drilled = stage_name(prefix, "drilled");

        document.insert_box(&base, self.box_params)?;
        document.cylindrical_hole_on_selected_face(
            &drilled,
            &base,
            FaceSelector::BestAlignedPlane {
                normal_hint: self.hole_normal_hint,
            },
            self.hole_radius,
        )?;

        Ok(RecipeBuildResult {
            final_shape: drilled.clone(),
            stages: vec![base, drilled],
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RoundedDrilledBlockRecipe {
    pub drilled_block: DrilledBlockRecipe,
    pub fillet_curve_kind: CurveKind,
    pub fillet_radius: f64,
}

impl RoundedDrilledBlockRecipe {
    pub fn build(
        &self,
        document: &mut ModelDocument,
        prefix: impl AsRef<str>,
    ) -> Result<RecipeBuildResult, Error> {
        let prefix = prefix.as_ref();
        let mut build = self.drilled_block.build(document, prefix)?;
        let rounded = stage_name(prefix, "rounded");

        document.fillet_selected_edge(
            &rounded,
            build.final_shape(),
            EdgeSelector::FirstByCurveKind(self.fillet_curve_kind),
            self.fillet_radius,
        )?;

        build.final_shape = rounded.clone();
        build.stages.push(rounded);
        Ok(build)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SelectorDrivenRoundedBlockRecipe {
    pub box_params: BoxParams,
    pub hole_face_selector: FaceSelector,
    pub hole_radius: f64,
    pub fillet_edge_selector: EdgeSelector,
    pub fillet_radius: f64,
}

impl SelectorDrivenRoundedBlockRecipe {
    pub fn build(
        &self,
        document: &mut ModelDocument,
        prefix: impl AsRef<str>,
    ) -> Result<RecipeBuildResult, Error> {
        let prefix = prefix.as_ref();
        let base = stage_name(prefix, "base");
        let drilled = stage_name(prefix, "drilled");
        let rounded = stage_name(prefix, "rounded");

        document.insert_box(&base, self.box_params)?;
        document.cylindrical_hole_on_selected_face(
            &drilled,
            &base,
            self.hole_face_selector,
            self.hole_radius,
        )?;
        document.fillet_selected_edge(
            &rounded,
            &drilled,
            self.fillet_edge_selector,
            self.fillet_radius,
        )?;

        Ok(RecipeBuildResult {
            final_shape: rounded.clone(),
            stages: vec![base, drilled, rounded],
        })
    }
}

fn stage_name(prefix: &str, suffix: &str) -> String {
    if prefix.is_empty() {
        suffix.to_owned()
    } else {
        format!("{prefix}_{suffix}")
    }
}
