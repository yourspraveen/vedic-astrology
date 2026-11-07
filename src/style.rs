//! Chart style choice, shared by the desktop drawing (`chart.rs`), the HTML report and the web
//! version. Kept apart from the drawing code so the web build needs no window libraries.

/// How a birth chart is laid out.
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub enum Style {
    /// South Indian: signs fixed in a 4×4 frame, Aries second from the top left.
    South,
    /// North Indian: houses fixed in a diamond, the ascendant at the top.
    North,
    /// East Indian (Bengal): signs fixed, Aries in the top middle, running anticlockwise.
    East,
}
