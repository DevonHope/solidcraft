//! Contracts and result types for the commands in `cmd/drawing.rs`.
//!
//! The handlers still read their parameters themselves. Each parameter type here reads its key
//! the way its handler does and fails in the handler's words, so the check and the handler say the
//! same thing about the same mistake.

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};
use solidcraft_doc::ids;
use solidcraft_doc::metadata::MAX_KEYS;
use solidcraft_geom::cell::Projection;

use super::check::whole;
use super::drawing_view::{SheetId, ViewRecordResult};
use super::present;

/// Add a drawing of this design, with one ISO A3 landscape sheet and the ISO border, and open
/// it. One undo step of the model.
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct CreateParams {
    /// The drawing's name. Default: the design's name.
    #[serde(default, deserialize_with = "present", skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String")]
    pub name: Option<String>,
    /// The drawing's content language, a BCP 47 tag such as `en` or `de-CH`. Default: en.
    #[serde(default, deserialize_with = "present", skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String")]
    pub language: Option<String>,
    /// The unit the drawing gives lengths in: `follow_model` (each view takes its design's own
    /// units) or one length unit such as `mm` or `in`. Default: follow_model.
    #[serde(default, deserialize_with = "units", skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String")]
    pub units: Option<String>,
}

/// List the design's drawings.
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct ListParams {}

/// Open one of the design's drawings. The drawing commands then change it, and the model's wait
/// until `drawing.close`.
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct OpenParams {
    pub drawing: DrawingId,
}

/// Close the open drawing, if one is open.
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct CloseParams {}

/// Delete one of the design's drawings, closing it first if it is open. One undo step of the
/// model.
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct DeleteParams {
    pub drawing: DrawingId,
}

/// Describe the open drawing. Read-only.
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct InspectParams {}

/// Change the open drawing's own settings. Keys not given stay as they are. One undo step of the
/// drawing.
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct SetParams {
    /// The drawing's new name.
    #[serde(default, deserialize_with = "present", skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String")]
    pub name: Option<String>,
    /// The drawing's content language, a BCP 47 tag such as `en` or `de-CH`.
    #[serde(default, deserialize_with = "present", skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String")]
    pub language: Option<String>,
    /// The unit the drawing gives lengths in: `follow_model` (each view takes its design's own
    /// units) or one length unit such as `mm` or `in`.
    #[serde(default, deserialize_with = "units", skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String")]
    pub units: Option<String>,
    /// The projection its views are drawn in: `first` or `third` angle.
    #[serde(default, deserialize_with = "present", skip_serializing_if = "Option::is_none")]
    #[schemars(with = "ProjectionParam")]
    pub projection: Option<ProjectionParam>,
}

/// Put a title block on a sheet of the open drawing, or take its block off. One undo step of the
/// drawing.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct TitleBlockSetParams {
    /// The sheet, by id (`drawing.inspect`). A drawing with one sheet may leave it out.
    #[serde(default, deserialize_with = "present", skip_serializing_if = "Option::is_none")]
    #[schemars(with = "SheetId")]
    pub sheet: Option<SheetId>,
    pub block: BlockParam,
}

/// Which title block a sheet gets.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BlockParam {
    /// SolidCraft's own 180 x 54 mm block, in the corner of the sheet's drawing border.
    BuiltIn,
    /// No title block on the sheet.
    None,
}

/// The projection a drawing's views are drawn in.
#[derive(Clone, Copy, Debug)]
pub struct ProjectionParam(pub Projection);

impl<'de> Deserialize<'de> for ProjectionParam {
    /// What the handler's `projection` reads, in its words.
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        match Value::deserialize(d)? {
            Value::String(text) if text == "first" => Ok(ProjectionParam(Projection::First)),
            Value::String(text) if text == "third" => Ok(ProjectionParam(Projection::Third)),
            _ => Err(D::Error::custom(whole("`projection` is \"first\" or \"third\""))),
        }
    }
}

impl JsonSchema for ProjectionParam {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "Projection".into()
    }
    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "description": "The projection a drawing's views are drawn in (ISO 5456-2): first angle, where a view sits on the side away from the one it looks from, or third angle, where it sits on the side it looks from.",
            "type": "string",
            "enum": ["first", "third"]
        })
    }
}

/// Read the open drawing's properties. Read-only.
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct PropertiesParams {}

