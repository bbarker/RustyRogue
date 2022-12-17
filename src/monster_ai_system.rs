use crate::{
    components::*, gamelog::GameLog, map::Map, player::get_player_entities_with_pos, RunState,
};

use super::{Monster, Viewshed};
use bracket_lib::terminal::DistanceAlg;
use specs::prelude::*;

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        ReadExpect<'a, RunState>,
        WriteExpect<'a, Map>,
        ReadStorage<'a, Player>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, EventWantsToMelee>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut log,
            runstate,
            map,
            players,
            mut viewsheds,
            monsters,
            names,
            mut positions,
            mut wants_to_melee,
        ) = data;

        if *runstate == RunState::MonsterTurn {
            let player_entities_with_pos =
                get_player_entities_with_pos(&entities, &players, &positions);

            (&entities, &mut viewsheds, &monsters, &names, &mut positions)
                .join()
                .for_each(|(entity, viewshed, _monster, name, pos)| {
                    player_entities_with_pos
                        .iter()
                        .for_each(|(player_entity, player_pos)| {
                            if viewshed.visible_tiles.contains(&(*player_pos).into()) {
                                let distance = DistanceAlg::Pythagoras
                                    .distance2d((*pos).into(), (*player_pos).into());
                                if distance < 1.5 {
                                    wants_to_melee
                                        .insert(
                                            entity,
                                            EventWantsToMelee {
                                                target: *player_entity,
                                            },
                                        )
                                        .unwrap_or_else(|_| {
                                            panic!(
                                                "Unable to insert attack on player from {}",
                                                name.name,
                                            )
                                        });
                                    log.entries.push(format!("{} shouts insults", name.name));
                                } else {
                                    let path = bracket_lib::prelude::a_star_search(
                                        pos.idx(map.width_psnu),
                                        player_pos.idx(map.width_psnu),
                                        &*map,
                                    );
                                    if path.success && path.steps.len() > 1 {
                                        *pos = map.idx_to_pos(path.steps[1]);
                                        viewshed.dirty = true;
                                    }
                                }
                            }
                        })
                });
        }
    }
}
