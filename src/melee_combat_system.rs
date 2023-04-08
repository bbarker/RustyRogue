use crate::{
    components::{DefenseBonus, Equipped, MeleePowerBonus},
    gamelog::GameLog,
};

use super::{CombatStats, EventIncomingDamage, EventWantsToMelee, Name};
use bracket_lib::terminal::console;
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
        ReadStorage<'a, DefenseBonus>,
        ReadStorage<'a, MeleePowerBonus>,
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
            melee_bonus,
            defense_bonus,
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
                    (&entities, &melee_bonus, &equipped)
                        .join()
                        .filter(|(_e, _mb, eq)| eq.owner == entity)
                        .map(|(_e, mb, _eq)| mb.bonus)
                        .sum::<i16>()
                } else {
                    0
                };
                let defensive_bonus = target_stats_opt.map_or(0, |ts| {
                    if ts.hp > 0 {
                        (&entities, &defense_bonus, &equipped)
                            .join()
                            .filter(|(_e, _db, eq)| eq.owner == target)
                            .map(|(_e, db, _eq)| db.bonus)
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
                    console::log(format!(
                        "power {} and power with bonus {}",
                        stats.power, stats_power_with_bonus
                    )); // FIXME: we seem to be getting 0 power bonus even when items are equipped
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
