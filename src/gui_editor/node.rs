use super::drawable::Drawable;
use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Stroke, StrokeKind, Vec2};

const BACKGROUND_COLOR: Color32 = Color32::from_rgb(0x33, 0x33, 0x33);
use super::TEXT_COLOR;

pub struct Node {
    pub id: usize,
    pub label: String,

    pub separate_header: bool,
    pub size: Vec2,

    pub top_left: Pos2,
    pub foreground_color: Color32,

    pub is_final: bool,
}

impl Node {
    pub fn new(id: usize, label: String, starting_position: Pos2, fg_color: Color32, separate_header: bool) -> Self {
        Node {
            id: id, 
            label: label,
            top_left: starting_position,
            size: egui::vec2(100.0, 100.0),
            foreground_color: fg_color,

            separate_header: separate_header,

            is_final: false
        }
    }
    pub fn change_position(&mut self, delta: Vec2) {
        self.top_left.x += delta.x;
        self.top_left.y += delta.y;
    }
    pub fn rect(&self) -> egui::Rect {
        Rect::from_points(
            &[
                self.top_left,
                Pos2::new(self.top_left.x + self.size.x, self.top_left.y + self.size.y)
            ]
        )
    }

    pub fn get_input_edge(&self) -> Pos2 {
        let (current_size, center) = {
            let rect = self.rect();
            (rect.size(), rect.center())
        };
        Pos2::new(
            center.x - current_size.x / 2.0,
            center.y
        ) 
    }

    pub fn get_output_edge(&self) -> Pos2 {
        let (current_size, center) = {
            let rect = self.rect();
            (rect.size(), rect.center())
        };
        Pos2::new(
            center.x + current_size.x / 2.0,
            center.y
        )
    }
}

impl Drawable for Node {
    fn draw(&self, painter: &Painter) {
        let whole_rect = Rect::from_points(
            &[
                self.top_left,
                Pos2::new(self.top_left.x + self.size.x, self.top_left.y + self.size.y)
            ]
        );

        painter.rect_filled(whole_rect, 10, BACKGROUND_COLOR);
        painter.rect_stroke(whole_rect, 10, Stroke::new(2.5, self.foreground_color), StrokeKind::Inside);
        
        let label_position = whole_rect.center();
        painter.text(label_position, Align2::CENTER_CENTER, self.label.as_str(), FontId::monospace(15.0), TEXT_COLOR);

        if self.separate_header {
            let foreground_rect = Rect::from_points(&[
                self.top_left,
                Pos2::new(self.top_left.x + self.size.x, self.top_left.y + self.size.y * 0.25)
            ]);
            painter.rect_filled(foreground_rect, 10, self.foreground_color);

            let header_pos = Pos2::new(
                self.top_left.x + 50.0,
                self.top_left.y + 13.0
            );
            painter.text(header_pos, Align2::CENTER_CENTER, format!("{}", self.id), FontId::monospace(15.0), TEXT_COLOR);
        }
    }
}

impl super::Serializable for Node {
    fn serialize(&self) -> String {
        let fg: u32 = {
            let x = self.foreground_color;

            0u32 
                | ((x.r() as u32) << 16) 
                | ((x.g() as u32) << 8) 
                | (x.b() as u32)
        };
        format!("{}, {}, {}, {}, {}", self.id, self.label, self.top_left, self.separate_header, fg)
    }
}
impl super::Deserializable for Node {
    fn deserialize(from: &String) -> Result<Self, String> {
        let pieces: Vec<&str> = from.split(", ").collect();

        let id: usize = pieces.get(0).ok_or("No id")?.parse().map_err(|_| String::from("Could not parse id"))?;
        let label: String = pieces.get(1).ok_or("No label")?.to_string();
        let pos_str = pieces.get(2).ok_or("No position")?.trim_matches(|c| c == '[' || c == ']');
        let pos_vec: Vec<u32> = pos_str.split_whitespace().map(|n| n.parse::<u32>().map_err(|_| "Could not parse position".to_owned())).collect::<Result<Vec<_>, _>>()?;
        let header: bool = pieces.get(3).ok_or("No header")?.parse().map_err(|_| String::from("Could not parse header"))?;
        let top_left = Pos2::new(pos_vec[0] as f32, pos_vec[1] as f32);
        let color: u32 = pieces.get(4).ok_or("No fg")?.parse().map_err(|_| String::from("Could not parse color"))?;
        let (r, g, b) = (
            ((color & 0x00FF0000) >> 16) as u8,
            ((color & 0x0000FF00) >> 8) as u8,
            (color & 0x000000FF) as u8
        );

        Ok(
            Node::new(
                id, label, top_left, Color32::from_rgb(r, g, b), header
            )
        )
    }
}    
