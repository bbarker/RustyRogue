use crate::{
    components::{Name, PlayerPosition, Position, Positionable},
    map::Map,
};

use super::{Monster, Viewshed};
use bracket_lib::prelude::console;
use specs::prelude::*;

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, PlayerPosition>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (map, ppos, mut viewsheds, monsters, names, mut positions) = data;

        (&mut viewsheds, &monsters, &names, &mut positions)
            .join()
            .for_each(|(mut viewshed, _monster, name, position)| {
                if viewshed.visible_tiles.contains(&ppos.pos().into()) {
                    console::log(format!("{} shouts insults", name.name));
                    let path = bracket_lib::prelude::a_star_search(
                        position.idx(map.width_psnu),
                        ppos.pos().idx(map.width_psnu),
                        &*map,
                    );
                    if path.success && path.steps.len() > 1 {
                        *position = map.idx_to_xy(path.steps[1]);
                        viewshed.dirty = true;
                    }
                }
            });
    }
}
