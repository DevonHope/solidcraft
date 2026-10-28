use std::f64::consts::TAU;

use super::*;
use crate::cell::Projection;
use crate::drawing_sheet::{DrawingBorderProfile, Orientation, PaperSize, Sheet};
use crate::drawing_view::{Scale, StandardExtents, standard_layout};
use crate::paper::MAX_RECORDS;
use crate::title_block::TitleBlock;
use crate::{Plane, Vec3};

fn a3() -> SheetLayout {
    DrawingBorderProfile::iso().layout(&Sheet::standard(PaperSize::A3, Orientation::Landscape)).unwrap()
}

fn built_in_block(sheet: &SheetLayout) -> TitleBlockLayout {
    TitleBlock::built_in().layout(sheet).unwrap()
}

fn marks<'a>(layout: &'a SheetLayout) -> SheetMarks<'a> {
    SheetMarks { object: 1, layout }
}

/// A view at `scale` whose basis sits away from the world origin, so a run's coordinates are read
/// in the view's own plane and nothing else.
fn frame(scale: Scale, centre: Vec2, at: Vec2) -> ViewFrame {
    let basis = Plane { origin: Vec3::new(10.0, -20.0, 30.0), x: Vec3::X, y: Vec3::Y };
    ViewFrame::new(basis, scale, centre, at).unwrap()
}

fn strokes(group: &Group) -> Vec<(usize, bool, f64, Option<Vec<f64>>)> {
    group
        .items
        .iter()
        .map(|item| match item {
            Item::Stroke { path, closed, width_mm, pattern } => (path.len(), *closed, *width_mm, pattern.as_ref().map(|d| d.lengths_mm.clone())),
            Item::Fill { .. } => (0, false, 0.0, None),
        })
        .collect()
}

/// The box round every point a group's strokes pass through.
fn box_of(group: &Group) -> Aabb2 {
    group
        .items
        .iter()
        .flat_map(|item| match item {
            Item::Stroke { path, .. } => path.clone(),
            Item::Fill { region } => region.outer.segs.clone(),
        })
        .flat_map(|seg| [seg.start(), seg.end()])
        .collect()
}

/// How far `p` lies from the segment `a`-`b`.
fn off_segment(p: Vec2, a: Vec2, b: Vec2) -> f64 {
    let d = b - a;
    let t = if d.len2() > 0.0 { ((p - a).dot(d) / d.len2()).clamp(0.0, 1.0) } else { 0.0 };
    p.dist(a + d * t)
}

fn square(centre: Vec2, half: f64) -> Vec<Vec2> {
    let c = [centre + Vec2::new(-half, -half), centre + Vec2::new(half, -half), centre + Vec2::new(half, half), centre + Vec2::new(-half, half)];
    vec![c[0], c[1], c[2], c[3], c[0]]
}

#[test]
fn the_sheet_and_its_border_come_from_the_layout() {
    let layout = a3();
    let built = scene(marks(&layout), None, &[]).unwrap();
    assert_eq!(built.size_mm, Vec2::new(420.0, 297.0));
    let kinds: Vec<_> = built.groups.iter().map(|g| (g.object, g.kind)).collect();
    assert_eq!(kinds, [(1, ObjectKind::SheetBorder), (1, ObjectKind::DrawingBorder)]);

    // The paper's own edge: one thin closed rectangle round the whole sheet.
    assert_eq!(strokes(&built.groups[0]), [(4, true, SHEET_EDGE_LINE_MM, None)]);
    assert_eq!(box_of(&built.groups[0]), layout.sheet_border);

    let border = &built.groups[1];
    // The drawing border, the grid reference border, a line at each field edge on two sides of the
    // sheet, and the four centring marks.
    let edges = 2 * (layout.columns.len() - 1) + 2 * (layout.rows.len() - 1);
    assert_eq!(border.items.len(), 2 + edges + 4);
    assert_eq!(strokes(border)[..2], [(4, true, 0.7, None), (4, true, 0.7, None)]);
    assert!(strokes(border)[2..].iter().all(|s| *s == (1, false, 0.7, None)));
    // Everything the border group draws lies in the margins or on them, never in the drawing area.
    assert_eq!(box_of(border), layout.sheet_border);
    assert_eq!(Aabb2::from_iter([layout.drawing_border.min, layout.drawing_border.max]), layout.drawing_border);
}

