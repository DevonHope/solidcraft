//! Drawing view commands. `drawing.view.snapshot` (W07, #5866) works one orthographic view of the
//! design out and returns it without saving anything: it reads the evaluated model and changes
//! nothing. The rest (W12) write the views a sheet keeps (`solidcraft_doc::drawing_view::View`).
//!
//! Of those, `base`, `projected`, `set` and `delete` **evaluate nothing**: what a view draws is
//! worked out when it is drawn, not when it is placed, so each is a document edit and one undo step
//! of the drawing. `standard` is the exception and reads the model, because the scale it fits and
//! the four places it puts the views in follow the part's extents on four planes.
//!
//! The view's settings are `doc::drawing_view::ViewSettings`; its basis, scale and map onto
//! paper are `geom::drawing_view`'s. Without a scale it takes the largest ISO 5455 scale at
//! which the view, centred where it goes, lies inside the sheet's drawing border.

use std::sync::Arc;

use serde::Serialize;
use serde_json::Value;
use solidcraft_doc::drawing::{Drawing, MAX_VIEWS, Sheet as SheetRecord};
use solidcraft_doc::drawing_view::{Looks, TangentEdges, View, ViewDirection, ViewStyle};
use solidcraft_doc::ids::{self, IdSource};
use solidcraft_doc::{MAX_PLACEMENT_DEPTH, MAX_PLACEMENTS};
use solidcraft_geom::drawing_paper::kept_clear;
use solidcraft_geom::drawing_sheet::{DrawingBorderProfile, Sheet};
use solidcraft_geom::drawing_view::{NamedView, PRESETS, ProjectedSide, Scale, StandardExtents, ViewFrame, projected_basis, standard_layout};
use solidcraft_geom::paper::MAX_COORDINATE_MM;
use solidcraft_geom::{Aabb2, Plane, Vec2};

use super::CommandSpec;
use crate::params::bad;
use crate::schema::drawing_view::{
    BaseParams, BoxResult, DPI_RANGE, ExtentsResult, LinesResult, LooksResult, ProjectedParams, SavedViewResult, ScaleResult, SheetId,
    SnapshotParams, SnapshotResult, SourceResult, StandardParams, StandardResult, ViewDeleteParams, ViewDeletedResult, ViewRecordResult, ViewResult,
    ViewSetParams, ViewStyleField, ViewStyleResult,
};
use crate::schema::{Point2, TypedContract, output_schema_for};
use crate::{Result, Session};

const CMD: &str = "drawing.view.snapshot";
const CONTRACT: TypedContract<SnapshotParams> = TypedContract::new();
const BASE: TypedContract<BaseParams> = TypedContract::new();
const PROJECTED: TypedContract<ProjectedParams> = TypedContract::new();
const SET: TypedContract<ViewSetParams> = TypedContract::new();
const DELETE: TypedContract<ViewDeleteParams> = TypedContract::new();
const STANDARD: TypedContract<StandardParams> = TypedContract::new();

/// The space `drawing.view.standard` leaves between neighbouring views when the call doesn't say.
const DEFAULT_GAP_MM: f64 = 10.0;

