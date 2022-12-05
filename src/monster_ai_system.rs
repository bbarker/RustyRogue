use crate::{
    components::{EventWantsToMelee, Name, Player, PlayerPosition, Position, Positionable},
    map::Map,
};

use super::{Monster, Viewshed};
use bracket_lib::{prelude::console, terminal::DistanceAlg};
use specs::prelude::*;

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, Map>,
        ReadStorage<'a, Player>,
        ReadExpect<'a, PlayerPosition>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, EventWantsToMelee>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            map,
            players,
            player_pos,
            mut viewsheds,
            monsters,
            names,
            mut positions,
            mut wants_to_melee,
        ) = data;

        let player_entities = (&entities, &players)
            .join()
            .map(|(ent, _)| ent)
            .collect::<Vec<_>>();

        (&entities, &mut viewsheds, &monsters, &names, &mut positions)
            .join()
            .for_each(|(entity, viewshed, _monster, name, pos)| {
                player_entities.iter().for_each(|player_entity| {
                    if viewshed.visible_tiles.contains(&player_pos.pos().into()) {
                        let distance = DistanceAlg::Pythagoras
                            .distance2d((*pos).into(), player_pos.pos().into());
                        if distance < 1.5 {
                            wants_to_melee
                                .insert(
                                    entity,
                                    EventWantsToMelee {
                                        target: *player_entity,
                                    },
                                )
                                .unwrap_or_else(|_| {
                                    panic!("Unable to insert attack on player from {}", name.name,)
                                });
                            console::log(format!("{} shouts insults", name.name));
                        } else {
                            let path = bracket_lib::prelude::a_star_search(
                                pos.idx(map.width_psnu),
                                player_pos.pos().idx(map.width_psnu),
                                &*map,
                            );
                            if path.success && path.steps.len() > 1 {
                                *pos = map.idx_to_xy(path.steps[1]);
                                viewshed.dirty = true;
                            }
                        }
                    }
                })
            });
    }
}
