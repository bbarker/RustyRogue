use bracket_lib::terminal::RGB;
use specs::prelude::*;

use crate::{
    components::{CombatStats, Name, Player, Position, Renderable, Viewshed},
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
