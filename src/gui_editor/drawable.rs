use egui::Painter;

pub trait Drawable {
    fn draw(&self, painter: &Painter);
}