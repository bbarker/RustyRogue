use bracket_lib::prelude::{BTerm, FontCharType, GameState, VirtualKeyCode, RGB};
use specs::prelude::*;
use specs_derive::Component;
use std::cmp::{max, min};

#[derive(Component)]
struct Position {
    xx: i32,
    yy: i32,
}

#[derive(Component)]
struct Renderable {
    glyph: FontCharType,
    fg: RGB,
    bg: RGB,
}

struct State {
    ecs: World,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        (&positions, &renderables).join().map(|(pos, render)| {
            ctx.set(pos.xx, pos.yy, render.fg, render.bg, render.glyph);
        });
    }
}

fn main() {
    let mut gs = State { ecs: World::new() };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();

    // Note we aren't storing the entity, just telling the World it is there.
    // FIXME: unit discard warning?
    buildEntityAt(gs);
    buildEntitiesHappyFolk(gs);
}

fn buildEntityAt(mut gs: State) -> Entity {
    gs.ecs
        .create_entity()
        .with(Position { xx: 40, yy: 25 })
        .with(Renderable {
            glyph: bracket_lib::prelude::to_cp437('@'),
            fg: RGB::named(bracket_lib::prelude::YELLOW),
            bg: RGB::named(bracket_lib::prelude::BLACK),
        })
        .build()
}

fn buildEntitiesHappyFolk(mut gs: State) -> Vec<Entity> {
    (0..10)
        .map(|ii| {
            gs.ecs
                .create_entity()
                .with(Position { xx: 7 * ii, yy: 20 })
                .with(Renderable {
                    glyph: bracket_lib::prelude::to_cp437('☺'),
                    fg: RGB::named(bracket_lib::prelude::BLACK),
                    bg: RGB::named(bracket_lib::prelude::BLACK),
                })
                .build()
        })
        .collect()
}
