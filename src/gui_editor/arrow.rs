use egui::{epaint::CubicBezierShape, Color32, Pos2};
use super::drawable::Drawable;
use super::TEXT_COLOR;

pub struct Arrow {
    pub id: usize,

    pub start: Pos2,
    pub end: Pos2,

    pub labels: Vec<String>,

    pub id_from_node: usize,
    pub id_to_node: Option<usize>,

    stroke: egui::Stroke
}

impl Arrow {
    pub fn new(id: usize, start: Pos2, end: Pos2, from: usize, to: Option<usize>) -> Self {
        Arrow {
            id: id,

            start: start,
            end: end,

            labels: vec![],

            id_from_node: from,
            id_to_node: to,
            stroke: egui::Stroke::new(1.5, egui::Color32::WHITE)
        }
    }
    
    pub fn add_label(&mut self, label: String) {
        self.labels.push(label)
    }
    pub fn remove_label_by_index(&mut self, index: usize) {
        if index >= self.labels.len() { return }
        self.labels.remove(index);
    }

    pub fn is_near_curve(&self, pos: Pos2) -> bool {
        let off = self.get_control_offset();
        let (start, c1, c2, end) = (self.start, off.0, off.1, self.end);

        let min_x = start.x.min(c1.x).min(c2.x).min(end.x);
        let max_x = start.x.max(c1.x).max(c2.x).max(end.x);
        let min_y = start.y.min(c1.y).min(c2.y).min(end.y);
        let max_y = start.y.max(c1.y).max(c2.y).max(end.y);

        if pos.x < min_x || pos.x > max_x || pos.y < min_y || pos.y > max_y {
            return false;
        }
        let steps = 6;
        let mut prev = start;
        for i in 1..=steps {
            let t = i as f32 / steps as f32;
            let pt = cubic_bezier_point(t, &[start, c1, c2, end]);
            let d = Arrow::distance_point_to_segment(pos, prev, pt);
            if d < 6.0 { return true }
            prev = pt;
        }
        return false;
    }

    pub fn distance_point_to_segment(p: Pos2, a: Pos2, b: Pos2) -> f32 {
        let ab = b - a;
        let ap = p - a;
        let ab_len_sq = ab.length_sq();
        if ab_len_sq == 0.0 {
            return ap.length();
        }
        let t = (ap.dot(ab) / ab_len_sq).clamp(0.0, 1.0);
        let closest = a + ab * t;
        (p - closest).length()
    }

    pub fn get_control_offset(&self) -> (Pos2, Pos2) {
        let (h_offset, v_offset) = {
        if let Some(to_node) = self.id_to_node {
            if self.id_from_node == to_node {
                (80.0, 120.0)
            } else {
                (((self.end.x - self.start.x).abs() * 0.4).max(40.0), 0.0)
            }
        } else {
            (((self.end.x - self.start.x).abs() * 0.4).max(40.0), 0.0)
        }
        };
        let control1 = self.start + egui::vec2(h_offset, v_offset);
        let control2 = self.end - egui::vec2(h_offset, -v_offset);
        (control1, control2)
    }
}

impl Drawable for Arrow {
    fn draw(&self, painter: &egui::Painter) {
        let (control1, control2) = self.get_control_offset();

        let (p0, p1, p2, p3) = (self.start, control1, control2, self.end);

        let bezier = egui::Shape::CubicBezier(
            CubicBezierShape {
                points: [p0, p1, p2, p3],
                closed: false,
                fill: Color32::TRANSPARENT,
                stroke: self.stroke.into()
            }
        );
        painter.add(bezier);

        let start_label_position = cubic_bezier_point(0.5, &[p0, p1, p2, p3]) + egui::vec2(0.0, 12.0);

        for (i, label) in self.labels.iter().enumerate() {
            painter.text(
                start_label_position + egui::vec2(0.0,i as f32 * 16.0),
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::monospace(15.0),
                TEXT_COLOR
            );
        }

        let tip_points = {
            if let Some(to) = self.id_to_node {
                if to == self.id_from_node {
                    vec![
                        Pos2::new(self.end.x - 10.0, self.end.y + 0.5),
                        self.end,
                        Pos2::new(self.end.x - 1.0, self.end.y + 15.0),
                    ]
                } else {
                    vec![
                        Pos2::new(self.end.x - 10.0, self.end.y + 10.0),
                        self.end,
                        Pos2::new(self.end.x - 10.0, self.end.y - 10.0),
                    ]  
                }
            } else {
                vec![
                    Pos2::new(self.end.x - 10.0, self.end.y + 10.0),
                    self.end,
                    Pos2::new(self.end.x - 10.0, self.end.y - 10.0),
                ]
            }
        };
        
        painter.add(egui::Shape::line(
            tip_points,
            self.stroke
        ));
    }
}


