use egui::{Color32, Pos2, Rect};

use super::Deserializable;
use super::Serializable;
use super::node::Node;
use super::arrow::Arrow;
use super::drawable::Drawable;
pub const FG: Color32 = Color32::from_rgb(0x00, 0x71, 0xEB);
const BG: [Color32; 2] = [Color32::TRANSPARENT, Color32::from_rgb(0x25, 0x25, 0x25)];

use std::sync::mpsc::Sender;

pub struct NodeEditor {
    dragging_arrow: Option<Arrow>,
    selected_node_id: Option<usize>,
    selected_arrow_id: Option<usize>,

    to_remove_next_frame: Option<(usize, usize)>,
    new_label: String,

    nodes: Vec<Option<Node>>,
    arrows: Vec<Option<Arrow>>,

    input: String,
    n_tapes: u8,

    channel_sender: Sender<TMInfo>
}

impl NodeEditor {
    pub fn new(tx: Sender<TMInfo>) -> Self {
        NodeEditor {
            dragging_arrow: None,
            selected_node_id: None,
            selected_arrow_id: None,

            to_remove_next_frame: None,
            new_label: String::new(),

            nodes: vec![],
            arrows: vec![],

            n_tapes: 1,
            input: String::new(),

            channel_sender: tx
        }
    }

    pub fn delete_node(&mut self, node_id: usize) {
        if node_id >= self.nodes.len() { return }
        self.nodes[node_id] = None;
        for maybe_arrow in &mut self.arrows {
            if let Some(arrow) = maybe_arrow {
                if arrow.id_from_node == node_id || arrow.id_to_node.unwrap() == node_id {
                    *maybe_arrow = None;
                }
            }
        }
    }

    pub fn insert_new_node(&mut self, mut node: Node) {
        let mut new_node_id = self.nodes.len();
        for (i, maybe_node) in self.nodes.iter().enumerate() {
            if maybe_node.is_none() {
                new_node_id = i;
                break;
            }
        };
        node.id = new_node_id;
        if new_node_id == self.nodes.len() { self.nodes.push(Some(node)) }
        else { self.nodes[new_node_id] = Some(node) }
    }

    pub fn delete_arrow(&mut self, arrow_id: usize) {
        if arrow_id >= self.arrows.len() { return }
        self.arrows[arrow_id] = None;
    }
    
    pub fn insert_new_arrow(&mut self, mut arrow: Arrow) {
        let mut new_arrow_id = self.arrows.len();
        for (i, maybe_arrow) in self.arrows.iter().enumerate() {
            if maybe_arrow.is_none() {
                new_arrow_id = i;
                break;
            }
        }
        arrow.id = new_arrow_id;
        if new_arrow_id == self.arrows.len() { self.arrows.push(Some(arrow)) }
        else { self.arrows[new_arrow_id] = Some(arrow) }
    }

    fn from_serialized(&mut self, ser: std::fs::File) -> Result<(), String> {
        use std::io::{BufRead, BufReader};
        let reader = BufReader::new(ser);
        self.arrows.drain(..);
        self.nodes.drain(..);
        
        let lines = reader.lines().into_iter();

        let (mut in_nodes, mut in_arrows) = (false, false);

        for maybe_line in lines {
            let line = maybe_line.map_err(|_| "No line here".to_owned())?;

            if line.starts_with("Nodes = [") { in_nodes = true; continue }
            if line.starts_with("Arrows = [") { in_arrows = true; continue; }
            if line.starts_with("]") { in_nodes = false; in_arrows = false; continue }

            if in_nodes {
                self.nodes.push(
                    match line.as_str() {
                        "none" => None,
                        _ => Some(Node::deserialize(&line)?)
                    }
                );
                continue;
            }
            if in_arrows {
                self.arrows.push(
                    match line.as_str() {
                        "none" => None,
                        _ => Some(Arrow::deserialize(&line)?)
                    }
                );
                continue;
            }
        }
        
        Ok(())
    }    

