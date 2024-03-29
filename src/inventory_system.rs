use std::collections::HashSet;
use std::fmt::{Display, Formatter};

use crate::util_ecs::EcsActionMsgData;
use crate::{
    components::{
        AreaOfEffect, CombatStats, Confusion, Consumable, Equipped, EventIncomingDamage,
        EventWantsToDropItem, EventWantsToRemoveItem, EventWantsToUseItem, InflictsDamage, IsItem,
        Item, ProvidesHealing,
    },
    equipment::{get_equipped_items, EquipSlot},
    map::Map,
    player::PLAYER_NAME,
    util::fmt_list,
};
use bracket_lib::prelude::field_of_view;
use frunk::Monoid;
use itertools::Itertools;
use specs::{prelude::*, world::EntitiesRes};

use super::{gamelog::GameLog, EventWantsToPickupItem, InBackpack, Name, Player, Position};

pub struct ItemCollectionSystem {}

use crate::entity_action_msg_no_ecs;

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

type ItemUseSystemData<'a> = (
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

type EquipData<'a, 'b, I> = (
    &'a Read<'b, EntitiesRes>,
    &'a mut WriteStorage<'b, InBackpack>,
    I,
    &'a mut WriteStorage<'b, Equipped>,
    &'a ReadStorage<'b, Name>,
);

#[derive(Clone, Debug)]
struct EquipBonusChanges {
    defense: i16,
    power: i16,
}

impl EquipBonusChanges {
    fn unequip(item: &Item) -> Self {
        match item {
            Item::Equippable(equip) => Self {
                defense: -equip.defense_bonus(),
                power: -equip.power_bonus(),
            },
            _ => Self {
                defense: 0,
                power: 0,
            },
        }
    }
}

impl Display for EquipBonusChanges {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let def_signed = (self.defense > 0)
            .then(|| "+".to_string())
            .unwrap_or(String::empty());
        let pow_signed = (self.power > 0)
            .then(|| "+".to_string())
            .unwrap_or(String::empty());
        let def_str = (self.defense != 0)
            .then(|| format!("DEF: {def_signed}{} ", self.defense))
            .unwrap_or(String::empty());
        let pow_str = (self.power != 0)
            .then(|| format!("POW: {pow_signed}{} ", self.power))
            .unwrap_or(String::empty());
        let result = vec![def_str, pow_str]
            .into_iter()
            .filter(|s| !s.is_empty())
            .join("; ");
        write!(formatter, "{}", result)
    }
}

#[derive(Clone, Debug)]
struct EquipChanges {
    init_equipped: HashSet<(Entity, Item)>,
    equipped: HashSet<(Entity, Item)>,
    unequipped: HashSet<(Entity, Item)>,
}

impl EquipChanges {
    fn new(init_equipped: HashSet<(Entity, Item)>) -> Self {
        Self {
            init_equipped,
            equipped: HashSet::new(),
            unequipped: HashSet::new(),
        }
    }

    fn unequip(self, entity: Entity, item: Item) -> Self {
        let unequipped = {
            let mut uneq_cloned = self.unequipped.clone();
            uneq_cloned.insert((entity, item));
            uneq_cloned
        };
        EquipChanges {
            init_equipped: self.init_equipped,
            equipped: self.equipped,
            unequipped,
        }
    }

    fn equip(self, entity: Entity, item: Item) -> Self {
        let unequipped = {
            let mut uneq_cloned = self.unequipped.clone();
            uneq_cloned.remove(&(entity, item.clone()));
            uneq_cloned
        };
        let equip_entry = (entity, item);
        let equipped = if !self.init_equipped.contains(&equip_entry) {
            let mut eq_cloned = self.equipped.clone();
            eq_cloned.insert(equip_entry);
            eq_cloned
        } else {
            self.equipped
        };
        EquipChanges {
            init_equipped: self.init_equipped,
            equipped,
            unequipped,
        }
    }

