use crate::{
    components::{debug_name, Equipped, Item},
    gamelog::GameLog,
};

use super::{CombatStats, EventIncomingDamage, EventWantsToMelee};
use bevy::prelude::*;

pub fn melee_combat_system(
    mut commands: Commands,
    mut log: ResMut<GameLog>,
    mut inflict_damage: Query<&mut EventIncomingDamage>,
    names_query: Query<&Name>,
    combat_stats_query: Query<&CombatStats>,
    wants_melee_query: Query<(Entity, &Name, &CombatStats, &EventWantsToMelee)>,
    equipped_items_query: Query<(Entity, &Item, &Equipped)>,
) {
    wants_melee_query.for_each(|(entity, name, stats, wants_melee)| {
        let target = wants_melee.target;
        let debug_name = debug_name();
        let target_name = names_query.get(target).unwrap_or(&debug_name);
        let target_stats_opt = combat_stats_query.get(target);

        let offensive_bonus = if stats.hp > 0 {
            equipped_items_query
                .iter()
                .filter(|(_e, _itm, eq)| eq.owner == entity)
                .filter_map(|(_, item, _)| item.equip_opt().map(|et| et.power_bonus()))
                .sum::<i16>()
        } else {
            0
        };

        let defensive_bonus = target_stats_opt.map_or(0, |ts| {
            if ts.hp > 0 {
                equipped_items_query
                    .iter()
                    .filter(|(_e, _itm, eq)| eq.owner == target)
                    .filter_map(|(_, item, _)| item.equip_opt().map(|et| et.defense_bonus()))
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
                    name.as_str(),
                    target_name.as_str(),
                    damage
                ));
            } else {
                log.entries.push(format!(
                    "{} is unable to hurt {}.",
                    name.as_str(),
                    target_name.as_str()
                ));
            }
        }
    });

    // Assuming EventWantsToMelee has a clear method or you can use commands to remove them
    // FIXME: I think this has to be moved into the for_each but for some reason it isn't showing
    // as an error yet
    commands.remove::<EventWantsToMelee>();
}
