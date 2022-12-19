use specs::prelude::*;

use crate::{
    components::{CombatStats, EventWantsToDrinkPotion, Potion},
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

pub struct PotionUseSystem {}

impl<'a> System<'a> for PotionUseSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Player>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, EventWantsToDrinkPotion>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Potion>,
        WriteStorage<'a, CombatStats>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, players, mut log, mut wants_drink, names, potions, mut combat_stats) = data;

        (
            &entities,
            &players,
            &mut wants_drink,
            &mut combat_stats,
            &names,
        )
            .join()
            .for_each(|(_player_entity, _player, drink, stats, player_name)| {
                let potion = potions.get(drink.potion);
                match potion {
                    None => {}
                    Some(potion) => {
                        stats.hp = u16::min(stats.max_hp, stats.hp + potion.heal_amount);
                        if player_name.name == PLAYER_NAME {
                            log.entries.push(format!(
                                "You drink the {}, healing {} hp.",
                                names.get(drink.potion).unwrap().name,
                                potion.heal_amount
                            ));
                            entities.delete(drink.potion).unwrap_or_else(|_| {
                                panic!("Delete potion failed for player {}", player_name.name)
                            });
                        }
                    }
                }
            });
        wants_drink.clear();
    }
}
