//! A sheet keeps the **views** it draws (`crate::drawing_view::View`: what each draws, which way it
//! looks, where its centre sits on the paper and what it draws differently from the view it is
//! projected from) and a copy of its **title block**, as it already keeps a copy of its drawing
//! border; a drawing says which **projection** its views are drawn in, first or third angle.
//!
//! No step. A drawing from before this change has no views and no title block, and reads as first
//! angle (`drawing::default_projection`), which is what it would have been drawn in: it had no
//! views to draw either way. The drawing format number does not move, because it is already the
//! number pending changes are written under; the last release refuses any drawing of it, as it does
//! one with `in-design`.
