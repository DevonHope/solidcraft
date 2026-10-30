//! Contracts and result types for the commands in `cmd/drawing_view.rs`.
//!
//! The view's own settings are `solidcraft_doc::drawing_view::ViewSettings`, which a saved view
//! (W12) keeps too. The contract lists the same fields, each read by that type's own reader, and
//! `the_contract_lists_the_view_settings` keeps the two lists the same.

use std::borrow::Cow;

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Value, json};
use solidcraft_doc::drawing_view::{TangentEdges, ViewDirection, ViewSettings, ViewStyle};
use solidcraft_doc::ids;
use solidcraft_geom::drawing_sheet::{Orientation, PaperSize, SheetSize};
use solidcraft_geom::drawing_view::{NamedView, ProjectedSide, Scale};
use solidcraft_geom::paper::{ObjectKind, Scene};
use solidcraft_render::VisualStyle;

use super::Point2;
use super::check::whole;

/// The lowest and highest picture resolution a sheet takes, in dots per inch.
pub const DPI_RANGE: std::ops::RangeInclusive<f64> = 25.0..=2400.0;

/// What `drawing.view.snapshot` takes: the view's settings, the page it fits on, and where its
/// centre goes.
#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct SnapshotParams {
    pub view: ViewParam,
    #[serde(deserialize_with = "super::present", skip_serializing_if = "Option::is_none")]
    #[schemars(with = "ScaleParam")]
    pub scale: Option<ScaleParam>,
    pub display_style: StyleParam,
    pub reference_style: StyleParam,
    /// Whether reference parts hide the model's lines behind them (false when not given).
    pub reference_hides_model: bool,
    /// Model millimetres added on every side of the view's extents, so reference parts around
    /// the model show inside its boundary: 0 to 100000 (0 when not given).
    pub reference_margin_mm: f64,
    pub tangent_edges: TangentEdgesParam,
    pub sheet: SheetParam,
    /// Where the view's centre goes on the sheet, in paper millimetres from its lower-left
    /// corner (the sheet's centre when not given).
    #[serde(deserialize_with = "super::present", skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Point2")]
    pub position_mm: Option<Point2>,
}

impl SnapshotParams {
    pub fn settings(&self) -> ViewSettings {
        ViewSettings {
            view: self.view.0,
            scale: self.scale.map(|s| s.0),
            display_style: self.display_style.0,
            reference_style: self.reference_style.0,
            reference_hides_model: self.reference_hides_model,
            reference_margin_mm: self.reference_margin_mm,
            tangent_edges: self.tangent_edges.0,
        }
    }
}

// Each parameter type below reads as the type it wraps, so the contract accepts exactly what the
// view's settings do; it is there for its schema.

/// Which way the view looks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize)]
#[serde(transparent)]
pub struct ViewParam(pub ViewDirection);

impl JsonSchema for ViewParam {
    fn schema_name() -> Cow<'static, str> {
        "DrawingView".into()
    }
    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        let names: Vec<Value> = NamedView::ALL.iter().filter_map(|v| serde_json::to_value(v).ok()).chain([json!("iso")]).collect();
        let vector = json!({"type": "array", "items": {"type": "number"}, "minItems": 3, "maxItems": 3});
        json_schema!({
            "description": "Which way the view looks: a named view (front when not given; iso is iso_front_right), or {direction, up, origin?}: the line of sight, the way up the paper (not along the direction) and the point the view's plane passes through (the world origin when not given).",
            "anyOf": [
                {"type": "string", "enum": names},
                {"type": "object", "properties": {"direction": vector, "up": vector, "origin": vector}, "required": ["direction", "up"], "additionalProperties": false}
            ]
        })
    }
}

/// A drawing scale, paper:model.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(transparent)]
pub struct ScaleParam(pub Scale);

impl JsonSchema for ScaleParam {
    fn schema_name() -> Cow<'static, str> {
        "DrawingScale".into()
    }
    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "description": "The scale as paper:model text, such as \"1:2\" or \"5:1\", from 1:100000 to 1000:1. When not given, the largest ISO 5455 scale at which the view fits the sheet's drawing area.",
            "type": "string",
            "pattern": "^[0-9]+(\\.[0-9]+)?:[0-9]+(\\.[0-9]+)?$"
        })
    }
}

