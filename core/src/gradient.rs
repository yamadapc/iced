use crate::Color;
use crate::Direction;

/// A gradient of colors
#[derive(Debug, Clone, PartialEq)]
pub enum Gradient {
    /// A linear gradient
    LinearGradient(LinearGradient),
}

/// A linear gradient of colors
#[derive(Debug, Clone, PartialEq)]
pub struct LinearGradient {
    /// The direction of the gradient
    pub direction: Direction,
    /// Each of the color stops
    pub stops: Vec<GradientStop>,
}

/// A gradient color stop
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct GradientStop {
    /// The percentage of this color step
    pub percentage: f32,
    /// The color of this step
    pub color: Color,
}
