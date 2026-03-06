#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct RSymbol(char);

impl From<char> for RSymbol {
    fn from(value: char) -> Self { Self(value) }
}
