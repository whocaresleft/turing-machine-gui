use super::Dimension;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Symbol(Dimension);

impl From<Dimension> for Symbol {
    fn from(value: Dimension) -> Self { Self(value) }
}

pub const BLANK: Symbol = Symbol(0);