    fn gather_all_information(&self) -> TMInfo {
        let mut set = std::collections::HashSet::<char>::new();
        let mut transitions = vec![];
        for maybe_arrow in &self.arrows {
            if let Some(arrow) = maybe_arrow {
                for label in &arrow.labels {
                    let mut chars = vec![];
                    for char in label.chars() {
                        if char != '/' {
                            set.insert(char);
                            chars.push(char);
                        }
                    }
                    transitions.push((arrow.id_from_node as u8, chars[0], chars[1], arrow.id_to_node.unwrap_or(0) as u8));
                }
            }
        }
        TMInfo {
            n_states: self.nodes.len() as u8,
            alphabet: set.iter().collect(),
            input: self.input.clone(),
            transitions: transitions,
        }
    }
}

impl super::Serializable for NodeEditor {
    fn serialize(&self) -> String {
        let mut result = String::from("Nodes = [\n");
        for maybe_node in &self.nodes {
            result.push_str(format!("{}\n",
                match maybe_node {
                    None => String::from("none"),
                    Some(node) => node.serialize(),
                }
            ).as_str());
        }
        result.push_str("]\nArrows = [\n");
        for maybe_arrow in &self.arrows {
            result.push_str(
                format!("{}\n",
                    match maybe_arrow {
                        None => String::from("none"),
                        Some(arrow) => arrow.serialize()
                    }
                ).as_str()
            );
        }
        result.push(']');
        result
    }

}

pub struct TMInfo {
    pub n_states: u8,
    pub alphabet: String,
    pub transitions: Vec<(u8, char, char, u8)>,
    pub input: String
}

