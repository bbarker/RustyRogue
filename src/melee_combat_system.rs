use crate::{
    components::{Equipped, Item},
    gamelog::GameLog,
};

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
        ReadStorage<'a, Equipped>,
        ReadStorage<'a, Item>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut log,
            names,
            combat_stats,
            mut inflict_damage,
            mut wants_melee,
            equipped,
            items,
        ) = data;

        let debug_name = Name {
            name: "<no name for entity>".to_string(),
        };
        (&entities, &names, &combat_stats, &mut wants_melee)
            .join()
            .for_each(|(entity, name, stats, wants_melee)| {
                let target = wants_melee.target;
                let target_name = names.get(target).unwrap_or(&debug_name);
                let target_stats_opt = combat_stats.get(target);
                let offensive_bonus = if stats.hp > 0 {
                    (&entities, &items, &equipped)
                        .join()
                        .filter(|(_e, _itm, eq)| eq.owner == entity)
                        .filter_map(|(_e, item, _eq)| item.equip_opt().map(|et| et.melee_bonus()))
                        .sum::<i16>()
                } else {
                    0
                };
                let defensive_bonus = target_stats_opt.map_or(0, |ts| {
                    if ts.hp > 0 {
                        (&entities, &items, &equipped)
                            .join()
                            .filter(|(_e, _itm, eq)| eq.owner == target)
                            .filter_map(|(_e, item, _eq)| {
                                item.equip_opt().map(|et| et.defense_bonus())
                            })
                            .sum::<i16>()
                    } else {
                        0
                    }
                });

                if let Some(target_stats) = target_stats_opt {
                    let stats_power_with_bonus: u16 = if offensive_bonus > 0 {
                        let sp: u16 = stats
                            .power
                            .saturating_add(offensive_bonus.try_into().unwrap());
                        sp
                    } else {
                        stats
                            .power
                            .saturating_sub(offensive_bonus.abs().try_into().unwrap())
                    };
                    let defense_with_bonus: u16 = if defensive_bonus > 0 {
                        let dp: u16 = target_stats
                            .defense
                            .saturating_add(defensive_bonus.try_into().unwrap());
                        dp
                    } else {
                        target_stats
                            .defense
                            .saturating_sub(defensive_bonus.abs().try_into().unwrap())
                    };
                    let damage = stats_power_with_bonus.saturating_sub(defense_with_bonus);
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
