//! A drawing sheet's marks, and the files they are written as.
//!
//! `drawing.sheet.paper` works a sheet's marks out and returns them in the paper format the SVG,
//! DXF, PDF and PNG writers read (`solidcraft_geom::paper::Scene`); `drawing.export` draws the
//! same marks into a file. Neither changes the drawing. The sheet's own furniture comes from its
//! layouts and each view from the design, through [`crate::drawing_view::drawn`], so [`marks`] is
//! the one place that turns a drawing's records into marks — and the heaviest work a drawing
//! command does, because every view's hidden lines are worked out.
//!
//! `drawing.update` draws the views that aren't current and keeps their lines, and
//! `drawing.status` says which those are without drawing anything ([`crate::drawing_view`]'s
//! freshness). The two marks commands reuse the kept lines of a current view.

use std::path::Path;

use serde::Serialize;
use serde_json::Value;
use solidcraft_doc::drawing::{Drawing, Sheet};
use solidcraft_doc::ids;
use solidcraft_doc::{MAX_PLACEMENT_DEPTH, MAX_PLACEMENTS};
use solidcraft_geom::drawing_paper::{self, SheetMarks, TitleBlockMarks, ViewMarks};
use solidcraft_geom::drawing_sheet::SheetLayout;
use solidcraft_geom::paper::Scene;

use super::CommandSpec;
use crate::drawing_view::{Drawn, Freshness};
use crate::params::bad;
use crate::schema::drawing_view::{
    ExportParams, ExportResult, PaperParams, PaperResult, PaperViewResult, ScaleResult, SceneResult, SheetId, SnappedResult, StatusParams,
    StatusResult, StatusSheetResult, StatusViewResult, UpdateParams, UpdateResult, UpdatedSheetResult, UpdatedViewResult,
};
use crate::schema::{TypedContract, output_schema_for};
use crate::{Result, Session};

const CMD: &str = "drawing.sheet.paper";
const EXPORT: &str = "drawing.export";
const UPDATE: &str = "drawing.update";
const STATUS: &str = "drawing.status";
const CONTRACT: TypedContract<PaperParams> = TypedContract::new();
const EXPORT_CONTRACT: TypedContract<ExportParams> = TypedContract::new();
const UPDATE_CONTRACT: TypedContract<UpdateParams> = TypedContract::new();
const STATUS_CONTRACT: TypedContract<StatusParams> = TypedContract::new();

/// The longest path a call may name, as the other file commands take it.
const MAX_PATH: usize = 4096;

pub static COMMANDS: &[CommandSpec] = &[CommandSpec::new(CMD, "Paper Scene", crate::checked!("drawing.sheet.paper", paper))
    .in_drawing()
    .icon("export")
    .noundo()
    .contract(&CONTRACT)
    .output(output_schema_for::<PaperResult>)
    .params(
        "sheet?: the sheet's id (drawing.inspect; a drawing with one sheet may leave it out) → scene: the sheet's marks in paint order, in the format the paper writers read — the sheet border, the drawing border with its grid and marks, the title block, then one group per view with its hidden lines under its visible ones; views: what each view settled on ({id, scale: {text, factor, fitted}, chord_mm, visible_runs, hidden_runs}). Every body the design places, whatever the browser shows. A view with no scale of its own is fitted to the drawing area around where it sits. Read-only: saves nothing.",
    ),
    CommandSpec::new(EXPORT, "Export Drawing", crate::checked!("drawing.export", export))
        .in_drawing()
        .icon("export")
        .noundo()
        .contract(&EXPORT_CONTRACT)
        .output(output_schema_for::<ExportResult>)
        .params(
            "format: \"svg\" (dxf, pdf and png follow); path: the file to write; sheet?: the sheet's id (a drawing with one sheet may leave it out); overwrite?: replace a file already at that path, which is refused without it → path, format, bytes, snapped (what the writer moved onto its grid), views (as drawing.sheet.paper reports them). The sheet is drawn as drawing.sheet.paper gives it, so the same sheet writes the same bytes. Read-only: the drawing is not changed.",
        ),
    CommandSpec::new(UPDATE, "Update Drawing", crate::checked!("drawing.update", update))
        .in_drawing()
        .icon("update")
        .noundo()
        .contract(&UPDATE_CONTRACT)
        .output(output_schema_for::<UpdateResult>)
        .params(
            "sheet?: the sheet's id (drawing.inspect; every sheet when not given) → sheets: [{id, views: [{id, was: current|out_of_date|not_drawn, scale: {text, factor, fitted}, chord_mm, visible_runs, hidden_runs}]}], drawn: the views drawn now, kept: the views that were current. Draws the lines of every view that isn't current and keeps them, so drawing.sheet.paper and drawing.export take them as they are. A view is out of date once the design, or the view's own direction or scale, has changed since it was drawn; moving it on the sheet changes nothing. Saves nothing: a drawing records no lines.",
        ),
    CommandSpec::new(STATUS, "Drawing Status", crate::checked!("drawing.status", status))
        .in_drawing()
        .icon("inspect")
        .noundo()
        .contract(&STATUS_CONTRACT)
        .output(output_schema_for::<StatusResult>)
        .params(
            "sheet?: the sheet's id (drawing.inspect; every sheet when not given) → sheets: [{id, views: [{id, status: current|out_of_date|not_drawn}]}], current, out_of_date, not_drawn: how many views are each. Draws nothing: the views that aren't current are the ones drawing.update draws.",
        ),
];

