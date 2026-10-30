use serde_json::json;

use super::*;

const EPS: f64 = 1e-9;

fn run(s: &mut Session, id: &str, p: Value) -> Value {
    s.execute(id, &p).unwrap()
}

/// A block 100 x 50 x 20 mm along X, Y and Z, from the origin.
fn block() -> Session {
    let mut s = Session::default();
    run(&mut s, "solid.box", json!({"length": 100, "width": 50, "height": 20, "body_name": "Block"}));
    s
}

fn snap(s: &mut Session, p: Value) -> Value {
    run(s, CMD, p)
}

fn refused(s: &mut Session, p: Value) -> String {
    s.execute(CMD, &p).err().map(|e| e.to_string()).unwrap_or_else(|| panic!("accepted: {p}"))
}

fn point(v: &Value) -> Vec2 {
    Vec2::new(v[0].as_f64().unwrap(), v[1].as_f64().unwrap())
}

/// The boundary's size and centre on the sheet.
fn boundary(v: &Value) -> (Vec2, Vec2) {
    let (min, max) = (point(&v["boundary_mm"]["min"]), point(&v["boundary_mm"]["max"]));
    (max - min, (min + max) * 0.5)
}

/// The block's boundary in each view at 1:2, worked by hand: front 50 x 10, top 50 x 25, right
/// 25 x 10, and from an isometric corner (100 + 50)/sqrt(2)/2 by (100 + 50 + 2 x 20)/sqrt(6)/2.
/// The centre lands on the position, and the extents are the model's own millimetres.
#[test]
fn a_blocks_boundary_in_each_view_is_its_size_at_the_scale() {
    let mut s = block();
    let views = [
        (json!("front"), (50.0, 10.0)),
        (json!("top"), (50.0, 25.0)),
        (json!("right"), (25.0, 10.0)),
        (json!("iso"), (150.0 / 2f64.sqrt() / 2.0, 190.0 / 6f64.sqrt() / 2.0)),
        (json!({"direction": [0, 1, 0], "up": [0, 0, 1]}), (50.0, 10.0)),
    ];
    for (view, (w, h)) in views {
        let v = snap(&mut s, json!({"view": view, "scale": "1:2", "position_mm": [100, 120]}));
        let (size, centre) = boundary(&v);
        assert!((size - Vec2::new(w, h)).len() < EPS, "{view}: {size:?}");
        assert!((centre - Vec2::new(100.0, 120.0)).len() < EPS, "{view}: {centre:?}");
        assert_eq!((v["scale"]["text"].as_str(), v["scale"]["fitted"].as_bool()), (Some("1:2"), Some(false)));
    }
    let front = snap(&mut s, json!({"scale": "1:2"}));
    assert_eq!(front["view"]["named"], "front");
    assert_eq!((front["view"]["direction"].clone(), front["view"]["x"].clone()), (json!([0.0, 1.0, 0.0]), json!([1.0, 0.0, 0.0])));
    assert!((point(&front["extents"]["min"]) - Vec2::ZERO).len() < EPS && (point(&front["extents"]["max"]) - Vec2::new(100.0, 20.0)).len() < EPS);
    assert_eq!((front["extents"]["status"].as_str(), front["source"]["bodies"].as_u64()), (Some("approximate"), Some(1)));
    assert_eq!(front["source"]["timeline_at_end"], true);
    let iso = snap(&mut s, json!({"view": "iso", "scale": "1:2"}));
    assert_eq!(iso["view"]["named"], "iso_front_right");
}

/// With no scale the view takes the largest preset scale that fits the sheet's drawing border,
/// centred on the sheet: the 100 x 20 front view on A3 landscape at 2:1 (5:1 would be 500 mm
/// wide), boundary 110..310 x 128.5..168.5. A scale given is used as it is, even when it overhangs.
#[test]
fn without_a_scale_the_view_fits_and_centres_on_its_sheet() {
    let mut s = block();
    let v = snap(&mut s, json!({}));
    assert_eq!((v["scale"]["text"].as_str(), v["scale"]["fitted"].as_bool()), (Some("2:1"), Some(true)));
    assert!((point(&v["boundary_mm"]["min"]) - Vec2::new(110.0, 128.5)).len() < EPS, "{v}");
    assert!((point(&v["boundary_mm"]["max"]) - Vec2::new(310.0, 168.5)).len() < EPS, "{v}");
    let a4 = snap(&mut s, json!({"sheet": {"size": {"standard": "a4"}, "orientation": "portrait"}}));
    assert_eq!(a4["scale"]["text"], "1:1", "A4 portrait has 95 mm to the border each side: 190 mm of room, so 2:1 (200 mm) doesn't fit");
    let near_the_left = snap(&mut s, json!({"position_mm": [80, 148.5]}));
    assert_eq!(near_the_left["scale"]["text"], "1:1", "70 mm to the left border: 140 mm of room");
    let overhanging = snap(&mut s, json!({"scale": "5:1"}));
    assert!((boundary(&overhanging).0 - Vec2::new(500.0, 100.0)).len() < EPS);
    let off_the_border = refused(&mut s, json!({"position_mm": [5, 148.5]}));
    assert!(off_the_border.contains("no preset scale from 50:1 to 1:10000"), "{off_the_border}");
    assert_eq!(a4["sheet"], json!({"size": {"standard": "a4"}, "orientation": "portrait", "dpi": 300.0}), "the sheet, its defaults filled in");
}

