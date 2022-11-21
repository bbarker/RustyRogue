use crate::Position;

pub struct Rect {
    pub x1: u32,
    pub x2: u32,
    pub y1: u32,
    pub y2: u32,
}

impl Rect {
    pub fn new(xx: u32, yy: u32, ww: u32, hh: u32) -> Rect {
        Rect {
            x1: xx,
            y1: yy,
            x2: xx + ww,
            y2: yy + hh,
        }
    }

    /// Returns true if this overlaps with other
    pub fn intersect(&self, other: &Rect) -> bool {
        self.x1 <= other.x2 && self.x2 >= other.x1 && self.y1 <= other.y2 && self.y2 >= other.y1
    }

    pub fn center(&self) -> Position {
        Position {
            xx: (self.x1 + self.x2) / 2,
            yy: (self.y1 + self.y2) / 2,
        }
    }
}
