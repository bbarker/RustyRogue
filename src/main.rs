use bracket_lib::prelude::{BTerm, GameState, VirtualKeyCode, RGB};
use specs::prelude::*;

pub mod components;
pub mod display_state;
pub mod map;
pub mod rect;

use components::*;
use display_state::*;
use map::*;

pub type PsnU = u16;

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

        let map = self.ecs.fetch::<Map>();
        draw_map(ctx, &map);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        (&positions, &renderables).join().for_each(|(pos, render)| {
            ctx.set(pos.xx, pos.yy, render.fg, render.bg, render.glyph);
        })
    }
}

struct LeftWalker<'a> {
    display: &'a DisplayState,
}

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

    let map = new_map_rooms_and_corridors(&gs.display);
    let player_posn = map.rooms.first().unwrap().center();

    gs.ecs.insert(map);

    // Note we aren't storing the entity, just telling the World it is there.
    // FIXME: unit discard warning?
    build_entity_player(&mut gs, player_posn);

    bracket_lib::prelude::main_loop(context, gs).unwrap()
}

fn build_entity_player(gs: &mut State, position: Position) -> Entity {
    gs.ecs
        .create_entity()
        .with(position)
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
            let map = gs.ecs.fetch::<Map>();
            let destination_ix = map.xy_idx(try_xx, try_yy);
            if map.map[destination_ix] != TileType::Wall {
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
            VirtualKeyCode::Left | VirtualKeyCode::Numpad4 | VirtualKeyCode::A => {
                try_move_player(-1, 0, gs)
            }
            VirtualKeyCode::Right | VirtualKeyCode::Numpad6 | VirtualKeyCode::D => {
                try_move_player(1, 0, gs)
            }
            VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::W => {
                try_move_player(0, -1, gs)
            }
            VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::S => {
                try_move_player(0, 1, gs)
            }
            _ => {}
        },
    }
}