/// Scale and position only scale and move the boundary; the margin grows each side by the
/// margin times the scale; the browser's eye counts for nothing; nothing in the design changes.
#[test]
fn scale_position_and_margin_set_the_boundary_and_the_design_is_untouched() {
    let mut s = block();
    let (undo, doc) = (s.undo.len(), s.doc.clone());
    let dirty = s.is_dirty();
    let one = boundary(&snap(&mut s, json!({"scale": "1:1", "position_mm": [0, 0]})));
    let half = boundary(&snap(&mut s, json!({"scale": "1:2", "position_mm": [0, 0]})));
    assert!((one.0 - half.0 * 2.0).len() < EPS && one.1.len() < EPS && half.1.len() < EPS);
    let margin = boundary(&snap(&mut s, json!({"scale": "1:2", "position_mm": [0, 0], "reference_margin_mm": 5})));
    assert!((margin.0 - (half.0 + Vec2::new(5.0, 5.0))).len() < EPS, "2.5 mm a side at 1:2: {margin:?}");
    let first = snap(&mut s, json!({"view": "top"}));
    assert_eq!(snap(&mut s, json!({"view": "top"})), first, "the same call gives the same answer");
    assert_eq!((s.undo.len(), s.is_dirty()), (undo, dirty));
    assert!(std::sync::Arc::ptr_eq(&doc, &s.doc), "nothing in the design changes");
    run(&mut s, "browser.visibility", json!({"items": ["Block"], "visible": false}));
    assert_eq!(snap(&mut s, json!({"view": "top"}))["boundary_mm"], first["boundary_mm"], "a hidden body is still drawn");
}

/// What the snapshot refuses, each with nothing changed: bad settings, the shaded styles until
/// the drawing renderer, the sheet's resolution out of range, an empty design, a position off
/// the sheet's limit, and a view that would reach past the paper's coordinates.
#[test]
fn hostile_snapshots_are_refused() {
    let mut s = block();
    let doc = s.doc.clone();
    let cases = [
        (json!({"scale": "0:1"}), "scale"),
        (json!({"scale": "-1:2"}), "scale"),
        (json!({"scale": "1e400:1"}), "scale"),
        (json!({"scale": "1:2:3"}), "scale"),
        (json!({"scale": 2}), "scale"),
        (json!({"scale": null}), "scale"),
        (json!({"view": "Front"}), "view is"),
        (json!({"view": {"direction": [0, 0, 0], "up": [0, 0, 1]}}), "direction"),
        (json!({"view": {"direction": [1e308, 1e308, 0], "up": [0, 0, 1]}}), "direction"),
        (json!({"view": {"direction": [0, 0, 1], "up": [0, 0, 2]}}), "along the view direction"),
        (json!({"display_style": "shaded"}), "not supported yet"),
        (json!({"reference_style": "shaded_visible"}), "not supported yet"),
        (json!({"display_style": "Shaded"}), "unknown variant"),
        (json!({"sheet": {"dpi": 24.9}}), "dpi"),
        (json!({"sheet": {"dpi": 2400.1}}), "dpi"),
        (json!({"sheet": {"size": {"custom": {"sides_mm": [0, 10]}}}}), "sheet"),
        (json!({"reference_margin_mm": -1}), "reference_margin_mm"),
        (json!({"position_mm": [1e308, 0]}), "point"),
        (json!({"position_mm": [5000.1, 0], "scale": "1:1"}), "position"),
        (json!({"scael": "1:2"}), "scael"),
    ];
    for (p, says) in cases {
        let e = refused(&mut s, p.clone());
        assert!(e.contains(says), "{p}: {e}");
        assert!(std::sync::Arc::ptr_eq(&doc, &s.doc), "{p}");
    }
    let mut big = Session::default();
    run(&mut big, "solid.box", json!({"length": 300, "width": 10, "height": 10}));
    let e = refused(&mut big, json!({"scale": "1000:1"}));
    assert!(e.contains("past ±100000 mm"), "{e}");
    let e = refused(&mut Session::default(), json!({}));
    assert!(e.contains("no bodies"), "{e}");
}

