//! Drawing commands (#3890, K4): make, list, open, close and delete a design's drawings, and read
//! and change the open drawing's own settings. A drawing lives in its design's file
//! ([`solidcraft_doc::Document::drawings`]); while one is open, the drawing commands change it and
//! the model's wait ([`super::Part`]). Each change of a drawing is one undo step of that drawing,
//! and evaluates nothing; making and deleting one are steps of the model, whose list of drawings
//! they change.

use std::collections::BTreeSet;
use std::sync::Arc;

use serde::Serialize;
use serde_json::{Map, Value};
use solidcraft_doc::drawing::{self, Drawing, Source, Units};
use solidcraft_doc::ids::{self, IdSource};
use solidcraft_doc::metadata::{self, Owner};
use solidcraft_geom::drawing_sheet::{DrawingBorderProfile, Orientation, PaperSize, Sheet};

use super::CommandSpec;
use crate::params::{bad, str_};
use crate::schema::drawing::{
    CloseParams, CloseResult, CreateParams, DeleteParams, DeleteResult, ExcludedResult, InspectParams, InspectResult, ListParams, ListResult,
    ListedDrawing, OpenParams, OrientationResult, PropertiesParams, PropertiesRemoveParams, PropertiesRemoveResult, PropertiesResult,
    PropertiesSetParams, PropertiesSetResult, SetParams, SheetComponentResult, SheetResult, SourceResult, UnitsResult,
};
use crate::schema::{TypedContract, output_schema_for};
use crate::{Result, Session};

const CREATE: TypedContract<CreateParams> = TypedContract::new();
const LIST: TypedContract<ListParams> = TypedContract::new();
const OPEN: TypedContract<OpenParams> = TypedContract::new();
const CLOSE: TypedContract<CloseParams> = TypedContract::new();
const DELETE: TypedContract<DeleteParams> = TypedContract::new();
const INSPECT: TypedContract<InspectParams> = TypedContract::new();
const SET: TypedContract<SetParams> = TypedContract::new();
const PROPERTIES: TypedContract<PropertiesParams> = TypedContract::new();
const PROPERTIES_SET: TypedContract<PropertiesSetParams> = TypedContract::new();
const PROPERTIES_REMOVE: TypedContract<PropertiesRemoveParams> = TypedContract::new();

pub static COMMANDS: &[CommandSpec] = &[
    CommandSpec::new("drawing.create", "Create Drawing", crate::checked!("drawing.create", create))
        .any_part()
        .icon("new")
        .doc_only()
        .contract(&CREATE)
        .output(output_schema_for::<InspectResult>)
        .params(
            "name?: the drawing's name (default: the design's); language?: the drawing's content language, a BCP 47 tag (default en); units?: \"follow_model\" (default) or one length unit such as \"mm\". Adds a drawing of this design, one ISO A3 landscape sheet with the ISO border, and opens it (one undo step of the model) → as drawing.inspect",
        ),
    CommandSpec::new("drawing.list", "List Drawings", crate::checked!("drawing.list", list))
        .any_part()
        .icon("list")
        .noundo()
        .doc_only()
        .contract(&LIST)
        .output(output_schema_for::<ListResult>)
        .params("→ drawings: [{id, name, sheets (how many), open}], by name, then id"),
    CommandSpec::new("drawing.open", "Open Drawing", crate::checked!("drawing.open", open))
        .any_part()
        .icon("open")
        .noundo()
        .doc_only()
        .contract(&OPEN)
        .output(output_schema_for::<InspectResult>)
        .params("drawing: id (from drawing.list) → as drawing.inspect. The drawing commands then change it; the model's wait until drawing.close"),
    CommandSpec::new("drawing.close", "Close Drawing", crate::checked!("drawing.close", close))
        .any_part()
        .icon("close")
        .noundo()
        .doc_only()
        .contract(&CLOSE)
        .output(output_schema_for::<CloseResult>)
        .params("→ closed: the drawing's id, or null when none was open"),
    CommandSpec::new("drawing.delete", "Delete Drawing", crate::checked!("drawing.delete", delete))
        .any_part()
        .icon("delete")
        .doc_only()
        .contract(&DELETE)
        .output(output_schema_for::<DeleteResult>)
        .params("drawing: id → deleted: its id; closed when it was open (one undo step of the model)"),
    CommandSpec::new("drawing.inspect", "Inspect Drawing", crate::checked!("drawing.inspect", inspect))
        .in_drawing()
        .icon("inspect")
        .noundo()
        .doc_only()
        .contract(&INSPECT)
        .output(output_schema_for::<InspectResult>)
        .params(
            "→ {id, name, units, language, properties, sources: [{id}], sheets: [{id, name, size_mm, orientation, dpi, documents, excluded}]}. Read-only.",
        ),
    CommandSpec::new("drawing.set", "Drawing Settings", crate::checked!("drawing.set", set))
        .in_drawing()
        .icon("settings")
        .doc_only()
        .contract(&SET)
        .output(output_schema_for::<InspectResult>)
        .params("name?, language?: a BCP 47 tag, units?: \"follow_model\" or one length unit such as \"mm\". One undo step."),
    CommandSpec::new("drawing.properties", "Drawing Properties", crate::checked!("drawing.properties", properties))
        .in_drawing()
        .icon("settings")
        .noundo()
        .doc_only()
        .contract(&PROPERTIES)
        .output(output_schema_for::<PropertiesResult>)
        .params(
            "→ metadata: the drawing's properties as saved (other tools' keys too); properties: project, part_number, revision, author, organization, description and license, each its text or null. The design properties' rules (document.metadata).",
        ),
    CommandSpec::new("drawing.properties.set", "Change Drawing Properties", crate::checked!("drawing.properties.set", properties_set))
        .in_drawing()
        .icon("settings")
        .doc_only()
        .contract(&PROPERTIES_SET)
        .output(output_schema_for::<PropertiesSetResult>)
        .params(
            "key and value, or values: {key: value, …} (all or nothing, one undo step), as document.metadata.set takes them → metadata, changed: [keys]",
        ),
    CommandSpec::new("drawing.properties.remove", "Remove Drawing Property", crate::checked!("drawing.properties.remove", properties_remove))
        .in_drawing()
        .icon("delete")
        .doc_only()
        .contract(&PROPERTIES_REMOVE)
        .output(output_schema_for::<PropertiesRemoveResult>)
        .params("key, or keys: [keys] → metadata, removed: [keys that were there]"),
];

