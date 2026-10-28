//! One drawing sheet as a paper scene (W17b): the sheet and its border from a
//! [`SheetLayout`], the title block from a [`TitleBlockLayout`], and each view's lines through its
//! [`ViewFrame`], as a [`Scene`] in sheet millimetres for the writers (SVG, PNG, DXF, PDF).
//!
//! [`scene`] is pure and composes laid-out geometry only: it works nothing out for itself, so
//! `drawing_sheet`, `title_block` and `drawing_view` stay the single places that decide where
//! anything goes. The same input always gives the same scene, byte for byte — nothing here is
//! ordered by a hash.
//!
//! What a group holds, in paint order:
//! - the sheet's own edge ([`ObjectKind::SheetBorder`]), a thin line;
//! - the drawing border, the grid reference border with the lines between its fields, the centring
//!   marks and the trimming marks ([`ObjectKind::DrawingBorder`], which the kind's own notes say
//!   the marks and the grid belong to);
//! - the title block's cells and then its outline, so the outline's wider line finishes the
//!   block's edges where a cell lies against them ([`ObjectKind::TitleBlock`]);
//! - one group per view ([`ObjectKind::View`]), its hidden lines before its visible ones, so a
//!   visible line wins wherever the two fall together.
//!
//! A view's runs arrive in the coordinates of its own plane, as `mesh_lines` and the extents give
//! them, and [`ViewFrame::paper_map`] puts them on paper. Each run is then thinned to
//! [`TOLERANCE_MM`] on paper (Douglas–Peucker), because a mesh follows a curved face in far more
//! points than paper can show, and a scene holds at most [`crate::paper::MAX_RECORDS`] of them. A
//! run's accuracy on paper is therefore its sampling chord plus [`TOLERANCE_MM`].
//!
//! Text is not drawn: the paper scene has no text yet, so the grid's labels and the title block's
//! labels and values are not here. Nor is the projection symbol, which is a glyph too.

use crate::drawing_sheet::SheetLayout;
use crate::drawing_view::ViewFrame;
use crate::paper::{Dashes, Group, Invalid, Item, ObjectKind, Scene};
use crate::title_block::TitleBlockLayout;
use crate::{Aabb2, EPS, Loop2, Region2, Seg2, Vec2};

/// The sheet's own edge, drawn thin: it is the paper's edge, not part of the drawing's frame.
pub const SHEET_EDGE_LINE_MM: f64 = 0.35;
/// A view's visible lines.
pub const VISIBLE_LINE_MM: f64 = 0.5;
/// A view's hidden lines, dashed.
pub const HIDDEN_LINE_MM: f64 = 0.25;
/// ISO 128-2's dashed line at `d` = [`HIDDEN_LINE_MM`]: dashes 12 d long, gaps 3 d, as #7430's own
/// pictures draw it at 0.35 mm.
pub const HIDDEN_DASH_MM: [f64; 2] = [12.0 * HIDDEN_LINE_MM, 3.0 * HIDDEN_LINE_MM];
/// How far a thinned run may leave the one it came from, on paper. The same 0.01 mm the view's
/// lines are sampled to, so the two together stay inside 0.02 mm: a fifth of a 300 dpi dot.
pub const TOLERANCE_MM: f64 = 0.01;
/// How many points [`thinned`] may look at for each point of a run before it stops thinning it
/// (see its notes). A smooth run of 100,000 points takes about eight.
const WORK_PER_POINT: usize = 64;

type Result<T> = std::result::Result<T, Invalid>;

/// The sheet's furniture: the layout to draw, and the id of the object that holds it.
#[derive(Clone, Copy, Debug)]
pub struct SheetMarks<'a> {
    pub object: u64,
    pub layout: &'a SheetLayout,
}

/// A title block to draw, and the id of the object that holds it.
#[derive(Clone, Copy, Debug)]
pub struct TitleBlockMarks<'a> {
    pub object: u64,
    pub layout: &'a TitleBlockLayout,
}

/// One view's lines, in the coordinates of its own plane ([`crate::Plane::to_local`]), which
/// `frame` puts on paper. `hidden` is empty when the view's style leaves hidden lines out.
#[derive(Clone, Copy, Debug)]
pub struct ViewMarks<'a> {
    pub object: u64,
    pub frame: &'a ViewFrame,
    pub visible: &'a [Vec<Vec2>],
    pub hidden: &'a [Vec<Vec2>],
}

