use super::{BlocksTile, Map, Position};
use bevy::prelude::*;

pub struct MapIndexingSystem {}

pub fn map_indexing_system(
    mut map: ResMut<Map>,
    query: Query<(Entity, &Position, Option<&BlocksTile>)>,
) {
    // Perform map-based blocking before entity-based blocking, so that
    // populate_blocked won't clear entities that are blocking.
    map.populate_blocked();
    map.clear_content_index();

    query.for_each(|(entity, pos, blocker)| {
        let ix = map.pos_idx(*pos);
        if blocker.is_some() {
            map.blocked[ix] = true;
        }

        map.tile_content[ix].push(entity);
    });
}