/// How the model draws.
#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize)]
#[serde(transparent)]
pub struct StyleParam(pub VisualStyle);

impl JsonSchema for StyleParam {
    fn schema_name() -> Cow<'static, str> {
        "VisualStyle".into()
    }
    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        let names: Vec<Value> = VisualStyle::ALL.iter().filter_map(|s| serde_json::to_value(s).ok()).collect();
        json_schema!({
            "description": "How the model draws: visible (the default), visible_hidden, shaded, shaded_visible or shaded_visible_hidden.",
            "type": "string",
            "enum": names
        })
    }
}

/// Tangent edges hidden or shown.
#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize)]
#[serde(transparent)]
pub struct TangentEdgesParam(pub TangentEdges);

impl JsonSchema for TangentEdgesParam {
    fn schema_name() -> Cow<'static, str> {
        "TangentEdges".into()
    }
    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({"description": "Edges where faces meet smoothly: hide (the default) or show.", "type": "string", "enum": ["hide", "show"]})
    }
}

/// The page the view fits on: its size, the way it lies and its picture resolution.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct SheetParam {
    pub size: SheetSize,
    pub orientation: Orientation,
    pub dpi: f64,
}

impl Default for SheetParam {
    fn default() -> Self {
        SheetParam { size: SheetSize::Standard(PaperSize::A3), orientation: Orientation::Landscape, dpi: 300.0 }
    }
}

impl JsonSchema for SheetParam {
    fn schema_name() -> Cow<'static, str> {
        "SnapshotSheet".into()
    }
    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "description": "The page the view fits and centres on (A3 landscape at 300 dpi when not given).",
            "type": "object",
            "properties": {
                "size": size_schema(),
                "orientation": {"type": "string", "enum": ["landscape", "portrait"]},
                "dpi": {"type": "number", "minimum": *DPI_RANGE.start(), "maximum": *DPI_RANGE.end()}
            },
            "additionalProperties": false
        })
    }
}

/// A paper size as `SheetSize` reads it: `{"standard": "a3"}`, or `{"custom": {"sides_mm": [w, h]}}`.
pub(super) fn size_schema() -> Value {
    let papers: Vec<Value> = PaperSize::ALL.iter().filter_map(|p| serde_json::to_value(p).ok()).collect();
    let sides = json!({"type": "array", "items": {"type": "number", "exclusiveMinimum": 0}, "minItems": 2, "maxItems": 2});
    let standard = json!({"type": "object", "properties": {"standard": {"type": "string", "enum": papers}}, "required": ["standard"], "additionalProperties": false});
    let custom = json!({
        "type": "object",
        "properties": {"custom": {"type": "object", "properties": {"sides_mm": sides}, "required": ["sides_mm"], "additionalProperties": false}},
        "required": ["custom"],
        "additionalProperties": false
    });
    json!({"anyOf": [standard, custom]})
}

/// A view's extents on its own plane, in model millimetres.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ExtentsResult {
    pub min: [f64; 2],
    pub max: [f64; 2],
    /// How far the display mesh may lie inside a curved face: the largest display tolerance.
    pub error_mm: f64,
    /// `approximate`: measured on the display mesh.
    pub status: &'static str,
    pub provenance: &'static str,
}

/// A view's lines on its own plane, in model millimetres: polylines, so a circle is one run.
#[derive(Debug, Serialize, JsonSchema)]
pub struct LinesResult {
    /// Runs of points; each run is one connected stretch of visible line.
    pub visible: Vec<Vec<[f64; 2]>>,
    /// The stretches a nearer surface covers, for a dashed style.
    pub hidden: Vec<Vec<[f64; 2]>>,
    /// How far a curved face may lie from the lines drawn for it.
    pub chord_mm: f64,
    /// `approximate`: sampled from the display mesh to within `chord_mm`.
    pub status: &'static str,
    pub provenance: &'static str,
}

/// A box on the sheet, in paper millimetres.
#[derive(Debug, Serialize, JsonSchema)]
pub struct BoxResult {
    pub min: [f64; 2],
    pub max: [f64; 2],
}

/// The scale the view was drawn at.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ScaleResult {
    pub text: String,
    pub factor: f64,
    /// Whether the scale was worked out to fit the sheet rather than given.
    pub fitted: bool,
}

