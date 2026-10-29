//! Drawing views of the session's model (W07, #5866): what a view shows, how far it reaches, and
//! what a view saved on a sheet draws when it is drawn ([`drawn`], W12).
//!
//! A view shows every body the design places, where its occurrences place it. The browser's eye
//! counts for nothing: hiding a part in the model never changes a drawing.

use std::collections::BTreeMap;

use solidcraft_doc::drawing::{Drawing, Sheet};
use solidcraft_doc::drawing_view::{Looks, View as Saved, ViewStyle};
use solidcraft_doc::ids;
use solidcraft_geom::drawing_view::{PRESETS, Scale, ViewFrame, projected_basis};
use solidcraft_geom::{Aabb2, Plane, Vec2};
use solidcraft_render::VisualStyle;

use crate::mesh_lines::{self, View};
use crate::params::bad;
use crate::{Result, Session};

/// How far a view reaches on its own plane, in model millimetres.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Extents {
    /// The bodies' box in the plane's own coordinates ([`Plane::to_local`]); empty with no bodies.
    pub model: Aabb2,
    /// How far a curved face may lie outside its display mesh: the largest display tolerance.
    pub error_mm: f64,
    pub bodies: usize,
}

/// The extents of every body the design places, seen on `basis`.
pub fn extents(s: &Session, basis: &Plane) -> Extents {
    let st = s.world_state();
    st.bodies.iter().fold(Extents { model: Aabb2::EMPTY, error_mm: 0.0, bodies: 0 }, |e, body| Extents {
        model: e.model.union(&body.mesh_bounds_in(basis)),
        error_mm: e.error_mm.max(body.display_tol()),
        bodies: e.bodies + 1,
    })
}

/// The paper chord a view's lines are sampled to, and the model chord that gives: 0.01 mm on the
/// sheet divided by the scale, kept inside the range `mesh_lines` accepts.
pub const PAPER_CHORD_MM: f64 = 0.01;
const CHORD_RANGE: std::ops::RangeInclusive<f64> = 0.0001..=1.0;

/// The model chord for a view drawn at `factor` (paper : model), clamped to [`CHORD_RANGE`].
pub fn chord_mm(factor: f64) -> f64 {
    let wanted = if factor.is_finite() && factor > 0.0 { PAPER_CHORD_MM / factor } else { *CHORD_RANGE.end() };
    wanted.clamp(*CHORD_RANGE.start(), *CHORD_RANGE.end())
}

/// A view's lines on its own plane, in model millimetres: polylines rather than loose segments,
/// so a circle stays one run. Backend-neutral, so the sheet painter, SVG and DXF can all draw it.
///
/// The accuracy is "sampled ± `chord_mm`": the lines come from the bodies' export meshes, so a
/// curved face is followed to within the chord and no closer.
#[derive(Clone, Debug, PartialEq)]
pub struct ViewLines {
    pub visible: Vec<Vec<Vec2>>,
    pub hidden: Vec<Vec<Vec2>>,
    pub chord_mm: f64,
    /// Occlusion segments tested, for the snapshot's timings.
    pub segments: usize,
}

/// The lines of every body the design places, seen on `basis` and sampled to `chord`.
///
/// Every placed body counts, as [`extents`] does: hiding a part in the model never changes a
/// drawing. No sketch curves — a drawing shows the model, not what is being drawn on it.
pub fn lines(s: &Session, cmd: &str, basis: &Plane, chord: f64) -> Result<ViewLines> {
    let scene = mesh_lines::prepare(s, cmd, chord, |_| true)?;
    let view = View::orthographic(basis).map_err(|e| crate::params::bad(cmd, e))?;
    let drawing = mesh_lines::drawing(&scene, &[], &view).map_err(|e| crate::params::bad(cmd, e))?;
    let pieces = |hidden: bool| chain(drawing.lines.iter().filter(|l| l.hidden == hidden).map(|l| (l.a, l.b)).collect());
    Ok(ViewLines { visible: pieces(false), hidden: pieces(true), chord_mm: chord, segments: drawing.segments })
}