pub static COMMANDS: &[CommandSpec] = &[CommandSpec::new(CMD, "Drawing View Snapshot", crate::checked!("drawing.view.snapshot", snapshot))
    .icon("export")
    .noundo()
    .contract(&CONTRACT)
    .output(output_schema_for::<SnapshotResult>)
    .params("view?: front (default), back, top, bottom, left, right, iso (= iso_front_right), iso_front_left, iso_back_right, iso_back_left, or {direction, up, origin?}; scale?: \"paper:model\" such as \"1:2\" (default: the largest ISO 5455 scale that fits the sheet's drawing area); position_mm?: [x, y] for the view's centre on the sheet (default the sheet's centre); sheet?: {size: {standard: a3} or {custom: {sides_mm: [w, h]}}, orientation?: landscape|portrait, dpi?: 25..2400} (default A3 landscape, 300 dpi); display_style?, reference_style?: visible (default) or visible_hidden (the shaded styles aren't supported yet); reference_hides_model?: false; reference_margin_mm?: 0..100000 model mm on every side; tangent_edges?: hide (default); show isn't supported yet. Every body the design places, whatever the browser shows. Returns {view, scale: {text, factor, fitted}, position_mm, sheet, extents (model mm on the view's plane), lines: {visible, hidden: runs of [x, y] on the view's plane, chord_mm}, boundary_mm (on the sheet), source}. Read-only: saves nothing."),
    CommandSpec::new("drawing.view.base", "Base View", crate::checked!("drawing.view.base", base))
        .in_drawing()
        .icon("new")
        .doc_only()
        .contract(&BASE)
        .output(output_schema_for::<SavedViewResult>)
        .params(
            "sheet?: the sheet's id (drawing.inspect; a drawing with one sheet may leave it out); view?: front (default), back, top, bottom, left, right, iso (= iso_front_right), iso_front_left, iso_back_right, iso_back_left, or {direction, up, origin?}; component?: the component of the design it draws (0, the design itself, by default); position_mm?: [x, y], its centre on the paper (default the middle of the sheet's drawing border); style?: {scale?: \"paper:model\" such as \"1:2\", display_style?, reference_style?, reference_hides_model?, reference_margin_mm?, tangent_edges?}, each left out drawn as the drawing's default → view: the view as saved, with the basis it draws with. Saves a view and evaluates nothing: its lines are worked out when it is drawn. One undo step of the drawing.",
        ),
    CommandSpec::new("drawing.view.projected", "Projected View", crate::checked!("drawing.view.projected", projected))
        .in_drawing()
        .icon("project")
        .doc_only()
        .contract(&PROJECTED)
        .output(output_schema_for::<SavedViewResult>)
        .params(
            "from: a base view's id (drawing.inspect); side: left|right|above|below; offset_mm: how far from that view's centre, in paper mm along side (above 0); style?: as drawing.view.base takes it, each field left out drawn as the view it comes from draws it → view: the view as saved, with the basis the projection gives it. It draws the same component as the view it comes from and keeps that view's other coordinate, so it stays lined up with it. One undo step of the drawing.",
        ),
    CommandSpec::new("drawing.view.set", "View Settings", crate::checked!("drawing.view.set", view_set))
        .in_drawing()
        .icon("settings")
        .doc_only()
        .contract(&SET)
        .output(output_schema_for::<SavedViewResult>)
        .params(
            "view: the view's id; position_mm?: [x, y] for a base view's centre, whose projections move with it; offset_mm?: how far a projected view sits from the view it comes from; style?: as drawing.view.base takes it; clear?: [style fields] to give back to what the view follows (scale, display_style, reference_style, reference_hides_model, reference_margin_mm, tangent_edges). Keys not given stay as they are; a field both given and cleared is refused → view: the view as saved. One undo step of the drawing.",
        ),
    CommandSpec::new("drawing.view.standard", "Standard Views", crate::checked!("drawing.view.standard", standard))
        .in_drawing()
        .icon("align")
        .contract(&STANDARD)
        .output(output_schema_for::<StandardResult>)
        .params(
            "sheet?: the sheet's id (drawing.inspect; a drawing with one sheet may leave it out); gap_mm?: the space between neighbouring views, 0 to 1000 paper mm (10 by default) → views: the front view, the top and right views projected from it, and an isometric view, in that order; scale: what they are all drawn at. The four go in the largest band of the drawing border beside the title block, at the largest ISO 5455 scale that fits them, in the drawing's projection. Unlike the other drawing commands this one reads the model, because the scale and the places follow the part's extents on four planes. The sheet must draw no views yet. One undo step of the drawing.",
        ),
    CommandSpec::new("drawing.view.delete", "Delete View", crate::checked!("drawing.view.delete", view_delete))
        .in_drawing()
        .icon("delete")
        .doc_only()
        .contract(&DELETE)
        .output(output_schema_for::<ViewDeletedResult>)
        .params(
            "view: the view's id → deleted: the views deleted, the one asked for first. A base view takes the views projected from it with it, because a projection draws nothing on its own. One undo step of the drawing.",
        ),
];