/// The view's basis as resolved: its name when it has one, the line of sight and the paper axes.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ViewResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub named: Option<String>,
    pub direction: [f64; 3],
    pub x: [f64; 3],
    pub y: [f64; 3],
    pub origin: [f64; 3],
}

/// What the view was drawn from.
#[derive(Debug, Serialize, JsonSchema)]
pub struct SourceResult {
    pub bodies: usize,
    /// Whether the timeline marker is at the end; otherwise the view shows the rolled-back model.
    pub timeline_at_end: bool,
}

/// What `drawing.view.snapshot` returns.
#[derive(Debug, Serialize, JsonSchema)]
pub struct SnapshotResult {
    pub view: ViewResult,
    pub scale: ScaleResult,
    pub position_mm: [f64; 2],
    /// The sheet the view is placed on, its defaults filled in.
    pub sheet: SheetParam,
    pub display_style: String,
    pub reference_style: String,
    pub extents: ExtentsResult,
    pub lines: LinesResult,
    /// The view's box on the sheet: `position_mm` at its centre, the extents at the scale.
    pub boundary_mm: BoxResult,
    pub source: SourceResult,
    /// In the warn parameter-check mode: what the call's parameters got wrong.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub param_warnings: Vec<String>,
}

/// A sheet's id, as `drawing.inspect` gives it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(transparent)]
#[schemars(description = "A sheet's id, as drawing.inspect gives it.")]
pub struct SheetId(pub String);

impl<'de> Deserialize<'de> for SheetId {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        match Value::deserialize(d)? {
            Value::String(s) if ids::decode(&s).is_some() => Ok(SheetId(s)),
            _ => Err(D::Error::custom("must be a sheet's id (drawing.inspect)")),
        }
    }
}

/// A view's id, as `drawing.inspect` gives it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(transparent)]
#[schemars(description = "A view's id, as drawing.inspect gives it.")]
pub struct ViewId(pub String);

impl<'de> Deserialize<'de> for ViewId {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        match Value::deserialize(d)? {
            Value::String(s) if ids::decode(&s).is_some() => Ok(ViewId(s)),
            _ => Err(D::Error::custom("must be a view's id (drawing.inspect)")),
        }
    }
}

/// Which side of its base view a projected view sits on.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(transparent)]
pub struct SideParam(pub ProjectedSide);

impl JsonSchema for SideParam {
    fn schema_name() -> Cow<'static, str> {
        "ProjectedSide".into()
    }
    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "description": "Which side of the base view it sits on: left, right, above or below. In third angle a view shows the side of the model it sits on, in first angle the side away from it (ISO 5456-2), so the drawing's projection decides which way it looks.",
            "type": "string",
            "enum": ["left", "right", "above", "below"]
        })
    }
}

/// What a saved view draws differently from what it follows. A field left out is left as it is;
/// `drawing.view.set`'s `clear` gives one back to what it follows.
#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct ViewStyleParams {
    #[serde(deserialize_with = "super::present", skip_serializing_if = "Option::is_none")]
    #[schemars(with = "ScaleParam")]
    pub scale: Option<ScaleParam>,
    #[serde(deserialize_with = "super::present", skip_serializing_if = "Option::is_none")]
    #[schemars(with = "StyleParam")]
    pub display_style: Option<StyleParam>,
    #[serde(deserialize_with = "super::present", skip_serializing_if = "Option::is_none")]
    #[schemars(with = "StyleParam")]
    pub reference_style: Option<StyleParam>,
    /// Whether reference parts hide the model's lines behind them.
    #[serde(deserialize_with = "super::present", skip_serializing_if = "Option::is_none")]
    #[schemars(with = "bool")]
    pub reference_hides_model: Option<bool>,
    /// Model millimetres added on every side of the view's extents, so reference parts around the
    /// model show inside its boundary: 0 to 100000.
    #[serde(deserialize_with = "super::present", skip_serializing_if = "Option::is_none")]
    #[schemars(with = "f64")]
    pub reference_margin_mm: Option<f64>,
    #[serde(deserialize_with = "super::present", skip_serializing_if = "Option::is_none")]
    #[schemars(with = "TangentEdgesParam")]
    pub tangent_edges: Option<TangentEdgesParam>,
}