/// The sheet's scene: its furniture, its title block if it has one, then its views in order.
///
/// Refused when the sheet's lower-left corner is not the scene's origin, when a view's run holds a
/// point that is not finite, or when the scene breaks one of [`Scene::validate`]'s limits — above
/// all its record count, which the views' runs are thinned to stay inside.
pub fn scene(sheet: SheetMarks, title_block: Option<TitleBlockMarks>, views: &[ViewMarks]) -> Result<Scene> {
    let l = sheet.layout;
    let origin = l.sheet_border.min == Vec2::ZERO && l.sheet_border.max.is_finite();
    if !origin {
        return Err(Invalid("a sheet's lower-left corner is the scene's origin"));
    }
    let mut groups = vec![
        Group { object: sheet.object, kind: ObjectKind::SheetBorder, items: vec![outline(l.sheet_border, SHEET_EDGE_LINE_MM)] },
        Group { object: sheet.object, kind: ObjectKind::DrawingBorder, items: border_items(l) },
    ];
    if let Some(block) = title_block {
        groups.push(Group { object: block.object, kind: ObjectKind::TitleBlock, items: block_items(block.layout) });
    }
    for view in views {
        groups.push(Group { object: view.object, kind: ObjectKind::View, items: view_items(view)? });
    }
    let scene = Scene { size_mm: l.sheet_border.max, groups };
    scene.validate()?;
    Ok(scene)
}

/// A rectangle as a closed stroke, anticlockwise from its lower-left corner.
fn outline(rect: Aabb2, width_mm: f64) -> Item {
    let corners = [rect.min, Vec2::new(rect.max.x, rect.min.y), rect.max, Vec2::new(rect.min.x, rect.max.y)];
    let path = (0..4).map(|i| Seg2::Line { a: corners[i], b: corners[(i + 1) % 4] }).collect();
    Item::Stroke { path, closed: true, width_mm, pattern: None }
}

fn segment(a: Vec2, b: Vec2, width_mm: f64) -> Item {
    Item::Stroke { path: vec![Seg2::Line { a, b }], closed: false, width_mm, pattern: None }
}

/// The drawing border, the grid reference border and the lines between its fields, the centring
/// marks, and the trimming marks as fills.
fn border_items(l: &SheetLayout) -> Vec<Item> {
    let line = l.drawing_border_line_mm;
    let (inner, outer) = (l.drawing_border, l.grid_border);
    let mut items = vec![outline(inner, line)];
    // The band the grid reference border leaves between the two: with none, there are no fields to
    // divide and nothing to draw round them.
    let band = inner.min.x - outer.min.x;
    if band > 0.0 {
        items.push(outline(outer, line));
        // A field's far edge is the next one's near edge, so the last field's is the border itself.
        for edge in l.columns.iter().rev().skip(1).map(|f| f.to_mm) {
            items.push(segment(Vec2::new(edge, outer.min.y), Vec2::new(edge, inner.min.y), line));
            items.push(segment(Vec2::new(edge, inner.max.y), Vec2::new(edge, outer.max.y), line));
        }
        // Rows are listed top to bottom, so it is the first row's upper edge that is the border's.
        for edge in l.rows.iter().skip(1).map(|f| f.to_mm) {
            items.push(segment(Vec2::new(outer.min.x, edge), Vec2::new(inner.min.x, edge), line));
            items.push(segment(Vec2::new(inner.max.x, edge), Vec2::new(outer.max.x, edge), line));
        }
    }
    items.extend(l.centring_marks.iter().map(|[a, b]| segment(*a, *b, line)));
    items.extend(l.trimming_marks.iter().map(|mark| {
        let corners = [mark.min, Vec2::new(mark.max.x, mark.min.y), mark.max, Vec2::new(mark.min.x, mark.max.y)];
        Item::Fill { region: Region2 { outer: Loop2::polygon(&corners), holes: vec![] } }
    }));
    items
}

/// Each cell's rectangle, then the block's outline over them.
fn block_items(l: &TitleBlockLayout) -> Vec<Item> {
    let cells = l.cells.iter().map(|cell| outline(cell.rect, l.cell_line_mm));
    cells.chain([outline(l.outline, l.outline_line_mm)]).collect()
}

/// The view's hidden runs, dashed, then its visible ones.
fn view_items(view: &ViewMarks) -> Result<Vec<Item>> {
    let dashes = Dashes { lengths_mm: HIDDEN_DASH_MM.to_vec(), phase_mm: 0.0 };
    let mut items = Vec::with_capacity(view.hidden.len() + view.visible.len());
    for (runs, width_mm, pattern) in [(view.hidden, HIDDEN_LINE_MM, Some(dashes)), (view.visible, VISIBLE_LINE_MM, None)] {
        for run in runs {
            if let Some((path, closed)) = on_paper(view.frame, run)? {
                items.push(Item::Stroke { path, closed, width_mm, pattern: pattern.clone() });
            }
        }
    }
    Ok(items)
}

