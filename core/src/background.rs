use crate::Color;
use crate::Gradient;

/// The background of some element.
#[derive(Debug, Clone, PartialEq)]
pub enum Background {
    /// A solid color
    Color(Color),
    /// A gradient of colors
    Gradient(Gradient),
    // TODO: Add image variants
}

impl From<Color> for Background {
    fn from(color: Color) -> Self {
        Background::Color(color)
    }
}

impl From<Color> for Option<Background> {
    fn from(color: Color) -> Self {
        Some(Background::from(color))
    }
}
