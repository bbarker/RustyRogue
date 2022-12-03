use crate::{
    components::{Name, PlayerPosition, Position, Positionable},
    map::Map,
};

use super::{Monster, Viewshed};
use bracket_lib::{prelude::console, terminal::DistanceAlg};
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
        let (map, player_pos, mut viewsheds, monsters, names, mut positions) = data;

        (&mut viewsheds, &monsters, &names, &mut positions)
            .join()
            .for_each(|(mut viewshed, _monster, name, pos)| {
                if viewshed.visible_tiles.contains(&player_pos.pos().into()) {
                    let distance =
                        DistanceAlg::Pythagoras.distance2d((*pos).into(), player_pos.pos().into());
                    if distance < 1.5 {
                        // Attack goes here
                        console::log(format!("{} shouts insults", name.name));
                    } else {
                        let path = bracket_lib::prelude::a_star_search(
                            pos.idx(map.width_psnu),
                            player_pos.pos().idx(map.width_psnu),
                            &*map,
                        );
                        if path.success && path.steps.len() > 1 {
                            *pos = map.idx_to_xy(path.steps[1]);
                            viewshed.dirty = true;
                        }
                    }
                }
            });
    }
}