fn cubic_bezier_point(t: f32, p: &[Pos2; 4]) -> Pos2 {

    // B(t) = (1-t)^3 * P0 + 3(1-t)^2 *tP1 + 3(1-t) * t^2 P2 + t^3 * P3

    let u = 1.0 - t;
    let tt = t * t;
    let uu = u * u;
    let uuu = uu * u;
    let ttt = tt * t;

    let mut vec = p[0].to_vec2() * uuu;
    vec += p[1].to_vec2() * (3.0 * uu * t);
    vec += p[2].to_vec2() * (3.0 * u * tt);
    vec += p[3].to_vec2() * ttt;

    Pos2::new(vec.x, vec.y)
}

impl super::Serializable for Arrow {
    fn serialize(&self) -> String {
        let mut x = format!("{}, {}, {}, {}, {}, [", 
            self.id, 
            self.start, 
            self.end, 
            self.id_from_node, match 
            self.id_to_node { None => { String::from("none") } Some(id) => { format!("{}", id) } });
        for (i, label) in self.labels.iter().enumerate() {
            x.push_str(label.as_str());
            if i < self.labels.len() - 1 { x.push_str(", ") }
        }
        x.push(']');
        x
    }
}
impl super::Deserializable for Arrow {
    fn deserialize(from: &String) -> Result<Self, String> where Self: Sized {
        let pieces: Vec<&str> = from.split(", ").collect();

        let id: usize = pieces.get(0)
            .ok_or("No id")?
            .parse().map_err(|_| String::from("Could not parse id"))?;
        
        let start_str = pieces.get(1)
            .ok_or("No start")?
            .trim_matches(|c| c == '[' || c == ']');
        let start_vec: Vec<u32> = start_str.split_whitespace()
            .map(|n| n.parse::<u32>()
            .map_err(|_| "Could not parse start".to_owned()))
            .collect::<Result<Vec<_>, _>>()?;
        
        let end_str = pieces.get(2)
            .ok_or("No end")?
            .trim_matches(|c| c == '[' || c == ']');
        let end_vec: Vec<u32> = end_str.split_whitespace()
            .map(|n| n.parse::<u32>()
            .map_err(|_| "Could not parse end".to_owned()))
            .collect::<Result<Vec<_>, _>>()?;
        
        let from_id: usize = pieces.get(3)
            .ok_or("No id from")?
            .parse()
            .map_err(|_| String::from("Could not parse id from"))?;
        let to_id: Option<usize> = {
            let x = pieces.get(4).ok_or("No id to")?;
            if *x == "none" { None }
            else { Some(x.parse().map_err(|_| String::from("Could not parse to id"))?) }
        };

        let mut labels = Vec::<String>::with_capacity(pieces.len() - 5);
        for i in 5..pieces.len() {
            let label = pieces.get(i)
                .ok_or("No labels")?
                .trim_matches(|c| c == '[' || c == ']');
            labels.push(label.to_owned());
        }
        

        let mut arrow = Arrow::new(
            id, 
            Pos2::new(start_vec[0] as f32, start_vec[1] as f32), 
            Pos2::new(end_vec[0] as f32, end_vec[1] as f32),
            from_id,
            to_id
        );
        arrow.labels = labels;

        Ok(arrow)
    }
}