fn snapshot(s: &mut Session, p: &Value) -> Result<Value> {
    let request: SnapshotParams = crate::schema::read(CMD, p)?;
    let settings = request.settings();
    let basis = settings.validate().map_err(|e| bad(CMD, e.to_string()))?;
    if settings.display_style.is_shaded() || settings.reference_style.is_shaded() {
        return Err(bad(CMD, "not supported yet: shaded views need the drawing renderer (#5624)"));
    }
    // The mesh carries the placed body's own edge choices, so there is no switch to turn tangent
    // edges back on; refuse rather than quietly drawing them hidden.
    if settings.tangent_edges == TangentEdges::Show {
        return Err(bad(CMD, "not supported yet: tangent_edges show needs the mesh edge classification (#5866)"));
    }
    if !(request.sheet.dpi.is_finite() && DPI_RANGE.contains(&request.sheet.dpi)) {
        return Err(bad(CMD, format!("sheet.dpi is a number from {} to {}", DPI_RANGE.start(), DPI_RANGE.end())));
    }
    let sheet = Sheet { size: request.sheet.size, orientation: request.sheet.orientation };
    let paper = sheet.size_mm().map_err(|e| bad(CMD, e.to_string()))?;
    let border = DrawingBorderProfile::iso().layout(&sheet).map_err(|e| bad(CMD, e.to_string()))?.drawing_border;

    if !s.doc.placements_complete() {
        let limits = format!("deeper than {MAX_PLACEMENT_DEPTH} levels or has more than {MAX_PLACEMENTS} placements");
        return Err(bad(CMD, format!("the assembly is {limits}: a view of it would leave parts out")));
    }
    let extents = crate::drawing_view::extents(s, &basis);
    if extents.model.is_empty() {
        return Err(bad(CMD, "the view's source has no bodies"));
    }
    let shown = extents.model.inflated(settings.reference_margin_mm);
    let position = request.position_mm.map_or(paper * 0.5, |p| p.0);
    let (scale, fitted) = match settings.scale {
        Some(scale) => (scale, false),
        None => (fit(shown.size(), position, border)?, true),
    };
    let frame = ViewFrame::new(basis, scale, shown.center(), position).map_err(|e| bad(CMD, e.to_string()))?;
    let boundary = frame.boundary(&shown).map_err(|e| bad(CMD, e.to_string()))?;
    let on_paper = [boundary.min, boundary.max].iter().all(|c| c.is_finite() && c.x.abs() <= MAX_COORDINATE_MM && c.y.abs() <= MAX_COORDINATE_MM);
    if !on_paper {
        return Err(bad(CMD, format!("at scale {scale} the view reaches past ±{MAX_COORDINATE_MM} mm on paper: choose a smaller scale")));
    }

    let chord = crate::drawing_view::chord_mm(scale.factor());
    let lines = crate::drawing_view::lines(s, CMD, &basis, chord)?;

    let named = match settings.view {
        ViewDirection::Named(view) => Some(snake_name(view)),
        ViewDirection::Looking(_) => None,
    };
    let reply = SnapshotResult {
        view: ViewResult {
            named,
            // The line of sight, -(x × y), written so no component reads -0.
            direction: basis.y.cross(basis.x).into(),
            x: basis.x.into(),
            y: basis.y.into(),
            origin: basis.origin.into(),
        },
        scale: ScaleResult { text: scale.to_string(), factor: scale.factor(), fitted },
        position_mm: position.into(),
        sheet: request.sheet,
        display_style: snake_name(settings.display_style),
        reference_style: snake_name(settings.reference_style),
        extents: ExtentsResult {
            min: extents.model.min.into(),
            max: extents.model.max.into(),
            error_mm: extents.error_mm,
            status: "approximate",
            provenance: "display_mesh",
        },
        lines: LinesResult {
            visible: lines.visible.iter().map(|run| run.iter().map(|p| (*p).into()).collect()).collect(),
            hidden: lines.hidden.iter().map(|run| run.iter().map(|p| (*p).into()).collect()).collect(),
            chord_mm: lines.chord_mm,
            status: "approximate",
            provenance: "display_mesh",
        },
        boundary_mm: BoxResult { min: boundary.min.into(), max: boundary.max.into() },
        source: SourceResult { bodies: extents.bodies, timeline_at_end: s.doc.marker.is_none() },
        param_warnings: Vec::new(),
    };
    serde_json::to_value(reply).map_err(|e| crate::EngineError::Other(e.to_string()))
}

