use serde_json::json;
use solidcraft_geom::drawing_sheet::{Margins, Orientation, PaperSize, SheetSize};
use solidcraft_geom::drawing_view::{NamedView, ProjectedSide, Scale};

use crate::drawing_view::{ViewDirection, ViewStyle};

use super::*;

/// The seed every test drawing's ids come from.
const SEED: u64 = 3890;

/// A profile of its own: ISO 5457's filing margin and trimming marks, which the built-in leaves off.
fn own_border() -> DrawingBorderProfile {
    DrawingBorderProfile { filing_margin: true, trimming_mark_mm: Some([10.0, 5.0]), ..DrawingBorderProfile::iso() }
}

/// The design it is in, on two sheets: the first (A3, the ISO border) documents its component 7;
/// the second (A4 portrait, a border of its own, 600 dpi, left out of printing) the design itself.
fn plate() -> Drawing {
    let mut id_source = IdSource::seeded(SEED);
    let a3 = drawing_sheet::Sheet::standard(PaperSize::A3, Orientation::Landscape);
    let mut d = Drawing::new("Plate assembly", "en-GB", "Assembly", a3, Some(DrawingBorderProfile::iso()), &BTreeSet::new(), &mut id_source).unwrap();
    let id = d.allocate_id(&mut id_source).unwrap();
    d.sources.push(Source { id });
    let plate_id = d.sources[0].id;
    let id = d.allocate_id(&mut id_source).unwrap();
    d.sheets.push(Arc::new(Sheet {
        id,
        name: "Bolt".into(),
        sheet: drawing_sheet::Sheet::standard(PaperSize::A4, Orientation::Portrait),
        border: Some(own_border()),
        title_block: None,
        dpi: 600.0,
        documents: Some(SheetComponent { source: plate_id, component: 0 }),
        excluded: Excluded { count: false, print: true },
        views: Vec::new(),
    }));
    Arc::make_mut(&mut d.sheets[0]).documents = Some(SheetComponent { source: plate_id, component: 7 });
    // The first sheet draws the component it documents, with the built-in title block in the
    // corner of its border: the front view at 1:2, and the top view projected from it, which
    // follows its scale. Their ids come from the seed, so they are sorted into the record's order.
    let front = View {
        id: d.allocate_id(&mut id_source).unwrap(),
        source: plate_id,
        component: 7,
        position_mm: [140.0, 150.0],
        looks: Looks::Base { view: ViewDirection::Named(NamedView::Front) },
        style: ViewStyle { scale: Some(Scale::new(1.0, 2.0).unwrap()), ..ViewStyle::default() },
    };
    let top = View {
        id: d.allocate_id(&mut id_source).unwrap(),
        position_mm: [140.0, 240.0],
        looks: Looks::Projected { from: front.id, side: ProjectedSide::Above },
        style: ViewStyle::default(),
        ..front
    };
    let assembly = Arc::make_mut(&mut d.sheets[0]);
    assembly.title_block = Some(TitleBlock::built_in());
    assembly.views = vec![front, top];
    assembly.views.sort_by_key(|view| view.id);
    d.properties.set(crate::metadata::Owner::Design, "author", json!("Jane Doe")).unwrap();
    d.units = Units::Fixed { unit: "in".into() };
    d.validate().unwrap();
    d
}

/// One A4 sheet that documents nothing, no sources: the base the caps grow from.
fn bare() -> Drawing {
    let a4 = drawing_sheet::Sheet::standard(PaperSize::A4, Orientation::Landscape);
    Drawing::new("Bare", "en", "Sheet", a4, None, &BTreeSet::new(), &mut IdSource::seeded(SEED)).unwrap()
}

fn file(d: &Drawing) -> Value {
    serde_json::from_str(&d.to_json().unwrap()).unwrap()
}

fn read(file: &Value) -> Result<Drawing> {
    Drawing::from_json(&file.to_string())
}

/// Why `file` is refused, or "read".
fn refusal(file: &Value) -> String {
    read(file).map_or_else(|e| e.to_string(), |_| "read".into())
}

/// Why `d`'s file, changed by `edit`, is refused, or "read".
fn refused(d: &Drawing, edit: impl FnOnce(&mut Value)) -> String {
    let mut v = file(d);
    edit(&mut v);
    refusal(&v)
}

