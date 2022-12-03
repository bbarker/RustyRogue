use super::{BlocksTile, Map, Position};
use specs::prelude::*;

pub struct MapIndexingSystem {}

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, BlocksTile>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, pos, blockers) = data;

        map.populate_blocked();
        // map.clear_content_index();

        (&pos, &blockers).join().for_each(|(pos, _blocker)| {
            let ix = map.pos_idx(*pos);
            map.blocked[ix] = true;
        });
    }
}