/// The largest preset scale at which a view `size` (model mm), centred at `position`, lies inside
/// the drawing border: the room is twice the distance from the position to the nearer side,
/// each way.
fn fit(size: Vec2, position: Vec2, border: Aabb2) -> Result<Scale> {
    let room =
        Vec2::new(2.0 * (position.x - border.min.x).min(border.max.x - position.x), 2.0 * (position.y - border.min.y).min(border.max.y - position.y));
    Scale::fit(size, room).ok_or_else(|| {
        let name = |s: Option<&Scale>| s.map(|s| s.to_string()).unwrap_or_default();
        let presets = format!("from {} to {}", name(PRESETS.first()), name(PRESETS.last()));
        bad(CMD, format!("the view fits the sheet's drawing area at no preset scale {presets} here: give a scale, a position or a larger sheet"))
    })
}

/// A unit enum's name as it reads and writes (`iso_front_right`, `visible_hidden`).
fn snake_name(value: impl Serialize) -> String {
    serde_json::to_value(value).ok().and_then(|v| v.as_str().map(str::to_string)).unwrap_or_default()
}

/// A reply built from its result type, so it can't drift from the output schema.
fn reply(value: &impl Serialize) -> Result<Value> {
    serde_json::to_value(value).map_err(|e| crate::EngineError::Other(e.to_string()))
}

/// The sheet `named` names, by index. A drawing with one sheet needs no name; one with more does,
/// rather than a command quietly choosing.
pub(super) fn sheet_index(cmd: &str, d: &Drawing, named: Option<&SheetId>) -> Result<usize> {
    match named {
        Some(SheetId(text)) => {
            let id = ids::decode(text).ok_or_else(|| bad(cmd, "`sheet` must be a sheet's id (drawing.inspect)"))?;
            d.sheets.iter().position(|sheet| sheet.id == id).ok_or_else(|| bad(cmd, format!("the drawing has no sheet {}", ids::encode(id))))
        }
        None if d.sheets.len() == 1 => Ok(0),
        None => Err(bad(cmd, format!("the drawing has {} sheets: say which with `sheet` (drawing.inspect)", d.sheets.len()))),
    }
}

/// The view `id`: the index of the sheet it is on, and of the view on that sheet.
fn view_index(cmd: &str, d: &Drawing, id: u64) -> Result<(usize, usize)> {
    let found = d.sheets.iter().enumerate().find_map(|(i, sheet)| sheet.views.iter().position(|view| view.id == id).map(|j| (i, j)));
    found.ok_or_else(|| bad(cmd, format!("the drawing has no view {}", ids::encode(id))))
}

/// The basis a view draws with: a base view's own, or a projected view's quarter turn from the
/// view it comes from, in the drawing's projection.
fn basis_of(cmd: &str, d: &Drawing, sheet: &SheetRecord, view: &View) -> Result<Plane> {
    let fail = |e: solidcraft_geom::drawing_view::Invalid| bad(cmd, e.to_string());
    match view.looks {
        Looks::Base { view } => view.basis().map_err(fail),
        Looks::Projected { from, side } => {
            let parent = sheet.views.iter().find(|other| other.id == from);
            let looks = parent.map(|parent| parent.looks).ok_or_else(|| bad(cmd, "the view it is projected from is not on its sheet"))?;
            let Looks::Base { view } = looks else {
                return Err(bad(cmd, "a view is projected from a base view"));
            };
            projected_basis(&view.basis().map_err(fail)?, side, d.projection).map_err(fail)
        }
    }
}

/// The named view whose basis this is, if any. A projection of a principal view is exact, so a
/// top view projected from the front comes back named `top`.
fn named_as(basis: &Plane) -> Option<String> {
    NamedView::ALL.iter().find(|view| view.basis() == *basis).map(snake_name)
}

/// The view `id` of the open drawing, as the view commands answer.
fn saved(s: &Session, cmd: &str, id: u64) -> Result<Value> {
    let d = s.drawing(cmd)?;
    let (i, j) = view_index(cmd, d, id)?;
    let record = record_of(cmd, d, &d.sheets[i], &d.sheets[i].views[j])?;
    reply(&SavedViewResult { view: record, param_warnings: Vec::new() })
}

