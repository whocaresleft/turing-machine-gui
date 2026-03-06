use super::Dimension;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct State(Dimension);

impl From<Dimension> for State {
    fn from(value: Dimension) -> Self { Self(value) }
}

pub const START: State = State(0);