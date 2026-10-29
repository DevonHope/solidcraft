//! A drawing view's settings (W07, #5866): which way it looks, its scale and how it draws.
//! `drawing.view.snapshot` takes them whole; a [`View`] saved on a sheet (W12) keeps the same
//! fields less the direction ([`ViewStyle`]), each one optional so that a projected view follows
//! the view it is projected from.

use serde::{Deserialize, Deserializer, Serialize};
use solidcraft_geom::Plane;
use solidcraft_geom::drawing_sheet::MAX_SIDE_MM;
use solidcraft_geom::drawing_view::{Invalid, NamedView, ProjectedSide, Scale, basis_looking};
use solidcraft_render::VisualStyle;

/// The largest reference margin, in model millimetres.
pub const MAX_REFERENCE_MARGIN_MM: f64 = 1e5;

/// Which way a view looks: by name, or along any direction with an up.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub enum ViewDirection {
    Named(NamedView),
    Looking(Looking),
}

/// A view along `direction` with `up` toward the top of the paper, through `origin` (the world
/// origin when not given).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Looking {
    pub direction: [f64; 3],
    pub up: [f64; 3],
    #[serde(default)]
    pub origin: [f64; 3],
}

impl<'de> Deserialize<'de> for ViewDirection {
    /// One message for anything that is neither, rather than serde's "did not match any variant".
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<ViewDirection, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Either {
            Named(NamedView),
            Looking(Looking),
        }
        let message = "view is front, back, top, bottom, left, right, iso, iso_front_right, iso_front_left, iso_back_right, iso_back_left, or {direction, up, origin?}";
        Ok(match Either::deserialize(d).map_err(|_| serde::de::Error::custom(message))? {
            Either::Named(view) => ViewDirection::Named(view),
            Either::Looking(looking) => ViewDirection::Looking(looking),
        })
    }
}

impl Default for ViewDirection {
    fn default() -> Self {
        ViewDirection::Named(NamedView::Front)
    }
}

impl ViewDirection {
    /// The view's basis: a named view's exact one, or one built from the direction and up.
    pub fn basis(&self) -> Result<Plane, Invalid> {
        match *self {
            ViewDirection::Named(view) => Ok(view.basis()),
            ViewDirection::Looking(Looking { direction, up, origin }) => basis_looking(direction.into(), up.into(), origin.into()),
        }
    }
}

/// Tangent edges, where faces meet smoothly: left out (the default) or drawn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TangentEdges {
    #[default]
    Hide,
    Show,
}

/// What a view shows and how. Every field has a default: the front view at the scale that fits
/// its page, visible lines only, tangent edges hidden.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ViewSettings {
    pub view: ViewDirection,
    /// None: the largest preset scale at which the view fits its page ([`Scale::fit`]). Not
    /// written when None: a snapshot refuses a null scale.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<Scale>,
    pub display_style: VisualStyle,
    /// How reference parts draw (bill of materials type `reference`).
    pub reference_style: VisualStyle,
    /// Whether reference parts hide the model's lines behind them.
    pub reference_hides_model: bool,
    /// Model millimetres added on every side of the view's extents, so reference parts around the
    /// model show inside its boundary: 0 to [`MAX_REFERENCE_MARGIN_MM`].
    pub reference_margin_mm: f64,
    pub tangent_edges: TangentEdges,
}

impl ViewSettings {
    /// What reading alone can't check: the margin's range and the view's basis.
    pub fn validate(&self) -> Result<Plane, Invalid> {
        let margin = self.reference_margin_mm;
        if !(margin.is_finite() && (0.0..=MAX_REFERENCE_MARGIN_MM).contains(&margin)) {
            return Err(Invalid("reference_margin_mm is a number of model millimetres from 0 to 100000"));
        }
        self.view.basis()
    }
}

/// What a view draws differently from the view it follows: [`ViewSettings`]' fields, less the
/// direction (which [`Looks`] holds), each one optional. A field left out takes the view this one
/// is projected from, or [`ViewSettings`]'s own default where there is none to follow.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ViewStyle {
    /// None: as the view it follows draws, or the largest preset scale at which the view fits its
    /// page ([`Scale::fit`]) where it follows none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<Scale>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_style: Option<VisualStyle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_style: Option<VisualStyle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_hides_model: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_margin_mm: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tangent_edges: Option<TangentEdges>,
}