/// One saved view as the commands and `drawing.inspect` report it: what it draws, where it sits,
/// what it draws differently, and the basis it draws with, worked out for a projection too.
pub(super) fn record_of(cmd: &str, d: &Drawing, sheet: &SheetRecord, view: &View) -> Result<ViewRecordResult> {
    let basis = basis_of(cmd, d, sheet, view)?;
    let seen = ViewResult {
        named: named_as(&basis),
        // The line of sight, -(x × y), written so no component reads -0.
        direction: basis.y.cross(basis.x).into(),
        x: basis.x.into(),
        y: basis.y.into(),
        origin: basis.origin.into(),
    };
    let looks = match view.looks {
        Looks::Base { .. } => LooksResult::Base { view: seen },
        Looks::Projected { from, side } => LooksResult::Projected { from: ids::encode(from), side: snake_name(side), view: seen },
    };
    let style = ViewStyleResult {
        scale: view.style.scale.map(|scale| scale.to_string()),
        display_style: view.style.display_style.map(snake_name),
        reference_style: view.style.reference_style.map(snake_name),
        reference_hides_model: view.style.reference_hides_model,
        reference_margin_mm: view.style.reference_margin_mm,
        tangent_edges: view.style.tangent_edges.map(snake_name),
    };
    Ok(ViewRecordResult {
        id: ids::encode(view.id),
        sheet: ids::encode(sheet.id),
        source: ids::encode(view.source),
        component: view.component,
        position_mm: view.position_mm,
        looks,
        style,
    })
}

/// The middle of a sheet's drawing border, or of its paper when it has no border: where a view
/// goes when the call doesn't say.
fn middle(cmd: &str, sheet: &SheetRecord) -> Result<Vec2> {
    let fail = |e: solidcraft_geom::drawing_sheet::Invalid| bad(cmd, e.to_string());
    match &sheet.border {
        Some(profile) => Ok(profile.layout(&sheet.sheet).map_err(fail)?.drawing_border.center()),
        None => Ok(sheet.sheet.size_mm().map_err(fail)? * 0.5),
    }
}

/// Where a view sits `offset` paper millimetres on `side` of `at`, lined up with it: a projection
/// keeps the coordinate it shares with the view it comes from.
fn beside(at: [f64; 2], side: ProjectedSide, offset: f64) -> [f64; 2] {
    match side {
        ProjectedSide::Left => [at[0] - offset, at[1]],
        ProjectedSide::Right => [at[0] + offset, at[1]],
        ProjectedSide::Above => [at[0], at[1] + offset],
        ProjectedSide::Below => [at[0], at[1] - offset],
    }
}

/// Put `view` on `sheet`, in the record's order (by id) and inside the cap.
fn add(cmd: &str, sheet: &mut SheetRecord, view: View) -> Result<()> {
    if sheet.views.len() >= MAX_VIEWS {
        return Err(bad(cmd, format!("the sheet draws {MAX_VIEWS} views already")));
    }
    sheet.views.push(view);
    sheet.views.sort_by_key(|view| view.id);
    Ok(())
}

/// A paper distance a view is placed by: finite and above 0, so a projection sits clear of the
/// view it comes from rather than on top of it or on its other side.
fn offset(cmd: &str, offset_mm: f64) -> Result<f64> {
    if offset_mm.is_finite() && offset_mm > 0.0 {
        Ok(offset_mm)
    } else {
        Err(bad(cmd, "`offset_mm` is a number of paper millimetres above 0: a projected view sits clear of the view it comes from"))
    }
}

/// `style` with every field `clear` names given back to what the view follows.
fn cleared(mut style: ViewStyle, clear: &[ViewStyleField]) -> ViewStyle {
    for field in clear {
        match field {
            ViewStyleField::Scale => style.scale = None,
            ViewStyleField::DisplayStyle => style.display_style = None,
            ViewStyleField::ReferenceStyle => style.reference_style = None,
            ViewStyleField::ReferenceHidesModel => style.reference_hides_model = None,
            ViewStyleField::ReferenceMarginMm => style.reference_margin_mm = None,
            ViewStyleField::TangentEdges => style.tangent_edges = None,
        }
    }
    style
}

