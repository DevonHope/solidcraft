//! Drawing views of the session's model (W07, #5866): what a view shows, how far it reaches, and
//! what a view saved on a sheet draws when it is drawn ([`drawn`], W12).
//!
//! A view shows every body the design places, where its occurrences place it. The browser's eye
//! counts for nothing: hiding a part in the model never changes a drawing.
//!
//! **Freshness.** A view's lines are the costly part, and they depend on three things only: the
//! geometry of the bodies the design places ([`Source`]), the view's basis and its chord. Where the
//! view sits on the paper never changes them. The lines each view was last drawn with are kept in
//! the session's [`LinesCache`], never in the drawing: a drawing records nothing measured from the
//! design. A view is then [`Freshness::Current`] when its kept lines match all three,
//! [`Freshness::OutOfDate`] when the design (or the view's own direction or scale) has changed
//! since, and [`Freshness::NotDrawn`] before it is first drawn. `drawing.update` draws the views
//! that aren't current; `drawing.sheet.paper` and `drawing.export` draw them too when they must,
//! and reuse the kept lines when they can.

use std::collections::BTreeMap;
use std::sync::{Arc, Weak};

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
    lines_in(&scene, cmd, basis, chord)
}

/// The lines `scene` (the bodies' meshes, sampled to `chord`) shows on `basis`.
fn lines_in(scene: &mesh_lines::Scene, cmd: &str, basis: &Plane, chord: f64) -> Result<ViewLines> {
    let view = View::orthographic(basis).map_err(|e| bad(cmd, e))?;
    let drawing = mesh_lines::drawing(scene, &[], &view).map_err(|e| bad(cmd, e))?;
    let pieces = |hidden: bool| chain(drawing.lines.iter().filter(|l| l.hidden == hidden).map(|l| (l.a, l.b)).collect());
    Ok(ViewLines { visible: pieces(false), hidden: pieces(true), chord_mm: chord, segments: drawing.segments })
}

/// The geometry a view's lines were worked out from: every body the design places, where it
/// stands.
#[derive(Clone, Debug)]
pub(crate) enum Source {
    /// Each placed body's [`solidcraft_doc::ModelBody::geometry_key`], in the world's order. The
    /// same again when an evaluation rebuilds the same bodies in the same places, so undoing an
    /// edit of the part makes its views current again.
    Bodies(Vec<u64>),
    /// A body the evaluation cache does not key: the placed world itself, which every evaluation
    /// replaces. Held weakly, so it keeps no model alive, and no other world can take its place.
    World(Weak<solidcraft_doc::ModelState>),
}

impl Source {
    /// The geometry the session's design places now.
    pub(crate) fn of(s: &Session) -> Source {
        let world = s.world_state();
        match world.bodies.iter().map(|body| body.geometry_key()).collect::<Option<Vec<u64>>>() {
            Some(keys) => Source::Bodies(keys),
            None => Source::World(Arc::downgrade(&world)),
        }
    }

    fn same(&self, other: &Source) -> bool {
        match (self, other) {
            (Source::Bodies(a), Source::Bodies(b)) => a == b,
            (Source::World(a), Source::World(b)) => a.ptr_eq(b) && a.strong_count() > 0,
            _ => false,
        }
    }
}

/// A view's basis and chord, to the bit: with the [`Source`], all its lines depend on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Sampled([u64; 10]);

impl Sampled {
    fn of(basis: &Plane, chord: f64) -> Sampled {
        let [o, x, y] = [basis.origin, basis.x, basis.y];
        Sampled([o.x, o.y, o.z, x.x, x.y, x.z, y.x, y.y, y.z, chord].map(f64::to_bits))
    }

    fn chord_bits(&self) -> u64 {
        self.0[9]
    }
}

/// Whether a view's kept lines are the ones it would draw now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Freshness {
    /// Its kept lines match the design, its direction and its scale.
    Current,
    /// It was drawn, but the design, or the view's own direction or scale, has changed since.
    OutOfDate,
    /// It has not been drawn since the design was opened, or its kept lines were let go.
    NotDrawn,
}

/// How many views' lines a session keeps. Past it the longest unused are let go: they are only
/// drawn again, never lost.
const MAX_KEPT: usize = 256;

/// The lines views were last drawn with, by drawing and view. Shared with scratch copies of the
/// session, so the lines a worker draws are the window's too. Never saved: a drawing records
/// nothing measured from the design.
#[derive(Default)]
pub(crate) struct LinesCache {
    views: BTreeMap<(u64, u64), Kept>,
    clock: u64,
    /// Views whose lines were worked out rather than taken from here, for tests.
    #[cfg(test)]
    pub(crate) worked_out: usize,
}

struct Kept {
    source: Source,
    sampled: Sampled,
    lines: Arc<ViewLines>,
    used: u64,
}

impl LinesCache {
    fn freshness(&self, key: (u64, u64), source: &Source, sampled: Sampled) -> Freshness {
        match self.views.get(&key) {
            None => Freshness::NotDrawn,
            Some(kept) if kept.sampled == sampled && kept.source.same(source) => Freshness::Current,
            Some(_) => Freshness::OutOfDate,
        }
    }

    /// The view's kept lines when they are current.
    fn take(&mut self, key: (u64, u64), source: &Source, sampled: Sampled) -> Option<Arc<ViewLines>> {
        self.clock += 1;
        let now = self.clock;
        let kept = self.views.get_mut(&key).filter(|kept| kept.sampled == sampled && kept.source.same(source))?;
        kept.used = now;
        Some(kept.lines.clone())
    }

