use crate::{
    components::{Player, Position, Positionable},
    entity_action_msg,
    gamelog::GameLog,
    map::Map,
    RunState,
};

use super::{CombatStats, EventIncomingDamage};
// use specs::prelude::*;
use bevy::prelude::*;

pub struct DamageSystem {}

/*
impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, EventIncomingDamage>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut combat_stats, mut incoming_damage) = data;

        (&mut combat_stats, &incoming_damage)
            .join()
            .for_each(|(stats, damage)| {
                stats.hp -= damage.amount.iter().sum::<u16>().clamp(0, stats.hp);
            });
        incoming_damage.clear();
    }
}
*/
// Now we convert the above to a bevy-ecs system:
pub fn damage_system(
    mut commands: Commands,
    query: Query<(Entity, &mut CombatStats, With<EventIncomingDamage>)>,
) {
    query.for_each_mut(|(entity, stats, damage)| {
        stats.hp -= damage.amount.iter().sum::<u16>().clamp(0, stats.hp);
        commands.entity(entity).remove::<EventIncomingDamage>();
    });
    // TODO: see util_ecs for a possible WIP for a clear; for now we clear
    // individually, which should be fine.
}

pub fn delete_the_dead(
    mut commands: Commands,
    mut log: ResMut<GameLog>,
    mut map: ResMut<Map>,
    query: Query<(Entity, &CombatStats, &Position)>,
    players: Query<(Entity, &Player)>,
) -> Option<RunState> {
    let mut dead: Vec<Entity> = Vec::new();
    let mut newrunstate_opt = None;
    {
        query.for_each(|(ent, stats, pos)| {
            if stats.hp < 1 {
                dead.push(ent);
                // FIXME:
                log.entries.push(entity_action_msg_no_ecs!(
                    ecs,
                    "<SUBJ> {} dead.",
                    ent,
                    "are"
                ));

                if let Ok(_player) = players.get(ent) {
                    {
                        newrunstate_opt = Some(RunState::GameOver);
                    }
                }
                let ix = { map.pos_idx(pos.from()) };
                map.blocked[ix] = false;
            }
        });
    }
    dead.iter()
        .for_each(|victim| commands.entity(*victim).despawn());
    newrunstate_opt
}