/// The style field a call both gives and clears, if any: two instructions for one field, which is
/// a mistake rather than an order to obey.
fn contradicted(request: &ViewSetParams) -> Option<&'static str> {
    let style = &request.style;
    let fields: [(ViewStyleField, bool, &'static str); 6] = [
        (ViewStyleField::Scale, style.scale.is_some(), "scale"),
        (ViewStyleField::DisplayStyle, style.display_style.is_some(), "display_style"),
        (ViewStyleField::ReferenceStyle, style.reference_style.is_some(), "reference_style"),
        (ViewStyleField::ReferenceHidesModel, style.reference_hides_model.is_some(), "reference_hides_model"),
        (ViewStyleField::ReferenceMarginMm, style.reference_margin_mm.is_some(), "reference_margin_mm"),
        (ViewStyleField::TangentEdges, style.tangent_edges.is_some(), "tangent_edges"),
    ];
    fields.iter().find(|(field, given, _)| *given && request.clear.contains(field)).map(|(_, _, name)| *name)
}

fn base(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "drawing.view.base";
    let request: BaseParams = crate::schema::read(cmd, p)?;
    let direction = request.view.0;
    // Checked here rather than left to the record, so the message names the parameter.
    direction.basis().map_err(|e| bad(cmd, e.to_string()))?;
    // The record can't check the component: it never sees the design. The command can, and must,
    // or a view would be saved of something that isn't there. A drawing draws the design it is in
    // for now, so these are its components; a drawing of other designs would ask its source.
    let component = request.component;
    if component != 0 && !s.doc.components.iter().any(|in_design| in_design.id == component) {
        return Err(bad(cmd, format!("the design has no component {component}")));
    }
    let style = request.style.over(ViewStyle::default());
    let id = s.edit_drawing(cmd, |d| {
        let source = d.sources.first().map(|source| source.id);
        let source = source.ok_or_else(|| bad(cmd, "the drawing draws no design, so there is nothing to make a view of"))?;
        let i = sheet_index(cmd, d, request.sheet.as_ref())?;
        let at = match request.position_mm {
            Some(Point2(at)) => at,
            None => middle(cmd, &d.sheets[i])?,
        };
        // A fresh source each time: the ids are 65 bits of entropy, so a removed view's id never
        // comes back in practice, and the record refuses one that is in use.
        let id = d.allocate_id(&mut IdSource::from_entropy()).map_err(|e| bad(cmd, e.to_string()))?;
        let view = View { id, source, component: request.component, position_mm: [at.x, at.y], looks: Looks::Base { view: direction }, style };
        add(cmd, Arc::make_mut(&mut d.sheets[i]), view)?;
        Ok(id)
    })?;
    saved(s, cmd, id)
}

fn projected(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "drawing.view.projected";
    let request: ProjectedParams = crate::schema::read(cmd, p)?;
    let from = ids::decode(&request.from.0).ok_or_else(|| bad(cmd, "`from` must be a view's id (drawing.inspect)"))?;
    let away = offset(cmd, request.offset_mm)?;
    let side = request.side.0;
    let style = request.style.over(ViewStyle::default());
    let id = s.edit_drawing(cmd, |d| {
        let (i, j) = view_index(cmd, d, from)?;
        let parent = d.sheets[i].views[j];
        if !matches!(parent.looks, Looks::Base { .. }) {
            let named = ids::encode(from);
            return Err(bad(cmd, format!("view {named} is projected from another view: a projection is taken from a base view")));
        }
        let id = d.allocate_id(&mut IdSource::from_entropy()).map_err(|e| bad(cmd, e.to_string()))?;
        let view = View { id, position_mm: beside(parent.position_mm, side, away), looks: Looks::Projected { from, side }, style, ..parent };
        add(cmd, Arc::make_mut(&mut d.sheets[i]), view)?;
        Ok(id)
    })?;
    saved(s, cmd, id)
}

