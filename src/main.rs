#![windows_subsystem = "windows"]

mod turing;
use turing::*;
mod gui_editor;
use gui_editor::Editor;

fn main() -> eframe::Result<()> {

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1800.0, 1100.0])
            .with_min_inner_size([1000.0, 600.0])
            .with_transparent(true),
        ..Default::default()
    };
    eframe::run_native(
        "Whocaresleft?'s Turing machine editor", 
        options, 
        Box::new(|_cc| Ok(
            Box::new(
                Editor::new()
            )
        ))
    )
}