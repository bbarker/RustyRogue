use crate::display_state::*;

#[derive(PartialEq, Copy, Clone)]

pub enum TileType {
    wall,
    Floor,
}

pub fn xy_idx(display: &DisplayState, x: u32, y: u32) -> usize {
    ((y * display.width) + x) as usize // TODO: safer conversion
}