fn view_set(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "drawing.view.set";
    let request: ViewSetParams = crate::schema::read(cmd, p)?;
    let id = ids::decode(&request.view.0).ok_or_else(|| bad(cmd, "`view` must be a view's id (drawing.inspect)"))?;
    if let Some(field) = contradicted(&request) {
        return Err(bad(cmd, format!("`style.{field}` is given and cleared in the same call: say one or the other")));
    }
    let away = request.offset_mm.map(|offset_mm| offset(cmd, offset_mm)).transpose()?;
    s.edit_drawing(cmd, |d| {
        let (i, j) = view_index(cmd, d, id)?;
        let view = d.sheets[i].views[j];
        let style = cleared(request.style.over(view.style), &request.clear);
        match view.looks {
            Looks::Base { .. } => {
                if away.is_some() {
                    return Err(bad(cmd, "`offset_mm` is for a projected view: a base view moves by `position_mm`"));
                }
                let at = request.position_mm.map_or(view.position_mm, |Point2(at)| [at.x, at.y]);
                let moved = [at[0] - view.position_mm[0], at[1] - view.position_mm[1]];
                let sheet = Arc::make_mut(&mut d.sheets[i]);
                // Its projections move with it, so they stay lined up and keep their spacing.
                for other in &mut sheet.views {
                    if matches!(other.looks, Looks::Projected { from, .. } if from == id) {
                        other.position_mm = [other.position_mm[0] + moved[0], other.position_mm[1] + moved[1]];
                    }
                }
                sheet.views[j].position_mm = at;
                sheet.views[j].style = style;
            }
            Looks::Projected { from, side } => {
                if request.position_mm.is_some() {
                    let how = "a projected view stays lined up with the view it comes from, so it moves by `offset_mm`";
                    return Err(bad(cmd, format!("`position_mm` is for a base view: {how}")));
                }
                let parent = d.sheets[i].views.iter().find(|other| other.id == from).map(|parent| parent.position_mm);
                let parent = parent.ok_or_else(|| bad(cmd, "the view it is projected from is not on its sheet"))?;
                let sheet = Arc::make_mut(&mut d.sheets[i]);
                if let Some(away) = away {
                    sheet.views[j].position_mm = beside(parent, side, away);
                }
                sheet.views[j].style = style;
            }
        }
        Ok(())
    })?;
    saved(s, cmd, id)
}