/// The layout of `sheet`, which its views are fitted into. Without a drawing border there is no
/// layout to lay anything out in, and nothing can make such a sheet today: only a hand-written
/// file carries one.
fn layout_of(cmd: &str, sheet: &Sheet) -> Result<SheetLayout> {
    let profile = sheet.border.as_ref();
    let profile = profile.ok_or_else(|| bad(cmd, "the sheet has no drawing border, so there is nothing to lay its marks out in"))?;
    profile.layout(&sheet.sheet).map_err(|e| bad(cmd, e.to_string()))
}

/// Refuses an assembly the placements stop short of: its views would leave parts out.
fn complete(s: &Session, cmd: &str) -> Result<()> {
    if !s.doc.placements_complete() {
        let limits = format!("deeper than {MAX_PLACEMENT_DEPTH} levels or has more than {MAX_PLACEMENTS} placements");
        return Err(bad(cmd, format!("the assembly is {limits}: the views would leave parts out")));
    }
    Ok(())
}

/// The sheets a call names: the one `named`, or every sheet of the drawing.
fn chosen<'a>(cmd: &str, d: &'a Drawing, named: Option<&SheetId>) -> Result<Vec<&'a Sheet>> {
    match named {
        Some(_) => {
            let i = super::drawing_view::sheet_index(cmd, d, named)?;
            Ok(d.sheets.get(i).map(|sheet| &**sheet).into_iter().collect())
        }
        None => Ok(d.sheets.iter().map(|sheet| &**sheet).collect()),
    }
}

/// A reply built from its result type, so it can't drift from the output schema.
fn reply(value: &impl Serialize) -> Result<Value> {
    serde_json::to_value(value).map_err(|e| crate::EngineError::Other(e.to_string()))
}

/// The marks a sheet puts on paper, and what each of its views settled on: what
/// `drawing.sheet.paper` answers, and what `drawing.export` draws into a file.
fn marks(s: &Session, cmd: &str, named: Option<&SheetId>) -> Result<(Scene, Vec<Drawn>)> {
    let d = s.drawing(cmd)?;
    let i = super::drawing_view::sheet_index(cmd, d, named)?;
    let sheet = d.sheets[i].clone();

    // The sheet's furniture.
    let layout = layout_of(cmd, &sheet)?;
    let block = match &sheet.title_block {
        Some(block) => Some(block.layout(&layout).map_err(|e| bad(cmd, e.to_string()))?),
        None => None,
    };

    complete(s, cmd)?;
    let drawn = crate::drawing_view::drawn(s, cmd, d, &sheet, layout.drawing_border)?;
    let marks: Vec<ViewMarks> =
        drawn.iter().map(|view| ViewMarks { object: view.view, frame: &view.frame, visible: &view.visible, hidden: &view.hidden }).collect();
    let on_block = block.as_ref().map(|layout| TitleBlockMarks { object: sheet.id, layout });
    let scene = drawing_paper::scene(SheetMarks { object: sheet.id, layout: &layout }, on_block, &marks).map_err(|e| bad(cmd, e.to_string()))?;

    Ok((scene, drawn))
}

/// What each view settled on, as both commands report it.
fn reported(drawn: &[Drawn]) -> Vec<PaperViewResult> {
    drawn
        .iter()
        .map(|view| PaperViewResult {
            id: ids::encode(view.view),
            scale: ScaleResult { text: view.scale.to_string(), factor: view.scale.factor(), fitted: view.fitted },
            chord_mm: crate::drawing_view::chord_mm(view.scale.factor()),
            visible_runs: view.visible.len(),
            hidden_runs: view.hidden.len(),
        })
        .collect()
}

fn paper(s: &mut Session, p: &Value) -> Result<Value> {
    let request: PaperParams = crate::schema::read(CMD, p)?;
    let (scene, drawn) = marks(s, CMD, request.sheet.as_ref())?;
    reply(&PaperResult { scene: SceneResult(scene), views: reported(&drawn), param_warnings: Vec::new() })
}

