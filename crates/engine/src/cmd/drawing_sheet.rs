//! Drawing sheet commands: what a sheet of the open drawing puts on paper.
//!
//! `drawing.sheet.paper` works a sheet's marks out and returns them in the paper format the SVG,
//! DXF, PDF and PNG writers read (`solidcraft_geom::paper::Scene`). It saves nothing. The sheet's
//! own furniture comes from its layouts, and each view from the design through
//! [`crate::drawing_view::drawn`], so this is the one place that turns a drawing's records into
//! marks — and the heaviest drawing command, because every view's hidden lines are worked out.

use serde::Serialize;
use serde_json::Value;
use solidcraft_doc::ids;
use solidcraft_doc::{MAX_PLACEMENT_DEPTH, MAX_PLACEMENTS};
use solidcraft_geom::drawing_paper::{self, SheetMarks, TitleBlockMarks, ViewMarks};

use super::CommandSpec;
use crate::params::bad;
use crate::schema::drawing_view::{PaperParams, PaperResult, PaperViewResult, ScaleResult, SceneResult};
use crate::schema::{TypedContract, output_schema_for};
use crate::{Result, Session};

const CMD: &str = "drawing.sheet.paper";
const CONTRACT: TypedContract<PaperParams> = TypedContract::new();

pub static COMMANDS: &[CommandSpec] = &[CommandSpec::new(CMD, "Paper Scene", crate::checked!("drawing.sheet.paper", paper))
    .in_drawing()
    .icon("export")
    .noundo()
    .contract(&CONTRACT)
    .output(output_schema_for::<PaperResult>)
    .params(
        "sheet?: the sheet's id (drawing.inspect; a drawing with one sheet may leave it out) → scene: the sheet's marks in paint order, in the format the paper writers read — the sheet border, the drawing border with its grid and marks, the title block, then one group per view with its hidden lines under its visible ones; views: what each view settled on ({id, scale: {text, factor, fitted}, chord_mm, visible_runs, hidden_runs}). Every body the design places, whatever the browser shows. A view with no scale of its own is fitted to the drawing area around where it sits. Read-only: saves nothing.",
    )];

/// A reply built from its result type, so it can't drift from the output schema.
fn reply(value: &impl Serialize) -> Result<Value> {
    serde_json::to_value(value).map_err(|e| crate::EngineError::Other(e.to_string()))
}

fn paper(s: &mut Session, p: &Value) -> Result<Value> {
    let request: PaperParams = crate::schema::read(CMD, p)?;
    let d = s.drawing(CMD)?;
    let i = super::drawing_view::sheet_index(CMD, d, request.sheet.as_ref())?;
    let sheet = d.sheets[i].clone();

    // The sheet's furniture: without a drawing border there is no layout to lay anything out in,
    // and nothing can make such a sheet today — only a hand-written file carries one.
    let profile = sheet.border.as_ref();
    let profile = profile.ok_or_else(|| bad(CMD, "the sheet has no drawing border, so there is nothing to lay its marks out in"))?;
    let layout = profile.layout(&sheet.sheet).map_err(|e| bad(CMD, e.to_string()))?;
    let block = match &sheet.title_block {
        Some(block) => Some(block.layout(&layout).map_err(|e| bad(CMD, e.to_string()))?),
        None => None,
    };

    if !s.doc.placements_complete() {
        let limits = format!("deeper than {MAX_PLACEMENT_DEPTH} levels or has more than {MAX_PLACEMENTS} placements");
        return Err(bad(CMD, format!("the assembly is {limits}: the views would leave parts out")));
    }
    let drawn = crate::drawing_view::drawn(s, CMD, d, &sheet, layout.drawing_border)?;
    let marks: Vec<ViewMarks> =
        drawn.iter().map(|view| ViewMarks { object: view.view, frame: &view.frame, visible: &view.visible, hidden: &view.hidden }).collect();
    let on_block = block.as_ref().map(|layout| TitleBlockMarks { object: sheet.id, layout });
    let scene = drawing_paper::scene(SheetMarks { object: sheet.id, layout: &layout }, on_block, &marks).map_err(|e| bad(CMD, e.to_string()))?;

    let views = drawn
        .iter()
        .map(|view| PaperViewResult {
            id: ids::encode(view.view),
            scale: ScaleResult { text: view.scale.to_string(), factor: view.scale.factor(), fitted: view.fitted },
            chord_mm: crate::drawing_view::chord_mm(view.scale.factor()),
            visible_runs: view.visible.len(),
            hidden_runs: view.hidden.len(),
        })
        .collect();
    reply(&PaperResult { scene: SceneResult(scene), views, param_warnings: Vec::new() })
}

#[cfg(test)]
#[path = "drawing_sheet_tests.rs"]
mod tests;
