use bracket_lib::random::RandomNumberGenerator;
use bracket_lib::terminal::{BTerm, RGB};
use itertools::Itertools;
use std::cmp::{max, min};

use crate::{display_state::*, Position, PsnU, INIT_PLAYER_POSITION};

use crate::rect::*;

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

pub fn xy_idx(display: &DisplayState, xx: PsnU, yy: PsnU) -> usize {
    ((yy * display.width) + xx).try_into().unwrap()
}

pub fn idx_to_xy(display: &DisplayState, ix: usize) -> Position {
    let display_width: usize = display.width.try_into().unwrap();
    let xx = ix % display_width;
    let yy = ix / display_width;
    Position {
        xx: xx.try_into().unwrap(),
        yy: yy.try_into().unwrap(),
    }
}

pub fn new_map_rooms_and_corridors(
    display: &DisplayState,
    _player_position: &Position,
) -> Vec<TileType> {
    let mut map = vec![TileType::Wall; (display.width * display.height).try_into().unwrap()];

    let mut rooms: Vec<Rect> = Vec::new();
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

        let room_ok = rooms
            .iter()
            .all(|other_room| !new_room.intersect(other_room));
        if room_ok {
            map_room(display, &new_room, &mut map);
            rooms.push(new_room)
        }
    });

    map_horizontal_tunnel(display, &mut map, 25, 40, INIT_PLAYER_POSITION.yy);
    map
}

fn map_room(display: &DisplayState, room: &Rect, map: &mut [TileType]) {
    (room.x1..=room.x2)
        .cartesian_product(room.y1..=room.y2)
        .for_each(|(xx, yy)| {
            map[xy_idx(display, xx, yy)] = TileType::Floor;
        })
}

fn map_horizontal_tunnel(
    display: &DisplayState,
    map: &mut [TileType],
    x1: PsnU,
    x2: PsnU,
    yy: PsnU,
) {
    (min(x1, x2)..=max(x1, x2)).for_each(|xx| {
        let ix = xy_idx(display, xx, yy);
        if ix > 0 && ix < (display.width * display.height).try_into().unwrap() {
            map[ix] = TileType::Floor;
        }
    })
}

fn _map_vertical_tunnel(
    display: &DisplayState,
    map: &mut [TileType],
    y1: PsnU,
    y2: PsnU,
    xx: PsnU,
) {
    (min(y1, y2)..=max(y1, y2)).for_each(|yy| {
        let ix = xy_idx(display, xx, yy);
        if ix > 0 && ix < (display.width * display.height).try_into().unwrap() {
            map[ix] = TileType::Floor;
        }
    })
}

/// Makes a map with solid boundaries and randomly placed walls
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

pub fn draw_map(ctx: &mut BTerm, display: &DisplayState, map: &[TileType]) {
    map.iter().enumerate().for_each(|(ix, tile)| {
        let tile_pos = idx_to_xy(display, ix);
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