/// `drawing.view.standard`: the four views of the standard arrangement, saved on one sheet.
///
/// The only drawing command that reads the model. The scale and the four places come from the
/// part's extents on the four planes (`geom::drawing_view::standard_layout`), so there is nothing
/// to lay out without measuring it first.
fn standard(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "drawing.view.standard";
    let request: StandardParams = crate::schema::read(cmd, p)?;
    let gap = request.gap_mm.unwrap_or(DEFAULT_GAP_MM);

    // The sheet and what it keeps clear, before anything is measured: a sheet that can't take the
    // views is refused without paying for the extents.
    let d = s.drawing(cmd)?;
    let i = sheet_index(cmd, d, request.sheet.as_ref())?;
    let (sheet, projection) = (d.sheets[i].clone(), d.projection);
    let source = d.sources.first().map(|source| source.id);
    let source = source.ok_or_else(|| bad(cmd, "the drawing draws no design, so there is nothing to make views of"))?;
    if !sheet.views.is_empty() {
        return Err(bad(cmd, "the sheet already draws views: delete them first (drawing.view.delete)"));
    }
    let profile = sheet.border.as_ref().ok_or_else(|| bad(cmd, "the sheet has no drawing border for the views to be laid out in"))?;
    let paper = profile.layout(&sheet.sheet).map_err(|e| bad(cmd, e.to_string()))?;
    let clear = match &sheet.title_block {
        Some(block) => {
            let block = block.layout(&paper).map_err(|e| bad(cmd, e.to_string()))?;
            Some(kept_clear(&block, paper.drawing_border))
        }
        None => None,
    };

    if !s.doc.placements_complete() {
        let limits = format!("deeper than {MAX_PLACEMENT_DEPTH} levels or has more than {MAX_PLACEMENTS} placements");
        return Err(bad(cmd, format!("the assembly is {limits}: views of it would leave parts out")));
    }
    let measured = |view: NamedView| crate::drawing_view::extents(s, &view.basis());
    let (front, top, right, iso) =
        (measured(NamedView::Front), measured(NamedView::Top), measured(NamedView::Right), measured(NamedView::IsoFrontRight));
    if front.bodies == 0 {
        return Err(bad(cmd, "the design has no bodies, so there is nothing to draw"));
    }
    let extents = StandardExtents { front: front.model, top: top.model, right: right.model, iso: iso.model };
    let placed = standard_layout(&extents, paper.drawing_border, clear, projection, gap).map_err(|e| bad(cmd, e.to_string()))?;

    // Where each view's centre lands on the paper, taken from the frame the layout built rather
    // than worked out again: a projected frame derives the axis it shares with its parent, so this
    // is the one place that knows both numbers.
    let at = |frame: &ViewFrame, centre: Vec2| {
        let on_paper = frame.paper_map().apply(centre);
        [on_paper.x, on_paper.y]
    };
    let places = [
        at(&placed.front, extents.front.center()),
        at(&placed.top, extents.top.center()),
        at(&placed.right, extents.right.center()),
        at(&placed.iso, extents.iso.center()),
    ];
    let scale = placed.scale;
    let (top_side, right_side) = (placed.top_side, placed.right_side);

    let saved = s.edit_drawing(cmd, |d| {
        let i = sheet_index(cmd, d, request.sheet.as_ref())?;
        if !d.sheets[i].views.is_empty() {
            return Err(bad(cmd, "the sheet already draws views: delete them first (drawing.view.delete)"));
        }
        let mut ids = IdSource::from_entropy();
        let mut id = || d.allocate_id(&mut ids).map_err(|e| bad(cmd, e.to_string()));
        let (front_id, top_id, right_id, iso_id) = (id()?, id()?, id()?, id()?);
        // The two base views carry the scale the layout fitted; the projections follow them.
        let of =
            |id, position_mm, looks, scale| View { id, source, component: 0, position_mm, looks, style: ViewStyle { scale, ..ViewStyle::default() } };
        let views = [
            of(front_id, places[0], Looks::Base { view: ViewDirection::Named(NamedView::Front) }, Some(scale)),
            of(top_id, places[1], Looks::Projected { from: front_id, side: top_side }, None),
            of(right_id, places[2], Looks::Projected { from: front_id, side: right_side }, None),
            of(iso_id, places[3], Looks::Base { view: ViewDirection::Named(NamedView::IsoFrontRight) }, Some(scale)),
        ];
        let sheet = Arc::make_mut(&mut d.sheets[i]);
        for view in views {
            add(cmd, sheet, view)?;
        }
        Ok([front_id, top_id, right_id, iso_id])
    })?;

    let d = s.drawing(cmd)?;
    let on = &d.sheets[sheet_index(cmd, d, request.sheet.as_ref())?];
    let mut views = Vec::with_capacity(saved.len());
    for id in saved {
        let view = on.views.iter().find(|view| view.id == id).ok_or_else(|| bad(cmd, "a view just saved is missing"))?;
        views.push(record_of(cmd, d, on, view)?);
    }
    let fitted = ScaleResult { text: scale.to_string(), factor: scale.factor(), fitted: true };
    reply(&StandardResult { views, scale: fitted, param_warnings: Vec::new() })
}

fn view_delete(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "drawing.view.delete";
    let request: ViewDeleteParams = crate::schema::read(cmd, p)?;
    let id = ids::decode(&request.view.0).ok_or_else(|| bad(cmd, "`view` must be a view's id (drawing.inspect)"))?;
    let gone = s.edit_drawing(cmd, |d| {
        let (i, _) = view_index(cmd, d, id)?;
        let sheet = Arc::make_mut(&mut d.sheets[i]);
        let projections = sheet.views.iter().filter(|view| matches!(view.looks, Looks::Projected { from, .. } if from == id));
        let mut gone = vec![id];
        gone.extend(projections.map(|view| view.id));
        sheet.views.retain(|view| !gone.contains(&view.id));
        Ok(gone)
    })?;
    reply(&ViewDeletedResult { deleted: gone.iter().map(|id| ids::encode(*id)).collect(), param_warnings: Vec::new() })
}

#[cfg(test)]
#[path = "drawing_view_tests.rs"]
mod tests;