/// A reply built from its result type, so it can't drift from the output schema.
fn reply(value: &impl Serialize) -> Result<Value> {
    serde_json::to_value(value).map_err(|e| crate::EngineError::Other(e.to_string()))
}

/// `units` as given: `follow_model`, or a length unit the drawing's rules check.
fn units(cmd: &str, p: &Value) -> Result<Option<Units>> {
    match p.get("units") {
        None => Ok(None),
        Some(Value::String(u)) if u == "follow_model" => Ok(Some(Units::FollowModel)),
        Some(Value::String(u)) => Ok(Some(Units::Fixed { unit: u.clone() })),
        Some(_) => Err(bad(cmd, "`units` is \"follow_model\" or a length unit such as \"mm\"")),
    }
}

/// An optional text parameter: absent, or a string.
fn text<'a>(cmd: &str, p: &'a Value, key: &str) -> Result<Option<&'a str>> {
    match p.get(key) {
        None => Ok(None),
        Some(_) => str_(p, key).map(Some).ok_or_else(|| bad(cmd, format!("`{key}` must be text"))),
    }
}

/// The design's drawing `p.drawing` names, by id.
fn named(cmd: &str, s: &Session, p: &Value) -> Result<u64> {
    let id = str_(p, "drawing").and_then(ids::decode).ok_or_else(|| bad(cmd, "`drawing` must be a drawing's id (drawing.list)"))?;
    s.doc.drawing(id).map(|d| d.id).ok_or_else(|| bad(cmd, format!("the design has no drawing {}", ids::encode(id))))
}

fn create(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "drawing.create";
    if s.doc.features.is_empty() {
        return Err(bad(cmd, "the design is empty: there is nothing to draw yet"));
    }
    drawing::check_count(s.doc.drawings.len() + 1).map_err(|e| bad(cmd, e.to_string()))?;
    let name = text(cmd, p, "name")?.map_or_else(|| s.doc.name.clone(), str::to_string);
    let language = text(cmd, p, "language")?.unwrap_or("en");
    // Not the id of a drawing undo or redo could bring back either.
    let steps = s.undo.iter().chain(&s.redo).filter_map(|step| step.part);
    let taken: BTreeSet<u64> = s.doc.drawings.iter().map(|d| d.id).chain(steps).collect();
    let mut id_source = IdSource::from_entropy();
    let a3 = Sheet::standard(PaperSize::A3, Orientation::Landscape);
    let fail = |e: solidcraft_doc::DocError| bad(cmd, e.to_string());
    let mut d = Drawing::new(&name, language, "Sheet 1", a3, Some(DrawingBorderProfile::iso()), &taken, &mut id_source).map_err(fail)?;
    let source = d.allocate_id(&mut id_source).map_err(fail)?;
    d.sources.push(Source { id: source });
    if let Some(units) = units(cmd, p)? {
        d.units = units;
    }
    d.validate().map_err(fail)?;
    s.open_drawing = Some(d.id);
    s.doc_mut().put_drawing(Arc::new(d));
    inspect(s, &Value::Null)
}

