use bracket_lib::random::RandomNumberGenerator;
use bracket_lib::terminal::{BTerm, RGB};
use itertools::Itertools;
use std::cmp::{max, min};

use crate::{display_state::*, Position, PsnU};

use crate::rect::*;

#[derive(PartialEq, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

/// Makes a map with solid boundaries and randomly placed walls
/*
pub fn new_map_test(display: &DisplayState, player_position: &Position) -> Vec<TileType> {
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
*/

pub fn draw_map(ctx: &mut BTerm, map: &Map) {
    map.map.iter().enumerate().for_each(|(ix, tile)| {
        let tile_pos = map.idx_to_xy(ix);
        match tile {
            TileType::Floor => ctx.set(
                tile_pos.xx,
                tile_pos.yy,
                RGB::from_f32(0.5, 0.5, 0.5),
                RGB::from_f32(0., 0., 0.),
                bracket_lib::prelude::to_cp437('.'),
            ),
            TileType::Wall => ctx.set(
                tile_pos.xx,
                tile_pos.yy,
                RGB::from_f32(0., 1.0, 0.),
                RGB::from_f32(0., 0., 0.),
                bracket_lib::prelude::to_cp437('#'),
            ),
        }
    })
}

pub struct Map {
    pub map: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub width: usize,
    pub height: usize,
    pub width_psnu: PsnU,
    pub height_psnu: PsnU,
}

pub fn idx_to_xy(width: usize, ix: usize) -> Position {
    let xx = ix % width;
    let yy = ix / width;
    Position {
        xx: xx.try_into().unwrap(),
        yy: yy.try_into().unwrap(),
    }
}

pub fn xy_idx(width: PsnU, xx: PsnU, yy: PsnU) -> usize {
    ((yy * width) + xx).try_into().unwrap()
}

impl Map {
    pub fn idx_to_xy(self: &Map, ix: usize) -> Position {
        idx_to_xy(self.width, ix)
    }

    pub fn xy_idx(self: &Map, xx: PsnU, yy: PsnU) -> usize {
        xy_idx(self.width_psnu, xx, yy)
    }

    fn add_room(self: &mut Map, room: &Rect) {
        (room.x1..=room.x2)
            .cartesian_product(room.y1..=room.y2)
            .for_each(|(xx, yy)| {
                self.map[xy_idx(self.width_psnu, xx, yy)] = TileType::Floor;
            })
    }

    fn add_horizontal_tunnel(self: &mut Map, x1: PsnU, x2: PsnU, yy: PsnU) {
        (min(x1, x2)..=max(x1, x2)).for_each(|xx| {
            let ix = self.xy_idx(xx, yy);
            if ix > 0 && ix < (self.width * self.height) {
                self.map[ix] = TileType::Floor;
            }
        })
    }

    fn add_vertical_tunnel(self: &mut Map, y1: PsnU, y2: PsnU, xx: PsnU) {
        (min(y1, y2)..=max(y1, y2)).for_each(|yy| {
            let ix = self.xy_idx(xx, yy);
            if ix > 0 && ix < (self.width * self.height) {
                self.map[ix] = TileType::Floor;
            }
        })
    }
}

pub fn new_map_rooms_and_corridors(display: &DisplayState) -> Map {
    let mut map = Map {
        map: vec![TileType::Wall; (display.width * display.height).try_into().unwrap()],
        rooms: Vec::new(),
        width: display.width.try_into().unwrap(),
        height: display.height.try_into().unwrap(),
        width_psnu: display.width,
        height_psnu: display.height,
    };

    const MAX_ROOMS: u16 = 30;
    const MIN_SIZE: PsnU = 6;
    const MAX_SIZE: PsnU = 10;

    let mut rng = RandomNumberGenerator::new();

    (0..MAX_ROOMS).for_each(|_| {
        let ww = rng.range(MIN_SIZE, MAX_SIZE);
        let hh = rng.range(MIN_SIZE, MAX_SIZE);
        let xx = rng.range(1, display.width - ww - 1);
        let yy = rng.range(1, display.height - hh - 1);

        let new_room = Rect::new(xx, yy, ww, hh);

        let room_ok = map
            .rooms
            .iter()
            .all(|other_room| !new_room.intersect(other_room));
        if room_ok {
            map.add_room(&new_room);
            match map.rooms.last() {
                Some(prev_room) => {
                    let new_center = new_room.center();
                    let pre_center = prev_room.center();
                    if rng.range(0, 2) == 1 {
                        map.add_horizontal_tunnel(pre_center.xx, new_center.xx, pre_center.yy);
                        map.add_vertical_tunnel(pre_center.yy, new_center.yy, new_center.xx);
                    } else {
                        map.add_vertical_tunnel(pre_center.yy, new_center.yy, pre_center.xx);
                        map.add_horizontal_tunnel(pre_center.xx, new_center.xx, new_center.yy);
                    }
                }
                None => {}
            }

            map.rooms.push(new_room)
        }
    });

    map
}
// TODO: come back to this in Section 3 (Section 2 stretch goals)
// TODO: probably should regenerate the below to take into account any refactorings
/*
fn _wall_glyph(display: &DisplayState, map: &[TileType], xx: PsnU, yy: PsnU) -> FontCharType {
    let mut mask = 0;
    if yy > 0 && map[xy_idx(display, xx, yy - 1)] == TileType::Wall {
        mask += 1;
    }
    if yy < display.height - 1 && map[xy_idx(display, xx, yy + 1)] == TileType::Wall {
        mask += 2;
    }
    if xx > 0 && map[xy_idx(display, xx - 1, yy)] == TileType::Wall {
        mask += 4;
    }
    if xx < display.width - 1 && map[xy_idx(display, xx + 1, yy)] == TileType::Wall {
        mask += 8;
    }

    match mask {
        0 => to_cp437(' '),
        1 => to_cp437('╵'),
        2 => to_cp437('╷'),
        3 => to_cp437('│'),
        4 => to_cp437('╴'),
        5 => to_cp437('┘'),
        6 => to_cp437('┐'),
        7 => to_cp437('┤'),
        8 => to_cp437('╶'),
        9 => to_cp437('└'),
        10 => to_cp437('┌'),
        11 => to_cp437('├'),
        12 => to_cp437('─'),
        13 => to_cp437('┴'),
        14 => to_cp437('┬'),
        15 => to_cp437('┼'),
        _ => to_cp437('?'),
    }
}

*/