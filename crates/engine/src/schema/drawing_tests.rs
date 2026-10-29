use schemars::Schema;
use serde_json::{Map, Value, json};
use solidcraft_doc::ids;
use solidcraft_doc::metadata::MAX_KEYS;

use super::{
    CloseParams, CloseResult, CreateParams, DeleteParams, DeleteResult, InspectParams, InspectResult, ListParams, ListResult, OpenParams,
    PropertiesParams, PropertiesRemoveParams, PropertiesRemoveResult, PropertiesResult, PropertiesSetParams, PropertiesSetResult, SetParams,
    TitleBlockSetParams,
};
use crate::Session;
use crate::schema::tests::{assert_keys, assert_strict_keeps_todays_errors, conforms, sketch_rectangle};
use crate::schema::{ParamCheck, input_schema_for, output_schema_for};

#[test]
fn schemas_and_the_key_check_agree() {
    assert_keys(input_schema_for::<CreateParams>(), &["language", "name", "units"]);
    assert_keys(input_schema_for::<ListParams>(), &[]);
    assert_keys(input_schema_for::<OpenParams>(), &["drawing"]);
    assert_keys(input_schema_for::<CloseParams>(), &[]);
    assert_keys(input_schema_for::<DeleteParams>(), &["drawing"]);
    assert_keys(input_schema_for::<InspectParams>(), &[]);
    assert_keys(input_schema_for::<SetParams>(), &["language", "name", "projection", "units"]);
    assert_keys(input_schema_for::<TitleBlockSetParams>(), &["block", "sheet"]);
    assert_keys(input_schema_for::<PropertiesParams>(), &[]);
    assert_keys(input_schema_for::<PropertiesSetParams>(), &["key", "value", "values"]);
    assert_keys(input_schema_for::<PropertiesRemoveParams>(), &["key", "keys"]);
}

/// The commands that work without an open drawing.
#[test]
fn strict_keeps_todays_errors() {
    let missing = ids::encode(12_345);
    assert_strict_keeps_todays_errors(&[
        ("drawing.create", json!({"name": 5})),
        ("drawing.create", json!({"name": null})),
        ("drawing.create", json!({"language": ["en"]})),
        ("drawing.create", json!({"units": 5})),
        ("drawing.create", json!({"units": null})),
        ("drawing.open", json!({"drawing": 5})),
        ("drawing.open", json!({"drawing": "nope"})),
        ("drawing.open", json!({"drawing": missing})),
        ("drawing.delete", json!({"drawing": null})),
        ("drawing.delete", json!({"drawing": missing})),
    ]);
}

/// The commands that need an open drawing: the same rule as [`assert_strict_keeps_todays_errors`],
/// in a design whose drawing is open.
#[test]
fn strict_keeps_todays_errors_in_a_drawing() {
    let many: Map<String, Value> = (0..=MAX_KEYS).map(|i| (format!("key{i}"), json!("x"))).collect();
    let absent = ids::encode(12_345);
    let cases = [
        ("drawing.set", json!({"name": 5})),
        ("drawing.set", json!({"projection": "fourth"})),
        ("drawing.title_block.set", json!({})),
        ("drawing.title_block.set", json!({"block": "fancy"})),
        ("drawing.title_block.set", json!({"sheet": 5, "block": "built_in"})),
        ("drawing.view.base", json!({"view": "Front"})),
        ("drawing.view.base", json!({"style": {"scale": "0:1"}})),
        ("drawing.view.base", json!({"style": {"display_style": "hidden"}})),
        ("drawing.view.projected", json!({"side": "above", "offset_mm": 50})),
        ("drawing.view.projected", json!({"from": absent.clone(), "side": "above"})),
        ("drawing.view.projected", json!({"from": absent.clone(), "side": "sideways", "offset_mm": 50})),
        ("drawing.view.set", json!({})),
        ("drawing.view.set", json!({"view": absent.clone(), "clear": ["colour"]})),
        ("drawing.view.delete", json!({"view": "nope"})),
        ("drawing.view.standard", json!({"gap_mm": "wide"})),
        ("drawing.view.standard", json!({"sheet": 5})),
        ("drawing.set", json!({"language": false})),
        ("drawing.set", json!({"units": {"fixed": "mm"}})),
        ("drawing.set", json!({"units": "parsec"})),
        ("drawing.properties.set", json!({"key": 5, "value": "x"})),
        ("drawing.properties.set", json!({"key": "part_number"})),
        ("drawing.properties.set", json!({"value": "x"})),
        ("drawing.properties.set", json!({"values": ["part_number"]})),
        ("drawing.properties.set", json!({"values": many})),
        ("drawing.properties.set", json!({"values": {"part_number": 5}})),
        ("drawing.properties.remove", json!({"key": 3})),
        ("drawing.properties.remove", json!({"keys": [1]})),
        ("drawing.properties.remove", json!({"keys": "part_number"})),
        ("drawing.properties.remove", json!({})),
    ];
    for (id, p) in &cases {
        let error = |level: ParamCheck| {
            let mut s = Session { param_check: level, ..Session::default() };
            sketch_rectangle(&mut s);
            s.execute("drawing.create", &json!({})).unwrap_or_else(|e| panic!("{e}"));
            s.execute(id, p).err().map(|e| e.to_string())
        };
        let off = error(ParamCheck::Off);
        assert!(off.is_some(), "{id} {p} should fail today");
        assert_eq!(error(ParamCheck::Strict), off, "{id} {p}");
    }
}

