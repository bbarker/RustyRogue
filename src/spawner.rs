use bracket_lib::{
    random::RandomNumberGenerator,
    terminal::{FontCharType, BLACK, BLUE, CYAN, GREEN, ORANGE, PINK, RED, RGB, YELLOW},
};
use itertools::Itertools;
use specs::prelude::*;

use crate::{
    components::{
        AreaOfEffect, BlocksTile, CombatStats, Confusion, Consumable, InflictsDamage, Item,
        Monster, Name, Player, Position, ProvidesHealing, Ranged, Renderable, Viewshed,
    },
    map::Map,
    rect::Rect,
    State,
};

const MAX_ROOM_MONSTERS: u16 = 3; // TODO: Should be 4
const MAX_ROOM_ITEMS: u16 = 2;

type SimpleSpawner = fn(&mut World, Position) -> Entity;
pub fn player(gs: &mut State, position: Position) -> Entity {
    gs.ecs
        .create_entity()
        .with(position)
        .with(Renderable {
            glyph: bracket_lib::prelude::to_cp437('@'),
            fg: RGB::named(bracket_lib::prelude::YELLOW),
            bg: RGB::named(bracket_lib::prelude::BLACK),
            render_order: 2,
        })
        .with(Player {})
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(Name {
            name: "Player".to_string(),
        })
        .with(CombatStats {
            max_hp: 100, // TODO: Should be 30
            hp: 100,     // Should be 30
            defense: 2,
            power: 5,
        })
        .build()
}

pub fn health_potion(ecs: &mut World, position: Position) -> Entity {
    ecs.create_entity()
        .with(position)
        .with(Renderable {
            glyph: bracket_lib::prelude::to_cp437('¡'),
            fg: RGB::named(bracket_lib::prelude::RED),
            bg: RGB::named(bracket_lib::prelude::BLACK),
            render_order: 0,
        })
        .with(Name {
            name: "Health Potion".to_string(),
        })
        .with(Item {})
        .with(Consumable {})
        .with(ProvidesHealing { heal_amount: 8 })
        .build()
}

pub fn fireball_scroll(ecs: &mut World, position: Position) -> Entity {
    ecs.create_entity()
        .with(position)
        .with(Renderable {
            glyph: bracket_lib::prelude::to_cp437(')'),
            fg: RGB::named(ORANGE),
            bg: RGB::named(BLACK),
            render_order: 0,
        })
        .with(Name {
            name: "Fireball Scroll".to_string(),
        })
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(InflictsDamage { damage: 20 })
        .with(AreaOfEffect { radius: 3 })
        .build()
}

pub fn magic_missile_scroll(ecs: &mut World, position: Position) -> Entity {
    ecs.create_entity()
        .with(position)
        .with(Renderable {
            glyph: bracket_lib::prelude::to_cp437(')'),
            fg: RGB::named(CYAN),
            bg: RGB::named(BLACK),
            render_order: 0,
        })
        .with(Name {
            name: "Magic Missile Scroll".to_string(),
        })
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(InflictsDamage { damage: 8 })
        .build()
}

pub fn confusion_scroll(ecs: &mut World, position: Position) -> Entity {
    let rand_turns = {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        rng.range(1, 7)
    };
    let steps: Vec<(i8, i8)> = {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        (0..rand_turns)
            .map(|_| (rng.range(-1, 2), rng.range(-1, 2)))
            .collect()
    };
    ecs.create_entity()
        .with(position)
        .with(Renderable {
            glyph: bracket_lib::prelude::to_cp437(')'),
            fg: RGB::named(PINK),
            bg: RGB::named(BLACK),
            render_order: 0,
        })
        .with(Name {
            name: "Confusion Scroll".to_string(),
        })
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(Confusion {
            step_sequence: steps,
        })
        .build()
}

const WEIGHTED_ITEM_SPAWNERS: [(SimpleSpawner, u16); 4] = [
    (health_potion, 30),
    (fireball_scroll, 30),
    (magic_missile_scroll, 40),
    (confusion_scroll, 300),
];

const ITEM_WEIGHT_CUMULATIVE: [(SimpleSpawner, u16);
    WEIGHTED_ITEM_SPAWNERS.len()] = {
    // Note: const iterators don't exist at the moment, so we have to do this
    let mut ii = 0;
    let mut sum = 0;
    let mut cumulative = WEIGHTED_ITEM_SPAWNERS;

    while ii < WEIGHTED_ITEM_SPAWNERS.len() {
        sum += WEIGHTED_ITEM_SPAWNERS[ii].1;
        cumulative[ii].1 = sum;
        ii += 1;
    }
    cumulative
};