/// A drawing reads back as it was saved, and save, open and save again writes the same bytes, as
/// does another run from the same seed. The file holds each field under the record's name for it,
/// ids as their 13 characters, and no counter.
#[test]
fn a_drawing_reads_back_as_saved_and_saves_the_same_bytes_again() {
    let d = plate();
    let saved = d.to_json().unwrap();
    let reopened = Drawing::from_json(&saved).unwrap();
    assert_eq!(reopened, d);
    assert_eq!(reopened.to_json().unwrap(), saved);
    assert_eq!(plate().to_json().unwrap(), saved, "a seeded run makes the same drawing");
    let v = file(&d);
    let plate_id = ids::encode(d.sources[0].id);
    let [assembly_id, bolt_sheet_id] = [0, 1].map(|i| ids::encode(d.sheets[i].id));
    assert_eq!(v["id"], "1ge30wkvq38r0", "the seed's first id, as splitmix64 gives it");
    assert_eq!(v["format"], DRAWING.format_string(DRAWING.current()));
    assert_eq!(v.get("format_pending").is_some(), !DRAWING.pending_names().is_empty());
    assert_eq!(v.get("next_id"), None);
    assert_eq!((&v["name"], &v["language"]), (&json!("Plate assembly"), &json!("en-GB")));
    assert_eq!(v["sources"], json!([{"id": plate_id}]));
    assert_eq!(v["properties"], json!({"author": "Jane Doe"}));
    assert_eq!(v["units"], json!({"fixed": {"unit": "in"}}));
    assert_eq!((&v["sheets"][0]["id"], &v["sheets"][0]["dpi"]), (&json!(assembly_id), &json!(300.0)));
    assert_eq!(v["sheets"][0]["sheet"], json!({"size": {"standard": "a3"}, "orientation": "landscape"}));
    assert_eq!(v["sheets"][0]["documents"], json!({"source": plate_id, "component": 7}));
    assert_eq!(v["sheets"][0]["excluded"], json!({"count": false, "print": false}));
    // The drawing's projection is always written, and the block is a copy of the built-in. A view
    // says what it draws and where it sits, and nothing measured from the design; one that draws
    // just what it follows saves no style at all. The views are in the record's order, by id.
    assert_eq!(v["projection"], json!("first"));
    assert_eq!(v["sheets"][0]["title_block"], serde_json::to_value(TitleBlock::built_in()).unwrap());
    let view_of = |projected: bool| {
        let found = d.sheets[0].views.iter().find(|view| matches!(view.looks, Looks::Projected { .. }) == projected);
        let view = found.expect("the sheet's views");
        let saved = v["sheets"][0]["views"].as_array().expect("the views as saved");
        (view.id, saved.iter().find(|s| s["id"] == json!(ids::encode(view.id))).expect("the view as saved").clone())
    };
    let ((base_id, base), (top_id, top)) = (view_of(false), view_of(true));
    assert_eq!(
        base,
        json!({
            "id": ids::encode(base_id), "source": plate_id, "component": 7, "position_mm": [140.0, 150.0],
            "looks": {"base": {"view": "front"}}, "style": {"scale": "1:2"}
        })
    );
    assert_eq!(
        top,
        json!({
            "id": ids::encode(top_id), "source": plate_id, "component": 7, "position_mm": [140.0, 240.0],
            "looks": {"projected": {"from": ids::encode(base_id), "side": "above"}}
        })
    );
    let order: Vec<&Value> = v["sheets"][0]["views"].as_array().expect("the views as saved").iter().map(|view| &view["id"]).collect();
    assert!(order[0].as_str() < order[1].as_str(), "the views are saved by id: {order:?}");
    assert_eq!(
        v["sheets"][1],
        json!({
            "id": bolt_sheet_id, "name": "Bolt", "sheet": {"size": {"standard": "a4"}, "orientation": "portrait"},
            "border": serde_json::to_value(own_border()).unwrap(), "dpi": 600.0,
            "documents": {"source": plate_id, "component": 0}, "excluded": {"count": false, "print": true}
        })
    );
}

/// A hand-written file may leave out what has a default, and reads as a new drawing has it; the
/// rest is needed.
#[test]
fn a_file_may_leave_out_what_has_a_default() {
    let minimal = json!({
        "format": DRAWING.format_string(1),
        "id": "0000000000002",
        "name": "Bracket",
        "sources": [],
        "language": "de",
        "sheets": [{"id": "0000000000001", "name": "Blatt 1", "sheet": {"size": {"standard": "a3"}}}]
    });
    let d = read(&minimal).unwrap();
    assert_eq!((d.sheets[0].id, &d.units, d.properties.is_empty()), (1, &Units::FollowModel, true));
    let sheet = &d.sheets[0];
    assert_eq!(sheet.sheet, drawing_sheet::Sheet::standard(PaperSize::A3, Orientation::Landscape));
    assert_eq!((sheet.border, sheet.dpi, sheet.documents, sheet.excluded), (None, DEFAULT_DPI, None, Excluded::default()));
    assert_eq!((&sheet.title_block, sheet.views.as_slice(), d.projection), (&None, [].as_slice(), Projection::First));
    // A drawing the build before this change saved — the owner's own files, as 0e3074ac writes
    // them: the pending format number, and `in-design` alone, the one change that build knew. It
    // reads, and what this change added comes out as a new drawing's. The name is written out
    // rather than taken from `DRAWING.pending_names()` without its last entry, because pending
    // changes are listed in file-name order: `sheet-dimensions` and `sheet-notes` will sort
    // *before* `sheet-views`, and dropping the last name would build a file no build ever wrote.
    let mut before = minimal.clone();
    before["format"] = json!(DRAWING.format_string(DRAWING.current()));
    before["format_pending"] = json!(["in-design"]);
    let d = read(&before).unwrap();
    let sheet = &d.sheets[0];
    assert_eq!((&sheet.title_block, sheet.views.as_slice(), d.projection), (&None, [].as_slice(), Projection::First));
    assert_eq!(d.format_pending, DRAWING.pending_names(), "saved again, it lists this build's changes");
    // The format as this build writes it, however the file spelt it.
    let mut spelt = minimal.clone();
    spelt["format"] = json!("solidcraft-drawing/+1");
    assert_eq!(read(&spelt).unwrap().format, DRAWING.format_string(DRAWING.current()));
    for key in ["id", "name", "sources", "language", "sheets"] {
        let mut v = minimal.clone();
        v.as_object_mut().unwrap().remove(key);
        let why = refusal(&v);
        assert!(why.starts_with("damaged drawing: ") && why.contains(&format!("missing field `{key}`")), "{why}");
    }
}

/// Every cap on both sides: sheets, sources, the bytes of names (two-byte characters too).
#[test]
fn every_cap_holds_on_both_sides() {
    let base = file(&bare());
    let with = |key: &str, value: Value| {
        let mut v = base.clone();
        v[key] = value;
        v
    };
    let sheet = base["sheets"][0].clone();
    let sheets = |n: u64| {
        let one = |id: u64| {
            let mut s = sheet.clone();
            s["id"] = json!(ids::encode(id));
            s
        };
        Value::Array((1..=n).map(one).collect())
    };
    assert_eq!(read(&with("sheets", sheets(1000))).unwrap().sheets.len(), 1000);
    for n in [0, 1001] {
        assert_eq!(refusal(&with("sheets", sheets(n))), "damaged drawing: a drawing has 1 to 1000 sheets", "{n} sheets");
    }
    // Sources from id 1: the drawing's and the sheet's ids are the seed's, far from them. A drawing
    // draws the design it is in, by an id alone: a path is a field this build doesn't know.
    let sources = |n: u64| Value::Array((1..=n).map(|id| json!({"id": ids::encode(id)})).collect());
    for n in [0, 1] {
        assert_eq!(read(&with("sources", sources(n))).unwrap().sources.len(), n as usize);
    }
    assert_eq!(refusal(&with("sources", sources(2))), "damaged drawing: a drawing draws only the design it is in");
    let with_path = refusal(&with("sources", json!([{"id": ids::encode(1), "path": "plate.solidcraft"}])));
    assert!(with_path.starts_with("damaged drawing: unknown field `path`"), "{with_path}");
    let names = |name: &str| {
        let mut sheet_named = base.clone();
        sheet_named["sheets"][0]["name"] = json!(name);
        (refusal(&with("name", json!(name))), refusal(&sheet_named))
    };
    for fits in ["x".repeat(256), "é".repeat(128), "A".into(), "Blatt 1 – Übersicht".into()] {
        assert_eq!(names(fits.as_str()), ("read".to_string(), "read".to_string()), "{} bytes", fits.len());
    }
    let rule = "name is 1 to 256 bytes, not only spaces, without control characters";
    let sheet_id = ids::encode(bare().sheets[0].id);
    for broken in ["x".repeat(257), "é".repeat(128) + "a", String::new(), " ".into(), "two\nlines".into(), "bell\u{7}".into(), "nul\0".into()] {
        let expected = (format!("damaged drawing: the drawing's {rule}"), format!("damaged drawing: sheet {sheet_id}: its {rule}"));
        assert_eq!(names(broken.as_str()), expected, "{broken:?}");
    }
}