/// One run on paper, thinned: its pieces and whether it closes. None when what is left draws
/// nothing (a run of one point, or one that thins to a dot).
fn on_paper(frame: &ViewFrame, run: &[Vec2]) -> Result<Option<(Vec<Seg2>, bool)>> {
    if !run.iter().all(|p| p.is_finite()) {
        return Err(Invalid("a view's line holds a point that is not finite"));
    }
    let map = frame.paper_map();
    let mut points: Vec<Vec2> = run.iter().map(|p| map.apply(*p)).collect();
    // `chain` closes a run by walking back to the point it started from, which it matched on a
    // grid: the two agree to within the grid, not exactly. Make them the same point.
    let closed = points.len() > 3 && points.first().zip(points.last()).is_some_and(|(a, b)| a.dist(*b) <= EPS);
    if closed {
        let first = points[0];
        if let Some(last) = points.last_mut() {
            *last = first;
        }
    }
    // Dropped before the pieces are cut, never after: a stroke's pieces have to meet end to
    // start, so taking one out of the middle would leave a gap rather than a shorter path.
    points = thinned(&points, TOLERANCE_MM);
    points.dedup();
    let path: Vec<Seg2> = points.windows(2).map(|w| Seg2::Line { a: w[0], b: w[1] }).collect();
    Ok((!path.is_empty()).then_some((path, closed)))
}

/// `points` thinned to those more than `tol` from the chords between the ones kept
/// (Douglas–Peucker), its two ends always kept. Written with its own stack rather than by
/// recursion: a run can be as long as the mesh is fine, and the call stack is not.
///
/// The walk is quadratic on a run whose every point is a corner — a zigzag finer than the
/// tolerance, which splits off one point at a time — so it gives up after [`WORK_PER_POINT`]
/// points looked at for each point of the run and keeps the rest as they came. That costs
/// nothing in accuracy: an unthinned run is the one it came from. Such a run is near the record
/// cap by definition, since nothing about it thins, and the scene's own count then refuses it.
fn thinned(points: &[Vec2], tol: f64) -> Vec<Vec2> {
    if points.len() <= 2 {
        return points.to_vec();
    }
    let mut keep = vec![false; points.len()];
    let last = points.len() - 1;
    (keep[0], keep[last]) = (true, true);
    let mut spans = vec![(0, last)];
    let mut left = points.len().saturating_mul(WORK_PER_POINT);
    while let Some((from, to)) = spans.pop() {
        if left < to - from {
            // Out of work: everything still in hand stays, this span and the ones waiting.
            for (from, to) in spans.into_iter().chain([(from, to)]) {
                keep[from..=to].fill(true);
            }
            break;
        }
        left -= to - from;
        let (mut worst, mut at) = (tol, from);
        for i in from + 1..to {
            let d = off_chord(points[i], points[from], points[to]);
            if d > worst {
                (worst, at) = (d, i);
            }
        }
        if at > from {
            keep[at] = true;
            spans.push((from, at));
            spans.push((at, to));
        }
    }
    points.iter().zip(keep).filter_map(|(p, keep)| keep.then_some(*p)).collect()
}

/// How far `p` lies from the line through `a` and `b`, or from `a` itself when the two are the
/// same point, as the ends of a closed run are: there the farthest point out is what to keep.
fn off_chord(p: Vec2, a: Vec2, b: Vec2) -> f64 {
    let chord = b - a;
    let length = chord.len();
    if length <= f64::MIN_POSITIVE { p.dist(a) } else { chord.cross(p - a).abs() / length }
}

/// Where a title block leaves no room for a view: its outline, grown by half the line drawn on it,
/// within `border`, for [`crate::drawing_view::standard_layout`] to lay the views out beside.
///
/// Clipped to the border because the block sits on it: its right and bottom edges lie along the
/// border's, so half of the line drawn on them falls outside the drawing area, where no view goes
/// anyway.
pub fn kept_clear(block: &TitleBlockLayout, border: Aabb2) -> Aabb2 {
    block.outline.inflated(block.outline_line_mm / 2.0).intersection(&border)
}

#[cfg(test)]
#[path = "drawing_paper_tests.rs"]
mod tests;