const ITEM_WEIGHT_SUM: u16 = ITEM_WEIGHT_CUMULATIVE[ITEM_WEIGHT_CUMULATIVE.len() - 1].1;

pub fn random_item(ecs: &mut World, position: Position) -> Entity {
    let roll = {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        rng.range(0, ITEM_WEIGHT_SUM)
    };

    // TODO: if we get a lot of items, may want to consider a search
    ITEM_WEIGHT_CUMULATIVE
        .iter()
        .find(|(_, weight)| roll < *weight)
        .unwrap()
        .0(ecs, position)
}

pub fn random_monster(ecs: &mut World, position: Position) -> Entity {
    let pos_ix = {
        let map = ecs.read_resource::<Map>();
        map.pos_idx(position)
    };
    {
        let mut map = ecs.write_resource::<Map>();
        map.blocked[pos_ix] = true;
    }

    let roll = {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        rng.range(0, 4) // TODO: refactor monsters as an ADT?
                        // Some possibilities listed here: https://stackoverflow.com/questions/41637978/how-to-get-the-number-of-elements-variants-in-an-enum-as-a-constant-value
    };
    match roll {
        0 => goblin(ecs, position),
        1 => orc(ecs, position),
        2 => tarrasque(ecs, position),
        _ => troll(ecs, position),
    }
}

fn goblin(ecs: &mut World, position: Position) -> Entity {
    monster(
        ecs,
        position,
        bracket_lib::prelude::to_cp437('g'),
        "Goblin",
        RGB::named(RED),
    )
}

fn orc(ecs: &mut World, position: Position) -> Entity {
    monster(
        ecs,
        position,
        bracket_lib::prelude::to_cp437('o'),
        "Orc",
        RGB::named(GREEN),
    )
}

fn tarrasque(ecs: &mut World, position: Position) -> Entity {
    monster(
        ecs,
        position,
        bracket_lib::prelude::to_cp437('T'),
        "Tarrasque",
        RGB::named(YELLOW),
    )
}

fn troll(ecs: &mut World, position: Position) -> Entity {
    monster(
        ecs,
        position,
        bracket_lib::prelude::to_cp437('t'),
        "Troll",
        RGB::named(BLUE),
    )
}

fn monster<S: ToString>(
    ecs: &mut World,
    posn: Position,
    glyph: FontCharType,
    name: S,
    fg: RGB,
) -> Entity {
    ecs.create_entity()
        .with(Position {
            xx: posn.xx,
            yy: posn.yy,
        })
        .with(Renderable {
            glyph,
            fg,
            bg: RGB::named(bracket_lib::prelude::BLACK),
            render_order: 1,
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(Monster {})
        .with(Name {
            name: name.to_string(),
        })
        .with(CombatStats {
            max_hp: 16,
            hp: 16,
            defense: 1,
            power: 4,
        })
        .with(BlocksTile {})
        .build()
}

pub fn spawn_room(ecs: &mut World, room: &Rect) -> Vec<Entity> {
    let (num_monsters, num_items) = {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let monsters = rng.range(0, MAX_ROOM_MONSTERS + 1);
        let items = rng.range(0, MAX_ROOM_ITEMS + 1);
        (monsters, items)
    };
    vec![
        spawn_in_room(ecs, room, num_monsters, random_monster),
        spawn_in_room(ecs, room, num_items, random_item),
    ]
    .concat()
}

/// Fills a room with monsters and items
pub fn spawn_in_room(
    ecs: &mut World,
    room: &Rect,
    num_indices: u16,
    spawn_fn: fn(&mut World, Position) -> Entity,
) -> Vec<Entity> {
    let fill_indices: Vec<usize> = {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let map = &mut ecs.fetch_mut::<Map>();

        let mut free_room_indices: Vec<usize> = (room.x1..=room.x2)
            .cartesian_product(room.y1..=room.y2)
            .map(|(x, y)| map.xy_idx(x, y))
            .filter(|ix| !map.blocked[*ix] && map.tile_content[*ix].is_empty())
            .collect();

        (0..num_indices)
            .map(|_| {
                let idx = rng.range(0, free_room_indices.len());
                free_room_indices.remove(idx)
            })
            .collect()
    };

    fill_indices
        .iter()
        .map(|ix| {
            let pos = {
                let map = &mut ecs.fetch_mut::<Map>();
                map.idx_to_pos(*ix)
            };
            let entity = spawn_fn(
                ecs,
                Position {
                    xx: pos.xx,
                    yy: pos.yy,
                },
            );
            let map = &mut ecs.fetch_mut::<Map>();
            map.tile_content[*ix].push(entity);
            // TODO: ^ make our own entity wrapper to avoid having to remember to do this
            entity
        })
        .collect()
}
