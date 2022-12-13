use bracket_lib::prelude::{Algorithm2D, BaseMap};
use bracket_lib::random::RandomNumberGenerator;
use bracket_lib::terminal::{BTerm, DistanceAlg, Point, RGB};
use itertools::Itertools;
use specs::*;
use std::cmp::{max, min};

use crate::components::{xy_idx, Positionable};
use crate::{Position, PsnU, State};

use crate::rect::*;

const MOVE_THROUGH_WALLS: bool = true;

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

// TODO: factor out the iterator, and have alternative draw_map functions
//       that can be used in different contexts.
pub fn draw_map(ecs: &World, ctx: &mut BTerm) {
    // let mut viewsheds = ecs.write_storage::<Viewshed>();
    // let mut players = ecs.write_storage::<Player>();
    let map = ecs.fetch::<Map>();

    //(&mut players, &mut viewsheds)
    //    .join()
    //     .for_each(|(_player, _viewshed)| {
    map.tiles.iter().enumerate().for_each(|(ix, tile)| {
        let tile_pos = map.idx_to_xy(ix);
        // if viewshed.visible_tiles.contains(&tile_pos.to_point()) {
        if map.revealed_tiles[ix] {
            let (fg, glyph) = match tile {
                TileType::Floor => (
                    RGB::from_f32(0.5, 0.5, 0.5),
                    bracket_lib::prelude::to_cp437('.'),
                ),
                TileType::Wall => (
                    RGB::from_f32(0., 1.0, 0.),
                    bracket_lib::prelude::to_cp437('#'),
                ),
            };
            let fg = if !map.visible_tiles[ix] {
                fg.to_greyscale()
            } else {
                fg
            };
            ctx.set(
                tile_pos.xx,
                tile_pos.yy,
                fg,
                RGB::from_f32(0., 0., 0.),
                glyph,
            )
        }
    })
    //})
}

pub struct Map {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    width: usize,
    height: usize,
    tile_count: usize,
    pub width_psnu: PsnU,
    pub height_psnu: PsnU,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked: Vec<bool>,
    pub tile_content: Vec<Vec<Entity>>,
}

pub fn idx_to_xy(width: usize, ix: usize) -> Position {
    let xx = ix % width;
    let yy = ix / width;
    Position {
        xx: xx.try_into().unwrap(),
        yy: yy.try_into().unwrap(),
    }
}

impl Map {
    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn tile_count(&self) -> usize {
        self.tile_count
    }

    pub fn idx_to_xy(self: &Map, ix: usize) -> Position {
        idx_to_xy(self.width, ix)
    }

    pub fn xy_idx(self: &Map, xx: PsnU, yy: PsnU) -> usize {
        xy_idx(self.width_psnu, xx, yy)
    }

    pub fn pos_idx(self: &Map, pos: impl Positionable) -> usize {
        pos.idx(self.width_psnu)
    }

    fn is_exit_valid(&self, xx: PsnU, yy: PsnU) -> bool {
        if xx < 1 || xx > self.width_psnu - 2 || yy < 1 || yy > self.height_psnu - 2 {
            false
        } else {
            let ix = self.xy_idx(xx, yy);
            !self.blocked[ix]
        }
    }

    fn add_room(self: &mut Map, room: &Rect) {
        (room.x1..=room.x2)
            .cartesian_product(room.y1..=room.y2)
            .for_each(|(xx, yy)| {
                self.tiles[xy_idx(self.width_psnu, xx, yy)] = TileType::Floor;
            })
    }

    fn add_horizontal_tunnel(self: &mut Map, x1: PsnU, x2: PsnU, yy: PsnU) {
        (min(x1, x2)..=max(x1, x2)).for_each(|xx| {
            let ix = self.xy_idx(xx, yy);
            if ix > 0 && ix < (self.tile_count) {
                self.tiles[ix] = TileType::Floor;
            }
        })
    }

    fn add_vertical_tunnel(self: &mut Map, y1: PsnU, y2: PsnU, xx: PsnU) {
        (min(y1, y2)..=max(y1, y2)).for_each(|yy| {
            let ix = self.xy_idx(xx, yy);
            if ix > 0 && ix < (self.tile_count) {
                self.tiles[ix] = TileType::Floor;
            }
        })
    }

    pub fn populate_blocked(&mut self) {
        if MOVE_THROUGH_WALLS {
            self.blocked = vec![false; self.tile_count];
        } else {
            self.blocked = self.tiles.iter().map(|t| *t == TileType::Wall).collect();
        }
    }