/// Change the open drawing's properties, by the design properties' rules
/// (`document.metadata.set`): give `key` and `value`, or `values`. All or nothing, one undo step
/// of the drawing.
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct PropertiesSetParams {
    /// The property to change, such as `part_number` or another tool's key. Ignored when `values`
    /// is given.
    #[serde(default, deserialize_with = "present", skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String")]
    pub key: Option<String>,
    /// The key's new value: text for SolidCraft's own keys, any JSON value for another tool's;
    /// null removes the key. Required with `key`.
    #[serde(default)]
    pub value: Option<Value>,
    /// Several changes at once: an object of keys and their new values, as `value` takes them, at
    /// most 256 keys.
    #[serde(default, deserialize_with = "values", skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Map<String, Value>")]
    pub values: Option<Map<String, Value>>,
}

/// Remove properties from the open drawing: give `key`, or `keys`. One undo step of the drawing.
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct PropertiesRemoveParams {
    /// The property to remove. Ignored when `keys` is given.
    #[serde(default, deserialize_with = "present", skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String")]
    pub key: Option<String>,
    /// The properties to remove, at most 256.
    #[serde(default, deserialize_with = "keys", skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Vec<String>")]
    pub keys: Option<Vec<String>>,
}

/// A drawing's id, as `drawing.list` gives it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(transparent)]
#[schemars(description = "A drawing's id, as drawing.list gives it.")]
pub struct DrawingId(pub String);

impl<'de> Deserialize<'de> for DrawingId {
    /// What the handlers' `named` reads: text that decodes as an id.
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        match Value::deserialize(d)? {
            Value::String(s) if ids::decode(&s).is_some() => Ok(DrawingId(s)),
            _ => Err(D::Error::custom("must be a drawing's id (drawing.list)")),
        }
    }
}

/// `units` as the handlers' `units` reads it: text, else its message.
fn units<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    match Value::deserialize(d)? {
        Value::String(s) => Ok(Some(s)),
        _ => Err(D::Error::custom(whole("`units` is \"follow_model\" or a length unit such as \"mm\""))),
    }
}

/// `values` as `metadata::setting` reads it: an object of at most [`MAX_KEYS`] keys.
fn values<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Map<String, Value>>, D::Error> {
    match Value::deserialize(d)? {
        Value::Object(values) if values.len() <= MAX_KEYS => Ok(Some(values)),
        Value::Object(_) => Err(D::Error::custom(whole(format!("`values` can hold at most {MAX_KEYS} keys")))),
        _ => Err(D::Error::custom(whole("give `key` and `value`, or `values`: {key: value, …}"))),
    }
}

/// `keys` as `metadata::removing` reads it: a list of at most [`MAX_KEYS`] keys.
fn keys<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Vec<String>>, D::Error> {
    match Value::deserialize(d)? {
        Value::Array(keys) if keys.len() <= MAX_KEYS => keys
            .into_iter()
            .map(|k| match k {
                Value::String(k) => Some(k),
                _ => None,
            })
            .collect::<Option<Vec<String>>>()
            .map(Some)
            .ok_or_else(|| D::Error::custom("must be a list of keys")),
        _ => Err(D::Error::custom(whole(format!("give `key`, or `keys`: a list of at most {MAX_KEYS} keys")))),
    }
}

/// A drawing as `drawing.inspect` describes it, which `drawing.create`, `drawing.open` and
/// `drawing.set` return too.
#[derive(Debug, Serialize, JsonSchema)]
pub struct InspectResult {
    /// The drawing's id.
    pub id: String,
    pub name: String,
    /// The unit the drawing gives lengths in.
    pub units: UnitsResult,
    /// The drawing's content language, a BCP 47 tag.
    pub language: String,
    /// The projection its views are drawn in: `first` or `third` angle.
    pub projection: String,
    /// The drawing's properties as saved, other tools' keys too.
    pub properties: Map<String, Value>,
    /// The designs the drawing draws.
    pub sources: Vec<SourceResult>,
    pub sheets: Vec<SheetResult>,
    /// In the warn parameter-check mode: what the call's parameters got wrong.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub param_warnings: Vec<String>,
}

/// A drawing's length unit: `follow_model`, or `{"fixed": {"unit": "mm"}}`.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UnitsResult {
    /// Each view takes its design's own units.
    FollowModel,
    /// One length unit for the whole drawing.
    Fixed { unit: String },
}