/// The content language is a well-formed BCP 47 tag of 2 to 35 characters.
#[test]
fn the_language_is_a_bcp_47_tag_of_2_to_35_characters() {
    let base = file(&bare());
    let why = |tag: &str| {
        let mut v = base.clone();
        v["language"] = json!(tag);
        refusal(&v)
    };
    let longest = "en-abcdefgh-abcdefgh-abcdefgh-abcde";
    assert_eq!(longest.len(), 35);
    for tag in ["en", "de", "ast", "en-GB", "EN-gb", "zh-Hans-CN", "sr-Latn-RS", "es-419", longest] {
        assert_eq!(why(tag), "read", "{tag}");
    }
    let rule = "damaged drawing: the language is not a BCP 47 tag of 2 to 35 characters (such as `en` or `en-GB`)";
    let too_long = format!("{longest}f");
    let broken =
        ["e", "", too_long.as_str(), "en_GB", "en-", "-en", "en--GB", "e1", "x-private", "en-abcdefghi", "abcdefghi", "日本", "en GB", "en\n"];
    for tag in broken {
        assert_eq!(why(tag), rule, "{tag:?}");
    }
}

/// A fixed unit is a length unit of the expression unit table, written as the table writes it.
#[test]
fn a_fixed_unit_is_a_length_unit_of_the_expression_table() {
    let base = file(&bare());
    let units = |units: Value| {
        let mut v = base.clone();
        v["units"] = units;
        read(&v).map(|d| d.units)
    };
    for (unit, kind, _) in expr::UNITS {
        assert_eq!(units(json!({"fixed": {"unit": unit}})).is_ok(), kind == expr::Kind::Length, "{unit}");
    }
    assert_eq!(units(json!({"fixed": {"unit": "in"}})).unwrap(), Units::Fixed { unit: "in".into() });
    assert_eq!(units(json!("follow_model")).unwrap(), Units::FollowModel);
    let rule = "damaged drawing: the drawing's unit is not a length unit (such as `mm` or `in`)";
    for unit in ["furlong", "deg", "°", "none", "", " in", "in ", "IN"] {
        assert_eq!(units(json!({"fixed": {"unit": unit}})).unwrap_err().to_string(), rule, "{unit:?}");
    }
    for broken in [json!("follow_design"), json!({"fixed": "in"}), json!({"fixed": {}}), json!(null)] {
        assert!(units(broken.clone()).is_err(), "{broken}");
    }
}

/// A sheet's border is saved as a copy of its profile, field by field, and read back as saved, so
/// a later built-in never changes it; a profile's name in its place is refused.
#[test]
fn a_border_is_saved_as_a_copy_of_its_profile() {
    let v = file(&plate());
    assert_eq!(v["sheets"][0]["border"], serde_json::to_value(DrawingBorderProfile::iso()).unwrap());
    assert_eq!(v["sheets"][1]["border"], serde_json::to_value(own_border()).unwrap());
    assert_eq!(read(&v).unwrap().sheets[1].border, Some(own_border()));
    let mut edited = v.clone();
    edited["sheets"][0]["border"]["margins"]["left_mm"] = json!(15.0);
    let iso = DrawingBorderProfile::iso();
    let wider = DrawingBorderProfile { margins: Margins { left_mm: 15.0, ..iso.margins }, ..iso };
    assert_eq!(read(&edited).unwrap().sheets[0].border, Some(wider));
    edited["sheets"][0]["border"] = json!("iso");
    assert!(refusal(&edited).starts_with("damaged drawing: "));
    edited["sheets"][0]["border"] = Value::Null;
    // A sheet with no border has no corner for a title block to sit in either, so that sheet's
    // block goes with it (`a_title_block_is_a_copy_that_fits_the_sheets_drawing_border`).
    edited["sheets"][0]["title_block"] = Value::Null;
    assert_eq!(read(&edited).unwrap().sheets[0].border, None, "no drawing border");
}