/// The largest preset scale at which a view `size` (model mm), centred at `position`, lies inside
/// `border`: the room is twice the distance from the position to the nearer side, each way. What
/// a view with no scale of its own is drawn at.
pub fn fit(cmd: &str, size: Vec2, position: Vec2, border: Aabb2) -> Result<Scale> {
    let room =
        Vec2::new(2.0 * (position.x - border.min.x).min(border.max.x - position.x), 2.0 * (position.y - border.min.y).min(border.max.y - position.y));
    Scale::fit(size, room).ok_or_else(|| {
        let name = |s: Option<&Scale>| s.map(|s| s.to_string()).unwrap_or_default();
        let presets = format!("from {} to {}", name(PRESETS.first()), name(PRESETS.last()));
        bad(cmd, format!("the view fits the sheet's drawing area at no preset scale {presets} here: give a scale, a position or a larger sheet"))
    })
}

/// A view saved on a sheet, worked out: the frame it draws through, the scale it settled on and
/// the lines it draws.
#[derive(Clone, Debug)]
pub struct Drawn {
    /// The saved view's id.
    pub view: u64,
    pub frame: ViewFrame,
    pub scale: Scale,
    /// Whether the scale was fitted here rather than saved on the view.
    pub fitted: bool,
    pub visible: Vec<Vec<Vec2>>,
    /// Empty unless the view's style draws hidden lines.
    pub hidden: Vec<Vec<Vec2>>,
}

/// Every view `sheet` draws, worked out in the record's order, with `border` as its drawing area.
///
/// Base views are worked out first: a projection is lined up with the frame of the view it comes
/// from ([`ViewFrame::aligned`]), so that frame has to exist before it. The record keeps a
/// projection's parent a base view of the same sheet, so two passes are enough and there is no
/// chain to walk.
pub fn drawn(s: &Session, cmd: &str, d: &Drawing, sheet: &Sheet, border: Aabb2) -> Result<Vec<Drawn>> {
    let mut frames: BTreeMap<u64, (ViewFrame, Scale, bool)> = BTreeMap::new();
    for view in sheet.views.iter().filter(|view| matches!(view.looks, Looks::Base { .. })) {
        let Looks::Base { view: direction } = view.looks else { continue };
        let style = view.style;
        let basis = direction.basis().map_err(|e| bad(cmd, e.to_string()))?;
        let shown = measured(s, cmd, view, &basis, &style)?;
        let at = Vec2::new(view.position_mm[0], view.position_mm[1]);
        let (scale, fitted) = match style.scale {
            Some(scale) => (scale, false),
            None => (fit(cmd, shown.size(), at, border)?, true),
        };
        let frame = ViewFrame::new(basis, scale, shown.center(), at).map_err(|e| bad(cmd, e.to_string()))?;
        frames.insert(view.id, (frame, scale, fitted));
    }
    for view in sheet.views.iter().filter(|view| matches!(view.looks, Looks::Projected { .. })) {
        let Looks::Projected { from, side } = view.looks else { continue };
        let parent = sheet.views.iter().find(|other| other.id == from);
        let parent = parent.ok_or_else(|| bad(cmd, format!("view {} is projected from a view its sheet hasn't got", ids::encode(view.id))))?;
        let style = view.style.over(&parent.style);
        if view.style.scale.is_some() {
            let how = "a projected view is drawn at the scale of the view it comes from, so it takes no scale of its own";
            return Err(bad(cmd, format!("view {}: {how}", ids::encode(view.id))));
        }
        let (frame, scale, fitted) = *frames.get(&from).ok_or_else(|| bad(cmd, "a projection's base view was not worked out first"))?;
        let basis = projected_basis(&frame.basis(), side, d.projection).map_err(|e| bad(cmd, e.to_string()))?;
        let shown = measured(s, cmd, view, &basis, &style)?;
        let at = Vec2::new(view.position_mm[0], view.position_mm[1]);
        let turned = frame.aligned(side, d.projection, shown.center(), at).map_err(|e| bad(cmd, e.to_string()))?;
        frames.insert(view.id, (turned, scale, fitted));
    }

    // The lines, in the record's order, so the scene's groups come out the same way twice.
    let mut out = Vec::with_capacity(sheet.views.len());
    for view in &sheet.views {
        let (frame, scale, fitted) = *frames.get(&view.id).ok_or_else(|| bad(cmd, "a view was not worked out"))?;
        let style = match view.looks {
            Looks::Projected { from, .. } => {
                sheet.views.iter().find(|other| other.id == from).map_or(view.style, |parent| view.style.over(&parent.style))
            }
            Looks::Base { .. } => view.style,
        };
        let basis = frame.basis();
        let lines = lines(s, cmd, &basis, chord_mm(scale.factor()))?;
        let hidden = if shows_hidden(&style) { lines.hidden } else { Vec::new() };
        out.push(Drawn { view: view.id, frame, scale, fitted, visible: lines.visible, hidden });
    }
    Ok(out)
}