    fn bonus_changes(&self) -> EquipBonusChanges {
        let defense = self
            .equipped
            .iter()
            .filter_map(|(_, item)| item.equip_opt().map(|eq| eq.defense_bonus()))
            .sum::<i16>()
            - self
                .unequipped
                .iter()
                .filter_map(|(_, item)| item.equip_opt().map(|eq| eq.defense_bonus()))
                .sum::<i16>();
        let power = self
            .equipped
            .iter()
            .filter_map(|(_, item)| item.equip_opt().map(|eq| eq.power_bonus()))
            .sum::<i16>()
            - self
                .unequipped
                .iter()
                .filter_map(|(_, item)| item.equip_opt().map(|eq| eq.power_bonus()))
                .sum::<i16>();

        EquipBonusChanges { defense, power }
    }
}

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    type SystemData = ItemUseSystemData<'a>;

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
                    targets.first().iter().for_each(|target| {
                        let player_equip = get_equipped_items(&entities, &items, &equipped, player_entity);
                        let equipped_items: HashSet<(Entity, Item)> = player_equip.iter().map(|kv| (kv.1.1, Item::Equippable(kv.1.0.clone()))).collect();
                        let new_equip = Equipped::new(**target, &player_equip, &equip.allowed_slots);
                        let equip_changes = equip_slot(
                            (&entities, &mut backpack, &items, &mut equipped, &names)
                            , new_equip, useitem.item,  EquipChanges::new( equipped_items)
                        );
                        let ecs_data = EcsActionMsgData::new(&entities, &players, &names);
                        if let Some(equip_msg) = equip_message(ecs_data, equip_changes, player_entity) {
                            log.entries.push(equip_msg);
                        }
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
                        if used && player_name.name == PLAYER_NAME {
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

fn equip_slot<I: Join + Copy>(
    equip_data: EquipData<I>,
    new_equip: Equipped,
    new_equip_ent: Entity,
    equip_changes: EquipChanges,
) -> EquipChanges
where
    I::Type: IsItem,
{
    let (entities, backpack, items, equipped_items, names) = equip_data;

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

    let unequipped: HashSet<(Entity, Item)> = HashSet::from_iter(
        to_unequip
            .clone()
            .into_iter()
            .map(|(item_ent, _, itm)| {
                equipped_items.remove(item_ent);
                backpack
                    .insert(
                        item_ent,
                        InBackpack {
                            owner: new_equip.owner,
                        },
                    )
                    .unwrap();
                (item_ent, itm)
            })
            .collect_vec(),
    );
    let equip_changes = unequipped.iter().fold(equip_changes, |acc, (ent, itm)| {
        acc.unequip(*ent, itm.clone())
    });
    backpack.remove(new_equip_ent);

    equipped_items
        .insert(new_equip_ent, new_equip.clone())
        .unwrap();
    let newly_equipped = (entities, items)
        .join()
        .filter(|(ent, _itm)| new_equip_ent == *ent)
        .map(|(ent, item)| (ent, item.from()))
        .collect::<HashSet<(Entity, Item)>>();
    let equip_changes = newly_equipped
        .iter()
        .fold(equip_changes, |acc, (ent, itm)| {
            acc.equip(*ent, itm.clone())
        });

    // If unequipped was in the main hand, and it and the newly equipped are both 1-handed items,
    // we perform a second recursive call to equip unequipped item in the off-hand slot:
    match unequipped.iter().next() {
        None => equip_changes,
        Some((uneq_ent, uneq_item)) => {
            let was_in_mh = to_unequip
                .iter()
                .any(|uneq| uneq.1.slot == EquipSlot::MainHand);
            let old_eq_can_oh = || -> bool {
                uneq_item
                    .equip_opt()
                    .map_or_else(|| false, |eq| eq.is_oh_capable())
            };
            let new_eq_is_1h = || -> bool {
                let new_item_opt: Option<Item> = (entities, items)
                    .join()
                    .filter(|(ent, _itm)| new_equip_ent == *ent)
                    .map(|(_ent, itm)| itm.from())
                    .next();
                new_item_opt
                    .map(|new_item| !new_item.equip_opt().map_or_else(|| false, |eq| eq.is_2h()))
                    .unwrap_or(false)
            };
            if was_in_mh && old_eq_can_oh() && new_eq_is_1h() {
                equip_slot(
                    (entities, backpack, items, equipped_items, names),
                    Equipped {
                        owner: new_equip.owner,
                        slot: EquipSlot::OffHand,
                        slot_extra: None,
                    },
                    *uneq_ent,
                    equip_changes,
                )
            } else {
                equip_changes
            }
        }
    }
}

fn calculate_unequip<I: Join>(
    entities: &Read<EntitiesRes>,
    items: I, // FIXME: need this to be a reference, somehow
    equipped: &WriteStorage<Equipped>,
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

fn equip_message(
    ecs_data: EcsActionMsgData,
    equip_changes: EquipChanges,
    owner: Entity,
) -> Option<String> {
    let bonus_changes = equip_changes.clone().bonus_changes();
    let equip_names = equip_changes
        .equipped
        .into_iter()
        .map(|ei| ecs_data.names.get(ei.0).unwrap().name.clone())
        .collect::<Vec<String>>();
    let unequip_names = equip_changes
        .unequipped
        .into_iter()
        .map(|ei| ecs_data.names.get(ei.0).unwrap().name.clone())
        .collect::<Vec<String>>();
    let fmt_unequip_names = fmt_list(&unequip_names);
    let fmt_equip_names = fmt_list(&equip_names);
    match (equip_names.is_empty(), unequip_names.is_empty()) {
        (true, true) => None,
        (true, false) => Some(entity_action_msg_no_ecs!(
            ecs_data,
            "<SUBJ> {} {fmt_unequip_names} ({bonus_changes}).",
            owner,
            "unequip"
        )),
        (false, true) => Some(entity_action_msg_no_ecs!(
            ecs_data,
            "<SUBJ> {} {fmt_equip_names} ({bonus_changes}).",
            owner,
            "equip"
        )),
        (false, false) => Some(entity_action_msg_no_ecs!(
            ecs_data,
            "<SUBJ> {} {fmt_unequip_names} and {} {fmt_equip_names} ({bonus_changes}).",
            owner,
            "unequip",
            "equip"
        )),
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

pub struct ItemRemoveSystem {}

impl<'a> System<'a> for ItemRemoveSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, EventWantsToRemoveItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Item>,
        WriteStorage<'a, InBackpack>,
        WriteStorage<'a, Equipped>,
        WriteExpect<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut wants_remove,
            names,
            players,
            items,
            mut backpack,
            mut equipped,
            mut log,
        ) = data;
        (&entities, &wants_remove)
            .join()
            .for_each(|(entity, to_remove)| {
                equipped.remove(to_remove.item);
                backpack
                    .insert(to_remove.item, InBackpack { owner: entity })
                    .unwrap_or_else(|er| panic!("Unable to unequip item fully: {}", er));
                let item_name = names
                    .get(to_remove.item)
                    .map(|n| n.name.clone())
                    .unwrap_or_else(|| format!("item {}", to_remove.item.id()));
                let ecs_data = EcsActionMsgData::new(&entities, &players, &names);
                let unequipped_item = items.get(to_remove.item).expect(&format!(
                    "No item found for entity {}!",
                    to_remove.item.id()
                ));
                let bonus_changes = EquipBonusChanges::unequip(unequipped_item);
                log.entries.push(entity_action_msg_no_ecs!(
                    ecs_data,
                    "<SUBJ> {} {item_name} ({bonus_changes}).",
                    entity,
                    "unequip"
                ));
            });
        wants_remove.clear();
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        gui::backpack_items,
        init_state,
        player::{get_item, get_player_pos_unwrap, get_player_unwrap},
        spawner, State,
    };

    use super::*;

    fn use_first_backpack_item(gs: &mut State, player_entity: Entity) {
        let bpack_items = backpack_items(&gs.ecs, player_entity);
        {
            let mut intent = gs.ecs.write_storage::<EventWantsToUseItem>();
            intent
                .insert(
                    player_entity,
                    EventWantsToUseItem {
                        item: bpack_items[0].0,
                        target: None,
                    },
                )
                .unwrap();
        }
        gs.run_systems();
    }

    #[test]
    fn equip_item_removes_from_item_from_bag() {
        let (mut gs, _) = init_state(true, None);
        let player_entity = get_player_unwrap(&gs.ecs, PLAYER_NAME);
        let player_posn = get_player_pos_unwrap(&gs.ecs, PLAYER_NAME);

        spawner::dagger_at_level(5, &mut gs.ecs, player_posn);
        get_item(&mut gs.ecs); // pickup an item
        gs.run_systems();

        spawner::dagger_at_level(5, &mut gs.ecs, player_posn);
        get_item(&mut gs.ecs); // pickup an item
        gs.run_systems();

        let bpack_items = backpack_items(&gs.ecs, player_entity);

        assert_eq!(bpack_items.len(), 2);

        use_first_backpack_item(&mut gs, player_entity);

        gs.run_systems();
        let bpack_items = backpack_items(&gs.ecs, player_entity);
        assert_eq!(bpack_items.len(), 1);
    }

    #[test]
    fn main_hand_shifts_to_offhand() {
        let (mut gs, _) = init_state(true, None);
        let player_entity = get_player_unwrap(&gs.ecs, PLAYER_NAME);
        let player_posn = get_player_pos_unwrap(&gs.ecs, PLAYER_NAME);

        let _dagger1 = spawner::dagger_at_level(1, &mut gs.ecs, player_posn);
        get_item(&mut gs.ecs); // pickup an item
        gs.run_systems();
        use_first_backpack_item(&mut gs, player_entity);

        let shield = spawner::shield_at_level(1, &mut gs.ecs, player_posn);
        get_item(&mut gs.ecs); // pickup an item
        gs.run_systems();
        use_first_backpack_item(&mut gs, player_entity);

        let dagger2 = spawner::dagger_at_level(1, &mut gs.ecs, player_posn);
        get_item(&mut gs.ecs); // pickup an item
        gs.run_systems();

        let bpack_items = backpack_items(&gs.ecs, player_entity);
        assert_eq!(bpack_items.len(), 1);
        assert_eq!(bpack_items[0].0, dagger2);

        use_first_backpack_item(&mut gs, player_entity);
        let bpack_items = backpack_items(&gs.ecs, player_entity);
        assert_eq!(bpack_items.len(), 1);
        assert_eq!(bpack_items[0].0, shield);
    }

    #[test]
    fn equip_item_gives_nonempty_item_name() {
        let (mut gs, _) = init_state(true, None);
        let player_entity = get_player_unwrap(&gs.ecs, PLAYER_NAME);
        let player_posn = get_player_pos_unwrap(&gs.ecs, PLAYER_NAME);

        let shield = spawner::shield_at_level(1, &mut gs.ecs, player_posn);
        get_item(&mut gs.ecs); // pickup an item
        gs.run_systems();
        use_first_backpack_item(&mut gs, player_entity);
        let bpack_items = backpack_items(&gs.ecs, player_entity);
        assert_eq!(bpack_items.len(), 0);

        let names = gs.ecs.read_storage::<Name>();
        let shield_name = names.get(shield).unwrap();
        let expected_pickup_msg = format!("You pick up the {}.", shield_name.name);
        let expected_equip_submsg = format!(" equip {}", shield_name.name);
        let log = gs.ecs.fetch::<GameLog>();
        log.entries.iter().for_each(|e| println!("{}", e));
        assert!(log.entries.iter().any(|e| e == &expected_pickup_msg));
        assert!(log
            .entries
            .iter()
            .any(|e| e.contains(&expected_equip_submsg)));
    }
}
