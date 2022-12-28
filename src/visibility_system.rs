use crate::components::{Player, Positionable};

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
                    viewshed.dirty = false;
                    viewshed.visible_tiles.clear();
                    viewshed.visible_tiles = field_of_view((*pos).into(), viewshed.range.0, &*map);
                    viewshed.visible_tiles.retain(|pt| {
                        pt.x >= 0
                            && pt.x < map.width().try_into().unwrap()
                            && pt.y >= 0
                            && pt.y < map.height().try_into().unwrap()
                    });
                    if let Some(_p) = player.get(ent) {
                        map.visible_tiles = vec![false; map.tile_count()];
                        viewshed.visible_tiles.iter().for_each(|vis| {
                            let width = map.width().try_into().unwrap();
                            let ix = vis.idx(width);
                            map.revealed_tiles[ix] = true;
                            map.visible_tiles[ix] = true;
                        });
                    }
                }
            })
    }
}
