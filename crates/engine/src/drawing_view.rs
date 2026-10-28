//! Drawing views of the session's model (W07, #5866): what a view shows and how far it reaches.
//! The snapshot command uses it now; saved views (W14) and their pictures (W17b) will too.
//!
//! A view shows every body the design places, where its occurrences place it. The browser's eye
//! counts for nothing: hiding a part in the model never changes a drawing.

use std::collections::BTreeMap;

use solidcraft_geom::{Aabb2, Plane, Vec2};

use crate::mesh_lines::{self, View};
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