/// A sheet's title block is a copy of the block it was made from, as its border is a copy of its
/// profile, and it is read only where it can be drawn: on a drawing border, inside it, cells and
/// all. The very layout that draws it decides, so a block that reads is a block that draws.
#[test]
fn a_title_block_is_a_copy_that_fits_the_sheets_drawing_border() {
    let v = file(&plate());
    let assembly_id = ids::encode(plate().sheets[0].id);
    assert_eq!(v["sheets"][0]["title_block"], serde_json::to_value(TitleBlock::built_in()).unwrap());
    assert_eq!(v["sheets"][1].get("title_block"), None, "a sheet without a block saves no field");
    assert_eq!(read(&v).unwrap().sheets[0].title_block, Some(TitleBlock::built_in()));
    // A copy: an edited block reads as the file writes it, whatever the built-in says now.
    let mut edited = v.clone();
    edited["sheets"][0]["title_block"]["outline_line_mm"] = json!(0.5);
    let thinner = TitleBlock { outline_line_mm: 0.5, ..TitleBlock::built_in() };
    assert_eq!(read(&edited).unwrap().sheets[0].title_block, Some(thinner));
    // Nothing for it to sit on, a sheet too small for it (it is 180 x 54 mm; A5 portrait's border
    // is narrower), and a block that breaks the block's own rules.
    let without = refused(&plate(), |v| v["sheets"][0]["border"] = Value::Null);
    assert_eq!(without, format!("damaged drawing: sheet {assembly_id}: its title block sits on a drawing border, which it hasn't got"));
    let small = refused(&plate(), |v| v["sheets"][0]["sheet"] = json!({"size": {"standard": "a5"}, "orientation": "portrait"}));
    assert_eq!(small, format!("damaged drawing: sheet {assembly_id}: title block: a title block fits inside the drawing border"));
    // A block makes the border's own layout part of reading the sheet, which is stricter than the
    // profile's rules on their own: paper this border leaves no drawing space on is refused with a
    // block on it, and read without one.
    let paper = json!({"size": {"custom": {"sides_mm": [5000.0, 0.5]}}, "orientation": "landscape"});
    let no_space = refused(&plate(), |v| v["sheets"][0]["sheet"] = paper.clone());
    assert_eq!(no_space, format!("damaged drawing: sheet {assembly_id}: drawing sheet: the margins leave no drawing space"));
    assert_eq!(
        refused(&plate(), |v| {
            v["sheets"][0]["sheet"] = paper.clone();
            v["sheets"][0]["title_block"] = Value::Null;
        }),
        "read"
    );
    let inside_out = refused(&plate(), |v| v["sheets"][0]["title_block"]["cells"][0]["rect_mm"] = json!([10.0, 0.0, 0.0, 10.0]));
    assert!(inside_out.starts_with(&format!("damaged drawing: sheet {assembly_id}: title block: ")), "{inside_out}");
}

/// What a sheet's views keep: the record's order and its cap, every reference resolving — the
/// source each draws, and a base view of the same sheet for a projected one to be turned from —
/// and a view's id being an object's id like any other.
#[test]
fn a_sheets_views_keep_the_records_rules() {
    let d = plate();
    let assembly_id = ids::encode(d.sheets[0].id);
    let base = *d.sheets[0].views.iter().find(|view| matches!(view.looks, Looks::Base { .. })).expect("the base view");
    let top = *d.sheets[0].views.iter().find(|view| matches!(view.looks, Looks::Projected { .. })).expect("the projected view");
    let (base_id, top_id) = (ids::encode(base.id), ids::encode(top.id));
    assert_eq!(read(&file(&d)).unwrap().sheets[0].views, d.sheets[0].views);
    // Saved by id, lowest first: the order carries nothing, so a file that gives another is damaged.
    let swapped = refused(&d, |v| v["sheets"][0]["views"].as_array_mut().expect("the views").swap(0, 1));
    assert_eq!(swapped, format!("damaged drawing: sheet {assembly_id}: its views are saved by id, lowest first"));
    // The cap, and a sheet that draws nothing.
    let with_views = |n: u64| {
        let one = |id: u64| serde_json::to_value(View { id, ..base }).expect("a view as JSON");
        let mut v = file(&d);
        v["sheets"][0]["views"] = Value::Array((1..=n).map(one).collect());
        v
    };
    assert_eq!(read(&with_views(MAX_VIEWS as u64)).unwrap().sheets[0].views.len(), MAX_VIEWS);
    assert_eq!(refusal(&with_views(MAX_VIEWS as u64 + 1)), format!("damaged drawing: sheet {assembly_id}: it draws at most {MAX_VIEWS} views"));
    assert_eq!(read(&with_views(0)).unwrap().sheets[0].views.len(), 0);
    // The source it draws is one of the drawing's: nothing, or an object of another kind.
    for missing in [ids::encode(99), assembly_id.clone()] {
        let why = refused(&d, |v| v["sheets"][0]["views"][0]["source"] = json!(missing));
        let drawn = ids::encode(d.sheets[0].views[0].id);
        assert_eq!(why, format!("damaged drawing: sheet {assembly_id}: view {drawn}: it draws source {missing}, which the drawing doesn't have"));
    }
    // What a projected view is turned from: a base view of its own sheet, and no other. Nothing,
    // an object of another kind, itself — and a base view on the drawing's other sheet, which is
    // the one that resolves and is still not its own sheet's.
    let not_a_base =
        |why: &str| format!("damaged drawing: sheet {assembly_id}: view {top_id}: it is projected from {why}, which is not a base view of its sheet");
    for wrong in [ids::encode(99), assembly_id.clone(), top_id.clone()] {
        assert_eq!(refusal(&projected_from(&d, &wrong)), not_a_base(&wrong), "projected from {wrong}");
    }
    let elsewhere = ids::encode(7);
    let mut other_sheet = projected_from(&d, &elsewhere);
    other_sheet["sheets"][1]["views"] = Value::Array(vec![serde_json::to_value(View { id: 7, ..base }).expect("a view as JSON")]);
    assert_eq!(refusal(&other_sheet), not_a_base(&elsewhere));
    // A projection shows what its base view shows: the same component of the same design.
    let another = refused(&d, |v| {
        for view in v["sheets"][0]["views"].as_array_mut().expect("the views") {
            if view["looks"].get("projected").is_some() {
                view["component"] = json!(3);
            }
        }
    });
    assert_eq!(
        another,
        format!("damaged drawing: sheet {assembly_id}: view {top_id}: it is projected from view {base_id}, which draws another component")
    );
    // A view's id is an object's id: no two objects share one, whatever kind they are.
    assert_eq!(
        refused(&d, |v| v["sheets"][0]["views"][0]["id"] = json!(assembly_id)),
        format!("damaged drawing: two objects have the id {assembly_id}")
    );
    // In memory, where NaN can be, a view's own rule.
    let mut in_memory = plate();
    Arc::make_mut(&mut in_memory.sheets[0]).views[0].position_mm = [f64::NAN, 0.0];
    let drawn = ids::encode(in_memory.sheets[0].views[0].id);
    let rule = "drawing view: a view's position is finite and at most 5000 mm from the sheet's origin";
    assert_eq!(in_memory.validate().unwrap_err().to_string(), format!("sheet {assembly_id}: view {drawn}: {rule}"));
    for edge in [5000.0, -5000.0] {
        assert_eq!(refused(&d, |v| v["sheets"][0]["views"][0]["position_mm"] = json!([edge, 0.0])), "read", "{edge} mm");
    }
    let off_paper = refused(&d, |v| v["sheets"][0]["views"][0]["position_mm"] = json!([5000.1, 0.0]));
    assert_eq!(off_paper, format!("damaged drawing: sheet {assembly_id}: view {drawn}: {rule}"));
}