#[test]
fn a_border_with_no_grid_draws_only_its_own_rectangle_and_marks() {
    let plain = DrawingBorderProfile { grid: crate::drawing_sheet::GridRule::None, grid_border_mm: 0.0, ..DrawingBorderProfile::iso() };
    let layout = plain.layout(&Sheet::standard(PaperSize::A4, Orientation::Portrait)).unwrap();
    let built = scene(marks(&layout), None, &[]).unwrap();
    // The drawing border and the four centring marks: no grid reference border to divide.
    assert_eq!(built.groups[1].items.len(), 5);
}

#[test]
fn the_title_blocks_cells_are_drawn_under_its_outline() {
    let layout = a3();
    let block = built_in_block(&layout);
    let built = scene(marks(&layout), Some(TitleBlockMarks { object: 7, layout: &block }), &[]).unwrap();
    let group = built.groups.iter().find(|g| g.kind == ObjectKind::TitleBlock).unwrap();
    assert_eq!(group.object, 7);
    // Twelve cells at the cell line, then the outline over them at its own wider line, so the
    // block's edges finish in the outline wherever a cell lies against them.
    assert_eq!(group.items.len(), block.cells.len() + 1);
    let drawn = strokes(group);
    assert!(drawn[..block.cells.len()].iter().all(|s| *s == (4, true, block.cell_line_mm, None)));
    assert_eq!(drawn[block.cells.len()], (4, true, block.outline_line_mm, None));
    assert_eq!(box_of(group), block.outline);
    // The block sits on the drawing border's lower-right corner, as its own layout puts it.
    assert_eq!((block.outline.max.x, block.outline.min.y), (layout.drawing_border.max.x, layout.drawing_border.min.y));
}

#[test]
fn a_views_runs_land_on_paper_at_its_scale_with_the_hidden_ones_dashed() {
    let layout = a3();
    let (half, at) = (40.0, Vec2::new(150.0, 150.0));
    let view = frame(Scale::new(1.0, 2.0).unwrap(), Vec2::ZERO, at);
    let visible = vec![square(Vec2::ZERO, half)];
    let hidden = vec![vec![Vec2::new(-half, 0.0), Vec2::new(half, 0.0)]];
    let marks_of = ViewMarks { object: 3, frame: &view, visible: &visible, hidden: &hidden };
    let built = scene(marks(&layout), None, &[marks_of]).unwrap();

    let group = built.groups.iter().find(|g| g.kind == ObjectKind::View).unwrap();
    assert_eq!(group.object, 3);
    // Hidden first, so a visible line wins wherever the two fall together.
    assert_eq!(strokes(group), [(1, false, HIDDEN_LINE_MM, Some(HIDDEN_DASH_MM.to_vec())), (4, true, VISIBLE_LINE_MM, None)]);
    assert_eq!(HIDDEN_DASH_MM, [3.0, 0.75]);

    // The run is read in the view's own plane: its centre lands where the frame puts it, and 40 mm
    // of model is 20 mm of paper at 1:2.
    assert_eq!(box_of(group), Aabb2 { min: at - Vec2::new(20.0, 20.0), max: at + Vec2::new(20.0, 20.0) });
}