fn export(s: &mut Session, p: &Value) -> Result<Value> {
    let request: ExportParams = crate::schema::read(EXPORT, p)?;
    // The path is judged before anything is drawn: a call that cannot be written should not pay
    // for every view's hidden lines first.
    let path = request.path.trim();
    if path.is_empty() || path.len() > MAX_PATH {
        return Err(bad(EXPORT, format!("`path` is the file to write, 1 to {MAX_PATH} characters")));
    }
    solidcraft_io::vfs::check(path).map_err(|e| crate::EngineError::Other(e.to_string()))?;

    let (scene, drawn) = marks(s, EXPORT, request.sheet.as_ref())?;
    let prepared = scene.prepare().map_err(|e| bad(EXPORT, e.to_string()))?;
    let (bytes, report) = solidcraft_io::paper::paper_svg(&prepared).map_err(|e| bad(EXPORT, e.to_string()))?;
    let written = if request.overwrite {
        solidcraft_io::write_atomic(Path::new(path), &bytes, false)
    } else {
        // Never onto a file that is already there. `create_atomic` refuses one even if another
        // writer makes it while this call runs, so there is no window between looking and writing.
        solidcraft_io::safe_write::create_atomic(Path::new(path), &bytes)
    };
    written.map_err(|e| match e.kind() {
        std::io::ErrorKind::AlreadyExists => bad(EXPORT, format!("{path}: a file is already there; pass `overwrite: true` to replace it")),
        _ => crate::EngineError::Other(format!("{path}: {e}")),
    })?;
    let snapped = SnappedResult { widths: report.widths_snapped, pieces: report.pieces_merged, dashes: report.dashes_snapped };
    let written = ExportResult {
        path: path.to_string(),
        format: "svg".into(),
        bytes: bytes.len(),
        snapped,
        views: reported(&drawn),
        param_warnings: Vec::new(),
    };
    reply(&written)
}

fn update(s: &mut Session, p: &Value) -> Result<Value> {
    let request: UpdateParams = crate::schema::read(UPDATE, p)?;
    let d = s.drawing(UPDATE)?.clone();
    complete(s, UPDATE)?;
    let (mut drawn, mut kept) = (0, 0);
    let mut sheets = Vec::new();
    for sheet in chosen(UPDATE, &d, request.sheet.as_ref())? {
        let layout = layout_of(UPDATE, sheet)?;
        let mut views = Vec::with_capacity(sheet.views.len());
        for (view, was) in crate::drawing_view::draw(s, UPDATE, &d, sheet, layout.drawing_border)? {
            match was {
                Freshness::Current => kept += 1,
                Freshness::OutOfDate | Freshness::NotDrawn => drawn += 1,
            }
            views.push(UpdatedViewResult {
                id: ids::encode(view.view),
                was: was.into(),
                scale: ScaleResult { text: view.scale.to_string(), factor: view.scale.factor(), fitted: view.fitted },
                chord_mm: crate::drawing_view::chord_mm(view.scale.factor()),
                visible_runs: view.visible.len(),
                hidden_runs: view.hidden.len(),
            });
        }
        sheets.push(UpdatedSheetResult { id: ids::encode(sheet.id), views });
    }
    reply(&UpdateResult { sheets, drawn, kept, param_warnings: Vec::new() })
}

fn status(s: &mut Session, p: &Value) -> Result<Value> {
    let request: StatusParams = crate::schema::read(STATUS, p)?;
    let d = s.drawing(STATUS)?;
    complete(s, STATUS)?;
    let mut counts = [0usize; 3];
    let mut sheets = Vec::new();
    for sheet in chosen(STATUS, d, request.sheet.as_ref())? {
        let layout = layout_of(STATUS, sheet)?;
        let mut views = Vec::with_capacity(sheet.views.len());
        for (view, freshness) in crate::drawing_view::freshness(s, STATUS, d, sheet, layout.drawing_border)? {
            let at = match freshness {
                Freshness::Current => 0,
                Freshness::OutOfDate => 1,
                Freshness::NotDrawn => 2,
            };
            if let Some(count) = counts.get_mut(at) {
                *count += 1;
            }
            views.push(StatusViewResult { id: ids::encode(view), status: freshness.into() });
        }
        sheets.push(StatusSheetResult { id: ids::encode(sheet.id), views });
    }
    let [current, out_of_date, not_drawn] = counts;
    reply(&StatusResult { sheets, current, out_of_date, not_drawn, param_warnings: Vec::new() })
}

#[cfg(test)]
#[path = "drawing_sheet_tests.rs"]
mod tests;