impl ViewStyleParams {
    /// These fields over `style`, the view's style now: what the call gives, else what it had.
    pub fn over(&self, style: ViewStyle) -> ViewStyle {
        ViewStyle {
            scale: self.scale.map(|s| s.0).or(style.scale),
            display_style: self.display_style.map(|s| s.0).or(style.display_style),
            reference_style: self.reference_style.map(|s| s.0).or(style.reference_style),
            reference_hides_model: self.reference_hides_model.or(style.reference_hides_model),
            reference_margin_mm: self.reference_margin_mm.or(style.reference_margin_mm),
            tangent_edges: self.tangent_edges.map(|t| t.0).or(style.tangent_edges),
        }
    }
}

/// A style field a view gives back to what it follows (`drawing.view.set`'s `clear`).
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ViewStyleField {
    Scale,
    DisplayStyle,
    ReferenceStyle,
    ReferenceHidesModel,
    ReferenceMarginMm,
    TangentEdges,
}

/// Add a base view — one that looks where it says — to a sheet of the open drawing. One undo step
/// of the drawing; nothing is evaluated, because a view's lines are worked out when it is drawn.
#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct BaseParams {
    /// The sheet to draw it on. A drawing with one sheet may leave it out.
    #[serde(deserialize_with = "super::present", skip_serializing_if = "Option::is_none")]
    #[schemars(with = "SheetId")]
    pub sheet: Option<SheetId>,
    pub view: ViewParam,
    /// The component of the drawing's design it draws, as the design writes it (0, the design
    /// itself, when not given).
    pub component: u64,
    /// Where the view's centre goes, in paper millimetres from the sheet's lower-left corner (the
    /// centre of the sheet's drawing border when not given).
    #[serde(deserialize_with = "super::present", skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Point2")]
    pub position_mm: Option<Point2>,
    pub style: ViewStyleParams,
}

/// Add a view projected from a base view of the same sheet: a quarter turn from it, lined up with
/// it, drawing the same component. One undo step of the drawing.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct ProjectedParams {
    /// The base view it is projected from.
    pub from: ViewId,
    pub side: SideParam,
    /// How far from that view's centre, in paper millimetres along `side`: a projected view keeps
    /// the other coordinate of the view it comes from, so it stays lined up with it.
    /// `drawing.view.standard` works the spacing out from the model instead.
    pub offset_mm: f64,
    #[serde(default)]
    pub style: ViewStyleParams,
}

/// Change a saved view: move it, scale it, or draw it differently. Keys not given stay as they
/// are. One undo step of the drawing.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct ViewSetParams {
    pub view: ViewId,
    /// Where a base view's centre goes, in paper millimetres from the sheet's lower-left corner.
    /// Its projections move with it. A projected view takes `offset_mm` instead.
    #[serde(default, deserialize_with = "super::present", skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Point2")]
    pub position_mm: Option<Point2>,
    /// How far a projected view sits from the view it comes from, in paper millimetres along its
    /// side. A base view takes `position_mm` instead.
    #[serde(default, deserialize_with = "super::present", skip_serializing_if = "Option::is_none")]
    #[schemars(with = "f64")]
    pub offset_mm: Option<f64>,
    #[serde(default)]
    pub style: ViewStyleParams,
    /// Style fields to give back to what the view follows: the view it is projected from, or the
    /// drawing's defaults where it follows none. A field given in `style` as well is refused.
    #[serde(default)]
    pub clear: Vec<ViewStyleField>,
}

/// Delete a saved view. A base view takes the views projected from it with it. One undo step of
/// the drawing.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct ViewDeleteParams {
    pub view: ViewId,
}

/// Write a sheet of the open drawing to a file. Read-only: the drawing is not changed.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct ExportParams {
    pub format: ExportFormat,
    /// The file to write. An existing file is not replaced unless `overwrite` says so.
    pub path: String,
    /// The sheet to write. A drawing with one sheet may leave it out.
    #[serde(default, deserialize_with = "super::present", skip_serializing_if = "Option::is_none")]
    #[schemars(with = "SheetId")]
    pub sheet: Option<SheetId>,
    /// Replace a file that is already at `path`. Without it, writing onto one is refused.
    #[serde(default)]
    pub overwrite: bool,
}

/// The file a sheet is written as.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ExportFormat {
    Svg,
}