#[test]
fn a_closed_run_stays_closed_and_round_when_it_is_thinned() {
    let layout = a3();
    // A circle of radius 50 mm sampled every half degree: 720 pieces for a line paper can't tell
    // from 60. Its ends are the same point, so the first chord has no length at all.
    let points: Vec<Vec2> = (0..=720).map(|i| Vec2::from_angle(TAU * i as f64 / 720.0) * 50.0).collect();
    let circle = vec![points.clone()];
    let view = frame(Scale::new(1.0, 1.0).unwrap(), Vec2::ZERO, Vec2::new(200.0, 150.0));
    let built = scene(marks(&layout), None, &[ViewMarks { object: 3, frame: &view, visible: &circle, hidden: &[] }]).unwrap();
    let group = built.groups.iter().find(|g| g.kind == ObjectKind::View).unwrap();

    let Item::Stroke { path, closed, .. } = &group.items[0] else { panic!("the circle draws a stroke") };
    assert!(*closed, "the run came back closed");
    // Thinned, but not collapsed: a degenerate first chord must not take the whole circle with it.
    assert!((16..points.len() / 2).contains(&path.len()), "{} pieces", path.len());
    // Every point of the original lies within the tolerance of what is drawn.
    let drawn: Vec<Vec2> = path.iter().map(|s| s.start()).chain(path.last().map(|s| s.end())).collect();
    let off = |p: Vec2| drawn.windows(2).map(|w| off_segment(p, w[0], w[1])).fold(f64::INFINITY, f64::min);
    let worst = points.iter().map(|p| off(view.paper_map().apply(*p))).fold(0.0, f64::max);
    assert!(worst <= TOLERANCE_MM, "{worst} mm from the circle");
}

#[test]
fn a_straight_run_thins_to_its_two_ends() {
    let layout = a3();
    // 1000 points along 100 mm, each wandering a thousandth of a millimetre off the line.
    let wobble: Vec<Vec2> = (0..1000).map(|i| Vec2::new(i as f64 * 0.1, if i % 2 == 0 { 0.001 } else { -0.001 })).collect();
    let runs = vec![wobble];
    let view = frame(Scale::new(1.0, 1.0).unwrap(), Vec2::new(50.0, 0.0), Vec2::new(200.0, 150.0));
    let built = scene(marks(&layout), None, &[ViewMarks { object: 3, frame: &view, visible: &runs, hidden: &[] }]).unwrap();
    let group = built.groups.iter().find(|g| g.kind == ObjectKind::View).unwrap();
    assert_eq!(strokes(group), [(1, false, VISIBLE_LINE_MM, None)]);
}

#[test]
fn a_run_too_long_to_thin_is_refused_rather_than_cut_short() {
    let layout = a3();
    // A zigzag 0.2 mm deep: every point is its own corner, so none of it thins away.
    let zigzag: Vec<Vec2> = (0..=MAX_RECORDS).map(|i| Vec2::new(i as f64 * 0.5, if i % 2 == 0 { 0.1 } else { -0.1 })).collect();
    let runs = vec![zigzag];
    let view = frame(Scale::new(1.0, 1.0).unwrap(), Vec2::ZERO, Vec2::new(200.0, 150.0));
    let refused = scene(marks(&layout), None, &[ViewMarks { object: 3, frame: &view, visible: &runs, hidden: &[] }]);
    assert_eq!(refused.unwrap_err(), Invalid("more than 100,000 groups, items, loops and pieces"));
}

#[test]
fn a_run_the_thinning_gives_up_on_keeps_every_point_it_came_with() {
    let layout = a3();
    // A zigzag 0.2 mm deep: every point is a corner, which is the shape the walk is quadratic on,
    // so it runs out of work. What it has not looked at has to come through untouched — thinning
    // may draw a line with fewer points, never a different line.
    let points: Vec<Vec2> = (0..50_000).map(|i| Vec2::new(i as f64 * 0.001, if i % 2 == 0 { 0.1 } else { -0.1 })).collect();
    let runs = vec![points.clone()];
    let view = frame(Scale::new(1.0, 1.0).unwrap(), Vec2::ZERO, Vec2::new(200.0, 150.0));
    let built = scene(marks(&layout), None, &[ViewMarks { object: 3, frame: &view, visible: &runs, hidden: &[] }]).unwrap();
    let group = built.groups.iter().find(|g| g.kind == ObjectKind::View).unwrap();
    assert_eq!(strokes(group), [(points.len() - 1, false, VISIBLE_LINE_MM, None)]);
}

