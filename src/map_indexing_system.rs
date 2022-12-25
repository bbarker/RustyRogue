use super::{BlocksTile, Map, Position};
use specs::prelude::*;

pub struct MapIndexingSystem {}

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, BlocksTile>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, pos, blockers, entities) = data;

        // Perform map-based blocking before entity-based blocking, so that
        // populate_blocked won't clear entities that are blocking.
        map.populate_blocked();
        map.clear_content_index();

        (&pos, &entities).join().for_each(|(pos, entity)| {
            let ix = map.pos_idx(*pos);
            if let Some(_b) = blockers.get(entity) {
                map.blocked[ix] = true;
            }

            map.tile_content[ix].push(entity);
        });
    }
}

pub fn move_blocker(map: &mut Map, pos: &mut Position, new_pos: &Position) {
    let old_ix = map.pos_idx(*pos);
    let new_ix = map.pos_idx(*new_pos);
    map.blocked[new_ix] = true;
    *pos = *new_pos;
    map.blocked[old_ix] = false;
}
