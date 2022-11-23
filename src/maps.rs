use bracket_lib::terminal::{BTerm, RGB};
use itertools::Itertools;
use std::cmp::{max, min};

use crate::{display_state::*, Position, INIT_PLAYER_POSITION};

use crate::rect::*;

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

pub fn xy_idx(display: &DisplayState, xx: u32, yy: u32) -> usize {
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
    player_position: &Position,
) -> Vec<TileType> {
    let mut map = vec![TileType::Wall; (display.width * display.height).try_into().unwrap()];
    let room1 = Rect::new(20, 15, 10, 15);
    let room2 = Rect::new(35, 15, 10, 15);

    map_room(display, &room1, &mut map);
    map_room(display, &room2, &mut map);
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

fn map_horizontal_tunnel(display: &DisplayState, map: &mut [TileType], x1: u32, x2: u32, yy: u32) {
    (min(x1, x2)..=max(x1, x2)).for_each(|xx| {
        let ix = xy_idx(display, xx, yy);
        if ix > 0 && ix < (display.width * display.height).try_into().unwrap() {
            map[ix] = TileType::Floor;
        }
    })
}

fn map_vertical_tunnel(display: &DisplayState, map: &mut [TileType], y1: u32, y2: u32, xx: u32) {
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
