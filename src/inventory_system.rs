use std::collections::HashSet;

use bracket_lib::prelude::field_of_view;
use itertools::Itertools;
use specs::{prelude::*, world::EntitiesRes};

use crate::{
    components::{
        AreaOfEffect, CombatStats, Confusion, Consumable, Equipped, EventIncomingDamage,
        EventWantsToDropItem, EventWantsToUseItem, InflictsDamage, IsItem, Item, ProvidesHealing,
    },
    equipment::{get_equipped_items, EquipSlot},
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
        ReadStorage<'a, Item>,
        WriteStorage<'a, Equipped>,
        WriteStorage<'a, InBackpack>,
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
            items,
            mut equipped,
            mut backpack,
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
                if let Some(Item::Equippable(equip)) = items.get(useitem.item) {
                    let player_equip = get_equipped_items(&items, &equipped, player_entity);
                    targets.first().iter().for_each(|target| {
                        let new_equip = Equipped::new(player_entity, &player_equip, &equip.allowed_slots);
                        // TODO: warn on non-unit discard?:
                        equip_slot(&mut log, &entities, &mut backpack, &items, &mut equipped, &names, **target, new_equip);
                     });
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
fn equip_slot<I: Join>(
    log: &mut WriteExpect<GameLog>,
    entities: &Read<EntitiesRes>,
    backpack: &mut WriteStorage<InBackpack>,
    items: I,
    equipped_items: &mut WriteStorage<Equipped>,
    names: &ReadStorage<Name>,
    target_item_entity: Entity, // Should be first in 'targets'
    new_equip: Equipped,
) -> HashSet<(Entity, Item)>
where
    I: Copy,
    I::Type: IsItem,
{
    let to_unequip = calculate_unequip(
        entities,
        items,
        equipped_items,
        new_equip.owner,
        new_equip.slot.clone(),
    )
    .into_iter()
    .chain(new_equip.slot_extra.iter().flat_map(|slot2| {
        calculate_unequip(
            entities,
            items,
            equipped_items,
            new_equip.owner,
            slot2.clone(),
        )
    }))
    .collect_vec();

    HashSet::from_iter(
        to_unequip
            .into_iter()
            .map(|(item_ent, _, itm)| {
                equipped_items.remove(item_ent);
                let unequip_name = names.get(item_ent).unwrap();
                log.entries
                    .push(format!("You unequip {}.", unequip_name.name));
                backpack
                    .insert(
                        item_ent,
                        InBackpack {
                            owner: new_equip.owner,
                        },
                    )
                    .unwrap();
                backpack.remove(target_item_entity);
                equipped_items
                    .insert(target_item_entity, new_equip.clone())
                    .unwrap();
                let equip_name = names.get(target_item_entity).unwrap();
                log.entries.push(format!("You equip {}.", equip_name.name));
                (item_ent, itm)
            })
            .collect_vec(),
    )
}

fn calculate_unequip<I: Join>(
    entities: &Read<EntitiesRes>,
    items: I, // FIXME: need this to be a reference, somehow
    equipped: &mut WriteStorage<Equipped>,
    owner: Entity,
    slot: EquipSlot,
) -> Vec<(Entity, Equipped, Item)>
where
    I: Copy,
    I::Type: IsItem,
{
    (entities, equipped, items)
        .join()
        .filter(|(_, eq, _)| eq.owner == owner && eq.slot == slot)
        .map(|(ent, eq, item)| (ent, eq.clone(), item.from()))
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