impl eframe::App for NodeEditor {

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some((i, j)) = self.to_remove_next_frame.take() { self.arrows[i].as_mut().unwrap().remove_label_by_index(j);}
        egui::CentralPanel::default().show(ctx, |ui| {
            let (size_x, size_y) = {
                let size = ui.available_size_before_wrap();
                (size.x, size.y)
            };
            ui.horizontal(|ui| {

                // Buttons + editor
                ui.group(|ui| {
                    let size = egui::vec2(size_x * 0.73, size_y * 0.985);
                    ui.set_min_size(size);

                    ui.vertical(|ui| {
                        let (y_1, y_2) = match size.y {
                            0.0..527.0 => (size.y * 0.08, size.y * 0.86),
                            527.0..775.0 => (size.y * 0.08, size.y * 0.86),
                            _ => (size.y * 0.08, size.y * 0.89)
                        };
                        ui.group( |ui| {
                            ui.set_min_size(egui::vec2(size.x * 0.99, y_1));
                            ui.add_space(size.y * 0.015);
                            ui.horizontal(|ui| {
                                ui.add_space(10.0);
                                if let Some(node_id) = self.selected_node_id {
                                    ui.group(|ui| {
                                        ui.label(egui::RichText::new(format!("Selected Node: {}", node_id)).font(egui::FontId::monospace(20.0)));
                                    });
                                    if ui.add_sized([120.0, 40.0], egui::Button::new(
                                        egui::RichText::new("Delete Node").font(egui::FontId::monospace(20.0))
                                    )).clicked() {
                                        self.delete_node(node_id);
                                        self.selected_node_id = None;
                                    }
                                    if let Some(node) = &mut self.nodes[node_id] {
                                        ui.group(|ui| {
                                            ui.label(egui::RichText::new("Label: ").font(egui::FontId::monospace(20.0)));
                                            ui.add(
                                                egui::TextEdit::singleline(&mut node.label)
                                                    .font(egui::FontId::monospace(20.0))
                                                    .background_color(Color32::TRANSPARENT)
                                                    .desired_width(100.0)
                                            );
                                            ui.checkbox(&mut node.separate_header, 
                                                egui::RichText::new("Show header").font(egui::FontId::monospace(20.0))
                                            );
                                            if node.label.len() > 10 { node.label.truncate(10) }

                                            ui.add_space(10.0);

                                            ui.checkbox(&mut node.is_final, 
                                                egui::RichText::new("Final").font(egui::FontId::monospace(20.0))
                                            );
                                        });
                                    }
                                    ui.add_space(10.0);
                                } else if let Some(arrow_id) = self.selected_arrow_id {
                                    ui.group(|ui| {
                                        ui.label(egui::RichText::new(format!("Selected Arrow: {}", arrow_id)).font(egui::FontId::monospace(20.0)));
                                    });
                                    if ui.add_sized([120.0, 40.0], egui::Button::new(
                                        egui::RichText::new("Delete Arrow").font(egui::FontId::monospace(20.0))
                                    )).clicked() {
                                        self.delete_arrow(arrow_id);
                                        self.selected_arrow_id = None;
                                    }
                                    ui.group(|ui| {
                                        if let Some(arrow) = &mut self.arrows[arrow_id] {
                                            ui.label(egui::RichText::new("Label: ").font(egui::FontId::monospace(20.0)));
                                            ui.add(
                                                egui::TextEdit::singleline(&mut self.new_label)
                                                    .font(egui::FontId::monospace(20.0))
                                                    .background_color(Color32::TRANSPARENT)
                                                    .desired_width(100.0)
                                            );
                                            if ui.button(egui::RichText::new("Add label").font(egui::FontId::monospace(20.0))).clicked() {
                                                let x = std::mem::take(&mut self.new_label);
                                                arrow.add_label(x);
                                            }
                                        }
                                    });
                                } else {
                                    if ui.add_sized([120.0, 40.0], egui::Button::new(
                                        egui::RichText::new("New Node").font(egui::FontId::monospace(20.0))
                                    )).clicked() {
                                        let node = make_node(0, true);
                                        self.insert_new_node(node);
                                    }
                                    if ui.add_sized([120.0, 40.0], egui::Button::new(
                                        egui::RichText::new("Save").font(egui::FontId::monospace(20.0))
                                    )).clicked() {
                                        let export = self.serialize();
                                        if let Some(path) = rfd::FileDialog::new()
                                            .set_title("Save")
                                            .set_file_name("export.txt")
                                            .add_filter("Text", &["txt"])
                                            .save_file()
                                        {
                                            std::fs::write(path,  export).ok();
                                        }
                                    }
                                    if ui.add_sized([120.0, 40.0], egui::Button::new(
                                        egui::RichText::new("Load").font(egui::FontId::monospace(20.0))
                                    )).clicked() {
                                        if let Some(path) = rfd::FileDialog::new()
                                            .set_title("Load")
                                            .add_filter("Text", &["txt"])
                                            .pick_file()
                                        {
                                            if let Ok(file) = std::fs::File::open(path) {
                                                self.from_serialized(file).ok();
                                            } else { println!("ERROR file") }
                                        }
                                    }
                                    ui.group(|ui| {
                                        ui.label(egui::RichText::new("Number of tapes: ").font(egui::FontId::monospace(20.0)));
                                        egui::ComboBox::from_label("")
                                            .selected_text(egui::RichText::new(format!("{:?}", self.n_tapes)).font(egui::FontId::monospace(20.0)))
                                            .show_ui(ui, |ui| {
                                                ui.selectable_value(&mut self.n_tapes, 1, egui::RichText::new("1").font(egui::FontId::monospace(20.0)));
                                                ui.selectable_value(&mut self.n_tapes, 2, egui::RichText::new("2").font(egui::FontId::monospace(20.0)));
                                                ui.selectable_value(&mut self.n_tapes, 3, egui::RichText::new("3").font(egui::FontId::monospace(20.0)));
                                            }
                                        );
                                    });
                                    if ui.add_sized([120.0, 40.0], egui::Button::new(
                                        egui::RichText::new("Execute").font(egui::FontId::monospace(20.0))
                                    )).clicked() {
                                        let y = self.gather_all_information();
                                        self.channel_sender.send(y).ok();
                                    }
                                }
                            });
                        });
                        ui.group( |ui| {
                            ui.set_min_size(egui::vec2(size.x * 0.99, y_2));
                            let group_rect = Rect::from_min_max(Pos2::new(25.0 ,y_1 + 45.0), Pos2::new(size.x * 0.99 + 15.0, y_1 + 40.0 + y_2));
                            let response = ui.interact(group_rect, egui::Id::new("sandbox-bg"), egui::Sense::click());
                            if response.clicked() {
                                self.selected_node_id = None;
                                self.selected_arrow_id = None;
                                if let Some(pos) = response.interact_pointer_pos() {
                                    for maybe_arrow in &self.arrows {
                                        if let Some(arrow) = maybe_arrow {
                                            if arrow.is_near_curve(pos) {
                                                self.selected_arrow_id = Some(arrow.id);
                                            }
                                        }
                                    }
                                }
                            }

                            for maybe_node in &mut self.nodes {
                                if let Some(node) = maybe_node {
                                    let response = ui.interact(node.rect(), egui::Id::new(node.id), egui::Sense::click_and_drag());
                                    if response.dragged_by(egui::PointerButton::Primary) {
                                        node.change_position(response.drag_delta());

                                        let x_clamped = node.top_left.x.clamp(
                                            group_rect.min.x, group_rect.max.x - node.size.x
                                        );
                                        let y_clamped = node.top_left.y.clamp(
                                            group_rect.min.y, group_rect.max.y - node.size.y
                                        );
                                        node.top_left = Pos2::new(x_clamped, y_clamped);
                                    }
                                    if response.clicked_by(egui::PointerButton::Secondary) {
                                        let pos = node.get_output_edge();
                                        self.dragging_arrow = Some(make_arrow(self.arrows.len(), pos, pos, node.id));
                                    }
                                    if response.clicked_by(egui::PointerButton::Primary) {
                                        self.selected_node_id = Some(node.id);
                                        self.selected_arrow_id = None;
                                    }
                                    node.draw(ui.painter());
                                }
                            }
                                                    
                            if let Some(arrow) = &mut self.dragging_arrow {
                                if let Some(mouse_position) = ui.ctx().pointer_hover_pos() {
                                    arrow.end = mouse_position;
                                    arrow.draw(ui.painter());
                                }

                                if ui.input(|i| i.pointer.primary_clicked()) {
                                    if let Some(mut arrow) = self.dragging_arrow.take() {
                                        for maybe_node in &self.nodes {
                                            if let Some(node) = maybe_node {
                                                if node.rect().contains(arrow.end) {
                                                    arrow.id_to_node = Some(node.id);
                                                    if !is_present(&self.arrows, &arrow) {
                                                        arrow.end = node.get_input_edge();
                                                        self.insert_new_arrow(arrow);
                                                        break;  
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            for maybe_arrow in &mut self.arrows {
                                if let Some(arrow) = maybe_arrow {
                                    let (from, to) = (arrow.id_from_node, arrow.id_to_node.unwrap());
                                    arrow.start = self.nodes[from].as_ref().unwrap().get_output_edge();
                                    arrow.end = self.nodes[to].as_ref().unwrap().get_input_edge();
                                    arrow.draw(ui.painter());
                                }
                            }

                        });
                    });
                });

                // Arrows + input
                ui.group(|ui| {
                    let x = match size_x {
                        1550.0..=1800.0 => size_x * 0.25,
                        1160.0..1550.0 => size_x * 0.235,
                        0.0..1160.0 => size_x * 0.22,
                        _ => size_x * 0.25,
                    };
                    ui.set_min_size(egui::vec2(x, size_y * 0.985));

                    ui.vertical(|ui| {
                        let size = ui.available_size();
                        let (y_1, y_2, y_3) = match size.y {
                            0.0..527.0 => (size.y * 0.39, size.y * 0.39, size.y * 0.05),
                            527.0..775.0 => (size.y * 0.39, size.y * 0.39, size.y * 0.05),
                            _ => (size.y * 0.445, size.y * 0.445, size.y * 0.05)
                        };
                        ui.group(|ui| {
                            let s = egui::vec2(size.x * 0.965, y_1);
                            ui.set_min_size(s);
                            ui.set_max_size(s);
                            ui.vertical_centered(|ui| {
                                ui.heading(egui::RichText::new("Nodes").font(egui::FontId::monospace(20.0)));
                                ui.separator();
                                egui::ScrollArea::vertical().id_salt(0).show(ui, |ui| {
                                    for (i, maybe_node) in self.nodes.iter_mut().enumerate() {
                                        if let Some(node) = maybe_node {
                                            ui.label(
                                                egui::RichText::new(
                                                    format!("Node {}: {}", node.id, node.label))
                                                    .font(egui::FontId::monospace(20.0)
                                                ).background_color(BG[i & 0b1])
                                            );
                                        }
                                    }
                                });
                            });
                        });
                        ui.add_space(size_y * 0.001);
                        ui.group(|ui| {
                            ui.set_min_size(egui::vec2(size.x * 0.965, y_2));
                            ui.vertical_centered(|ui| {
                                ui.heading(egui::RichText::new("Arrows").font(egui::FontId::monospace(20.0)));
                                ui.separator();
                                egui::ScrollArea::vertical().id_salt(1).show(ui, |ui| {
                                    for (i, maybe_arrow) in self.arrows.iter_mut().enumerate() {
                                        if let Some(arrow) = maybe_arrow {
                                            let from_node = self.nodes[arrow.id_from_node].as_ref().unwrap();
                                            let to_node = self.nodes[arrow.id_to_node.unwrap()].as_ref().unwrap();
                                            for (j, label) in arrow.labels.iter_mut().enumerate() {
                                                ui.horizontal(|ui| {
                                                    ui.label(
                                                        egui::RichText::new(
                                                            format!("{} -> {} | ",
                                                            from_node.id,
                                                            to_node.id,
                                                        ))
                                                            .font(egui::FontId::monospace(20.0)
                                                        ).background_color(BG[i & 0b1])
                                                    );
                                                    ui.add(
                                                        egui::TextEdit::singleline(label)
                                                            .font(egui::FontId::monospace(20.0))
                                                            .desired_width(100.0)
                                                            .background_color(Color32::TRANSPARENT)
                                                    );
                                                    if label.len() > 2 * self.n_tapes as usize + 1 { label.truncate(2 * self.n_tapes as usize + 1) }
                                                    if ui.button("X").clicked() {
                                                        self.to_remove_next_frame = Some((i, j));
                                                    }
                                                });
                                            }
                                        }
                                    }
                                });
                            });
                        });
                        ui.add_space(size_y * 0.001);
                        ui.group(|ui| {
                            ui.set_min_size(egui::vec2(size.x * 0.965, y_3));
                            ui.vertical_centered(|ui| {
                                ui.heading(egui::RichText::new("Input").font(egui::FontId::monospace(20.0)));
                                ui.separator();
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.input)
                                        .font(egui::FontId::monospace(20.0))
                                        .desired_width(size.x * 0.9)
                                        .background_color(Color32::TRANSPARENT)
                                );
                            });
                        });
                    });
                });
            });
        });
    }
}

const STARTING_POSITION: Pos2 = Pos2::new(50.0, 150.0);
fn make_node(id: usize, header: bool) -> Node {
    Node::new(
        id,
        format!(""),
        STARTING_POSITION,
        FG,
        header
    )
}
fn make_arrow(id: usize, start: Pos2, end: Pos2, from: usize) -> Arrow {
    Arrow::new(
        id,
        start,
        end,
        from,
        None
    )  
}

fn is_present(arrows: &Vec<Option<Arrow>>, arrow: &Arrow) -> bool {
    assert!(arrow.id_to_node.is_some());
    let (from, to) = (arrow.id_from_node, arrow.id_to_node.unwrap());
    for maybe_arrow_ in arrows {
        if let Some(arrow_) = maybe_arrow_ {
            let (from_, to_) = (arrow_.id_from_node, arrow_.id_to_node.unwrap());
            if from == from_ && to == to_ { return true }
        }
    }
    false
}