impl<'de> Deserialize<'de> for ExportFormat {
    /// What the handler takes, in its words, and naming the formats that are coming rather than
    /// calling them unknown.
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let later = ["dxf", "pdf", "png"];
        match Value::deserialize(d)? {
            Value::String(name) if name == "svg" => Ok(ExportFormat::Svg),
            Value::String(name) if later.contains(&name.as_str()) => {
                Err(D::Error::custom(whole(format!("not supported yet: {name} comes with the drawing's other exports; `format` is \"svg\" today"))))
            }
            _ => Err(D::Error::custom(whole("`format` is \"svg\""))),
        }
    }
}

impl JsonSchema for ExportFormat {
    fn schema_name() -> Cow<'static, str> {
        "DrawingExportFormat".into()
    }
    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "description": "The file to write the sheet as. Only svg today; dxf, pdf and png follow.",
            "type": "string",
            "enum": ["svg"]
        })
    }
}

/// What `drawing.export` returns: the file it wrote and what went into it.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ExportResult {
    pub path: String,
    pub format: String,
    /// How long the file is, in bytes.
    pub bytes: usize,
    /// What the writer had to move onto its grid to draw the sheet: nothing, on a drawing that
    /// was laid out in whole nanometres.
    pub snapped: SnappedResult,
    /// Each view the sheet draws, as `drawing.sheet.paper` reports them.
    pub views: Vec<PaperViewResult>,
    /// In the warn parameter-check mode: what the call's parameters got wrong.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub param_warnings: Vec<String>,
}

/// What a writer moved onto its grid.
#[derive(Debug, Serialize, JsonSchema)]
pub struct SnappedResult {
    /// Line widths under half a grid step, drawn one step wide.
    pub widths: usize,
    /// Pieces too short for the grid, merged into the next.
    pub pieces: usize,
    /// Dashes and gaps under half a step, drawn one step long.
    pub dashes: usize,
}

/// Work out the marks a sheet of the open drawing puts on paper. Read-only.
#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct PaperParams {
    /// The sheet to work out. A drawing with one sheet may leave it out.
    #[serde(deserialize_with = "super::present", skip_serializing_if = "Option::is_none")]
    #[schemars(with = "SheetId")]
    pub sheet: Option<SheetId>,
}

/// A sheet's marks in paint order, in the paper format the SVG, DXF, PDF and PNG writers read
/// (`solidcraft_geom::paper::Scene`). Described here rather than mirrored field for field: the
/// format is defined and round-tripped in `geom`, and a second definition would drift from it.
#[derive(Debug, Serialize)]
#[serde(transparent)]
pub struct SceneResult(pub Scene);

impl JsonSchema for SceneResult {
    fn schema_name() -> Cow<'static, str> {
        "PaperScene".into()
    }
    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        let kinds: Vec<Value> = ObjectKind::ALL.iter().filter_map(|kind| serde_json::to_value(kind).ok()).collect();
        let point = json!({"type": "array", "items": {"type": "number"}, "minItems": 2, "maxItems": 2});
        let stroke = json!({
            "description": "A line `width_mm` wide along `path`, each piece starting where the one before ends; with `pattern` it is dashed.",
            "type": "object",
            "properties": {
                "kind": {"const": "stroke"},
                "path": {"type": "array", "items": {"type": "object", "description": "A piece of the path: a line {line: {a, b}} or an arc, as `geom::Seg2` writes it."}},
                "closed": {"type": "boolean"},
                "width_mm": {"type": "number", "exclusiveMinimum": 0},
                "pattern": {"type": "object", "properties": {"lengths_mm": {"type": "array", "items": {"type": "number"}}, "phase_mm": {"type": "number"}}}
            },
            "required": ["kind", "path", "closed", "width_mm"]
        });
        let fill = json!({
            "description": "A region painted solid, as `geom::Region2` writes it.",
            "type": "object",
            "properties": {"kind": {"const": "fill"}, "region": {"type": "object"}},
            "required": ["kind", "region"]
        });
        json_schema!({
            "description": "The sheet's marks in paint order: one group per object that drew any, each group's items painted in order, and the groups painted in the order they are given. The same format the paper writers read.",
            "type": "object",
            "properties": {
                "size_mm": point,
                "groups": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "object": {"description": "The id of the object that drew these items, as a number.", "type": "integer"},
                            "kind": {"type": "string", "enum": kinds},
                            "items": {"type": "array", "items": {"anyOf": [stroke, fill]}}
                        },
                        "required": ["object", "kind", "items"]
                    }
                }
            },
            "required": ["size_mm", "groups"]
        })
    }
}

