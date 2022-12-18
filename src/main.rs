#![feature(const_cmp)]

use bracket_lib::{
    prelude::{BTerm, GameState},
    random::RandomNumberGenerator,
};
use inventory_system::ItemCollectionSystem;
use map_indexing_system::MapIndexingSystem;
use spawner::spawn_room;
use specs::prelude::*;

mod components;
mod damage_system;
mod display_state;
mod gamelog;
mod gui;
mod inventory_system;
mod map;
mod map_indexing_system;
mod melee_combat_system;
mod monster_ai_system;
mod rect;
mod spawner;
mod visibility_system;

mod player;

use components::*;
use damage_system::*;
use display_state::*;
use gui::*;
use map::*;
use melee_combat_system::*;
use monster_ai_system::*;
use player::*;
use visibility_system::VisibilitySystem;

pub type PsnU = u16;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
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
        let mut pickup = ItemCollectionSystem {};
        pickup.run_now(&self.ecs);

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
            RunState::ShowInventory => {
                if gui::show_inventory(self, ctx) == gui::ItemMenuResult::Cancel {
                    newrunstate = RunState::AwaitingInput;
                }
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
    gs.ecs.register::<EventWantsToPickupItem>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<Item>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Position>();
    gs.ecs.register::<Potion>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Viewshed>();

    gs.ecs.insert(RunState::PreRun);
    gs.ecs.insert(gamelog::GameLog {
        entries: vec!["Welcome to Rusty Rogue!".to_string()],
    });
    gs.ecs.insert(RandomNumberGenerator::new());

    let map = new_map_rooms_and_corridors(&gs);

    let player_posn = map.rooms.first().unwrap().center();
    gs.ecs.insert(map);
    populate_rooms(&mut gs.ecs);

    // FIXME: unit discard warning?
    spawner::player(&mut gs, player_posn);
    gs.ecs.insert(player_posn);

    bracket_lib::prelude::main_loop(context, gs).unwrap()
}

fn populate_rooms(ecs: &mut World) -> Vec<Entity> {
    let rooms = {
        let map = ecs.read_resource::<Map>();
        map.rooms.clone()
    };
    rooms
        .iter()
        .skip(1)
        .flat_map(|room| spawn_room(ecs, room))
        .collect()
}
