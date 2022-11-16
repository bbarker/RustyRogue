#![feature(once_cell)]

use bracket_lib::prelude::{BTerm, FontCharType, GameState, VirtualKeyCode, RGB};
use bracket_lib::terminal::BTermBuilder;
use specs::prelude::*;
use specs_derive::Component;
use std::cell::OnceCell;
use std::cmp::{max, min};
use std::convert::TryInto;
use std::sync::Arc;

#[derive(Component)]
struct Position {
    xx: u32,
    yy: u32,
}

#[derive(Component)]
struct Renderable {
    glyph: FontCharType,
    fg: RGB,
    bg: RGB,
}

#[derive(Component)]
struct LeftMover {}

struct State<'a> {
    ecs: World,
    context: &'a BTerm,
    display: DisplayState,
}

const STATE: OnceCell<State> = OnceCell::new();

fn init_state<'a>(ctxt: &'a BTerm) -> State {
    let ctxt = BTermBuilder::simple80x50()
        .with_title("Rusty Rogue")
        .build()
        .unwrap(); // TODO: better error handling from software tools
    let termWidth = ctxt.get_char_size().0;
    let termHeight = ctxt.get_char_size().1;
    State {
        ecs: World::new(),
        context: &ctxt,
        display: DisplayState {
            width: termWidth,
            height: termHeight,
        },
    }
}
impl State {
    fn run_systems(&mut self) {
        let mut lw = LeftWalker {};
        lw.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

struct DisplayState {
    width: u32,
    height: u32,
}

impl DisplayState {
    fn width_i32(&self) -> i32 {
        self.width as i32
    }
    fn height_i32(&self) -> i32 {
        self.height as i32
    }
}
impl GameState for State {
    // TODO: read about lifetimes ^^
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

// TODO: add a ref to global state?
struct LeftWalker {}

impl<'a> System<'a> for LeftWalker {
    type SystemData = (ReadStorage<'a, LeftMover>, WriteStorage<'a, Position>);

    fn run(&mut self, (lefty, mut pos): Self::SystemData) {
        (&lefty, &mut pos).join().for_each(|(_lefty, pos)| {
            if pos.xx == 0 {
                pos.xx = STATE.get_or_init(init_state).display.width - 1
            } else {
                pos.xx -= 1
            }
        })
    }
}

#[derive(PartialEq, Clone, Copy)]
enum TileType {
    Wall,
    Floor,
}

fn main() {
    use bracket_lib::prelude::BTermBuilder;

    let mut gs = init_state();

    STATE.set(gs);

    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<LeftMover>();
    gs.ecs.register::<Player>();

    // Note we aren't storing the entity, just telling the World it is there.
    // FIXME: unit discard warning?
    build_entity_player(&mut gs);
    build_entities_happy_folk(&mut gs);

    bracket_lib::prelude::main_loop(*(gs.context), gs).unwrap()
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

fn try_move_player(delta_x: i32, delta_y: i32, gs: &mut State) {
    let mut positions = gs.ecs.write_storage::<Position>();
    let mut players = gs.ecs.write_storage::<Player>();
    (&mut players, &mut positions)
        .join()
        .for_each(|(_player, pos)| {
            let xx_i32 = i32::try_from(pos.xx).unwrap();
            let yy_i32 = i32::try_from(pos.yy).unwrap();
            pos.xx =
                u32::try_from(min(gs.display.width_i32() - 1, max(0, xx_i32 + delta_x))).unwrap();
            pos.yy =
                u32::try_from(min(gs.display.height_i32() - 1, max(0, yy_i32 + delta_y))).unwrap();
        })
}

fn player_input(gs: &mut State, ctx: &mut BTerm) {
    match ctx.key {
        None => {}
        Some(key) => match key {
            // Player Movement
            VirtualKeyCode::Left => try_move_player(-1, 0, gs),
            VirtualKeyCode::Right => try_move_player(1, 0, gs),
            VirtualKeyCode::Up => try_move_player(0, -1, gs),
            VirtualKeyCode::Down => try_move_player(0, 1, gs),
            _ => {}
        },
    }
}
