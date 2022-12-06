use specs::{
    shred::{Fetch, FetchMut},
    storage::MaskedStorage,
    world::EntitiesRes,
    Entity, Join, Read, Storage,
};

use crate::components::{Player, Position};

pub fn get_player_entities_with_pos(
    entities: &Read<EntitiesRes>,
    players: &Storage<Player, Fetch<MaskedStorage<Player>>>,
    positions: &Storage<Position, FetchMut<MaskedStorage<Position>>>,
) -> Vec<(Entity, Position)> {
    (entities, players, positions)
        .join()
        .map(|(ent, _, pos)| (ent, *pos))
        .collect::<Vec<_>>()
}
