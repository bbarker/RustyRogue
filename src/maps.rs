use crate::{display_state::*, Position};

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

pub fn xy_idx(display: &DisplayState, xx: u32, yy: u32) -> usize {
    ((yy * display.width) + xx).try_into().unwrap()
}

fn new_map(display: &DisplayState, player_position: Position) -> Vec<TileType> {
    let mut map = vec![TileType::Floor; (display.width * display.height).try_into().unwrap()];
    (0..display.width).for_each(|xx| {
        map[xy_idx(display, xx, 0)] = TileType::Wall;
        map[xy_idx(display, xx, display.height - 1)] = TileType::Wall;
    });
    (0..display.height).for_each(|yy| {
        map[xy_idx(display, 0, yy)] = TileType::Wall;
        map[xy_idx(display, display.width - 1, yy)] = TileType::Wall;
    });

    let mut rng = bracket_lib::random::RandomNumberGenerator::new();
    (0..300).for_each(|_i| {
        let xx = rng
            .roll_dice(1, display.width_i32() - 1)
            .try_into()
            .unwrap();
        let yy = rng
            .roll_dice(1, display.height_i32() - 1)
            .try_into()
            .unwrap();
        let ix = xy_idx(display, xx, yy);
        if ix != xy_idx(display, player_position.xx, player_position.yy) {
            map[ix] = TileType::Wall;
        }
    });
    map
}