/// Every form each command takes, in warn: the reply matches its output schema and carries no
/// warnings, so the contract accepts the call too.
#[test]
fn replies_match_the_output_schemas() {
    let mut s = Session::default();
    sketch_rectangle(&mut s);
    s.param_check = ParamCheck::Warn;
    let mut run = |id: &str, p: Value, schema: Schema| -> Value {
        let r = s.execute(id, &p).unwrap_or_else(|e| panic!("{id} {p}: {e}"));
        let schema = schema.as_value();
        assert!(conforms(schema, schema, &r), "{id}: {r} doesn't match {schema}");
        assert!(r.get("param_warnings").is_none(), "{id} {p}: {r}");
        r
    };
    let inspect = output_schema_for::<InspectResult>;
    let made = run("drawing.create", json!({}), inspect());
    run("drawing.create", json!({"name": "Plate drawing", "language": "de-CH", "units": "mm"}), inspect());
    run("drawing.inspect", json!({}), inspect());
    let fixed = run("drawing.set", json!({"name": "Plate", "language": "en", "units": "in"}), inspect());
    assert_eq!(fixed["units"], json!({"fixed": {"unit": "in"}}));
    run("drawing.set", json!({"units": "follow_model"}), inspect());
    run("drawing.properties.set", json!({"key": "part_number", "value": "P-1"}), output_schema_for::<PropertiesSetResult>());
    run("drawing.properties.set", json!({"values": {"revision": "B", "author": "A. Person"}}), output_schema_for::<PropertiesSetResult>());
    run("drawing.properties.set", json!({"key": "author", "value": null}), output_schema_for::<PropertiesSetResult>());
    let read = run("drawing.properties", json!({}), output_schema_for::<PropertiesResult>());
    assert_eq!((&read["properties"]["part_number"], &read["properties"]["author"]), (&json!("P-1"), &Value::Null));
    run("drawing.properties.remove", json!({"keys": ["revision", "revision"]}), output_schema_for::<PropertiesRemoveResult>());
    run("drawing.properties.remove", json!({"key": "part_number"}), output_schema_for::<PropertiesRemoveResult>());
    let listed = run("drawing.list", json!({}), output_schema_for::<ListResult>());
    assert_eq!(listed["drawings"].as_array().map(Vec::len), Some(2));
    run("drawing.close", json!({}), output_schema_for::<CloseResult>());
    let none = run("drawing.close", json!({}), output_schema_for::<CloseResult>());
    assert_eq!(none["closed"], Value::Null);
    run("drawing.open", json!({"drawing": made["id"]}), inspect());
    run("drawing.delete", json!({"drawing": made["id"]}), output_schema_for::<DeleteResult>());
}