/// A view shows every body the design places, so a design whose assembly walk stops at its depth
/// limit before some occurrences is refused, and one that ends at the limit is drawn: a chain of
/// components one inside the next, one level at a time across the limit.
#[test]
fn a_view_of_an_assembly_past_the_walks_limit_is_refused() {
    let mut s = block();
    let nest = |s: &mut Session, level: u64| {
        let (component, parent) = (1000 + level, if level == 0 { 0 } else { 1000 + level - 1 });
        let doc = s.doc_mut();
        doc.components.push(solidcraft_doc::Component { id: component, name: format!("L{level}"), parent, ..Default::default() });
        let name = format!("L{level}");
        // From JSON, so a field another change adds to occurrences takes its default here.
        let occurrence = json!({"id": 5000 + level, "component": component, "parent": parent, "name": name});
        doc.occurrences.push(serde_json::from_value(occurrence).unwrap());
    };
    // The deepest occurrence's path is MAX_PLACEMENT_DEPTH + 1 long: walked, its children not.
    for level in 0..=MAX_PLACEMENT_DEPTH as u64 {
        nest(&mut s, level);
    }
    assert!(s.doc.placements_complete());
    snap(&mut s, json!({"scale": "1:1"}));
    nest(&mut s, MAX_PLACEMENT_DEPTH as u64 + 1);
    assert!(!s.doc.placements_complete());
    let e = refused(&mut s, json!({"scale": "1:1"}));
    assert!(e.contains("deeper than 64 levels or has more than 100000 placements"), "{e}");
}

#[test]
fn the_chord_follows_the_scale_and_stays_in_its_range() {
    let chord = crate::drawing_view::chord_mm;
    // 0.01 mm on the sheet, divided by the scale.
    assert!((chord(1.0) - 0.01).abs() < EPS, "{}", chord(1.0));
    assert!((chord(0.5) - 0.02).abs() < EPS, "1:2 samples twice as coarsely: {}", chord(0.5));
    assert!((chord(100.0) - 0.0001).abs() < EPS, "100:1 reaches the fine end: {}", chord(100.0));
    // Past either end it is clamped, so `prepare` never sees a tolerance it refuses.
    assert!((chord(1000.0) - 0.0001).abs() < EPS, "{}", chord(1000.0));
    assert!((chord(0.001) - 1.0).abs() < EPS, "{}", chord(0.001));
    for broken in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!((chord(broken) - 1.0).abs() < EPS, "{broken} gave {}", chord(broken));
    }
}

#[test]
fn a_snapshot_returns_the_views_lines_chained_into_polylines() {
    let mut s = block();
    let reply = snap(&mut s, json!({"scale": "1:1"}));
    let lines = &reply["lines"];
    assert_eq!(lines["status"], "approximate");
    assert_eq!(lines["provenance"], "display_mesh");
    assert!((lines["chord_mm"].as_f64().unwrap() - 0.01).abs() < EPS);
    let runs = |which: &str| lines[which].as_array().expect("lines are runs of points").clone();
    let visible = runs("visible");
    assert!(!visible.is_empty(), "the front view of a block has visible lines");
    // Chaining is the point: loose segments would leave every run two points long.
    let longest = visible.iter().map(|r| r.as_array().map_or(0, Vec::len)).max().unwrap_or(0);
    assert!(longest > 2, "the block's outline was left as loose segments: longest run {longest}");
    // Every point is a pair of finite model millimetres on the view's plane.
    for run in visible.iter().chain(runs("hidden").iter()) {
        for point in run.as_array().expect("a run is a list of points") {
            let p = point.as_array().expect("a point is [x, y]");
            assert_eq!(p.len(), 2, "{point}");
            assert!(p.iter().all(|v| v.as_f64().is_some_and(f64::is_finite)), "{point}");
        }
    }
}

#[test]
fn showing_tangent_edges_is_refused_while_the_mesh_decides_them() {
    let mut s = block();
    let e = refused(&mut s, json!({"scale": "1:1", "tangent_edges": "show"}));
    assert!(e.contains("not supported yet: tangent_edges show"), "{e}");
    // The default still works, so the refusal is the option's and not the command's.
    snap(&mut s, json!({"scale": "1:1", "tangent_edges": "hide"}));
}

