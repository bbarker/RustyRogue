use crate::{components::*, gamelog::GameLog, map::Map, RunState};

use super::{Monster, Viewshed};
use bevy::prelude::*;
use bracket_lib::terminal::DistanceAlg;

fn monster_ai_system(
    mut commands: Commands,
    mut log: ResMut<GameLog>,
    runstate: Res<RunState>,
    mut map: ResMut<Map>,
    mut query: Query<(Entity, &mut Viewshed, &Monster, &Name, &mut Position)>,
    player_query: Query<(Entity, &Position, With<Player>)>,
    mut wants_to_melee_query: Query<&mut EventWantsToMelee>,
    mut confused_query: Query<&mut Confusion>,
) {
    if *runstate == RunState::MonsterTurn {
        let player_entities_with_pos: Vec<(Entity, Position)> = player_query.iter().collect();

        query
            .iter_mut()
            .for_each(|(entity, mut viewshed, _monster, name, mut pos)| {
                let is_confused = confused_query.get_mut(entity).is_ok();
                let can_act = !is_confused;

                if can_act {
                    for (player_entity, player_pos) in &player_entities_with_pos {
                        if viewshed.visible_tiles.contains(&(*player_pos).into()) {
                            let distance = DistanceAlg::Pythagoras
                                .distance2d((*pos).into(), (*player_pos).into());
                            if distance < 1.5 {
                                commands.entity(entity).insert(EventWantsToMelee {
                                    target: *player_entity,
                                });
                                log.entries
                                    .push(format!("{} shouts insults", name.as_str()));
                            } else {
                                let path = bracket_lib::prelude::a_star_search(
                                    pos.idx(map.width_psnu),
                                    player_pos.idx(map.width_psnu),
                                    &*map,
                                );
                                if path.success && path.steps.len() > 1 {
                                    let new_pos = map.idx_to_pos(path.steps[1]);
                                    map.move_blocker(&mut *pos, &new_pos);
                                    viewshed.dirty = true;
                                }
                            }
                        }
                    }
                } else if let Ok(mut confusion) = confused_query.get_mut(entity) {
                    if let Some(step) = confusion.step_sequence.pop() {
                        let try_pos = map.dest_from_delta(&*pos, step.0 as i32, step.1 as i32);
                        if !map.blocked[map.pos_idx(try_pos)] {
                            map.move_blocker(&mut *pos, &try_pos);
                            viewshed.dirty = true;
                        }
                    } else {
                        commands.entity(entity).remove::<Confusion>();
                    }
                }
            });
    }
}
