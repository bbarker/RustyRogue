use std::collections::HashSet;

use bracket_lib::prelude::field_of_view;
use itertools::Itertools;
use specs::{prelude::*, world::EntitiesRes};

use crate::{
    components::{
        AreaOfEffect, CombatStats, Confusion, Consumable, Equipped, EventIncomingDamage,
        EventWantsToDropItem, EventWantsToUseItem, InflictsDamage, Item, ProvidesHealing,
    },
    equipment::{get_equipped_items, EquipSlot, EquipSlotAllowed},
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
                    .unwrap_or_else(|er| panic!("Unable to insert item into backpack!: {}", er));

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
        WriteStorage<'a, Confusion>,
        ReadStorage<'a, AreaOfEffect>,
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, EventIncomingDamage>,
        ReadStorage<'a, Consumable>,
        WriteStorage<'a, Item>,
        WriteStorage<'a, Equipped>,
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
            mut confused,
            aoe,
            mut combat_stats,
            mut incoming_damage,
            consumables, // TODO: consider removing in a separate commit; should just need Item
            mut items,
            mut equipped,
        ) = data;

        let delete_if_consumed = |item: Entity, used: bool, player_name: &Name| {
            let consumable = consumables.get(item);
            match consumable {
                None => {}
                Some(_) => {
                    if used {
                        entities.delete(item).unwrap_or_else(|er| {
                            panic!("Delete item failed for player {}: {}", player_name.name, er)
                        });
                    }
                }
            }
        };

        (&entities, &players, &mut wants_use_item, &names)
            .join()
            .for_each(|(player_entity, _player, useitem, player_name)| {
                let targets = match useitem.target {
                    None => vec![player_entity],
                    Some(target) => {
                        let area_effect = aoe.get(useitem.item);
                        match area_effect {
                            None => {
                                // Single-tile target
                                map.tile_content[map.pos_idx(target)].to_vec()
                            }
                            Some(ae) => {
                                let fov_tiles =
                                    field_of_view(target, ae.radius.into(), &*map);
                                let blast_tiles = fov_tiles.into_iter().filter(|pos| {
                                    pos.x >= 0
                                        && pos.x < map.width_psnu as i32
                                        && pos.y >= 0
                                        && pos.y < map.height_psnu as i32
                                });
                                blast_tiles
                                    .into_iter()
                                    .flat_map(|pos| map.tile_content[map.pos_idx(pos)].clone())
                                    .collect()
                            }
                        }
                    }
                };
                let item_equippable = match items.get(useitem.item) {
                     Some(Item::Equippable(equip)) => {
                        targets.first().iter().for_each(|target| {
                        match equip.allowed_slots {
                            // TODO: here we need to get the current equipment map
                            // to see which slot, if any, is available - alternatively
                            // we pick the first one and do the check later - see how
                            // the book does it.
                            // OK: strategy should be:
                            // If 1H: unequip if needed and equip
                            // If 2H (Both): unequip if needed and equip
                            // If Either: equip in first open slot, otherwise:
                            //    Do a shift: unequip/equip item in first slot, then
                            //    take the item in first slot, and do the same for the second slot
                            //
                            // TODO: define function to calculate new_equip
                            EquipSlotAllowed::SingleSlot(slot) => equip_slot(&entities, items, equipped, target, new_equip),
                            EquipSlotAllowed::Both(slot1, slot2) => None, // ???
                            EquipSlotAllowed::Either(slot1, slot2) => {
                                let player_equip = get_equipped_items(&items, &equipped, player_entity);

                                None //TODO : rm, fix return value
                            }
                        };
                    });
                    }
                    _ => None,
                };

                let item_heals = healing.get(useitem.item);
                match item_heals {
                    None => {}
                    Some(healer) => {
                        targets.iter().for_each(|target| {
                            let stats = combat_stats.get_mut(*target).unwrap_or_else(|| {
                                panic!("Unable to get combat stats for target {}!", target.id())
                            });
                            stats.hp = u16::min(stats.max_hp, stats.hp + healer.heal_amount);
                            delete_if_consumed(useitem.item, /* used = */ true, player_name);
                            if player_name.name == PLAYER_NAME {
                                log.entries.push(format!(
                                    "You consume the {}, healing {} hp.",
                                    names.get(useitem.item).unwrap().name,
                                    healer.heal_amount
                                ));
                            }
                        });
                    }
                }
                let item_damages = damaging.get(useitem.item);
                match item_damages {
                    None => {}
                    Some(damage) => {
                        let used = targets
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
                        delete_if_consumed(useitem.item, used, player_name);
                        if player_name.name == PLAYER_NAME {
                            log.entries.push(format!(
                                "You use the {}, inflicting {} damage.",
                                names.get(useitem.item).unwrap().name,
                                damage.damage
                            ));
                        }
                    }
                }
                let confuser_item = confused.get(useitem.item).cloned();
                match confuser_item {
                    None => {}
                    Some(confusion) => {
                        let used = targets
                            .iter()
                            .map(|victim| {
                                confused
                                    .insert(*victim, confusion.clone())
                                    .unwrap_or_else(|er| {
                                        panic!(
                                            "Unable to insert confusion component for target {}!: {}",
                                            victim.id(),
                                            er
                                        )
                                    });
                                if player_name.name == PLAYER_NAME {
                                    log.entries.push(format!(
                                        "You use {} on {}, confusing them.",
                                        names.get(useitem.item).unwrap().name,
                                        names.get(*victim).unwrap().name
                                    ));
                                }
                            })
                            .count()
                            > 0;
                        delete_if_consumed(useitem.item, used, player_name);
                    }
                }
            });
        wants_use_item.clear();
    }
}

/// Utility method to help with equipping - should not be relied on to fully equip an item, but only
/// equips in the given slot
fn equip_slot(
    entities: &Read<EntitiesRes>,
    items: &ReadStorage<Item>,
    equippeds: &mut WriteStorage<Equipped>,
    item_entity: Entity, // Should be first in 'targets'
    new_equip: Equipped,
) -> HashSet<(Entity, Item)> {
    // TODO: make this a set, in case they are the same (i.e. a 2 hander)
    let to_unequip = calculate_unequip(
        entities,
        items,
        equippeds,
        new_equip.owner,
        item_entity,
        new_equip.slot,
    )
    .into_iter()
    .chain(new_equip.slot_extra.iter().flat_map(|slot2| {
        calculate_unequip(
            entities,
            items,
            equippeds,
            new_equip.owner,
            item_entity,
            *slot2,
        )
    }))
    .collect_vec();

    HashSet::from_iter(
        to_unequip
            .into_iter()
            .map(|(ent, eqp, itm)| {
                equippeds.remove(ent);
                equippeds.insert(item_entity, new_equip.clone());
                (ent, itm)
            })
            .collect_vec(),
    )
}

fn calculate_unequip(
    entities: &Read<EntitiesRes>,
    items: &ReadStorage<Item>,
    equipped: &mut WriteStorage<Equipped>,
    owner: Entity,
    item_entity: Entity,
    slot: EquipSlot,
) -> Vec<(Entity, Equipped, Item)> {
    (entities, equipped, items)
        .join()
        .filter(|(_, eq, _)| eq.owner == owner && eq.slot == slot)
        .map(|(ent, eq, item)| (ent, eq.clone(), item.clone()))
        .collect()
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
                    .unwrap_or_else(|er| panic!("Unable to drop item!: {}", er));
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