/// `d`'s file with the projected view of its first sheet turned from `from`, wherever the record's
/// order put that view.
fn projected_from(d: &Drawing, from: &str) -> Value {
    let mut v = file(d);
    for view in v["sheets"][0]["views"].as_array_mut().expect("the views") {
        if view["looks"].get("projected").is_some() {
            view["looks"]["projected"]["from"] = json!(from);
        }
    }
    v
}

/// What reading refuses: each damage the rules name, refused rather than patched, and files this
/// build can't read.
#[test]
fn damaged_files_are_refused_and_never_patched() {
    let d = plate();
    let (drawing_id, plate_id) = (ids::encode(d.id), ids::encode(d.sources[0].id));
    let [assembly_id, bolt_sheet_id] = [0, 1].map(|i| ids::encode(d.sheets[i].id));
    // Two objects with one id, of one kind and of two kinds, the drawing itself among them.
    let two = |id: &str| format!("damaged drawing: two objects have the id {id}");
    assert_eq!(refused(&d, |v| v["sources"][0]["id"] = json!(drawing_id)), two(&drawing_id));
    assert_eq!(refused(&d, |v| v["sheets"][1]["id"] = json!(assembly_id)), two(&assembly_id));
    assert_eq!(refused(&d, |v| v["sources"][0]["id"] = json!(bolt_sheet_id)), two(&bolt_sheet_id));
    assert_eq!(refused(&d, |v| v["id"] = json!(assembly_id)), two(&assembly_id));
    // An id that isn't one, where an object has it and where a reference names it; no number form,
    // and no counter.
    let not_an_id = "damaged drawing: an id is 13 characters of lowercase Crockford base32, not all 0";
    for text in ["0000000000000", "FZZZZZZZZZZZZ", "000000000001", "0000000000001 ", "000000000000o", "g000000000000"] {
        assert_eq!(refused(&d, |v| v["id"] = json!(text)), not_an_id, "{text:?}");
        assert_eq!(refused(&d, |v| v["sources"][0]["id"] = json!(text)), not_an_id, "{text:?}");
        assert_eq!(refused(&d, |v| v["sheets"][1]["documents"]["source"] = json!(text)), not_an_id, "{text:?}");
    }
    assert!(refused(&d, |v| v["sources"][0]["id"] = json!(5)).starts_with("damaged drawing: invalid type"));
    assert!(refused(&d, |v| v["next_id"] = json!(6)).contains("unknown field `next_id`"));
    for zero in [|d: &mut Drawing| d.id = 0, |d: &mut Drawing| d.sources[0].id = 0] {
        let mut in_memory = d.clone();
        zero(&mut in_memory);
        assert_eq!(in_memory.validate().unwrap_err().to_string(), "an object has the id 0, which is never one");
    }
    // A reference to nothing, or to an object of another kind; a source a sheet documents removed.
    for missing in [ids::encode(99), assembly_id.clone()] {
        let expected = format!("damaged drawing: sheet {bolt_sheet_id}: it documents source {missing}, which the drawing doesn't have");
        assert_eq!(refused(&d, |v| v["sheets"][1]["documents"]["source"] = json!(missing)), expected);
    }
    let without_plate = refused(&d, |v| {
        v["sources"].as_array_mut().unwrap().remove(0);
    });
    assert_eq!(without_plate, format!("damaged drawing: sheet {assembly_id}: it documents source {plate_id}, which the drawing doesn't have"));
    // The paper, its border and its resolution, through the sheet's own rules. A sheet at the
    // limits of a custom size has no drawing space left to put a title block in, so that sheet's
    // block goes with its paper here (`a_title_block_is_a_copy_that_fits_the_sheets_drawing_border`
    // reads it the other way round).
    let custom = |sides: Value| {
        refused(&d, |v| {
            v["sheets"][0]["sheet"]["size"] = json!({"custom": {"sides_mm": sides}});
            v["sheets"][0]["title_block"] = Value::Null;
        })
    };
    assert_eq!(custom(json!([5000.0, 0.5])), "read");
    for sides in [json!([5000.1, 100.0]), json!([0.0, 100.0]), json!([-1.0, 100.0])] {
        let expected = format!("damaged drawing: sheet {assembly_id}: drawing sheet: a custom side is finite, above 0 and at most 5000 mm");
        assert_eq!(custom(sides), expected);
    }
    let why = custom(json!([1e10, 100.0]));
    assert!(why.starts_with("damaged drawing: the number ") && why.ends_with(" is out of range"), "checked before the record: {why}");
    let nan = d.to_json().unwrap().replacen("600.0", "NaN", 1);
    assert!(Drawing::from_json(&nan).unwrap_err().to_string().starts_with("not a SolidCraft drawing: "), "NaN isn't JSON");
    assert_eq!(
        refused(&d, |v| v["sheets"][1]["border"]["margins"]["top_mm"] = json!(-1.0)),
        format!("damaged drawing: sheet {bolt_sheet_id}: drawing sheet: a margin is finite and 0 to 5000 mm")
    );
    for dpi in [25.0, 2400.0] {
        assert_eq!(refused(&d, |v| v["sheets"][1]["dpi"] = json!(dpi)), "read", "{dpi}");
    }
    for dpi in [24.9, 2400.1, 0.0, -300.0] {
        let expected = format!("damaged drawing: sheet {bolt_sheet_id}: its pictures are 25 to 2400 dpi");
        assert_eq!(refused(&d, |v| v["sheets"][1]["dpi"] = json!(dpi)), expected, "{dpi}");
    }
    // In memory, where NaN can be, the same rules.
    let mut in_memory = plate();
    Arc::make_mut(&mut in_memory.sheets[1]).dpi = f64::NAN;
    assert_eq!(in_memory.validate().unwrap_err().to_string(), format!("sheet {bolt_sheet_id}: its pictures are 25 to 2400 dpi"));
    let sheet = Arc::make_mut(&mut in_memory.sheets[1]);
    sheet.dpi = DEFAULT_DPI;
    sheet.sheet.size = SheetSize::Custom { sides_mm: [f64::NAN, 100.0] };
    assert!(in_memory.validate().unwrap_err().to_string().starts_with(&format!("sheet {bolt_sheet_id}: drawing sheet: a custom side")));
    // An unknown field anywhere.
    let unknown: [fn(&mut Value); 8] = [
        |v| v["views"] = json!([]),
        |v| v["sources"][0]["name"] = json!("Plate"),
        |v| v["sheets"][0]["scale"] = json!(1),
        |v| v["sheets"][0]["sheet"]["colour"] = json!("white"),
        |v| v["sheets"][0]["border"]["style"] = json!("iso"),
        |v| v["sheets"][0]["documents"]["view"] = json!(1),
        |v| v["sheets"][0]["excluded"]["export"] = json!(true),
        |v| v["units"]["fixed"]["precision"] = json!(2),
    ];
    for (i, edit) in unknown.into_iter().enumerate() {
        let why = refused(&d, edit);
        assert!(why.starts_with("damaged drawing: ") && why.contains("unknown field"), "{i}: {why}");
    }
    // A newer build's file is refused as one before anything in it is read: a field this build
    // doesn't know, or a number out of its range.
    let (current, newer) = (DRAWING.current(), DRAWING.current() + 1);
    let why = refused(&d, |v| {
        v["format"] = json!(DRAWING.format_string(newer));
        v["views"] = json!([]);
        v["sheets"][0]["dpi"] = json!(1e10);
    });
    assert_eq!(
        why,
        format!(
            "this drawing was saved by a newer SolidCraft (file format {newer}; this version reads formats up to {current}): update SolidCraft to open it"
        )
    );
    // A list of pending changes means nothing beside a format made of none; beside the pending
    // number, a change this build doesn't know is a newer build's.
    let why = refused(&d, |v| v["format_pending"] = json!(["a-later-change"]));
    if DRAWING.pending_names().is_empty() {
        assert_eq!(why, "read");
    } else {
        assert!(why.contains("newer SolidCraft"), "{why}");
    }
    // A design is never read as a drawing, nor a drawing as a design.
    let design = crate::Document::new("Plate").to_json();
    let design_format = crate::format::format_string(crate::format::FORMAT);
    assert_eq!(Drawing::from_json(&design).unwrap_err().to_string(), format!("not a SolidCraft drawing (format `{design_format}`)"));
    let drawing_format = DRAWING.format_string(current);
    assert_eq!(
        crate::Document::from_json(&d.to_json().unwrap()).unwrap_err().to_string(),
        format!("not a SolidCraft design (format `{drawing_format}`)")
    );
    // Not an object, not JSON, nested too deeply, too large.
    assert_eq!(Drawing::from_json("[]").unwrap_err().to_string(), "not a SolidCraft drawing (expected a JSON object)");
    assert!(Drawing::from_json("").unwrap_err().to_string().starts_with("not a SolidCraft drawing: "));
    let deep: Value = serde_json::from_str(&format!("{}1{}", "[".repeat(100), "]".repeat(100))).unwrap();
    assert_eq!(refused(&d, |v| v["views"] = deep), "damaged drawing: nested too deeply");
    let deeper = format!("{{\"format\": \"{drawing_format}\", \"views\": {}1{}}}", "[".repeat(200), "]".repeat(200));
    assert!(Drawing::from_json(&deeper).unwrap_err().to_string().starts_with("not a SolidCraft drawing: "));
    let text = d.to_json().unwrap();
    let at_cap = text.clone() + &" ".repeat(MAX_DRAWING_BYTES - text.len());
    assert_eq!(Drawing::from_json(&at_cap).unwrap(), d);
    assert_eq!(Drawing::from_json(&(at_cap + " ")).unwrap_err().to_string(), "drawing too large (at most 16 MiB)");
}