/// One design a drawing draws.
#[derive(Debug, Serialize, JsonSchema)]
pub struct SourceResult {
    /// The source's id in the drawing.
    pub id: String,
}

/// One sheet of a drawing.
#[derive(Debug, Serialize, JsonSchema)]
pub struct SheetResult {
    /// The sheet's id.
    pub id: String,
    pub name: String,
    /// The paper's width and height in mm, as it lies.
    pub size_mm: [f64; 2],
    pub orientation: OrientationResult,
    /// The resolution the sheet's pictures render at, in dots per inch.
    pub dpi: f64,
    /// The component the sheet documents, which its title block reads, or null.
    pub documents: Option<SheetComponentResult>,
    pub excluded: ExcludedResult,
    /// The title block drawn in the corner of its drawing border, or null.
    pub title_block: Option<TitleBlockResult>,
    /// The views it draws, by id.
    pub views: Vec<ViewRecordResult>,
}

/// A sheet's title block as it is saved: a copy of the block it was made from, so a later built-in
/// never changes it.
#[derive(Debug, Serialize, JsonSchema)]
pub struct TitleBlockResult {
    /// The block's width and height in mm.
    pub size_mm: [f64; 2],
    pub outline_line_mm: f64,
    pub cell_line_mm: f64,
    /// Its cells' names, in the order the block keeps them.
    pub cells: Vec<String>,
}

/// Which way a sheet lies. Landscape puts the long side across.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OrientationResult {
    Landscape,
    Portrait,
}

/// A component of one of the drawing's designs.
#[derive(Debug, Serialize, JsonSchema)]
pub struct SheetComponentResult {
    /// The source's id.
    pub source: String,
    /// The component's id in that design (0: the design itself).
    pub component: u64,
}

/// What a sheet is left out of.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ExcludedResult {
    /// The sheet count ("sheet 2 of 3" on the other sheets).
    pub count: bool,
    /// Printing and exporting every sheet.
    pub print: bool,
}

/// What `drawing.list` returns.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ListResult {
    /// The design's drawings, by name, then id.
    pub drawings: Vec<ListedDrawing>,
    /// In the warn parameter-check mode: what the call's parameters got wrong.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub param_warnings: Vec<String>,
}

/// One drawing in `drawing.list`.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ListedDrawing {
    pub id: String,
    pub name: String,
    /// How many sheets it has.
    pub sheets: usize,
    /// Whether it is the drawing open now.
    pub open: bool,
}

/// What `drawing.close` returns.
#[derive(Debug, Serialize, JsonSchema)]
pub struct CloseResult {
    /// The drawing's id, or null when none was open.
    pub closed: Option<String>,
    /// In the warn parameter-check mode: what the call's parameters got wrong.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub param_warnings: Vec<String>,
}

/// What `drawing.delete` returns.
#[derive(Debug, Serialize, JsonSchema)]
pub struct DeleteResult {
    /// The deleted drawing's id.
    pub deleted: String,
    /// In the warn parameter-check mode: what the call's parameters got wrong.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub param_warnings: Vec<String>,
}

/// What `drawing.properties` returns.
#[derive(Debug, Serialize, JsonSchema)]
pub struct PropertiesResult {
    /// The drawing's properties as saved, other tools' keys too.
    pub metadata: Map<String, Value>,
    /// SolidCraft's own keys (project, part_number, revision, author, organization, description
    /// and license), each its text, or null when it is not set.
    pub properties: BTreeMap<String, Option<String>>,
    /// In the warn parameter-check mode: what the call's parameters got wrong.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub param_warnings: Vec<String>,
}

/// What `drawing.properties.set` returns.
#[derive(Debug, Serialize, JsonSchema)]
pub struct PropertiesSetResult {
    /// The drawing's properties after the change.
    pub metadata: Map<String, Value>,
    /// The keys whose values changed.
    pub changed: Vec<String>,
    /// In the warn parameter-check mode: what the call's parameters got wrong.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub param_warnings: Vec<String>,
}

/// What `drawing.properties.remove` returns.
#[derive(Debug, Serialize, JsonSchema)]
pub struct PropertiesRemoveResult {
    /// The drawing's properties after the change.
    pub metadata: Map<String, Value>,
    /// The keys that were there and are now gone.
    pub removed: Vec<String>,
    /// In the warn parameter-check mode: what the call's parameters got wrong.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub param_warnings: Vec<String>,
}

#[cfg(test)]
#[path = "drawing_tests.rs"]
mod tests;
