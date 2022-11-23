use crate::{Position, PSN_U};

pub struct Rect {
    pub x1: PSN_U,
    pub x2: PSN_U,
    pub y1: PSN_U,
    pub y2: PSN_U,
}

impl Rect {
    pub fn new(xx: PSN_U, yy: PSN_U, ww: PSN_U, hh: PSN_U) -> Rect {
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