/// A run never gives an id twice: not one in use, and not a removed object's. Only another run from
/// the same seed, which tests and replays start from the same file, can give a removed id again.
#[test]
fn a_run_never_gives_an_id_twice() {
    let mut id_source = IdSource::seeded(SEED);
    let a4 = drawing_sheet::Sheet::standard(PaperSize::A4, Orientation::Landscape);
    let mut d = Drawing::new("Ids", "en", "Sheet", a4, None, &BTreeSet::new(), &mut id_source).unwrap();
    let mut given = BTreeSet::from([d.id, d.sheets[0].id]);
    for _ in 0..64 {
        let id = d.allocate_id(&mut id_source).unwrap();
        d.sheets.push(Arc::new(Sheet { id, ..(*d.sheets[0]).clone() }));
        assert!(given.insert(id), "{id:#x} given twice");
    }
    let removed = d.sheets.pop().map_or(0, |s| s.id);
    for _ in 0..10_000 {
        let id = d.allocate_id(&mut id_source).unwrap();
        assert!(id != removed && given.insert(id), "{id:#x} given twice");
    }
    // The same seed, from the start, over a file it made: every draw up to the removed id is in use.
    let reopened = Drawing::from_json(&d.to_json().unwrap()).unwrap();
    let again = reopened.allocate_id(&mut IdSource::seeded(SEED)).unwrap();
    assert!(!reopened.ids().any(|id| id == again));
    assert_eq!(again, removed);
}

