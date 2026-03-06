#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Left,
    Right,
}

// TRUE = LEFT, FALSE = RIGHT
impl From<bool> for Direction {
    fn from(value: bool) -> Self {
        if value {
            Self::Left
        } else {
            Self::Right
        }
    }
}