/// A block with a drawing of it open: what the saved-view commands work on.
fn drawn() -> Session {
    let mut s = block();
    run(&mut s, "drawing.create", json!({}));
    s
}

/// A block in a component, with a drawing of it open: what a view of a component needs to exist.
fn drawn_in_a_component() -> (Session, u64) {
    let mut s = block();
    run(&mut s, "component.create", json!({"name": "Plate"}));
    let component = s.doc.components.first().map(|in_design| in_design.id).expect("the component just made");
    run(&mut s, "drawing.create", json!({}));
    (s, component)
}

/// Why `id` refused `p`.
fn why(s: &mut Session, id: &str, p: Value) -> String {
    s.execute(id, &p).err().map(|e| e.to_string()).unwrap_or_else(|| panic!("accepted: {id} {p}"))
}

/// The view a call saved.
fn view_of(reply: &Value) -> Value {
    reply["view"].clone()
}

/// A view's id, as the next call names it.
fn id_of(view: &Value) -> String {
    view["id"].as_str().expect("the view's id").to_string()
}

/// The sheet `drawing.create` makes: A3 landscape.
fn a3_sheet() -> Sheet {
    Sheet::standard(solidcraft_geom::drawing_sheet::PaperSize::A3, solidcraft_geom::drawing_sheet::Orientation::Landscape)
}

/// The drawing area of that sheet with the ISO border.
fn a3_border() -> solidcraft_geom::Aabb2 {
    DrawingBorderProfile::iso().layout(&a3_sheet()).expect("the ISO border on A3").drawing_border
}

/// A base view is saved in the middle of the sheet's drawing area unless the call says where, it
/// draws the design the drawing draws, and placing it **evaluates nothing**: what a view draws is
/// worked out when it is drawn, not when it is placed.
#[test]
fn a_base_view_is_saved_where_it_is_put_and_evaluates_nothing() {
    let mut s = drawn();
    let (evaluations, steps) = (s.evaluations, s.undo.len());
    let made = view_of(&run(&mut s, "drawing.view.base", json!({})));
    assert_eq!(point(&made["position_mm"]), a3_border().center(), "the middle of the drawing area");
    assert_eq!(made["looks"]["base"]["view"]["named"], json!("front"), "the front view unless the call says");
    assert_eq!(made["component"], json!(0));
    assert_eq!(made.get("style"), None, "a view that draws as the drawing does saves no style");
    assert_eq!(s.evaluations, evaluations, "nothing was evaluated");
    assert_eq!((s.undo.len(), s.undo.last().and_then(|step| step.part)), (steps + 1, Some(s.open_drawing.expect("a drawing open"))));
    // It comes back from `drawing.inspect`, on the sheet it was put on, and undo takes it away.
    let sheets = run(&mut s, "drawing.inspect", json!({}))["sheets"].clone();
    assert_eq!(sheets[0]["views"], json!([made]));
    assert_eq!(made["sheet"], sheets[0]["id"]);
    run(&mut s, "edit.undo", json!({}));
    assert_eq!(run(&mut s, "drawing.inspect", json!({}))["sheets"][0]["views"], json!([]));
}

/// Where a call puts a base view, and what it draws differently, are saved as given.
#[test]
fn a_base_view_takes_a_place_a_direction_and_a_style() {
    let (mut s, component) = drawn_in_a_component();
    let p = json!({"view": "iso", "component": component, "position_mm": [300.0, 200.0], "style": {"scale": "1:2", "tangent_edges": "show"}});
    let made = view_of(&run(&mut s, "drawing.view.base", p));
    assert_eq!(point(&made["position_mm"]), Vec2::new(300.0, 200.0));
    assert_eq!(made["looks"]["base"]["view"]["named"], json!("iso_front_right"), "iso is iso_front_right");
    assert_eq!((made["component"].clone(), made["style"].clone()), (json!(component), json!({"scale": "1:2", "tangent_edges": "show"})));
    // What it refuses: a direction with no basis, a sheet it hasn't got, and a component the
    // design hasn't got — which the record can't check, because it never sees the design.
    let along = json!({"view": {"direction": [0, 0, 1], "up": [0, 0, 1]}});
    assert!(why(&mut s, "drawing.view.base", along).contains("up"), "an up along the line of sight");
    let elsewhere = json!({"sheet": "0000000000001"});
    assert!(why(&mut s, "drawing.view.base", elsewhere).contains("has no sheet"));
    let absent = json!({"component": component + 1000});
    assert!(why(&mut s, "drawing.view.base", absent).contains("has no component"), "a view of nothing is refused");
}