impl ViewStyle {
    /// Nothing of its own: it draws just what it follows.
    pub fn is_empty(&self) -> bool {
        *self == ViewStyle::default()
    }

    /// This style over `followed`, the style of the view it is projected from: its own fields
    /// where it has them, that view's where it hasn't.
    pub fn over(&self, followed: &ViewStyle) -> ViewStyle {
        ViewStyle {
            scale: self.scale.or(followed.scale),
            display_style: self.display_style.or(followed.display_style),
            reference_style: self.reference_style.or(followed.reference_style),
            reference_hides_model: self.reference_hides_model.or(followed.reference_hides_model),
            reference_margin_mm: self.reference_margin_mm.or(followed.reference_margin_mm),
            tangent_edges: self.tangent_edges.or(followed.tangent_edges),
        }
    }

    /// The settings a view looking `view` draws with in this style: what the style says, and
    /// [`ViewSettings`]'s defaults for the rest.
    pub fn settings(&self, view: ViewDirection) -> ViewSettings {
        let default = ViewSettings::default();
        ViewSettings {
            view,
            scale: self.scale,
            display_style: self.display_style.unwrap_or(default.display_style),
            reference_style: self.reference_style.unwrap_or(default.reference_style),
            reference_hides_model: self.reference_hides_model.unwrap_or(default.reference_hides_model),
            reference_margin_mm: self.reference_margin_mm.unwrap_or(default.reference_margin_mm),
            tangent_edges: self.tangent_edges.unwrap_or(default.tangent_edges),
        }
    }

    /// What reading alone can't check, by [`ViewSettings`]'s own rule: the margin's range.
    pub fn validate(&self) -> Result<(), Invalid> {
        self.settings(ViewDirection::default()).validate().map(|_| ())
    }
}

/// Which way a view looks: by itself, or projected from a base view of its sheet.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Looks {
    /// Where it says: a base view.
    Base { view: ViewDirection },
    /// From the base view `from`, on `side` of it: its direction is that view's turned a quarter
    /// turn in the drawing's projection
    /// ([`projected_basis`](solidcraft_geom::drawing_view::projected_basis)), so none of it is
    /// saved here. Only a base view is projected from, so no view follows a chain of them.
    Projected {
        #[serde(with = "crate::ids")]
        from: u64,
        side: ProjectedSide,
    },
}

/// A view saved on a sheet (W12): what it draws, which way it looks, where it sits on the paper
/// and what it draws differently from the view it follows. Nothing measured from the design is
/// here — a view's extents and the lines it draws are worked out from the design again
/// (`drawing.update`), so editing the part it draws rewrites no view.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct View {
    /// Unique among every object of its drawing.
    #[serde(with = "crate::ids")]
    pub id: u64,
    /// The drawing's source it draws ([`Source`](crate::drawing::Source)'s id).
    #[serde(with = "crate::ids")]
    pub source: u64,
    /// The component of that design it draws, as the design writes it (0: the design itself).
    #[serde(default)]
    pub component: u64,
    /// Where the view's centre sits on the paper, in millimetres from the sheet's lower-left
    /// corner, at most [`MAX_SIDE_MM`] along each side: what
    /// [`ViewFrame::new`](solidcraft_geom::drawing_view::ViewFrame::new) places it by.
    pub position_mm: [f64; 2],
    pub looks: Looks,
    #[serde(default, skip_serializing_if = "ViewStyle::is_empty")]
    pub style: ViewStyle,
}

impl View {
    /// What reading alone can't check: where it sits, which way it looks, and its style's rules.
    /// That the view it is projected from is a base view of the same sheet is the drawing's rule
    /// ([`Drawing::validate`](crate::drawing::Drawing::validate)), which alone sees the sheet.
    pub fn validate(&self) -> Result<(), Invalid> {
        let on_paper = self.position_mm.iter().all(|c| c.is_finite() && c.abs() <= MAX_SIDE_MM);
        if !on_paper {
            return Err(Invalid("a view's position is finite and at most 5000 mm from the sheet's origin"));
        }
        if let Looks::Base { view } = self.looks {
            view.basis()?;
        }
        self.style.validate()
    }
}

#[cfg(test)]
#[path = "drawing_view_tests.rs"]
mod tests;