/// Properties follow the design metadata's rules: other tools' data and a well-known key of another
/// kind are kept as they are, never read as numbers, while the same number elsewhere is refused.
#[test]
fn properties_are_kept_as_they_are() {
    let mut v = file(&plate());
    v["properties"]["com.example.pdm"] = json!({"huge": 1e300, "ids": [1, 2, 3]});
    v["properties"]["revision"] = json!(3);
    let d = read(&v).unwrap();
    assert_eq!(d.properties.get("com.example.pdm"), Some(&json!({"huge": 1e300, "ids": [1, 2, 3]})));
    assert_eq!((d.properties.revision(), d.properties.get("revision")), (None, Some(&json!(3))));
    assert_eq!(read(&file(&d)).unwrap(), d);
    v["sheets"][0]["dpi"] = json!(1e300);
    let why = refusal(&v);
    assert!(why.starts_with("damaged drawing: the number ") && why.ends_with(" is out of range"), "{why}");
}

/// A drawing whose JSON would be over the read limit isn't saved. Other tools' properties are kept
/// as they came, and indenting them can grow a file read in under the limit past it, so the save
/// refuses rather than write a file that won't open: every drawing saved opens again.
#[test]
fn a_drawing_too_large_to_reopen_is_not_saved() {
    let file = |zeros: usize| {
        let items = vec!["0"; zeros].join(",");
        let sheet = r#"{"id":"0000000000001","name":"S","sheet":{"size":{"standard":"a4"}}}"#;
        let format = DRAWING.format_string(1);
        format!(
            r#"{{"format":"{format}","id":"0000000000002","name":"A","sources":[],"language":"en","sheets":[{sheet}],"properties":{{"com.example":{{"items":[{items}]}}}}}}"#
        )
    };
    // The case found in review (#3891): 2,000,000 zeros read in at about 4 MB and indent past 16 MiB.
    let big = Drawing::from_json(&file(2_000_000)).unwrap();
    assert_eq!(big.to_json().unwrap_err().to_string(), "drawing too large to save (at most 16 MiB)");
    let small = Drawing::from_json(&file(100_000)).unwrap();
    assert_eq!(Drawing::from_json(&small.to_json().unwrap()).unwrap(), small);
}

/// A new drawing has one sheet, the drawing's id the source's first and the sheet's the next,
/// follows its designs' units and renders pictures at 300 dpi; its id is none of its design's other
/// drawings'; a name, language or border that breaks the rules is refused.
#[test]
fn a_new_drawing_has_one_sheet_and_keeps_the_rules() {
    let d = bare();
    let mut seeded = IdSource::seeded(SEED);
    let first = seeded.next(&BTreeSet::new()).unwrap();
    let second = seeded.next(&BTreeSet::new()).unwrap();
    assert_eq!((d.id, d.sheets.len(), d.sheets[0].id, d.sheets[0].dpi, &d.units), (first, 1, second, DEFAULT_DPI, &Units::FollowModel));
    assert_eq!((d.format, d.format_pending), (DRAWING.format_string(DRAWING.current()), DRAWING.pending_names()));
    let a4 = drawing_sheet::Sheet::standard(PaperSize::A4, Orientation::Landscape);
    let beside = Drawing::new("Beside", "en", "Sheet", a4, None, &BTreeSet::from([first]), &mut IdSource::seeded(SEED)).unwrap();
    assert!(beside.id != first && beside.sheets[0].id != beside.id, "the design's drawing keeps its id");
    let thin = DrawingBorderProfile { drawing_border_line_mm: 0.0, ..DrawingBorderProfile::iso() };
    let broken = [("", "en", "Sheet", None), ("Bare", "en_GB", "Sheet", None), ("Bare", "en", "\n", None), ("Bare", "en", "Sheet", Some(thin))];
    for (name, language, sheet, border) in broken {
        let made = Drawing::new(name, language, sheet, a4, border, &BTreeSet::new(), &mut IdSource::seeded(SEED));
        assert!(made.is_err(), "{name:?} {language:?} {sheet:?} {border:?}");
    }
}

/// Hostile input never panics: a valid file's text cut, overwritten and given lies, and its JSON
/// values replaced, repeated and dropped. Each read gives an error, or a drawing that keeps the
/// rules and reads back as it saves.
#[test]
fn mutated_files_give_an_error_or_a_valid_drawing_never_a_panic() {
    let good = plate().to_json().unwrap();
    let tree: Value = serde_json::from_str(&good).unwrap();
    let nodes = count(&tree) as u64;
    let lies = [
        json!(null),
        json!(0),
        json!(-1),
        json!(1),
        json!(1e10),
        json!(u64::MAX),
        json!(""),
        json!("x".repeat(300)),
        json!("0000000000000"),
        json!("0000000000001"),
        json!("fzzzzzzzzzzzz"),
        json!("FZZZZZZZZZZZZ"),
        json!([]),
        json!({}),
        json!(true),
        json!("follow_model"),
        json!({"fixed": {"unit": "deg"}}),
    ];
    let words = ["NaN", "1e400", "-0", "18446744073709551616", "\"", "{", "]", ",", "null", "\u{0}", "é", "o", "I"];
    let mut seed: u64 = 0x9e37_79b9_7f4a_7c15;
    let mut rnd = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    let mut read_back = 0;
    for i in 0..10_000 {
        let text = if i % 2 == 0 {
            let mut v = tree.clone();
            let lie = &lies[(rnd() % lies.len() as u64) as usize];
            mutate(&mut v, &mut ((rnd() % nodes) as usize), lie, rnd() % 3);
            v.to_string()
        } else {
            let mut s = good.clone().into_bytes();
            for _ in 0..1 + rnd() % 3 {
                // A run of up to 15 bytes taken out, overwritten by one byte or replaced by a lie.
                let at = (rnd() % (s.len() as u64 + 1)) as usize;
                let cut = ((rnd() % 16) as usize).min(s.len() - at);
                let byte = [(rnd() >> 56) as u8];
                let put: &[u8] = match rnd() % 3 {
                    0 => &[],
                    1 => &byte,
                    _ => words[(rnd() % words.len() as u64) as usize].as_bytes(),
                };
                let next = [&s[..at], put, &s[at + cut..]].concat();
                s = next;
                if rnd() % 8 == 0 {
                    s.truncate(at);
                }
            }
            String::from_utf8_lossy(&s).into_owned()
        };
        if let Ok(d) = Drawing::from_json(&text) {
            assert!(d.validate().is_ok() && d.ids().all(|id| id != 0), "{text}");
            assert_eq!(Drawing::from_json(&d.to_json().unwrap()).unwrap(), d, "{text}");
            read_back += 1;
        }
    }
    assert!(read_back > 50, "only {read_back} mutated files read: the mutations hardly reach the record's rules");
}

