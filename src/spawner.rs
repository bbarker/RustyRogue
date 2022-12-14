use bracket_lib::{
    random::RandomNumberGenerator,
    terminal::{FontCharType, BLUE, GREEN, RED, RGB, YELLOW},
};
use specs::prelude::*;

use crate::{
    components::{BlocksTile, CombatStats, Monster, Name, Player, Position, Renderable, Viewshed},
    State,
};

pub fn player(gs: &mut State, position: Position) -> Entity {
    gs.ecs
        .create_entity()
        .with(position)
        .with(Renderable {
            glyph: bracket_lib::prelude::to_cp437('@'),
            fg: RGB::named(bracket_lib::prelude::YELLOW),
            bg: RGB::named(bracket_lib::prelude::BLACK),
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

pub fn random_monster(ecs: &mut World, position: Position) -> Entity {
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
