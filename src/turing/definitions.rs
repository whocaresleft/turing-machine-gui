#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction { Left, Right }

#[derive(Clone, Copy)]
pub enum State { State(usize) }
impl State {
    pub fn make_state(inner: usize) -> Self {
        State::State(inner)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum LSymbol {
    Symbol(usize),
    Shift(Direction)
}
impl LSymbol {
    pub fn make_shift(inner: Direction) -> Self {
        LSymbol::Shift(inner)
    }
    pub fn make_symbol(inner: usize) -> Self {
        LSymbol::Symbol(inner)
    }
}

pub const BLANK_L_SYMBOL: LSymbol = LSymbol::Symbol(0);
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum RSymbol {
    Symbol(char)
}

