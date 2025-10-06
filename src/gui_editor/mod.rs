pub mod arrow;
pub mod node;
pub mod ui;
pub mod drawable;

pub const TEXT_COLOR: egui::Color32 = egui::Color32::from_rgb(0xE6, 0xE6, 0xE6);
pub use ui::NodeEditor as Editor;

pub trait Serializable {
    fn serialize(&self) -> String;
}
pub trait Deserializable {
    fn deserialize(from: &String) -> Result<Self, String> where Self: Sized;
}

use super::turing::{Computation, Alphabet, TuringMachine, Tape};