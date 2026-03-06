use super::{LSymbol};

pub struct Tape<const K: usize> {

    traces: [Vec<LSymbol>; K],
    heads: [usize; K],
    extends_on_end: bool,
}