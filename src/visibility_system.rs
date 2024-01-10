use crate::{
    components::{Player, Positionable},
    player::PLAYER_NAME,
};

use super::{Map, Position, Viewshed};
use bevy::prelude::*;
use bracket_lib::prelude::field_of_view;

pub fn visibility_system(
    map: ResMut<Map>,
    query: Query<(Entity, &Viewshed, &Position)>,
    player_query: Query<(Entity, &Player, &Name)>,
) {
    query.iter().for_each(|(ent, viewshed, pos)| {
        if viewshed.dirty {
            viewshed.dirty = false;
            viewshed.visible_tiles.clear();
            viewshed.visible_tiles = field_of_view((*pos).into(), viewshed.range.0, &mut map);
            viewshed.visible_tiles.retain(|pt| {
                pt.x >= 0
                    && pt.x < map.width().try_into().unwrap()
                    && pt.y >= 0
                    && pt.y < map.height().try_into().unwrap()
            });
            player_query
                .iter()
                .for_each(|player_ent, _player, player_name| {
                    if ent == player_ent && player_name == PLAYER_NAME {
                        map.visible_tiles = vec![false; map.tile_count()];
                        viewshed.visible_tiles.iter().for_each(|vis| {
                            let width = map.width().try_into().unwrap();
                            let ix = vis.idx(width);
                            map.revealed_tiles[ix] = true;
                            map.visible_tiles[ix] = true;
                        });
                    }
                })
        }
    })
}
