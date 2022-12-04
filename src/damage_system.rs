use bracket_lib::prelude::console;

use crate::{
    components::{Position, Positionable},
    map::Map,
};

use super::{CombatStats, EventIncomingDamage};
use specs::prelude::*;

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, EventIncomingDamage>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut combat_stats, mut incoming_damage) = data;

        (&mut combat_stats, &incoming_damage)
            .join()
            .for_each(|(mut stats, damage)| {
                stats.hp -= damage.amount.iter().sum::<u16>();
            });
        incoming_damage.clear();
    }
}

pub fn delete_the_dead(ecs: &mut World) {
    let mut dead: Vec<Entity> = Vec::new();
    {
        let entities = ecs.entities();
        let combat_stats = ecs.read_storage::<CombatStats>();
        let positions = ecs.read_storage::<Position>();
        (&entities, &combat_stats, &positions)
            .join()
            .for_each(|(ent, stats, pos)| {
                if stats.hp < 1 {
                    dead.push(ent);
                    let ix = {
                        let map = ecs.fetch::<Map>();
                        map.pos_idx(pos.from())
                    };
                    let map = &mut ecs.fetch_mut::<Map>();
                    map.blocked[ix] = false;
                }
            });
    }
    dead.iter().for_each(|victim| {
        ecs.delete_entity(*victim).unwrap_or_else(|_| {
            console::log(format!("Unable to delete entity with id {}", victim.id()))
        })
    });
}
