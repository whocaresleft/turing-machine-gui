//#![windows_subsystem = "windows"]

mod turing;
use turing::*;
mod gui_editor;
use gui_editor::Editor;

use std::sync::mpsc::{self, Sender, Receiver};
use gui_editor::ui::TMInfo;

fn main() -> eframe::Result<()> {

    let (tx, rx): (Sender<TMInfo>, Receiver<TMInfo>) = mpsc::channel();

    std::thread::spawn(move || {
        let mut c = Computation::<1>::new();
        loop {
            if let Ok(info) = rx.recv() {
                println!("Received info");
                let alphabet = {
                    let mut x = Alphabet::new('*');
                    for char in info.alphabet.chars() {
                        println!("Char: {}", char);
                        if char != 'L' && char != 'R' {
                            x.add_symbol(char).ok();
                            println!("writing: {}", x.get_l_symbol(&char).unwrap());
                        }
                    }
                    x
                };
                if let Some(mut machine) = TuringMachine::<1>::new(info.n_states as usize, alphabet.len() as usize).ok() {
                    let mut real_content: Vec<LSymbol> = vec![];
                    for char in info.input.chars() {
                        print!("Converting {} to ", char);
                        real_content.push(
                            match char {
                                'L' => machine.sx(),
                                'R' => machine.dx(),
                                _ => alphabet.get_l_symbol(&char).unwrap_or(0),
                            }
                        );
                        println!("{}", real_content[real_content.len() - 1]);
                    }
                    println!("Tape contains: {:?}", &real_content);
                    for t in info.transitions {
                        print!("Original {:?}", t);
                        let (q, x, a, t) = {

                            let x: LSymbol = match t.1 {
                                'L' => machine.sx(),
                                'R' => machine.dx(),
                                _ => alphabet.get_l_symbol(&t.1).unwrap_or(0),
                            };
                            let y: LSymbol = match t.2 {
                                'L' => machine.sx(),
                                'R' => machine.dx(),
                                _ => alphabet.get_l_symbol(&t.2).unwrap_or(0),
                            };

                            (t.0, x, y, t.3)
                        };
                        println!(" => Rewritten {:?}", (q,x,a,t));
                        machine.add_transition(q, [x], [a], t).ok();
                    }
                    c.use_alphabet(alphabet);
                    c.use_machine(machine);
                    c.use_tapes([Tape::with_content(real_content, true)]);
                    c.set_input_string(info.input);

                    c.start();
                }
            }
        }
    });

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
                Editor::new(tx)
            )
        ))
    )
}