/// A projected view is a quarter turn from the base view it comes from, lined up with it and
/// drawing the same component. Which way it looks is the drawing's projection: in first angle the
/// view above the front looks from below, so it is the bottom view; in third angle it is the top
/// (ISO 5456-2).
#[test]
fn a_projected_view_is_turned_from_its_base_view_and_lined_up_with_it() {
    let (mut s, component) = drawn_in_a_component();
    let base = view_of(&run(&mut s, "drawing.view.base", json!({"position_mm": [150.0, 150.0], "component": component})));
    let from = id_of(&base);
    let above = view_of(&run(&mut s, "drawing.view.projected", json!({"from": from, "side": "above", "offset_mm": 70})));
    let projected = above["looks"]["projected"].clone();
    assert_eq!((projected["from"].clone(), projected["side"].clone()), (json!(from), json!("above")));
    assert_eq!(projected["view"]["named"], json!("bottom"), "first angle: the view above looks from below");
    assert_eq!(point(&above["position_mm"]), Vec2::new(150.0, 220.0), "lined up with it, 70 mm above");
    let shows = (above["source"].clone(), above["component"].clone());
    assert_eq!(shows, (base["source"].clone(), json!(component)), "it draws what its base view draws");
    // The same side in third angle is the top view.
    let mut third = drawn();
    run(&mut third, "drawing.set", json!({"projection": "third"}));
    let base = id_of(&view_of(&run(&mut third, "drawing.view.base", json!({}))));
    let above = view_of(&run(&mut third, "drawing.view.projected", json!({"from": base, "side": "above", "offset_mm": 70})));
    assert_eq!(above["looks"]["projected"]["view"]["named"], json!("top"));
    // What it refuses: a projection of a projection, an offset that puts it on its base view, and
    // a view that isn't there.
    let on_a_projection = json!({"from": id_of(&above), "side": "left", "offset_mm": 50});
    assert!(why(&mut third, "drawing.view.projected", on_a_projection).contains("a projection is taken from a base view"));
    for offset in [0.0, -50.0] {
        let p = json!({"from": base, "side": "left", "offset_mm": offset});
        assert!(why(&mut third, "drawing.view.projected", p).contains("above 0"), "{offset} mm");
    }
    let nowhere = json!({"from": "0000000000001", "side": "left", "offset_mm": 50});
    assert!(why(&mut third, "drawing.view.projected", nowhere).contains("has no view"));
}

/// Moving a base view moves the views projected from it with it, so they stay lined up and keep
/// their spacing. A projected view itself moves along its own side, by `offset_mm`.
#[test]
fn a_base_views_projections_move_with_it() {
    let mut s = drawn();
    let base = id_of(&view_of(&run(&mut s, "drawing.view.base", json!({"position_mm": [150.0, 150.0]}))));
    let above = id_of(&view_of(&run(&mut s, "drawing.view.projected", json!({"from": base, "side": "above", "offset_mm": 70}))));
    let right = id_of(&view_of(&run(&mut s, "drawing.view.projected", json!({"from": base, "side": "right", "offset_mm": 90}))));
    let moved = view_of(&run(&mut s, "drawing.view.set", json!({"view": base, "position_mm": [200.0, 120.0]})));
    assert_eq!(point(&moved["position_mm"]), Vec2::new(200.0, 120.0));
    let placed = |s: &mut Session, id: &str| point(&view_of(&run(s, "drawing.view.set", json!({"view": id})))["position_mm"]);
    assert_eq!(placed(&mut s, &above), Vec2::new(200.0, 190.0), "70 mm above it still");
    assert_eq!(placed(&mut s, &right), Vec2::new(290.0, 120.0), "90 mm to its right still");
    // A projected view moves by its offset, and keeps the coordinate it shares with its base view.
    let further = view_of(&run(&mut s, "drawing.view.set", json!({"view": above, "offset_mm": 100})));
    assert_eq!(point(&further["position_mm"]), Vec2::new(200.0, 220.0));
    // Each takes the parameter that means something for it.
    assert!(why(&mut s, "drawing.view.set", json!({"view": base, "offset_mm": 50})).contains("`offset_mm` is for a projected view"));
    let how = why(&mut s, "drawing.view.set", json!({"view": above, "position_mm": [0.0, 0.0]}));
    assert!(how.contains("stays lined up"), "{how}");
}

