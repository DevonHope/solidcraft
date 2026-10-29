use std::sync::Arc;

use serde_json::json;
use solidcraft_geom::Vec2;

use super::*;

fn run(s: &mut Session, id: &str, p: Value) -> Value {
    s.execute(id, &p).unwrap_or_else(|e| panic!("{id}: {e}"))
}

fn why(s: &mut Session, id: &str, p: Value) -> String {
    s.execute(id, &p).err().map(|e| e.to_string()).unwrap_or_else(|| panic!("accepted: {id} {p}"))
}

fn point(v: &Value) -> Vec2 {
    Vec2::new(v[0].as_f64().unwrap_or_default(), v[1].as_f64().unwrap_or_default())
}

/// A block 100 x 50 x 20 mm with a drawing of it open: one A3 sheet, the ISO border.
fn drawn() -> Session {
    let mut s = Session::default();
    run(&mut s, "solid.box", json!({"length": 100, "width": 50, "height": 20, "body_name": "Block"}));
    run(&mut s, "drawing.create", json!({}));
    s
}

/// That drawing with the built-in title block and the four standard views on its sheet.
fn standard() -> Session {
    let mut s = drawn();
    run(&mut s, "drawing.title_block.set", json!({"block": "built_in"}));
    run(&mut s, "drawing.view.standard", json!({}));
    s
}

/// The kinds of the scene's groups, in paint order.
fn kinds(scene: &Value) -> Vec<String> {
    let groups = scene["groups"].as_array().expect("the scene's groups");
    groups.iter().filter_map(|group| group["kind"].as_str().map(str::to_string)).collect()
}

/// A full sheet's scene: its own furniture in paint order, then a group for every view it draws,
/// and the same bytes for the same sheet twice over — which is what the phase asks of an export.
#[test]
fn a_sheets_paper_scene_holds_its_furniture_and_a_group_for_every_view() {
    let mut s = standard();
    let reply = run(&mut s, "drawing.sheet.paper", json!({}));
    let scene = reply["scene"].clone();
    assert_eq!(point(&scene["size_mm"]), Vec2::new(420.0, 297.0), "A3 landscape");
    let kinds = kinds(&scene);
    assert_eq!(&kinds[..3], ["sheet_border", "drawing_border", "title_block"], "the sheet's furniture first");
    assert_eq!(kinds[3..].iter().filter(|kind| *kind == "view").count(), 4, "one group per view");
    assert_eq!(kinds.len(), 7);
    // What each view settled on: the one scale the standard layout fitted, saved on the base views
    // and followed by the projections, and lines for all four.
    let views = reply["views"].as_array().expect("the views").clone();
    assert_eq!(views.len(), 4);
    let scale = views[0]["scale"]["text"].clone();
    for view in &views {
        assert_eq!(view["scale"]["text"], scale, "one scale for the set: {view}");
        assert_eq!(view["scale"]["fitted"], json!(false), "the standard layout saved it");
        assert!(view["visible_runs"].as_u64().unwrap_or_default() > 0, "a view drew no lines: {view}");
        assert_eq!(view["hidden_runs"], json!(0), "hidden lines are not drawn unless the style asks");
    }
    assert_eq!(run(&mut s, "drawing.sheet.paper", json!({})), reply, "the same sheet gives the same scene");
}

/// A sheet with nothing on it is its own furniture and no more, and a view with no scale of its
/// own is fitted to the drawing area around where it sits.
#[test]
fn an_empty_sheet_is_furniture_and_a_view_without_a_scale_is_fitted() {
    let mut s = drawn();
    let bare = run(&mut s, "drawing.sheet.paper", json!({}));
    assert_eq!(kinds(&bare["scene"]), ["sheet_border", "drawing_border"], "no block, no views");
    assert_eq!(bare["views"], json!([]));
    run(&mut s, "drawing.view.base", json!({}));
    let reply = run(&mut s, "drawing.sheet.paper", json!({}));
    let view = reply["views"][0].clone();
    assert_eq!(view["scale"]["fitted"], json!(true), "it had no scale, so one was fitted");
    assert!(view["scale"]["factor"].as_f64().unwrap_or_default() > 0.0);
    assert_eq!(kinds(&reply["scene"]), ["sheet_border", "drawing_border", "view"]);
    // The chord a view's lines are sampled to follows its scale: 0.01 mm on paper.
    let chord = view["chord_mm"].as_f64().expect("the chord");
    let factor = view["scale"]["factor"].as_f64().expect("the factor");
    assert!((chord * factor - 0.01).abs() < 1e-9, "chord {chord} at factor {factor}");
}

