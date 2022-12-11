use crate::gamelog::GameLog;

use super::{CombatStats, EventIncomingDamage, EventWantsToMelee, Name};
use specs::prelude::*;

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, CombatStats>,
        WriteStorage<'a, EventIncomingDamage>,
        WriteStorage<'a, EventWantsToMelee>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut log, names, combat_stats, mut inflict_damage, mut wants_melee) = data;

        let debug_name = Name {
            name: "<no name for entity>".to_string(),
        };
        (&entities, &names, &combat_stats, &mut wants_melee)
            .join()
            .for_each(|(_ent, name, stats, wants_melee)| {
                let target = wants_melee.target;
                let target_name = names.get(target).unwrap_or(&debug_name);
                let target_stats_opt = combat_stats.get(target);
                if let Some(target_stats) = target_stats_opt {
                    let damage = u16::max(0, stats.power - target_stats.defense);
                    if damage > 0 && target_stats.hp > 0 {
                        EventIncomingDamage::new_damage(&mut inflict_damage, target, damage);
                        log.entries.push(format!(
                            "{} hits {} for {} hp.",
                            name.name, target_name.name, damage
                        ));
                    } else {
                        log.entries.push(format!(
                            "{} is unable to hurt {}.",
                            name.name, target_name.name
                        ));
                    }
                }
            });
        wants_melee.clear();
    }
}