/// A view's style is changed field by field, and `clear` gives a field back to what the view
/// follows. Saying both for one field is a mistake, not an order.
#[test]
fn a_views_style_is_set_and_cleared_field_by_field() {
    let mut s = drawn();
    let view = id_of(&view_of(&run(&mut s, "drawing.view.base", json!({"style": {"scale": "1:2"}}))));
    let set = json!({"view": view, "style": {"display_style": "visible_hidden", "reference_margin_mm": 5.0}});
    let styled = view_of(&run(&mut s, "drawing.view.set", set));
    assert_eq!(styled["style"], json!({"scale": "1:2", "display_style": "visible_hidden", "reference_margin_mm": 5.0}), "kept and added");
    let cleared = view_of(&run(&mut s, "drawing.view.set", json!({"view": view, "clear": ["scale", "reference_margin_mm"]})));
    assert_eq!(cleared["style"], json!({"display_style": "visible_hidden"}));
    let both = json!({"view": view, "style": {"display_style": "visible"}, "clear": ["display_style"]});
    assert!(why(&mut s, "drawing.view.set", both).contains("given and cleared"));
    let margin = json!({"view": view, "style": {"reference_margin_mm": -1.0}});
    assert!(why(&mut s, "drawing.view.set", margin).contains("reference_margin_mm"), "the record's own rule");
    let all = json!({"view": view, "clear": ["display_style"]});
    assert_eq!(view_of(&run(&mut s, "drawing.view.set", all)).get("style"), None, "nothing of its own left");
}

/// Deleting a base view takes the views projected from it with it: a projection draws nothing on
/// its own, and the drawing would refuse to keep one whose base view is gone.
#[test]
fn deleting_a_base_view_takes_its_projections() {
    let mut s = drawn();
    let base = id_of(&view_of(&run(&mut s, "drawing.view.base", json!({"position_mm": [150.0, 150.0]}))));
    let above = id_of(&view_of(&run(&mut s, "drawing.view.projected", json!({"from": base, "side": "above", "offset_mm": 70}))));
    let other = id_of(&view_of(&run(&mut s, "drawing.view.base", json!({"position_mm": [300.0, 150.0]}))));
    // A projection goes on its own without touching its base view.
    assert_eq!(run(&mut s, "drawing.view.delete", json!({"view": above}))["deleted"], json!([above]));
    let above = id_of(&view_of(&run(&mut s, "drawing.view.projected", json!({"from": base, "side": "above", "offset_mm": 70}))));
    let deleted = run(&mut s, "drawing.view.delete", json!({"view": base}))["deleted"].clone();
    assert_eq!(deleted, json!([base, above]), "the base view first, then what went with it");
    let left = run(&mut s, "drawing.inspect", json!({}))["sheets"][0]["views"].clone();
    assert_eq!(left.as_array().map(Vec::len), Some(1));
    assert_eq!(left[0]["id"], json!(other), "the other base view stands");
    assert!(why(&mut s, "drawing.view.delete", json!({"view": base})).contains("has no view"));
}

/// The projection is the drawing's, and changing it while projected views stand is refused: the
/// other angle turns each of them the other way, so it would quietly show other sides.
#[test]
fn the_drawings_projection_is_refused_while_projections_stand() {
    let mut s = drawn();
    assert_eq!(run(&mut s, "drawing.inspect", json!({}))["projection"], json!("first"), "first angle unless the drawing says");
    let base = id_of(&view_of(&run(&mut s, "drawing.view.base", json!({}))));
    assert_eq!(run(&mut s, "drawing.set", json!({"projection": "third"}))["projection"], json!("third"), "base views alone are no bar");
    run(&mut s, "drawing.view.projected", json!({"from": base, "side": "right", "offset_mm": 80}));
    let why_not = why(&mut s, "drawing.set", json!({"projection": "first"}));
    assert!(why_not.contains("projected views") && why_not.contains("drawing.view.delete"), "{why_not}");
    assert_eq!(run(&mut s, "drawing.set", json!({"projection": "third"}))["projection"], json!("third"), "the angle it already has is no change");
    assert!(why(&mut s, "drawing.set", json!({"projection": "fourth"})).contains("\"first\" or \"third\""));
}

