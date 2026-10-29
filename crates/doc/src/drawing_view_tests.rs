use serde_json::json;
use solidcraft_geom::Vec3;

use super::*;

/// Every field has its default, and the settings read and write back the same.
#[test]
fn view_settings_default_to_a_front_view_that_fits_its_page() {
    let settings: ViewSettings = serde_json::from_value(json!({})).unwrap();
    assert_eq!(settings, ViewSettings::default());
    assert_eq!(settings.view, ViewDirection::Named(NamedView::Front));
    assert_eq!((settings.scale, settings.display_style, settings.tangent_edges), (None, VisualStyle::Visible, TangentEdges::Hide));
    assert_eq!(settings.validate().unwrap(), NamedView::Front.basis());
    assert_eq!(serde_json::to_value(settings).unwrap().get("scale"), None, "no scale is written, not null");
    let full = json!({
        "view": {"direction": [0.0, 1.0, 0.0], "up": [0.0, 0.0, 1.0], "origin": [1.0, 2.0, 3.0]},
        "scale": "1:2", "display_style": "visible_hidden", "reference_style": "shaded",
        "reference_hides_model": true, "reference_margin_mm": 5.0, "tangent_edges": "show",
    });
    let read: ViewSettings = serde_json::from_value(full.clone()).unwrap();
    assert_eq!(serde_json::to_value(read).unwrap(), full);
    assert_eq!(read.validate().unwrap().origin, Vec3::new(1.0, 2.0, 3.0));
    let iso: ViewSettings = serde_json::from_value(json!({"view": "iso"})).unwrap();
    assert_eq!(iso.view, ViewDirection::Named(NamedView::IsoFrontRight));
}

/// What the settings refuse: unknown fields, unknown names, a basis that can't be built and a
/// margin out of range.
#[test]
fn view_settings_refuse_what_they_cant_draw() {
    for not_settings in [
        json!({"scael": "1:2"}),
        json!({"view": 3}),
        json!({"view": "Front"}),
        json!({"view": {"direction": [0, 1, 0]}}),
        json!({"view": {"direction": [0, 1, 0], "up": [0, 0, 1], "roll": 1}}),
        json!({"scale": 0.5}),
        json!({"display_style": "hidden"}),
        json!({"tangent_edges": "thin"}),
    ] {
        assert!(serde_json::from_value::<ViewSettings>(not_settings.clone()).is_err(), "{not_settings}");
    }
    let refused = [
        json!({"view": {"direction": [0, 0, 0], "up": [0, 0, 1]}}),
        json!({"view": {"direction": [0, 0, 1], "up": [0, 0, -1]}}),
        json!({"reference_margin_mm": -0.001}),
        json!({"reference_margin_mm": 100000.001}),
    ];
    for settings in refused {
        let read: ViewSettings = serde_json::from_value(settings.clone()).unwrap();
        assert!(read.validate().is_err(), "{settings}");
    }
    let edge: ViewSettings = serde_json::from_value(json!({"reference_margin_mm": 100000.0})).unwrap();
    assert!(edge.validate().is_ok());
}