    pub fn clear_content_index(&mut self) {
        self.tile_content.iter_mut().for_each(|vc| vc.clear());
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, ix: usize) -> bool {
        self.tiles[ix.clamp(0, self.tile_count - 1)] == TileType::Wall
    }

    fn get_available_exits(&self, ix: usize) -> bracket_lib::prelude::SmallVec<[(usize, f32); 10]> {
        let mut exits = bracket_lib::prelude::SmallVec::new();

        let pos = self.idx_to_xy(ix);
        let north = self.xy_idx(pos.xx, pos.yy - 1);
        let south = self.xy_idx(pos.xx, pos.yy + 1);
        let east = self.xy_idx(pos.xx + 1, pos.yy);
        let west = self.xy_idx(pos.xx - 1, pos.yy);

        let north_west = self.xy_idx(pos.xx - 1, pos.yy - 1);
        let north_east = self.xy_idx(pos.xx + 1, pos.yy - 1);
        let south_west = self.xy_idx(pos.xx - 1, pos.yy + 1);
        let south_east = self.xy_idx(pos.xx + 1, pos.yy + 1);

        // Cardinal directions
        if self.is_exit_valid(pos.xx, pos.yy - 1) {
            exits.push((north, 1.0))
        }
        if self.is_exit_valid(pos.xx, pos.yy + 1) {
            exits.push((south, 1.0))
        }
        if self.is_exit_valid(pos.xx + 1, pos.yy) {
            exits.push((east, 1.0))
        }
        if self.is_exit_valid(pos.xx - 1, pos.yy) {
            exits.push((west, 1.0))
        }

        // Diagonals
        if self.is_exit_valid(pos.xx - 1, pos.yy - 1) {
            exits.push((north_west, 1.414))
        }
        if self.is_exit_valid(pos.xx + 1, pos.yy - 1) {
            exits.push((north_east, 1.414))
        }
        if self.is_exit_valid(pos.xx - 1, pos.yy + 1) {
            exits.push((south_west, 1.414))
        }
        if self.is_exit_valid(pos.xx + 1, pos.yy + 1) {
            exits.push((south_east, 1.414))
        }

        exits
    }

    fn get_pathing_distance(&self, ix1: usize, ix2: usize) -> f32 {
        let p1 = self.idx_to_xy(ix1);
        let p2 = self.idx_to_xy(ix2);
        DistanceAlg::Pythagoras.distance2d(p1.into(), p2.into())
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width_psnu, self.height_psnu)
    }
}
pub fn new_map_rooms_and_corridors(gs: &State) -> Map {
    let map_width = gs.display.width.try_into().unwrap();
    let display_height: usize = gs.display.height.try_into().unwrap();
    let map_height = display_height - crate::gui::PANEL_HEIGHT;
    let map_tile_count: usize = map_width * map_height;
    let mut map = Map {
        tiles: vec![TileType::Wall; map_tile_count],
        rooms: Vec::new(),
        width: map_width,
        height: map_height,
        tile_count: map_tile_count,
        width_psnu: gs.display.width,
        height_psnu: gs.display.height,
        revealed_tiles: vec![false; map_tile_count],
        visible_tiles: vec![false; map_tile_count],
        blocked: vec![false; map_tile_count],

        /// The map_indexing system already visits each tile in the map to looking for blocking tiles
        /// so we can instead alter that scan to populate
        /// which entities are at each tile, preventing us from having to iterate over the join of all
        /// entities and positions again. We store the result in `tile_content`.
        tile_content: vec![Vec::new(); map_tile_count],
    };

    const MAX_ROOMS: u16 = 30;
    const MIN_SIZE: PsnU = 6;
    const MAX_SIZE: PsnU = 10;

    let mut rng = gs.ecs.write_resource::<RandomNumberGenerator>();

    (0..MAX_ROOMS).for_each(|_| {
        let ww = rng.range(MIN_SIZE, MAX_SIZE);
        let hh = rng.range(MIN_SIZE, MAX_SIZE);
        let xx = rng.range(1, map_width as u16 - ww - 1);
        let yy = rng.range(1, map_height as u16 - hh - 1);

        let new_room = Rect::new(xx, yy, ww, hh);

        let room_ok = map
            .rooms
            .iter()
            .all(|other_room| !new_room.intersect(other_room));
        if room_ok {
            map.add_room(&new_room);
            if let Some(prev_room) = map.rooms.last() {
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