/// What `drawing.sheet.paper` returns: the sheet's marks, and what each view settled on.
#[derive(Debug, Serialize, JsonSchema)]
pub struct PaperResult {
    pub scene: SceneResult,
    /// Each view the sheet draws, in the order its groups appear in the scene.
    pub views: Vec<PaperViewResult>,
    /// In the warn parameter-check mode: what the call's parameters got wrong.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub param_warnings: Vec<String>,
}

/// What one view of the sheet was drawn at.
#[derive(Debug, Serialize, JsonSchema)]
pub struct PaperViewResult {
    pub id: String,
    pub scale: ScaleResult,
    /// How far a curved face may lie from the lines drawn for it, in model millimetres.
    pub chord_mm: f64,
    /// Runs of visible line, and of hidden line where the view's style draws them.
    pub visible_runs: usize,
    pub hidden_runs: usize,
}

/// Draw the lines of every view of the open drawing that isn't current, on one sheet or on all
/// of them, and keep them for `drawing.sheet.paper` and `drawing.export`. A view that is current
/// is kept as it is. Saves nothing: a drawing records no lines.
#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct UpdateParams {
    /// The sheet to update. Every sheet of the drawing when not given.
    #[serde(deserialize_with = "super::present", skip_serializing_if = "Option::is_none")]
    #[schemars(with = "SheetId")]
    pub sheet: Option<SheetId>,
}

/// Say which views of the open drawing are current, out of date or not drawn yet, on one sheet or
/// on all of them. Draws nothing.
#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct StatusParams {
    /// The sheet to look at. Every sheet of the drawing when not given.
    #[serde(deserialize_with = "super::present", skip_serializing_if = "Option::is_none")]
    #[schemars(with = "SheetId")]
    pub sheet: Option<SheetId>,
}

/// How fresh a view's lines are.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FreshnessResult {
    /// Its lines are the ones it would draw now.
    Current,
    /// It was drawn, but the design, or the view's own direction or scale, has changed since.
    OutOfDate,
    /// It has not been drawn since the design was opened.
    NotDrawn,
}

impl From<crate::drawing_view::Freshness> for FreshnessResult {
    fn from(freshness: crate::drawing_view::Freshness) -> Self {
        match freshness {
            crate::drawing_view::Freshness::Current => FreshnessResult::Current,
            crate::drawing_view::Freshness::OutOfDate => FreshnessResult::OutOfDate,
            crate::drawing_view::Freshness::NotDrawn => FreshnessResult::NotDrawn,
        }
    }
}

/// What `drawing.update` returns.
#[derive(Debug, Serialize, JsonSchema)]
pub struct UpdateResult {
    /// Each sheet updated, in the drawing's order.
    pub sheets: Vec<UpdatedSheetResult>,
    /// How many views had their lines drawn now.
    pub drawn: usize,
    /// How many views were current already, and kept their lines.
    pub kept: usize,
    /// In the warn parameter-check mode: what the call's parameters got wrong.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub param_warnings: Vec<String>,
}

/// One sheet `drawing.update` updated.
#[derive(Debug, Serialize, JsonSchema)]
pub struct UpdatedSheetResult {
    /// The sheet's id.
    pub id: String,
    /// Each of its views, in the sheet's order.
    pub views: Vec<UpdatedViewResult>,
}

/// One view `drawing.update` looked at.
#[derive(Debug, Serialize, JsonSchema)]
pub struct UpdatedViewResult {
    pub id: String,
    /// How fresh it was before the update; it is current after it.
    pub was: FreshnessResult,
    pub scale: ScaleResult,
    /// How far a curved face may lie from the lines drawn for it, in model millimetres.
    pub chord_mm: f64,
    /// Runs of visible line, and of hidden line where the view's style draws them.
    pub visible_runs: usize,
    pub hidden_runs: usize,
}

/// What `drawing.status` returns.
#[derive(Debug, Serialize, JsonSchema)]
pub struct StatusResult {
    /// Each sheet looked at, in the drawing's order.
    pub sheets: Vec<StatusSheetResult>,
    /// How many of the views are current, out of date and not drawn yet.
    pub current: usize,
    pub out_of_date: usize,
    pub not_drawn: usize,
    /// In the warn parameter-check mode: what the call's parameters got wrong.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub param_warnings: Vec<String>,
}

