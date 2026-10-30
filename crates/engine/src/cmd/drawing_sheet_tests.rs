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

/// The export writes the sheet as the SVG its scene gives, never onto a file that is already
/// there unless it is told to, and writes the same bytes for the same sheet.
#[test]
fn the_export_writes_the_sheet_and_never_overwrites_unasked() {
    let dir = std::env::temp_dir().join(format!("solidcraft-drawing-export-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a folder to write into");
    let path = dir.join("plate.svg");
    let _ = std::fs::remove_file(&path);
    let name = path.to_string_lossy().to_string();
    let mut s = standard();

    let written = run(&mut s, "drawing.export", json!({"format": "svg", "path": name}));
    assert_eq!((written["format"].clone(), written["path"].clone()), (json!("svg"), json!(name)));
    let bytes = std::fs::read(&path).expect("the file the call says it wrote");
    assert_eq!(written["bytes"].as_u64().unwrap_or_default() as usize, bytes.len());
    let text = String::from_utf8(bytes.clone()).expect("the SVG is text");
    assert!(text.contains("<svg") && text.contains("420"), "an A3 document: {}", &text[..text.len().min(120)]);
    assert_eq!(written["views"].as_array().map(Vec::len), Some(4), "the four standard views");
    assert_eq!(written["snapped"], json!({"widths": 0, "pieces": 0, "dashes": 0}), "nothing had to move onto the writer's grid");

    // Not onto a file that is already there, and the refused call leaves it as it was.
    let why_not = why(&mut s, "drawing.export", json!({"format": "svg", "path": name}));
    assert!(why_not.contains("already there") && why_not.contains("overwrite"), "{why_not}");
    assert_eq!(std::fs::read(&path).expect("still there"), bytes, "a refused export left the file alone");

    run(&mut s, "drawing.export", json!({"format": "svg", "path": name, "overwrite": true}));
    assert_eq!(std::fs::read(&path).expect("written again"), bytes, "the same sheet writes the same bytes");
    let _ = std::fs::remove_file(&path);
}

/// What the export refuses: a format that isn't there yet — named, rather than called unknown —
/// and a path that is no path. Neither writes anything.
#[test]
fn the_export_refuses_a_format_it_hasnt_got() {
    let mut s = standard();
    let gone = std::env::temp_dir().join(format!("solidcraft-drawing-never-{}.svg", std::process::id()));
    let name = gone.to_string_lossy().to_string();
    let _ = std::fs::remove_file(&gone);
    for (format, says) in [("dxf", "not supported yet"), ("pdf", "not supported yet"), ("png", "not supported yet"), ("ps", "`format` is")] {
        let why_not = why(&mut s, "drawing.export", json!({"format": format, "path": name}));
        assert!(why_not.contains(says), "{format}: {why_not}");
    }
    assert!(why(&mut s, "drawing.export", json!({"format": "svg", "path": "   "})).contains("`path`"));
    assert!(!gone.exists(), "a refused export wrote a file anyway");
}

/// The phase's done-when, in one test: a script makes a drawing of a plate, puts the standard
/// views on it, asks for hidden lines, and exports it — and the SVG holds four views inside a
/// bordered sheet with its title block, the hidden lines dashed, and two runs give the same bytes.
#[test]
fn a_script_draws_a_plate_and_exports_a_sheet_with_dashed_hidden_lines() {
    let mut s = standard();
    // Hidden lines are a style, so the script asks for them on each view it wants them on.
    let views = run(&mut s, "drawing.inspect", json!({}))["sheets"][0]["views"].as_array().expect("the views").clone();
    assert_eq!(views.len(), 4);
    for view in &views {
        let id = view["id"].as_str().expect("a view's id").to_string();
        run(&mut s, "drawing.view.set", json!({"view": id, "style": {"display_style": "visible_hidden"}}));
    }
    let drawn = run(&mut s, "drawing.sheet.paper", json!({}));
    let hidden: u64 = drawn["views"].as_array().expect("the views").iter().filter_map(|view| view["hidden_runs"].as_u64()).sum();
    assert!(hidden > 0, "no hidden lines to dash");

    let dir = std::env::temp_dir().join(format!("solidcraft-drawing-done-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a folder to write into");
    let path = dir.join("plate.svg");
    let _ = std::fs::remove_file(&path);
    let name = path.to_string_lossy().to_string();
    run(&mut s, "drawing.export", json!({"format": "svg", "path": name, "overwrite": true}));
    let first = std::fs::read(&path).expect("the file");
    let text = String::from_utf8(first.clone()).expect("the SVG is text");
    assert!(text.contains("<svg") && text.contains("420") && text.contains("297"), "an A3 sheet");
    // The dashed stroke, read off the document rather than looked for loosely: ISO 128-2's hidden
    // line is half the visible one (0.25 mm against 0.5), dashed 12d and gapped 3d — 3 and 0.75 mm.
    let dashed = text.lines().find(|line| line.contains("stroke-dasharray")).expect("a dashed stroke");
    assert!(dashed.contains("stroke-width=\"0.25\""), "{dashed}");
    assert!(dashed.contains("stroke-dasharray=\"3 0.75\""), "{dashed}");
    run(&mut s, "drawing.export", json!({"format": "svg", "path": name, "overwrite": true}));
    assert_eq!(std::fs::read(&path).expect("written again"), first, "two runs give identical bytes");
    let _ = std::fs::remove_file(&path);
}

/// Two runs of the same script, in two sessions: what in the SVG is the same and what isn't.
///
/// **The same drawing exported twice gives the same bytes. Two drawings of the same part do not**,
/// and in two ways: the writer puts each group's object id in the document (`<g data-object="…">`)
/// and a drawing's ids come from entropy, *and* a sheet keeps its views sorted by id (the record's
/// rule for later records), so the groups come out in a different order as well. The marks
/// themselves are identical, which is what this test holds down — see PROGRESS.md §8.2 for the
/// question it leaves open about a sheet's paint order.
#[test]
fn two_drawings_of_one_part_draw_the_same_marks_in_their_own_order() {
    let svg = |s: &mut Session| {
        let dir = std::env::temp_dir().join(format!("solidcraft-drawing-twice-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("a folder to write into");
        let path = dir.join(format!("sheet-{}.svg", s.uid));
        let name = path.to_string_lossy().to_string();
        run(s, "drawing.export", json!({"format": "svg", "path": name, "overwrite": true}));
        let text = std::fs::read_to_string(&path).expect("the file");
        let _ = std::fs::remove_file(&path);
        text
    };
    let (mut first, mut second) = (standard(), standard());
    let (a, b) = (svg(&mut first), svg(&mut second));
    // The same drawing, exported twice: the same bytes. That is reproducible output in practice.
    assert_eq!(svg(&mut first), a, "one drawing exported twice");
    // Two drawings of the same part: the same marks, and only the ids and the order of the groups
    // between them. Sorting the paths compares the marks without their order.
    assert_ne!(a, b, "if these matched, neither the ids nor the order would be in the document");
    let paths = |text: &str| {
        let mut lines: Vec<&str> = text.lines().filter(|line| line.starts_with("<path")).collect();
        lines.sort_unstable();
        lines.join("\n")
    };
    assert_eq!(paths(&a), paths(&b), "two drawings of one part drew different marks, not just a different order");
    let groups = |text: &str| text.lines().filter(|line| line.starts_with("<g ")).count();
    assert_eq!((groups(&a), groups(&b)), (7, 7), "the same seven groups either way");
}

/// The ids of the four standard views on that drawing, in the order the layout saves them:
/// front, then the top and right views projected from it, then the isometric view.
fn standard_views() -> (Session, Vec<String>) {
    let mut s = drawn();
    run(&mut s, "drawing.title_block.set", json!({"block": "built_in"}));
    let made = run(&mut s, "drawing.view.standard", json!({}));
    let ids = made["views"].as_array().expect("the views").iter().filter_map(|view| view["id"].as_str().map(str::to_string)).collect();
    (s, ids)
}

/// Each view's status in `drawing.status`, by id.
fn statuses(s: &mut Session) -> std::collections::BTreeMap<String, String> {
    let reply = run(s, "drawing.status", json!({}));
    let views = reply["sheets"][0]["views"].as_array().expect("the views").clone();
    views.iter().filter_map(|view| Some((view["id"].as_str()?.to_string(), view["status"].as_str()?.to_string()))).collect()
}

/// How many views' lines the session has worked out rather than taken as kept.
fn worked_out(s: &Session) -> usize {
    s.drawing_lines.lock().map(|cache| cache.worked_out).unwrap_or_default()
}

/// Every view in `statuses` is `status`.
fn all(statuses: &std::collections::BTreeMap<String, String>, status: &str) -> bool {
    !statuses.is_empty() && statuses.values().all(|s| s == status)
}

/// A view is not drawn until something draws it; an update draws the views that aren't current
/// and keeps them, and a second update draws nothing.
#[test]
fn an_update_draws_the_views_that_arent_current_and_only_those() {
    let (mut s, ids) = standard_views();
    let status = run(&mut s, "drawing.status", json!({}));
    assert_eq!((status["current"].clone(), status["out_of_date"].clone(), status["not_drawn"].clone()), (json!(0), json!(0), json!(4)));
    assert_eq!(worked_out(&s), 0, "laying the views out drew no lines");

    let updated = run(&mut s, "drawing.update", json!({}));
    assert_eq!((updated["drawn"].clone(), updated["kept"].clone()), (json!(4), json!(0)));
    let views = updated["sheets"][0]["views"].as_array().expect("the views").clone();
    assert_eq!(views.len(), 4);
    for view in &views {
        assert_eq!(view["was"], "not_drawn", "{view}");
        assert!(view["visible_runs"].as_u64().is_some_and(|n| n > 0), "every view draws lines: {view}");
    }
    assert_eq!(worked_out(&s), 4);
    let fresh = statuses(&mut s);
    assert!(all(&fresh, "current"), "{fresh:?}");
    assert_eq!(fresh.keys().cloned().collect::<std::collections::BTreeSet<_>>(), ids.into_iter().collect());

    let again = run(&mut s, "drawing.update", json!({}));
    assert_eq!((again["drawn"].clone(), again["kept"].clone()), (json!(0), json!(4)), "a current view is kept as it is");
    assert_eq!(worked_out(&s), 4);
    // Naming the drawing's one sheet updates that sheet.
    let sheet = updated["sheets"][0]["id"].clone();
    let named = run(&mut s, "drawing.update", json!({"sheet": sheet}));
    assert_eq!(named["sheets"].as_array().map(Vec::len), Some(1));
}

/// The paper scene and the export take the lines an update drew, and the marks are the ones
/// working them out again gives: keeping them changes nothing but the time.
#[test]
fn the_paper_scene_takes_the_kept_lines_and_draws_the_same_marks() {
    let (mut s, _) = standard_views();
    // Drawing the sheet keeps its lines too, so it is current after.
    let first = run(&mut s, "drawing.sheet.paper", json!({}));
    assert_eq!(worked_out(&s), 4);
    assert!(all(&statuses(&mut s), "current"));
    let kept = run(&mut s, "drawing.sheet.paper", json!({}));
    assert_eq!(worked_out(&s), 4, "the second scene took the kept lines");
    assert_eq!(kept["scene"], first["scene"]);
    let path = std::env::temp_dir().join(format!("solidcraft-kept-lines-{}.svg", std::process::id()));
    run(&mut s, "drawing.export", json!({"format": "svg", "path": path.to_string_lossy(), "overwrite": true}));
    let _ = std::fs::remove_file(&path);
    assert_eq!(worked_out(&s), 4, "the export took them as well");

    // Let the kept lines go: the scene worked out from nothing is the same scene.
    if let Ok(mut cache) = s.drawing_lines.lock() {
        *cache = crate::drawing_view::LinesCache::default();
    }
    let again = run(&mut s, "drawing.sheet.paper", json!({}));
    assert_eq!(again["scene"], first["scene"]);
}

/// Editing the part puts every view out of date, and undoing the edit makes them current again:
/// the bodies are known by their geometry, not by when they were built.
#[test]
fn editing_the_part_puts_its_views_out_of_date_and_undoing_it_makes_them_current() {
    let (mut s, _) = standard_views();
    run(&mut s, "drawing.update", json!({}));
    let drawing = s.open_drawing.map(ids::encode).expect("a drawing open");
    // The model's commands wait while a drawing is open.
    run(&mut s, "drawing.close", json!({}));
    run(&mut s, "solid.box", json!({"length": 10, "width": 10, "height": 10, "corner": [200, 0, 0], "body_name": "Lug"}));
    run(&mut s, "drawing.open", json!({"drawing": drawing}));
    let edited = statuses(&mut s);
    assert!(all(&edited, "out_of_date"), "{edited:?}");

    run(&mut s, "drawing.close", json!({}));
    run(&mut s, "edit.undo", json!({}));
    run(&mut s, "drawing.open", json!({"drawing": drawing}));
    let undone = statuses(&mut s);
    assert!(all(&undone, "current"), "{undone:?}");

    // Edited again and updated: the update says what it found, and draws them.
    run(&mut s, "drawing.close", json!({}));
    run(&mut s, "edit.redo", json!({}));
    run(&mut s, "drawing.open", json!({"drawing": drawing}));
    let updated = run(&mut s, "drawing.update", json!({}));
    assert_eq!((updated["drawn"].clone(), updated["kept"].clone()), (json!(4), json!(0)));
    let was: Vec<Value> = updated["sheets"][0]["views"].as_array().expect("the views").iter().map(|view| view["was"].clone()).collect();
    assert_eq!(was, vec![json!("out_of_date"); 4]);
    assert!(all(&statuses(&mut s), "current"));
}

/// Moving a view on the sheet leaves its lines as they are; giving it a scale of its own puts it
/// and the views projected from it out of date, and no other.
#[test]
fn a_views_lines_follow_its_direction_and_scale_not_where_it_sits() {
    let (mut s, ids) = standard_views();
    run(&mut s, "drawing.update", json!({}));
    let [front, top, right, iso] = [0, 1, 2, 3].map(|i| ids.get(i).cloned().expect("four views"));
    run(&mut s, "drawing.view.set", json!({"view": front, "position_mm": [150, 160]}));
    let moved = statuses(&mut s);
    assert!(all(&moved, "current"), "moving a view and the views projected from it redraws nothing: {moved:?}");

    run(&mut s, "drawing.view.set", json!({"view": front, "style": {"scale": "1:5"}}));
    let scaled = statuses(&mut s);
    for view in [&front, &top, &right] {
        assert_eq!(scaled.get(view).map(String::as_str), Some("out_of_date"), "{view}: {scaled:?}");
    }
    assert_eq!(scaled.get(&iso).map(String::as_str), Some("current"), "{scaled:?}");
    let updated = run(&mut s, "drawing.update", json!({}));
    assert_eq!((updated["drawn"].clone(), updated["kept"].clone()), (json!(3), json!(1)));
}

/// What `drawing.update` draws on a scratch copy of the session (the file worker's copy) is the
/// session's: its views are current, and its paper scene draws nothing again.
#[test]
fn an_update_drawn_on_a_scratch_copy_is_the_sessions() {
    let (mut s, _) = standard_views();
    let mut worker = s.scratch();
    let updated = run(&mut worker, "drawing.update", json!({}));
    assert_eq!(updated["drawn"], 4);
    assert!(all(&statuses(&mut s), "current"));
    let before = worked_out(&s);
    run(&mut s, "drawing.sheet.paper", json!({}));
    assert_eq!(worked_out(&s), before);
}

/// A new paper keeps a view drawn at a scale of its own current, and puts a fitted view out of date
/// when fitting it again where it sits gives another scale: lines follow the scale, not the paper.
#[test]
fn a_new_paper_puts_only_the_views_it_refits_out_of_date() {
    let mut s = drawn();
    let fitted = run(&mut s, "drawing.view.base", json!({"view": "front", "position_mm": [210, 148.5]}))["view"]["id"].clone();
    let fixed = json!({"view": "top", "position_mm": [100, 60], "style": {"scale": "1:1"}});
    let fixed = run(&mut s, "drawing.view.base", fixed)["view"]["id"].clone();
    let before = run(&mut s, "drawing.update", json!({}));
    let scale_of = |reply: &Value, id: &Value| {
        let views = reply["sheets"][0]["views"].as_array().cloned().unwrap_or_default();
        views.into_iter().find(|view| view["id"] == *id).map(|view| view["scale"]["text"].clone())
    };
    // On A3 the fitted front view has room for 2:1 about the sheet's centre; on A4 that point is
    // near the right and top edges, so it fits again at 1:1.
    run(&mut s, "drawing.sheet.set", json!({"size": {"standard": "a4"}}));
    let status = statuses(&mut s);
    let key = |id: &Value| id.as_str().map(str::to_string).unwrap_or_default();
    assert_eq!(status.get(&key(&fixed)).map(String::as_str), Some("current"), "{status:?}");
    assert_eq!(status.get(&key(&fitted)).map(String::as_str), Some("out_of_date"), "{status:?}");
    let after = run(&mut s, "drawing.update", json!({}));
    assert_eq!((after["drawn"].clone(), after["kept"].clone()), (json!(1), json!(1)));
    assert_ne!(scale_of(&after, &fitted), scale_of(&before, &fitted), "the fit changed the scale");
    assert_eq!(scale_of(&after, &fixed), Some(json!("1:1")));
}

/// An update changes nothing in the design: no undo step, nothing to save.
#[test]
fn an_update_is_no_edit_of_the_drawing() {
    let (mut s, _) = standard_views();
    let (doc, undo, revision) = (s.doc.clone(), s.undo.len(), s.revision);
    run(&mut s, "drawing.update", json!({}));
    run(&mut s, "drawing.status", json!({}));
    assert!(Arc::ptr_eq(&doc, &s.doc), "the design is the one it was");
    assert_eq!((s.undo.len(), s.revision), (undo, revision));
}
