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

#[derive(Component)]
struct LeftMover {}

struct State {
    ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        let mut lw = LeftWalker {};
        lw.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        player_input(self, ctx);

        self.run_systems();

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        (&positions, &renderables).join().for_each(|(pos, render)| {
            ctx.set(pos.xx, pos.yy, render.fg, render.bg, render.glyph);
        })
    }
}

#[derive(Component, Debug)]
struct Player {}

struct LeftWalker {}

impl<'a> System<'a> for LeftWalker {
    type SystemData = (ReadStorage<'a, LeftMover>, WriteStorage<'a, Position>);

    fn run(&mut self, (lefty, mut pos): Self::SystemData) {
        (&lefty, &mut pos).join().for_each(|(_lefty, pos)| {
            pos.xx -= 1;
            if pos.xx < 0 {
                pos.xx = 79; // TODO: reference from config/state
            }
        })
    }
}

fn main() {
    use bracket_lib::prelude::BTermBuilder;
    let context = BTermBuilder::simple80x50()
        .with_title("Rusty Rogue")
        .build()
        .unwrap(); // TODO: better error handling from software tools

    let mut gs = State { ecs: World::new() };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<LeftMover>();
    gs.ecs.register::<Player>();

    // Note we aren't storing the entity, just telling the World it is there.
    // FIXME: unit discard warning?
    build_entity_player(&mut gs);
    build_entities_happy_folk(&mut gs);

    bracket_lib::prelude::main_loop(context, gs).unwrap()
}

fn build_entity_player(gs: &mut State) -> Entity {
    gs.ecs
        .create_entity()
        .with(Position { xx: 40, yy: 25 })
        .with(Renderable {
            glyph: bracket_lib::prelude::to_cp437('@'),
            fg: RGB::named(bracket_lib::prelude::YELLOW),
            bg: RGB::named(bracket_lib::prelude::BLACK),
        })
        .with(Player {})
        .build()
}

fn build_entities_happy_folk(gs: &mut State) -> Vec<Entity> {
    (0..10)
        .map(|ii| {
            gs.ecs
                .create_entity()
                .with(Position { xx: 7 * ii, yy: 20 })
                .with(Renderable {
                    glyph: bracket_lib::prelude::to_cp437('☺'),
                    fg: RGB::named(bracket_lib::prelude::RED),
                    bg: RGB::named(bracket_lib::prelude::BLACK),
                })
                .with(LeftMover {})
                .build()
        })
        .collect()
}

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    (&mut players, &mut positions)
        .join()
        .for_each(|(_player, pos)| {
            pos.xx = min(79, max(0, pos.xx + delta_x));
            pos.yy = min(49, max(0, pos.yy + delta_y));
        })
}

fn player_input(gs: &mut State, ctx: &mut BTerm) {
    match ctx.key {
        None => {}
        Some(key) => match key {
            // Player Movement
            VirtualKeyCode::Left => try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::Right => try_move_player(1, 0, &mut gs.ecs),
            VirtualKeyCode::Up => try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::Down => try_move_player(0, 1, &mut gs.ecs),
            _ => {}
        },
    }
}
