use crate::{components::Player, map::pos_idx};

use super::{Map, Position, Viewshed};
use bracket_lib::prelude::field_of_view;
use specs::prelude::*;

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Player>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, entities, mut viewshed, pos, player) = data;

        (&entities, &mut viewshed, &pos)
            .join()
            .for_each(|(ent, viewshed, pos)| {
                if viewshed.dirty {
                    viewshed.visible_tiles.clear();
                    viewshed.visible_tiles = field_of_view(pos.to_point(), viewshed.range, &*map);
                    viewshed.visible_tiles.retain(|pt| {
                        pt.x >= 0
                            && pt.x < map.width.try_into().unwrap()
                            && pt.y >= 0
                            && pt.y < map.height.try_into().unwrap()
                    });
                    if let Some(_p) = player.get(ent) {
                        viewshed.visible_tiles.iter().for_each(|vis| {
                            let width = map.width.try_into().unwrap();
                            map.revealed_tiles[pos_idx(width, *vis)] = true;
                        });
                    }
                    viewshed.dirty = false;
                }
            })
    }
}