/// A view's extents on `basis`, with its reference margin added, refusing what cannot be drawn
/// yet: the shaded styles need the drawing renderer (#5624), and tangent edges are decided by the
/// mesh (#5866), exactly as `drawing.view.snapshot` refuses them.
fn measured(s: &Session, cmd: &str, view: &Saved, basis: &Plane, style: &ViewStyle) -> Result<Aabb2> {
    let named = ids::encode(view.id);
    let shaded = style.display_style.is_some_and(VisualStyle::is_shaded) || style.reference_style.is_some_and(VisualStyle::is_shaded);
    if shaded {
        return Err(bad(cmd, format!("view {named}: not supported yet: shaded views need the drawing renderer (#5624)")));
    }
    if style.tangent_edges == Some(solidcraft_doc::drawing_view::TangentEdges::Show) {
        return Err(bad(cmd, format!("view {named}: not supported yet: tangent_edges show needs the mesh edge classification (#5866)")));
    }
    let extents = extents(s, basis);
    if extents.model.is_empty() {
        return Err(bad(cmd, format!("view {named}: its source has no bodies")));
    }
    Ok(extents.model.inflated(style.reference_margin_mm.unwrap_or(0.0)))
}

/// Whether a view's style draws the lines a nearer surface covers.
fn shows_hidden(style: &ViewStyle) -> bool {
    matches!(style.display_style, Some(VisualStyle::VisibleHidden | VisualStyle::ShadedVisibleHidden))
}
/// Join segments end to end into polylines. A join where exactly two segments meet continues the
/// run; a junction of three or more ends it, so a corner never invents a path through it. Points
/// are matched on a grid relative to the drawing's extent, as the segments come from one
/// projection and share their endpoints exactly.
fn chain(segments: Vec<(Vec2, Vec2)>) -> Vec<Vec<Vec2>> {
    if segments.is_empty() {
        return Vec::new();
    }
    let extent = segments.iter().flat_map(|(a, b)| [a, b]).fold(1.0_f64, |s, p| s.max(p.x.abs()).max(p.y.abs()));
    let eps = extent * 1e-8;
    let key = |p: Vec2| [(p.x / eps).round() as i64, (p.y / eps).round() as i64];
    let mut at: BTreeMap<[i64; 2], Vec<usize>> = BTreeMap::new();
    for (i, (a, b)) in segments.iter().enumerate() {
        at.entry(key(*a)).or_default().push(i);
        at.entry(key(*b)).or_default().push(i);
    }
    // A segment whose endpoints quantise together is a dot: it joins nothing and is dropped.
    let mut used = vec![false; segments.len()];
    let mut runs = Vec::new();
    // Ends first, so an open run is walked from its end rather than cut in the middle; then
    // whatever is left, which is closed loops.
    let ends_first = (0..segments.len())
        .filter(|&i| {
            let (a, b) = segments[i];
            at.get(&key(a)).is_none_or(|l| l.len() != 2) || at.get(&key(b)).is_none_or(|l| l.len() != 2)
        })
        .chain(0..segments.len());
    for start in ends_first {
        if used[start] {
            continue;
        }
        let (a, b) = segments[start];
        used[start] = true;
        // Walk from the end that is not a plain join, so the run reads along the curve.
        let (mut tail, head) = if at.get(&key(a)).is_some_and(|l| l.len() == 2) { (a, b) } else { (b, a) };
        let mut points = vec![head, tail];
        while let Some(next) = at.get(&key(tail)).filter(|l| l.len() == 2).and_then(|l| l.iter().copied().find(|&i| !used[i])) {
            used[next] = true;
            let (a, b) = segments[next];
            tail = if key(a) == key(tail) { b } else { a };
            points.push(tail);
            if key(tail) == key(points[0]) {
                break;
            }
        }
        runs.push(points);
    }
    runs
}