/// A saved view writes what it draws, which way it looks and where it sits, and nothing that was
/// measured from the design: a view that draws just what it follows saves no style at all, and a
/// field the record doesn't know is refused rather than ignored.
#[test]
fn a_saved_view_writes_what_it_draws_and_nothing_measured() {
    let base = json!({
        "id": "0000000000001", "source": "0000000000002", "component": 0,
        "position_mm": [140.0, 150.0], "looks": {"base": {"view": "front"}}
    });
    let view: View = serde_json::from_value(base.clone()).unwrap();
    assert_eq!(view.looks, Looks::Base { view: ViewDirection::Named(NamedView::Front) });
    assert_eq!(view.style, ViewStyle::default());
    view.validate().unwrap();
    assert_eq!(serde_json::to_value(view).unwrap(), base, "a style of nothing writes no field");
    let projected = json!({
        "id": "0000000000003", "source": "0000000000002", "component": 4,
        "position_mm": [140.0, 240.0], "looks": {"projected": {"from": "0000000000001", "side": "above"}},
        "style": {"scale": "1:2", "tangent_edges": "show"}
    });
    let view: View = serde_json::from_value(projected.clone()).unwrap();
    assert_eq!(view.looks, Looks::Projected { from: 1, side: ProjectedSide::Above });
    assert_eq!(view.style.scale, Some(Scale::new(1.0, 2.0).unwrap()));
    view.validate().unwrap();
    assert_eq!(serde_json::to_value(view).unwrap(), projected);
    // Nothing measured: a view has no extents, no lines and no basis of its own, and its style is
    // under `style`, never beside it.
    for (key, value) in [("extents_mm", json!([1.0, 2.0])), ("lines", json!([])), ("basis", json!({"x": [1, 0, 0]})), ("scale", json!("1:2"))] {
        let mut v = base.clone();
        v[key] = value;
        assert!(serde_json::from_value::<View>(v).is_err(), "{key}");
    }
    // Where it sits: finite, and at most 5000 mm from the sheet's origin along each side.
    let at = |position_mm: [f64; 2]| View { position_mm, ..view }.validate();
    for on_paper in [[5000.0, -5000.0], [0.0, 0.0]] {
        assert!(at(on_paper).is_ok(), "{on_paper:?}");
    }
    for off in [[5000.1, 0.0], [0.0, f64::NAN], [f64::INFINITY, 0.0]] {
        assert!(at(off).is_err(), "{off:?}");
    }
    // A base view looks a way that can be built; a projected one takes its parent's, so a view it
    // could never look is a view that is never read.
    let looking = |up: [f64; 3]| Looks::Base { view: ViewDirection::Looking(Looking { direction: [0.0, 0.0, 1.0], up, origin: [0.0; 3] }) };
    assert!(View { looks: looking([0.0, 1.0, 0.0]), ..view }.validate().is_ok());
    assert!(View { looks: looking([0.0, 0.0, 1.0]), ..view }.validate().is_err(), "an up along the direction");
}

/// A view's style says only what it draws differently: what it leaves out comes from the view it is
/// projected from, and from the settings' own defaults where it follows none.
#[test]
fn a_views_style_takes_what_it_doesnt_say_from_the_view_it_follows() {
    let empty = ViewStyle::default();
    assert!(empty.is_empty());
    assert_eq!(serde_json::to_value(empty).unwrap(), json!({}), "a style of nothing writes nothing");
    assert_eq!(empty.settings(ViewDirection::default()), ViewSettings::default());
    let half = Scale::new(1.0, 2.0).unwrap();
    let parent = ViewStyle { scale: Some(half), tangent_edges: Some(TangentEdges::Show), ..ViewStyle::default() };
    assert!(!parent.is_empty());
    assert_eq!(empty.over(&parent), parent, "nothing of its own: it draws what its parent draws");
    let own = ViewStyle { display_style: Some(VisualStyle::VisibleHidden), tangent_edges: Some(TangentEdges::Hide), ..ViewStyle::default() };
    let settled = own.over(&parent);
    assert_eq!(settled.scale, Some(half), "the parent's, which it doesn't give");
    assert_eq!(settled.tangent_edges, Some(TangentEdges::Hide), "its own, over the parent's");
    assert_eq!(settled.display_style, Some(VisualStyle::VisibleHidden));
    // As settings: what the style says, and the settings' defaults for the rest.
    let settings = settled.settings(ViewDirection::Named(NamedView::Top));
    let default = ViewSettings::default();
    assert_eq!(settings.view, ViewDirection::Named(NamedView::Top));
    assert_eq!((settings.scale, settings.display_style, settings.tangent_edges), (Some(half), VisualStyle::VisibleHidden, TangentEdges::Hide));
    assert_eq!((settings.reference_style, settings.reference_margin_mm), (default.reference_style, default.reference_margin_mm));
    // The margin keeps the settings' own rule, through the style.
    let margin = |reference_margin_mm| ViewStyle { reference_margin_mm: Some(reference_margin_mm), ..ViewStyle::default() }.validate();
    assert!(margin(MAX_REFERENCE_MARGIN_MM).is_ok());
    for out_of_range in [-0.001, MAX_REFERENCE_MARGIN_MM + 0.001, f64::NAN] {
        assert!(margin(out_of_range).is_err(), "{out_of_range}");
    }
}