/// A view's style decides whether the lines a nearer surface covers are drawn at all.
///
/// The view here looks from an isometric corner, where a box really does hide three edges. Seen
/// square on, a box hides none: its back edges project exactly onto its front ones, so there is
/// nothing behind anything. Worth knowing before writing a hidden-line test against a plain box.
#[test]
fn hidden_lines_are_drawn_only_where_the_style_asks_for_them() {
    let mut s = drawn();
    let view = run(&mut s, "drawing.view.base", json!({"view": "iso"}))["view"]["id"].as_str().expect("the view's id").to_string();
    assert_eq!(run(&mut s, "drawing.sheet.paper", json!({}))["views"][0]["hidden_runs"], json!(0), "not drawn by default");
    run(&mut s, "drawing.view.set", json!({"view": view, "style": {"display_style": "visible_hidden"}}));
    let hidden = run(&mut s, "drawing.sheet.paper", json!({}))["views"][0]["hidden_runs"].as_u64().unwrap_or_default();
    assert!(hidden > 0, "a box from an isometric corner hides the three edges at its far corner");
    // Cleared again, they go.
    run(&mut s, "drawing.view.set", json!({"view": view, "clear": ["display_style"]}));
    assert_eq!(run(&mut s, "drawing.sheet.paper", json!({}))["views"][0]["hidden_runs"], json!(0));
    // And square on there is nothing to hide either way, which is the note above measured.
    let square = run(&mut s, "drawing.view.base", json!({"position_mm": [320.0, 200.0], "style": {"display_style": "visible_hidden"}}));
    let square = square["view"]["id"].as_str().expect("the view's id").to_string();
    let views = run(&mut s, "drawing.sheet.paper", json!({}))["views"].clone();
    let front = views.as_array().and_then(|all| all.iter().find(|view| view["id"] == json!(square))).expect("the front view");
    assert_eq!(front["hidden_runs"], json!(0), "a box's back edges land on its front ones");
}

/// What the scene refuses: a sheet with no drawing border to lay marks out in, a style the drawing
/// renderer isn't there for, and a projection carrying a scale of its own — which the commands
/// refuse, so only a hand-written file can hold one.
#[test]
fn the_paper_scene_refuses_what_it_cannot_draw() {
    let mut s = standard();
    let open = s.drawing("test").expect("a drawing open").clone();

    let mut bare = (*open).clone();
    let sheet = Arc::make_mut(&mut bare.sheets[0]);
    sheet.border = None;
    sheet.title_block = None;
    s.doc_mut().put_drawing(Arc::new(bare));
    assert!(why(&mut s, "drawing.sheet.paper", json!({})).contains("no drawing border"));
    s.doc_mut().put_drawing(open.clone());

    let mut shaded = (*open).clone();
    let sheet = Arc::make_mut(&mut shaded.sheets[0]);
    sheet.views[0].style.display_style = Some(solidcraft_render::VisualStyle::Shaded);
    s.doc_mut().put_drawing(Arc::new(shaded));
    assert!(why(&mut s, "drawing.sheet.paper", json!({})).contains("shaded views need the drawing renderer"));
    s.doc_mut().put_drawing(open.clone());

    let mut scaled = (*open).clone();
    let sheet = Arc::make_mut(&mut scaled.sheets[0]);
    let projected = sheet.views.iter_mut().find(|view| matches!(view.looks, solidcraft_doc::drawing_view::Looks::Projected { .. }));
    projected.expect("a projected view").style.scale = Some("1:5".parse().expect("a scale"));
    s.doc_mut().put_drawing(Arc::new(scaled));
    let why_not = why(&mut s, "drawing.sheet.paper", json!({}));
    assert!(why_not.contains("takes no scale of its own"), "{why_not}");
    // And the commands never write one.
    s.doc_mut().put_drawing(open);
    let base = run(&mut s, "drawing.inspect", json!({}))["sheets"][0]["views"][0]["id"].as_str().expect("a view").to_string();
    let p = json!({"from": base, "side": "above", "offset_mm": 70, "style": {"scale": "1:5"}});
    assert!(why(&mut s, "drawing.view.projected", p).contains("takes no scale of its own"));
}

/// The scene a sheet gives is one the paper writers can actually draw, and the same sheet gives
/// the same bytes twice. That is the phase's done-when bar the export command itself: the writer
/// is #7430's, already in the branch, and `drawing.export` is what still has to call it.
#[test]
fn the_scene_a_sheet_gives_is_one_the_svg_writer_draws() {
    let mut s = standard();
    let reply = run(&mut s, "drawing.sheet.paper", json!({}));
    let scene: solidcraft_geom::paper::Scene = serde_json::from_value(reply["scene"].clone()).expect("the scene reads back as a scene");
    let (bytes, _) = solidcraft_io::paper::paper_svg(&scene.prepare().expect("it prepares")).expect("it writes as SVG");
    let text = String::from_utf8(bytes.clone()).expect("the SVG is text");
    assert!(text.contains("<svg"), "not an SVG document: {}", &text[..text.len().min(120)]);
    assert!(text.contains("420") && text.contains("297"), "the A3 sheet's size is in the document");
    // The sheet's furniture and the four views all reach the document.
    let paths = text.matches("<path").count();
    assert!(paths >= 7, "only {paths} paths for a bordered sheet with a title block and four views");
    let again = run(&mut s, "drawing.sheet.paper", json!({}));
    let scene: solidcraft_geom::paper::Scene = serde_json::from_value(again["scene"].clone()).expect("the scene reads back");
    let (again, _) = solidcraft_io::paper::paper_svg(&scene.prepare().expect("it prepares")).expect("it writes as SVG");
    assert_eq!(again, bytes, "the same sheet gives the same SVG, byte for byte");
}