fn list(s: &mut Session, _p: &Value) -> Result<Value> {
    let drawings = s
        .doc
        .drawings
        .iter()
        .map(|d| ListedDrawing { id: ids::encode(d.id), name: d.name.clone(), sheets: d.sheets.len(), open: s.open_drawing == Some(d.id) })
        .collect();
    reply(&ListResult { drawings, param_warnings: Vec::new() })
}

fn open(s: &mut Session, p: &Value) -> Result<Value> {
    s.open_drawing = Some(named("drawing.open", s, p)?);
    inspect(s, &Value::Null)
}

fn close(s: &mut Session, _p: &Value) -> Result<Value> {
    reply(&CloseResult { closed: s.open_drawing.take().map(ids::encode), param_warnings: Vec::new() })
}

fn delete(s: &mut Session, p: &Value) -> Result<Value> {
    let id = named("drawing.delete", s, p)?;
    if s.open_drawing == Some(id) {
        s.open_drawing = None;
    }
    s.doc_mut().drawings.retain(|d| d.id != id);
    reply(&DeleteResult { deleted: ids::encode(id), param_warnings: Vec::new() })
}

/// The drawing open, as `drawing.inspect` answers, which making and opening one answer too.
fn inspect(s: &mut Session, _p: &Value) -> Result<Value> {
    let cmd = "drawing.inspect";
    let d = s.drawing(cmd)?;
    let mut sheets = Vec::with_capacity(d.sheets.len());
    for sheet in &d.sheets {
        let size = sheet.sheet.size_mm().map_err(|e| bad(cmd, e.to_string()))?;
        sheets.push(SheetResult {
            id: ids::encode(sheet.id),
            name: sheet.name.clone(),
            size_mm: [size.x, size.y],
            orientation: match sheet.sheet.orientation {
                Orientation::Landscape => OrientationResult::Landscape,
                Orientation::Portrait => OrientationResult::Portrait,
            },
            dpi: sheet.dpi,
            documents: sheet.documents.map(|c| SheetComponentResult { source: ids::encode(c.source), component: c.component }),
            excluded: ExcludedResult { count: sheet.excluded.count, print: sheet.excluded.print },
        });
    }
    reply(&InspectResult {
        id: ids::encode(d.id),
        name: d.name.clone(),
        units: match &d.units {
            Units::FollowModel => UnitsResult::FollowModel,
            Units::Fixed { unit } => UnitsResult::Fixed { unit: unit.clone() },
        },
        language: d.language.clone(),
        properties: d.properties.0.clone(),
        sources: d.sources.iter().map(|source| SourceResult { id: ids::encode(source.id) }).collect(),
        sheets,
        param_warnings: Vec::new(),
    })
}

fn set(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "drawing.set";
    s.edit_drawing(cmd, |d| {
        if let Some(name) = text(cmd, p, "name")? {
            d.name = name.to_string();
        }
        if let Some(language) = text(cmd, p, "language")? {
            d.language = language.to_string();
        }
        if let Some(units) = units(cmd, p)? {
            d.units = units;
        }
        Ok(())
    })?;
    inspect(s, &Value::Null)
}

/// The open drawing's properties, as `document.metadata` reads a design's
/// (`metadata::reading`): as saved, and SolidCraft's own keys each as text or null.
fn properties(s: &mut Session, _p: &Value) -> Result<Value> {
    let m = &s.drawing("drawing.properties")?.properties;
    let properties = metadata::keys(Owner::Design).map(|k| (k.name.to_string(), m.text(k.name).map(str::to_string))).collect();
    reply(&PropertiesResult { metadata: m.0.clone(), properties, param_warnings: Vec::new() })
}

/// The open drawing with the properties `next`, kept when they changed: one undo step.
fn keep_properties(s: &mut Session, cmd: &str, next: solidcraft_doc::Metadata) -> Result<Map<String, Value>> {
    s.edit_drawing(cmd, |d| {
        d.properties = next;
        Ok(())
    })?;
    Ok(s.drawing(cmd)?.properties.0.clone())
}

fn properties_set(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "drawing.properties.set";
    let (next, changed) = super::metadata::setting(cmd, &s.drawing(cmd)?.properties, p)?;
    let metadata = keep_properties(s, cmd, next)?;
    reply(&PropertiesSetResult { metadata, changed, param_warnings: Vec::new() })
}

fn properties_remove(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "drawing.properties.remove";
    let (next, removed) = super::metadata::removing(cmd, &s.drawing(cmd)?.properties, p)?;
    let metadata = keep_properties(s, cmd, next)?;
    reply(&PropertiesRemoveResult { metadata, removed, param_warnings: Vec::new() })
}

#[cfg(test)]
#[path = "drawing_tests.rs"]
mod tests;
