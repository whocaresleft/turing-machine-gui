use super::{symbol::{Symbol, BLANK}, direction::Direction, Dimension};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum LSymbol {
    Symbol(Symbol),
    Shift(Direction),
}

impl From<Symbol> for LSymbol {
    fn from(value: Symbol) -> Self {
        Self::Symbol(value)
    }
}
impl From<Dimension> for LSymbol {
    fn from(value: Dimension) -> Self {
        Self::Symbol(Symbol::from(value))
    }
}

impl From<Direction> for LSymbol {
    fn from(value: Direction) -> Self {
        Self::Shift(value)
    }
}
impl From<bool> for LSymbol {
    fn from(value: bool) -> Self {
        Self::Shift(Direction::from(value))
    }
}


pub const BLANK_L: LSymbol = LSymbol::Symbol(BLANK);
pub const LEFT_SHIFT: LSymbol = LSymbol::Shift(Direction::Left);
pub const RIGHT_SHIFT: LSymbol = LSymbol::Shift(Direction::Right);