/// One sheet `drawing.status` looked at.
#[derive(Debug, Serialize, JsonSchema)]
pub struct StatusSheetResult {
    /// The sheet's id.
    pub id: String,
    /// Each of its views, in the sheet's order.
    pub views: Vec<StatusViewResult>,
}

/// How fresh one view is.
#[derive(Debug, Serialize, JsonSchema)]
pub struct StatusViewResult {
    pub id: String,
    pub status: FreshnessResult,
}

/// Lay the four standard views out on a sheet of the open drawing: the front view, the top and
/// right views projected from it and lined up with it, and an isometric view in the cell they
/// leave free, all at the largest preset scale that fits. One undo step of the drawing.
#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct StandardParams {
    /// The sheet to lay them out on. A drawing with one sheet may leave it out.
    #[serde(deserialize_with = "super::present", skip_serializing_if = "Option::is_none")]
    #[schemars(with = "SheetId")]
    pub sheet: Option<SheetId>,
    /// The space left between neighbouring views, in paper millimetres: 0 to 1000, and 10 when not
    /// given.
    #[serde(deserialize_with = "super::present", skip_serializing_if = "Option::is_none")]
    #[schemars(with = "f64")]
    pub gap_mm: Option<f64>,
}

/// What `drawing.view.standard` returns: the views it saved and the scale it fitted them at.
#[derive(Debug, Serialize, JsonSchema)]
pub struct StandardResult {
    /// The four views: the front view, the top and right views projected from it, and the
    /// isometric view, in that order.
    pub views: Vec<ViewRecordResult>,
    /// The scale all four are drawn at, worked out to fit them on the sheet.
    pub scale: ScaleResult,
    /// In the warn parameter-check mode: what the call's parameters got wrong.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub param_warnings: Vec<String>,
}

/// A saved view's style, as the record keeps it: only the fields the view sets itself.
#[derive(Debug, Default, Serialize, JsonSchema)]
pub struct ViewStyleResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_style: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_style: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_hides_model: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_margin_mm: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tangent_edges: Option<String>,
}

/// Which way a saved view looks: by itself, or projected from a base view of its sheet. Either
/// way the basis it draws with is given, so a caller needn't work a projection out.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LooksResult {
    Base { view: ViewResult },
    Projected { from: String, side: String, view: ViewResult },
}

/// A view saved on a sheet, as the drawing keeps it. Nothing measured from the design is here:
/// its extents and lines come from `drawing.view.snapshot` and the sheet's paper scene.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ViewRecordResult {
    pub id: String,
    /// The sheet it is drawn on.
    pub sheet: String,
    /// The drawing's design it draws.
    pub source: String,
    pub component: u64,
    /// Its centre on the paper, in millimetres from the sheet's lower-left corner.
    pub position_mm: [f64; 2],
    pub looks: LooksResult,
    #[serde(skip_serializing_if = "ViewStyleResult::is_empty")]
    pub style: ViewStyleResult,
}

impl ViewStyleResult {
    /// Nothing of its own: the view draws just what it follows.
    pub fn is_empty(&self) -> bool {
        let fields = [self.scale.is_none(), self.display_style.is_none(), self.reference_style.is_none(), self.tangent_edges.is_none()];
        fields.iter().all(|set| *set) && self.reference_hides_model.is_none() && self.reference_margin_mm.is_none()
    }
}

/// What `drawing.view.base`, `drawing.view.projected` and `drawing.view.set` return: the view as
/// the drawing now keeps it.
#[derive(Debug, Serialize, JsonSchema)]
pub struct SavedViewResult {
    pub view: ViewRecordResult,
    /// In the warn parameter-check mode: what the call's parameters got wrong.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub param_warnings: Vec<String>,
}

/// What `drawing.view.delete` returns: the view, and the projections that went with it.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ViewDeletedResult {
    /// The views deleted, by id: the one asked for first.
    pub deleted: Vec<String>,
    /// In the warn parameter-check mode: what the call's parameters got wrong.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub param_warnings: Vec<String>,
}

#[cfg(test)]
#[path = "drawing_view_tests.rs"]
mod tests;