#[test]
fn hostile_input_is_refused() {
    let layout = a3();
    let view = frame(Scale::new(1.0, 1.0).unwrap(), Vec2::ZERO, Vec2::new(200.0, 150.0));
    for bad in [f64::NAN, f64::INFINITY, -f64::INFINITY] {
        let runs = vec![vec![Vec2::ZERO, Vec2::new(bad, 1.0)]];
        let refused = scene(marks(&layout), None, &[ViewMarks { object: 3, frame: &view, visible: &runs, hidden: &[] }]);
        assert_eq!(refused.unwrap_err(), Invalid("a view's line holds a point that is not finite"));
    }
    // A run of one point draws nothing, and an empty one is not a stroke at all.
    let runs = vec![vec![Vec2::ZERO], vec![]];
    let built = scene(marks(&layout), None, &[ViewMarks { object: 3, frame: &view, visible: &runs, hidden: &[] }]).unwrap();
    assert!(built.groups.iter().find(|g| g.kind == ObjectKind::View).unwrap().items.is_empty());

    // The scene's coordinates start at the sheet's lower-left corner.
    for shifted in [Vec2::new(1.0, 0.0), Vec2::new(0.0, -1.0)] {
        let moved = SheetLayout { sheet_border: layout.sheet_border.inflated(-1.0), ..a3() };
        let moved = SheetLayout { sheet_border: Aabb2 { min: shifted, ..moved.sheet_border }, ..moved };
        assert_eq!(scene(marks(&moved), None, &[]).unwrap_err(), Invalid("a sheet's lower-left corner is the scene's origin"));
    }
}

#[test]
fn the_standard_views_fit_beside_the_built_in_block_on_its_own_sheet() {
    let layout = a3();
    let block = built_in_block(&layout);
    let clear = kept_clear(&block, layout.drawing_border);
    // A plate 200 x 120 x 150 mm: the four views of it, each its own size on its own plane.
    let extents = StandardExtents {
        front: Aabb2 { min: Vec2::new(-100.0, -60.0), max: Vec2::new(100.0, 60.0) },
        top: Aabb2 { min: Vec2::new(-100.0, -75.0), max: Vec2::new(100.0, 75.0) },
        right: Aabb2 { min: Vec2::new(-75.0, -60.0), max: Vec2::new(75.0, 60.0) },
        iso: Aabb2 { min: Vec2::new(-90.0, -85.0), max: Vec2::new(90.0, 85.0) },
    };
    let placed = standard_layout(&extents, layout.drawing_border, Some(clear), Projection::Third, 10.0).unwrap();

    // One square run per view, the size of that view's extents, so each group's box is its boundary.
    let runs: Vec<Vec<Vec<Vec2>>> =
        [extents.front, extents.top, extents.right, extents.iso].iter().map(|e| vec![square(e.center(), e.size().x.min(e.size().y) / 2.0)]).collect();
    let frames = [placed.front, placed.top, placed.right, placed.iso];
    let views: Vec<ViewMarks> =
        frames.iter().zip(&runs).enumerate().map(|(i, (frame, visible))| ViewMarks { object: 10 + i as u64, frame, visible, hidden: &[] }).collect();

    let built = scene(marks(&layout), Some(TitleBlockMarks { object: 7, layout: &block }), &views).unwrap();
    let drawn: Vec<&Group> = built.groups.iter().filter(|g| g.kind == ObjectKind::View).collect();
    assert_eq!(drawn.len(), 4);
    for group in drawn {
        let at = box_of(group);
        assert_eq!(layout.drawing_border.intersection(&at), at, "a view leaves the drawing border: {at:?}");
        assert!(!at.meets(&clear), "a view lands on the title block: {at:?}");
    }
}