    fn keep(&mut self, key: (u64, u64), source: Source, sampled: Sampled, lines: Arc<ViewLines>) {
        self.clock += 1;
        self.views.insert(key, Kept { source, sampled, lines, used: self.clock });
        while self.views.len() > MAX_KEPT {
            let Some(oldest) = self.views.iter().min_by_key(|(_, kept)| kept.used).map(|(key, _)| *key) else { break };
            self.views.remove(&oldest);
        }
        #[cfg(test)]
        {
            self.worked_out += 1;
        }
    }
}

/// The session's kept lines. A panic elsewhere while it was held leaves nothing half-written in
/// it (each change is one insert or remove), so a poisoned lock is taken as it is.
fn kept(s: &Session) -> std::sync::MutexGuard<'_, LinesCache> {
    s.drawing_lines.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
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
pub fn drawn(s: &Session, cmd: &str, d: &Drawing, sheet: &Sheet, border: Aabb2) -> Result<Vec<Drawn>> {
    Ok(draw(s, cmd, d, sheet, border)?.into_iter().map(|(drawn, _)| drawn).collect())
}

/// [`drawn`], with how fresh each view's kept lines were before it: a current view's lines are
/// taken as kept, and the others are worked out and kept, one mesh scene per chord shared by
/// every view drawn at it.
pub fn draw(s: &Session, cmd: &str, d: &Drawing, sheet: &Sheet, border: Aabb2) -> Result<Vec<(Drawn, Freshness)>> {
    let placed = placed(s, cmd, d, sheet, border)?;
    let source = Source::of(s);
    let sampled: Vec<Sampled> = placed.iter().map(|p| Sampled::of(&p.frame.basis(), chord_mm(p.scale.factor()))).collect();
    let mut found: Vec<Option<Arc<ViewLines>>> = Vec::with_capacity(placed.len());
    let mut was = Vec::with_capacity(placed.len());
    {
        let mut cache = kept(s);
        for (p, at) in placed.iter().zip(&sampled) {
            was.push(cache.freshness((d.id, p.view), &source, *at));
            found.push(cache.take((d.id, p.view), &source, *at));
        }
    }
    // The lock is not held while drawing: a worker drawing one drawing must not stop the window
    // reading another's lines.
    let mut chords: Vec<u64> = sampled.iter().zip(&found).filter(|(_, lines)| lines.is_none()).map(|(at, _)| at.chord_bits()).collect();
    chords.sort_unstable();
    chords.dedup();
    for bits in chords {
        let chord = f64::from_bits(bits);
        let scene = mesh_lines::prepare(s, cmd, chord, |_| true)?;
        for (i, p) in placed.iter().enumerate() {
            if found.get(i).is_some_and(Option::is_some) || sampled.get(i).is_none_or(|at| at.chord_bits() != bits) {
                continue;
            }
            let lines = Arc::new(lines_in(&scene, cmd, &p.frame.basis(), chord)?);
            if let Some(at) = sampled.get(i) {
                kept(s).keep((d.id, p.view), source.clone(), *at, lines.clone());
            }
            if let Some(slot) = found.get_mut(i) {
                *slot = Some(lines);
            }
        }
    }

    let mut out = Vec::with_capacity(placed.len());
    for ((p, lines), was) in placed.into_iter().zip(found).zip(was) {
        let lines = lines.ok_or_else(|| bad(cmd, "a view's lines were not worked out"))?;
        let hidden = if shows_hidden(&p.style) { lines.hidden.clone() } else { Vec::new() };
        out.push((Drawn { view: p.view, frame: p.frame, scale: p.scale, fitted: p.fitted, visible: lines.visible.clone(), hidden }, was));
    }
    Ok(out)
}

/// How fresh each view of `sheet` is, in the record's order. Draws nothing: placing the views
/// reads only the bodies' extents.
pub fn freshness(s: &Session, cmd: &str, d: &Drawing, sheet: &Sheet, border: Aabb2) -> Result<Vec<(u64, Freshness)>> {
    let placed = placed(s, cmd, d, sheet, border)?;
    let source = Source::of(s);
    let cache = kept(s);
    let fresh = |p: &Placed| cache.freshness((d.id, p.view), &source, Sampled::of(&p.frame.basis(), chord_mm(p.scale.factor())));
    Ok(placed.iter().map(|p| (p.view, fresh(p))).collect())
}

/// A view saved on a sheet, placed: its frame, the scale it settled on and the style it draws
/// in, worked out without drawing a line.
struct Placed {
    view: u64,
    frame: ViewFrame,
    scale: Scale,
    fitted: bool,
    style: ViewStyle,
}

/// Every view of `sheet` placed, in the record's order.
///
/// Base views are placed first: a projection is lined up with the frame of the view it comes
/// from ([`ViewFrame::aligned`]), so that frame has to exist before it. The record keeps a
/// projection's parent a base view of the same sheet, so two passes are enough and there is no
/// chain to walk.
fn placed(s: &Session, cmd: &str, d: &Drawing, sheet: &Sheet, border: Aabb2) -> Result<Vec<Placed>> {
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

    // In the record's order, so the scene's groups come out the same way twice.
    let mut out = Vec::with_capacity(sheet.views.len());
    for view in &sheet.views {
        let (frame, scale, fitted) = *frames.get(&view.id).ok_or_else(|| bad(cmd, "a view was not worked out"))?;
        let style = match view.looks {
            Looks::Projected { from, .. } => {
                sheet.views.iter().find(|other| other.id == from).map_or(view.style, |parent| view.style.over(&parent.style))
            }
            Looks::Base { .. } => view.style,
        };
        out.push(Placed { view: view.id, frame, scale, fitted, style });
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
