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
        WriteStorage<'a, Confusion>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut log,
            runstate,
            mut map,
            players,
            mut viewsheds,
            monsters,
            names,
            mut positions,
            mut wants_to_melee,
            mut confused,
        ) = data;

        if *runstate == RunState::MonsterTurn {
            let player_entities_with_pos =
                get_player_entities_with_pos(&entities, &players, &positions);

            (&entities, &mut viewsheds, &monsters, &names, &mut positions)
                .join()
                .for_each(|(entity, viewshed, _monster, name, pos)| {
                    let (is_confused, confusion_opt) = match confused.get_mut(entity) {
                        conf @ Some(_) => (true, conf),
                        None => (false, None),
                    };

                    let can_act = !is_confused;

                    if can_act {
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
                                            .unwrap_or_else(|er| {
                                                panic!(
                                                    "Unable to insert attack on player from {}: {}",
                                                    name.name, er
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
                                            let new_pos = map.idx_to_pos(path.steps[1]);
                                            map.move_blocker(pos, &new_pos);
                                            viewshed.dirty = true;
                                        }
                                    }
                                }
                            })
                    } else if let Some(confusion) = confusion_opt {
                        if let Some(step) = confusion.step_sequence.pop() {
                            let try_pos = map.dest_from_delta(pos, step.0 as i32, step.1 as i32);
                            if !map.blocked[map.pos_idx(try_pos)] {
                                map.move_blocker(pos, &try_pos);
                                viewshed.dirty = true;
                            }
                        } else {
                            confused.remove(entity);
                        }
                    }
                });
        }
    }
}
