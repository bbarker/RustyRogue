use specs::prelude::*;

use crate::{
    components::{
        CombatStats, Consumable, EventIncomingDamage, EventWantsToDropItem, EventWantsToUseItem,
        InflictsDamage, ProvidesHealing,
    },
    map::Map,
    player::PLAYER_NAME,
};

use super::{gamelog::GameLog, EventWantsToPickupItem, InBackpack, Name, Player, Position};

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Player>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackpack>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, EventWantsToPickupItem>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, players, mut log, names, mut backpack, mut positions, mut wants_pickup) =
            data;

        // TODO: (multi-player: fix this to be player-specific -
        // prevent other players from picking up the same item)
        (&entities, &players, &mut wants_pickup).join().for_each(
            |(player_entity, _player, pickup)| {
                positions.remove(pickup.item);
                backpack
                    .insert(
                        pickup.item,
                        InBackpack {
                            owner: player_entity,
                        },
                    )
                    .unwrap_or_else(|_| panic!("Unable to insert item into backpack!"));

                if pickup.collected_by == player_entity {
                    log.entries.push(format!(
                        "You pick up the {}.",
                        names.get(pickup.item).unwrap().name
                    ));
                }
            },
        );
        wants_pickup.clear();
    }
}

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Map>,
        ReadStorage<'a, Player>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, EventWantsToUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, ProvidesHealing>,
        ReadStorage<'a, InflictsDamage>,
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, EventIncomingDamage>,
        ReadStorage<'a, Consumable>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            map,
            players,
            mut log,
            mut wants_use_item,
            names,
            healing,
            damaging,
            mut combat_stats,
            mut incoming_damage,
            consumables,
        ) = data;

        (
            &entities,
            &players,
            &mut wants_use_item,
            &mut combat_stats,
            &names,
        )
            .join()
            .for_each(|(_player_entity, _player, useitem, stats, player_name)| {
                let item_heals = healing.get(useitem.item);
                match item_heals {
                    None => {}
                    Some(healer) => {
                        stats.hp = u16::min(stats.max_hp, stats.hp + healer.heal_amount);
                        if player_name.name == PLAYER_NAME {
                            log.entries.push(format!(
                                "You drink the {}, healing {} hp.",
                                names.get(useitem.item).unwrap().name,
                                healer.heal_amount
                            ));
                            let consumable = consumables.get(useitem.item);
                            match consumable {
                                None => {}
                                Some(_) => {
                                    entities.delete(useitem.item).unwrap_or_else(|_| {
                                        panic!("Delete item failed for player {}", player_name.name)
                                    });
                                }
                            }
                        }
                    }
                }
                let item_damages = damaging.get(useitem.item);
                match item_damages {
                    None => {}
                    Some(damage) => {
                        let target_pos = useitem.target.unwrap_or_else(|| {
                            panic!("Unable to get target position for item requiring target!",)
                        });
                        let ix = map.pos_idx(target_pos);
                        let used = map.tile_content[ix]
                            .iter()
                            .map(|victim| {
                                EventIncomingDamage::new_damage(
                                    &mut incoming_damage,
                                    *victim,
                                    damage.damage,
                                )
                            })
                            .count()
                            > 0;
                        if player_name.name == PLAYER_NAME {
                            log.entries.push(format!(
                                "You use the {}, inflicting {} damage.",
                                names.get(useitem.item).unwrap().name,
                                damage.damage
                            ));
                            let consumable = consumables.get(useitem.item);
                            match consumable {
                                None => {}
                                Some(_) => {
                                    if used {
                                        entities.delete(useitem.item).unwrap_or_else(|_| {
                                            panic!(
                                                "Delete item failed for player {}",
                                                player_name.name
                                            )
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            });
        wants_use_item.clear();
    }
}

pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, EventWantsToDropItem>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, InBackpack>,
        WriteExpect<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut wants_drop, names, mut positions, mut backpack, mut log) = data;
        (&entities, &wants_drop)
            .join()
            .for_each(|(entity, to_drop)| {
                let dropper_pos = positions.get(entity).unwrap();
                positions
                    .insert(to_drop.item, *dropper_pos)
                    .unwrap_or_else(|_| panic!("Unable to drop item!"));
                backpack.remove(to_drop.item);
                let item_name = names
                    .get(to_drop.item)
                    .map(|n| n.name.clone())
                    .unwrap_or_else(|| format!("item {}", to_drop.item.id()));
                let dropper_name = names
                    .get(entity)
                    .map(|n| n.name.clone())
                    .unwrap_or_else(|| format!("Entity {}", entity.id()));
                log.entries
                    .push(format!("{} drops the {}.", dropper_name, item_name));
            });
        wants_drop.clear();
    }
}
