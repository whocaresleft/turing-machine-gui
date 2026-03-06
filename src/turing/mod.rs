mod state;
mod symbol;
mod alphabet;
mod tape;

pub use self::{
    state::State,
    alphabet::Alphabet,
    symbol::{
        symbol::{Symbol},
        l_symbol::{LSymbol, BLANK_L, LEFT_SHIFT, RIGHT_SHIFT},
        r_symbol::RSymbol,
    },
};

type Dimension = u64;