/// A sheet takes a copy of the built-in title block, gives it back, and refuses one that would not
/// fit its drawing border — the record's rule, through the layout that draws it.
#[test]
fn a_title_block_is_put_on_a_sheet_and_refused_where_it_cant_be_drawn() {
    let mut s = drawn();
    let sheets = run(&mut s, "drawing.title_block.set", json!({"block": "built_in"}))["sheets"].clone();
    let block = sheets[0]["title_block"].clone();
    assert_eq!(point(&block["size_mm"]), Vec2::new(180.0, 54.0));
    let cells = block["cells"].as_array().map(Vec::len);
    assert_eq!(cells, Some(solidcraft_geom::title_block::TitleBlock::built_in().cells.len()));
    assert_eq!(run(&mut s, "drawing.inspect", json!({}))["sheets"][0]["title_block"], block, "it is saved, not worked out again");
    assert_eq!(run(&mut s, "drawing.title_block.set", json!({"block": "none"}))["sheets"][0]["title_block"], json!(null));
    // A sheet too small for it: the block is 180 x 54 mm, and A5 portrait's drawing area is
    // narrower.
    run(&mut s, "drawing.sheet.set", json!({"size": {"standard": "a5"}, "orientation": "portrait"}));
    let tight = why(&mut s, "drawing.title_block.set", json!({"block": "built_in"}));
    assert!(tight.contains("a title block fits inside the drawing border"), "{tight}");
}

/// A sheet's paper, the way it lies, its name and its picture resolution change in one undo step,
/// and what isn't given stays. The drawing's own rules refuse what they refuse, a title block that
/// would no longer fit among them, and a refusal leaves the drawing as it was.
#[test]
fn a_sheets_paper_name_and_resolution_change_in_one_undo_step() {
    let mut s = drawn();
    let steps = s.undo.len();
    let changed = json!({"size": {"standard": "a4"}, "orientation": "portrait", "dpi": 150, "name": "Details"});
    let sheet = run(&mut s, "drawing.sheet.set", changed)["sheets"][0].clone();
    assert_eq!(point(&sheet["size_mm"]), Vec2::new(210.0, 297.0), "A4 portrait");
    assert_eq!((sheet["orientation"].clone(), sheet["dpi"].clone(), sheet["name"].clone()), (json!("portrait"), json!(150.0), json!("Details")));
    assert_eq!(s.undo.len(), steps + 1, "one undo step of the drawing");

    let custom = run(&mut s, "drawing.sheet.set", json!({"size": {"custom": {"sides_mm": [500, 300]}}}))["sheets"][0].clone();
    let mut sides = [point(&custom["size_mm"]).x, point(&custom["size_mm"]).y];
    sides.sort_by(f64::total_cmp);
    assert_eq!(sides, [300.0, 500.0]);
    assert_eq!((custom["orientation"].clone(), custom["name"].clone(), custom["dpi"].clone()), (json!("portrait"), json!("Details"), json!(150.0)));

    run(&mut s, "edit.undo", json!({}));
    run(&mut s, "edit.undo", json!({}));
    let back = run(&mut s, "drawing.inspect", json!({}))["sheets"][0].clone();
    assert_eq!((point(&back["size_mm"]), back["dpi"].clone()), (Vec2::new(420.0, 297.0), json!(300.0)), "A3 landscape at 300 dpi again");

    let before = s.drawing("test").map(Arc::clone).expect("a drawing open");
    for (p, says) in [
        (json!({"dpi": 10}), "dpi"),
        (json!({"name": "  "}), "name"),
        (json!({"size": {"custom": {"sides_mm": [0, 100]}}}), "sheet"),
        (json!({"size": {"custom": {"sides_mm": [6000, 100]}}}), "sheet"),
    ] {
        let e = why(&mut s, "drawing.sheet.set", p.clone());
        assert!(e.contains(says), "{p}: {e}");
        assert!(s.drawing("test").is_ok_and(|now| Arc::ptr_eq(now, &before)), "{p} left the drawing as it was");
    }
    run(&mut s, "drawing.title_block.set", json!({"block": "built_in"}));
    let tight = why(&mut s, "drawing.sheet.set", json!({"size": {"standard": "a5"}, "orientation": "portrait"}));
    assert!(tight.contains("a title block fits inside the drawing border"), "{tight}");
}

