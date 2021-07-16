/// A vertical, horizontal or diagonal direction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    /// From bottom to top
    Top,
    /// From left to right
    Right,
    /// From top to bottom
    Bottom,
    /// From right to left
    Left,
    /// From bottom left to top right
    TopRight,
    /// From bottom right to top left
    TopLeft,
    /// From top left to bottom right
    BottomRight,
    /// From top right to bottom left
    BottomLeft,
}
