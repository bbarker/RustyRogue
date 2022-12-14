#![feature(const_cmp)]

use bracket_lib::{
    prelude::{BTerm, GameState, VirtualKeyCode},
    random::RandomNumberGenerator,
};
use map_indexing_system::MapIndexingSystem;
use specs::prelude::*;

pub mod components;
pub mod damage_system;
pub mod display_state;
pub mod gamelog;
pub mod gui;
pub mod map;
pub mod map_indexing_system;
pub mod melee_combat_system;
pub mod monster_ai_system;
pub mod rect;
pub mod spawner;
pub mod system_with_players;
pub mod visibility_system;

use components::*;
use damage_system::*;
use display_state::*;
use gui::*;
use map::*;
use melee_combat_system::*;
use monster_ai_system::*;
use visibility_system::VisibilitySystem;

pub type PsnU = u16;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
}

pub struct State {
    pub ecs: World,
    pub display: DisplayState,
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);
        let mut mob = MonsterAI {};
        mob.run_now(&self.ecs);
        let mut map_index = MapIndexingSystem {};
        map_index.run_now(&self.ecs);
        let mut melee = MeleeCombatSystem {};
        melee.run_now(&self.ecs);
        let mut damage = DamageSystem {};
        damage.run_now(&self.ecs);

        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        self.display = calc_display_state(ctx);

        ctx.cls();

        // This might be a good candidate for mutual TCO, someday
        // Also, look into bracket/resource handling patterns
        let mut newrunstate = {
            let runstate = self.ecs.fetch::<RunState>();
            *runstate
        };
        match newrunstate {
            RunState::PreRun => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => newrunstate = player_input(self, ctx),
            RunState::PlayerTurn => {
                self.run_systems();
                newrunstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
        }
        {
            let mut runstate = self.ecs.fetch_mut::<RunState>();
            *runstate = newrunstate;
        }

        delete_the_dead(&mut self.ecs);

        draw_map(&self.ecs, ctx);
        draw_ui(&self.ecs, ctx, &self.display);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();

        (&positions, &renderables)
            .join()
            .filter(|(pos, _)| map.visible_tiles[pos.idx(self.display.width)])
            .for_each(|(pos, render)| {
                ctx.set(pos.xx, pos.yy, render.fg, render.bg, render.glyph);
            });
    }
}

fn main() {
    use bracket_lib::prelude::BTermBuilder;
    let context = {
        let mut ctxt = BTermBuilder::simple80x50()
            .with_title("Rusty Rogue")
            .build()
            .unwrap(); // TODO: better error handling from software tools
        ctxt.with_post_scanlines(true);
        // ^ gives a retro "scanlines and screen burn" effect
        ctxt
    };
    let mut gs = State {
        ecs: World::new(),
        display: calc_display_state(&context),
    };
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<EventIncomingDamage>();
    gs.ecs.register::<EventWantsToMelee>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Viewshed>();

    gs.ecs.insert(RunState::PreRun);
    gs.ecs.insert(gamelog::GameLog {
        entries: vec!["Welcome to Rusty Rogue!".to_string()],
    });
    gs.ecs.insert(RandomNumberGenerator::new());

    let map = new_map_rooms_and_corridors(&gs);
    build_monsters(&mut gs.ecs, &map);

    let player_posn = map.rooms.first().unwrap().center();
    gs.ecs.insert(map);

    // FIXME: unit discard warning?
    spawner::player(&mut gs, player_posn);
    gs.ecs.insert(player_posn);

    bracket_lib::prelude::main_loop(context, gs).unwrap()
}

fn build_monsters(ecs: &mut World, map: &Map) -> Vec<Entity> {
    map.rooms
        .iter()
        .skip(1)
        .map(|room| {
            let posn = room.center();
            spawner::random_monster(ecs, posn)
        })
        .collect()
}

fn try_move_player(delta_x: i32, delta_y: i32, gs: &mut State) -> RunState {
    let entities = gs.ecs.entities();
    let mut log = gs.ecs.write_resource::<gamelog::GameLog>();

    let mut positions = gs.ecs.write_storage::<Position>();
    let mut players = gs.ecs.write_storage::<Player>();
    let mut viewsheds = gs.ecs.write_storage::<Viewshed>();
    let mut wants_to_melee = gs.ecs.write_storage::<EventWantsToMelee>();
    if let Some((entity, _player, pos, viewshed)) =
        (&entities, &mut players, &mut positions, &mut viewsheds)
            .join()
            .next()
    {
        let xx_i32 = i32::try_from(pos.xx).unwrap();
        let yy_i32 = i32::try_from(pos.yy).unwrap();
        let try_xx: PsnU = (xx_i32 + delta_x)
            .clamp(0, gs.display.width_i32() - 1)
            .try_into()
            .unwrap();
        let try_yy: PsnU = (yy_i32 + delta_y)
            .clamp(0, gs.display.height_i32() - PANEL_HEIGHT as i32 - 1)
            .try_into()
            .unwrap();
        let combat_stats = gs.ecs.read_storage::<CombatStats>();
        let map = gs.ecs.fetch::<Map>();
        let destination_ix = map.xy_idx(try_xx, try_yy);
        let combat = map.tile_content[destination_ix]
            .iter()
            .filter(|potential_target| potential_target.id() != entity.id())
            .any(|potential_target| {
                if let Some(_c_stats) = combat_stats.get(*potential_target) {
                    log.entries
                        .push("I stab thee with righteous fury!".to_string());
                    wants_to_melee
                        .insert(
                            entity,
                            EventWantsToMelee {
                                target: *potential_target,
                            },
                        )
                        .expect("Add target failed");
                    true
                } else {
                    false
                }
            });
        if !combat && !map.blocked[destination_ix] {
            pos.xx = try_xx;
            pos.yy = try_yy;
            viewshed.dirty = true;
            RunState::PlayerTurn
        } else if combat {
            RunState::PlayerTurn
        } else {
            RunState::AwaitingInput
        }
    } else {
        RunState::AwaitingInput
    }
}

fn player_input(gs: &mut State, ctx: &mut BTerm) -> RunState {
    match ctx.key {
        None => RunState::AwaitingInput,
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
            // Diagonals
            VirtualKeyCode::Numpad7 | VirtualKeyCode::Q => try_move_player(-1, -1, gs),
            VirtualKeyCode::Numpad9 | VirtualKeyCode::E => try_move_player(1, -1, gs),
            VirtualKeyCode::Numpad1 | VirtualKeyCode::Z => try_move_player(-1, 1, gs),
            VirtualKeyCode::Numpad3 | VirtualKeyCode::X => try_move_player(1, 1, gs),

            _ => RunState::AwaitingInput,
        },
    }
}