/// The standard arrangement, end to end: four views on one sheet at one fitted scale, each inside
/// the drawing border and clear of the title block, the two projections lined up with the front
/// view and turned the way the drawing's projection asks.
#[test]
fn the_standard_views_land_inside_the_border_and_off_the_title_block() {
    for (projection, sides) in [("first", ("below", "left")), ("third", ("above", "right"))] {
        let mut s = drawn();
        run(&mut s, "drawing.set", json!({"projection": projection}));
        run(&mut s, "drawing.title_block.set", json!({"block": "built_in"}));
        let steps = s.undo.len();
        let reply = run(&mut s, "drawing.view.standard", json!({}));
        let views = reply["views"].as_array().expect("the four views").clone();
        assert_eq!(views.len(), 4, "front, top, right and the isometric");
        // The front and isometric views carry the fitted scale; the projections follow them.
        let scale = reply["scale"]["text"].as_str().expect("the scale").to_string();
        assert!(reply["scale"]["fitted"].as_bool().unwrap_or_default(), "it was worked out, not given");
        assert_eq!(views[0]["style"]["scale"], json!(scale));
        assert_eq!(views[3]["style"]["scale"], json!(scale));
        assert_eq!((views[1].get("style"), views[2].get("style")), (None, None), "a projection follows its base view");
        // Which way each looks, and which side it sits on: the drawing's projection decides, and
        // the layout is the one that says so (`StandardLayout::top_side`).
        assert_eq!(views[0]["looks"]["base"]["view"]["named"], json!("front"));
        assert_eq!(views[3]["looks"]["base"]["view"]["named"], json!("iso_front_right"));
        let from = views[0]["id"].clone();
        for (view, side) in [(&views[1], sides.0), (&views[2], sides.1)] {
            let projected = view["looks"]["projected"].clone();
            assert_eq!((projected["from"].clone(), projected["side"].clone()), (from.clone(), json!(side)), "{projection} angle");
        }
        // In first angle the view below the front looks from above, so it is the top view, and the
        // one to the left looks from the right; in third angle each sits on the side it looks from.
        let named = |view: &Value| view["looks"]["projected"]["view"]["named"].clone();
        assert_eq!((named(&views[1]), named(&views[2])), (json!("top"), json!("right")), "{projection} angle shows top and right either way");
        // Every view inside the drawing border, and none of them on the title block.
        let border = a3_border();
        let block = kept_clear(
            &solidcraft_geom::title_block::TitleBlock::built_in().layout(&DrawingBorderProfile::iso().layout(&a3_sheet()).unwrap()).unwrap(),
            border,
        );
        for view in &views {
            let at = point(&view["position_mm"]);
            assert!(border.contains(at), "{projection}: a view's centre is outside the drawing border at {at:?}");
            assert!(!block.contains(at), "{projection}: a view's centre is on the title block at {at:?}");
        }
        // All four are one undo step of the drawing, and it takes all four away. This is the only
        // command that is `in_drawing()` without being `doc_only()`, so the step is worth asserting.
        let part = s.undo.last().and_then(|step| step.part);
        assert_eq!((s.undo.len(), part), (steps + 1, Some(s.open_drawing.expect("a drawing open"))), "{projection}");
        run(&mut s, "edit.undo", json!({}));
        assert_eq!(run(&mut s, "drawing.inspect", json!({}))["sheets"][0]["views"], json!([]), "{projection}: undone");
        run(&mut s, "edit.redo", json!({}));
        let back = run(&mut s, "drawing.inspect", json!({}))["sheets"][0]["views"].as_array().map(Vec::len);
        assert_eq!(back, Some(4), "{projection}: redone");
    }
}

/// What the standard arrangement refuses: a sheet that already draws views, a design with nothing
/// in it, a gap out of range, and a sheet with no drawing border to lay them out in.
#[test]
fn the_standard_views_are_refused_where_they_cannot_be_laid_out() {
    let mut s = drawn();
    run(&mut s, "drawing.view.standard", json!({}));
    let again = why(&mut s, "drawing.view.standard", json!({}));
    assert!(again.contains("already draws views") && again.contains("drawing.view.delete"), "{again}");
    // A gap wider than the sheet: the layout refuses it by its own rule, naming the presets.
    let mut fresh = drawn();
    let wide = why(&mut fresh, "drawing.view.standard", json!({"gap_mm": 900.0}));
    assert!(wide.contains("no preset scale fits"), "{wide}");
    for gap in [-1.0, 1001.0] {
        assert!(why(&mut fresh, "drawing.view.standard", json!({"gap_mm": gap})).contains("0 to 1000 mm"), "{gap}");
    }
    // No border on the sheet: nowhere to lay them out. The paper is set by hand, since no command
    // changes a sheet's border yet.
    let open = fresh.drawing("test").expect("a drawing open").clone();
    let mut bare = (*open).clone();
    let sheet = Arc::make_mut(&mut bare.sheets[0]);
    sheet.border = None;
    sheet.title_block = None;
    fresh.doc_mut().put_drawing(Arc::new(bare));
    assert!(why(&mut fresh, "drawing.view.standard", json!({})).contains("no drawing border"));
}