/// A design holding drawings, as plain JSON: each drawing inline under `drawings`, as its own
/// record with its own format. While the change is pending the design names `drawings` in its
/// `format_pending`, so a build without it refuses the design (`format::tests`) rather than open it
/// and save it without them. They read back in their order, by name, then id, and save as the same
/// bytes; a design without drawings writes no `drawings`.
#[test]
fn a_design_keeps_its_drawings_inline_in_plain_json() {
    let named = |name: &str, id: u64| Arc::new(Drawing { name: name.into(), id, ..plate() });
    let mut doc = crate::Document::new("Plate");
    doc.drawings = vec![named("Details", 7), Arc::new(plate()), named("Details", 3), named("Assembly", u64::MAX)];
    let read = crate::Document::from_json(&doc.to_json()).unwrap();
    let order: Vec<_> = read.drawings.iter().map(|d| (d.name.as_str(), d.id)).collect();
    assert_eq!(order, [("Assembly", u64::MAX), ("Details", 3), ("Details", 7), ("Plate assembly", plate().id)]);
    let saved = read.to_json();
    assert_eq!(crate::Document::from_json(&saved).unwrap().to_json(), saved, "save, open, save: the same bytes");
    let v: Value = serde_json::from_str(&saved).unwrap();
    assert_eq!(v["drawings"][3], file(&plate()));
    assert!(v["format_pending"].as_array().is_some_and(|names| names.contains(&json!("drawings"))), "{}", v["format_pending"]);
    let empty: Value = serde_json::from_str(&crate::Document::new("Plate").to_json()).unwrap();
    assert_eq!(empty.get("drawings"), None);
}

/// What a design refuses in its drawings, each time the whole design, never by dropping a drawing:
/// one a newer build saved (a newer number, or a change this build doesn't know), a damaged one,
/// `drawings` that isn't a list, two with one id, and more than a design may hold, counted before
/// any is read. A drawing's numbers are its own family's to check: other tools' data in its
/// properties is kept, a sheet's number out of range refused.
#[test]
fn a_drawing_a_design_cant_read_refuses_the_design() {
    let mut doc = crate::Document::new("Plate");
    doc.drawings = vec![Arc::new(plate())];
    let design: Value = serde_json::from_str(&doc.to_json()).unwrap();
    let refused = |edit: &dyn Fn(&mut Value)| {
        let mut v = design.clone();
        edit(&mut v);
        crate::Document::from_json(&v.to_string()).map_or_else(|e| e.to_string(), |d| format!("read {}", d.drawings.len()))
    };
    let (current, newer) = (DRAWING.current(), DRAWING.current() + 1);
    assert_eq!(
        refused(&|v| v["drawings"][0]["format"] = json!(DRAWING.format_string(newer))),
        format!(
            "drawing 1: this drawing was saved by a newer SolidCraft (file format {newer}; this version reads formats up to {current}): update SolidCraft to open it"
        )
    );
    let later = refused(&|v| v["drawings"][0]["format_pending"] = json!(["a-later-change"]));
    assert!(if DRAWING.pending_names().is_empty() { later == "read 1" } else { later.contains("newer SolidCraft") }, "{later}");
    let damaged = refused(&|v| v["drawings"][0]["language"] = json!("e"));
    assert!(damaged.starts_with("drawing 1: damaged drawing: the language"), "{damaged}");
    assert_eq!(refused(&|v| v["drawings"] = json!({})), "damaged design: `drawings` is not a list");
    let twice = refused(&|v| {
        let again = v["drawings"][0].clone();
        v["drawings"].as_array_mut().unwrap().push(again);
    });
    assert_eq!(twice, format!("damaged design: two drawings have the id {}", ids::encode(plate().id)));
    let at_cap: Vec<Value> = (1..=MAX_DRAWINGS as u64)
        .map(|id| {
            let mut d = file(&plate());
            d["id"] = json!(ids::encode(id));
            d
        })
        .collect();
    assert_eq!(refused(&|v| v["drawings"] = Value::Array(at_cap.clone())), format!("read {MAX_DRAWINGS}"));
    let over = refused(&|v| v["drawings"] = Value::Array(vec![json!({}); MAX_DRAWINGS + 1]));
    assert_eq!(over, format!("damaged design: a design holds at most {MAX_DRAWINGS} drawings"));
    assert_eq!(refused(&|v| v["drawings"][0]["properties"]["com.example"] = json!({"huge": 1e300})), "read 1");
    let out_of_range = refused(&|v| v["drawings"][0]["sheets"][0]["dpi"] = json!(1e300));
    assert!(out_of_range.starts_with("drawing 1: damaged drawing: the number "), "{out_of_range}");
}

/// The number of values in `v`, itself included.
fn count(v: &Value) -> usize {
    1 + match v {
        Value::Array(a) => a.iter().map(count).sum::<usize>(),
        Value::Object(o) => o.values().map(count).sum(),
        _ => 0,
    }
}

/// Change the `pick`th value of `v`, depth first: `how` 1 repeats a list's first element (two
/// objects with one id), 2 drops an object's first field, and otherwise `lie` takes its place.
fn mutate(v: &mut Value, pick: &mut usize, lie: &Value, how: u64) -> bool {
    if *pick == 0 {
        match (v, how) {
            (Value::Array(a), 1) if !a.is_empty() => {
                let first = a[0].clone();
                a.push(first);
            }
            (Value::Object(o), 2) if !o.is_empty() => {
                let key = o.keys().next().cloned().unwrap_or_default();
                o.remove(&key);
            }
            (v, _) => *v = lie.clone(),
        }
        return true;
    }
    *pick -= 1;
    match v {
        Value::Array(a) => a.iter_mut().any(|x| mutate(x, pick, lie, how)),
        Value::Object(o) => o.values_mut().any(|x| mutate(x, pick, lie, how)),
        _ => false,
    }
}
