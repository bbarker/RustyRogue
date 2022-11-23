use bracket_lib::prelude::{BTerm, FontCharType, GameState, VirtualKeyCode, RGB};
use specs::prelude::*;
use specs_derive::Component;

pub mod display_state;
pub mod maps;
pub mod rect;

use display_state::*;
use maps::*;

pub type PsnU = u16;

#[derive(Component)]
pub struct Position {
    xx: PsnU,
    yy: PsnU,
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
    display: DisplayState,
    ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        let mut lw = LeftWalker {
            display: &self.display,
        };
        lw.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        self.display = calc_display_state(ctx);

        ctx.cls();
        player_input(self, ctx);

        self.run_systems();

        let map = self.ecs.fetch::<Vec<TileType>>();
        draw_map(ctx, &self.display, &map);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        (&positions, &renderables).join().for_each(|(pos, render)| {
            ctx.set(pos.xx, pos.yy, render.fg, render.bg, render.glyph);
        })
    }
}

#[derive(Component, Debug)]
struct Player {}

struct LeftWalker<'a> {
    display: &'a DisplayState,
}

const INIT_PLAYER_POSITION: Position = Position { xx: 40, yy: 25 };

impl<'a> System<'a> for LeftWalker<'a> {
    type SystemData = (ReadStorage<'a, LeftMover>, WriteStorage<'a, Position>);

    fn run(&mut self, (lefty, mut pos): Self::SystemData) {
        (&lefty, &mut pos).join().for_each(|(_lefty, pos)| {
            if pos.xx == 0 {
                pos.xx = self.display.width - 1;
            } else {
                pos.xx -= 1
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

    let mut gs = State {
        ecs: World::new(),
        display: calc_display_state(&context),
    };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<LeftMover>();
    gs.ecs.register::<Player>();

    gs.ecs.insert(new_map_rooms_and_corridors(
        &gs.display,
        &INIT_PLAYER_POSITION,
    ));

    // Note we aren't storing the entity, just telling the World it is there.
    // FIXME: unit discard warning?
    build_entity_player(&mut gs);

    bracket_lib::prelude::main_loop(context, gs).unwrap()
}

fn build_entity_player(gs: &mut State) -> Entity {
    gs.ecs
        .create_entity()
        .with(INIT_PLAYER_POSITION)
        .with(Renderable {
            glyph: bracket_lib::prelude::to_cp437('@'),
            fg: RGB::named(bracket_lib::prelude::YELLOW),
            bg: RGB::named(bracket_lib::prelude::BLACK),
        })
        .with(Player {})
        .build()
}

/*
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
*/

fn try_move_player(delta_x: i32, delta_y: i32, gs: &mut State) {
    let mut positions = gs.ecs.write_storage::<Position>();
    let mut players = gs.ecs.write_storage::<Player>();
    (&mut players, &mut positions)
        .join()
        .for_each(|(_player, pos)| {
            let xx_i32 = i32::try_from(pos.xx).unwrap();
            let yy_i32 = i32::try_from(pos.yy).unwrap();
            let try_xx: PsnU = (xx_i32 + delta_x)
                .clamp(0, gs.display.width_i32() - 1)
                .try_into()
                .unwrap();
            let try_yy: PsnU = (yy_i32 + delta_y)
                .clamp(0, gs.display.height_i32() - 1)
                .try_into()
                .unwrap();
            let destination_ix = xy_idx(&gs.display, try_xx, try_yy);
            let map = gs.ecs.fetch::<Vec<TileType>>();
            if map[destination_ix] != TileType::Wall {
                pos.xx = try_xx;
                pos.yy = try_yy;
            }
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
