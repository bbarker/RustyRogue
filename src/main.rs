#![feature(const_trait_impl)]
#![feature(type_ascription)]

#[macro_use]
extern crate macro_attr;
#[macro_use]
extern crate enum_derive;

use bracket_lib::{
    prelude::{BTerm, GameState},
    random::RandomNumberGenerator,
    terminal::console,
};
use inventory_system::{ItemCollectionSystem, ItemDropSystem, ItemRemoveSystem, ItemUseSystem};
use itertools::Itertools;
use map_indexing_system::MapIndexingSystem;
use spawner::spawn_room;
use specs::saveload::SimpleMarker;
use specs::{prelude::*, saveload::SimpleMarkerAllocator};

mod components;
mod damage_system;
mod display_state;
mod equipment;
mod gamelog;
mod gui;
mod inventory_system;
mod map;
mod map_indexing_system;
mod melee_combat_system;
mod monster_ai_system;
mod player;
mod random_table;
mod rect;
mod saveload_system;
mod spawner;
mod util;
mod util_ecs;
mod visibility_system;

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
    ShowDropItem,
    ShowTargeting {
        range: u16,
        item: Entity,
    },
    ShowRemoveItem,
    MainMenu {
        menu_selection: gui::MainMenuSelection,
    },
    KeyBindingsMenu,
    SaveGame,
    NextLevel,
    GameOver,
}

pub struct State {
    pub ecs: World,
    pub display: DisplayState,
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<World; DisplayState = {:#?}>", self.display)
    }
}

impl State {
    fn run_systems(&mut self) {
        // These systems are required to be mutable by run_now, but
        // there seems to be nothing to mutate (so far)
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
        let mut potions = ItemUseSystem {};
        potions.run_now(&self.ecs);
        let mut drop_items = ItemDropSystem {};
        drop_items.run_now(&self.ecs);
        let mut item_remove = ItemRemoveSystem {};
        item_remove.run_now(&self.ecs);

        self.ecs.maintain();
    }

    fn entities_to_remove_on_level_change(&mut self) -> Vec<Entity> {
        let entities = self.ecs.entities();
        let player_entity = get_player_unwrap(&self.ecs, PLAYER_NAME);
        let player_items = owned_items(&self.ecs, player_entity)
            .iter()
            .map(|it| it.0)
            .collect_vec();
        entities
            .join()
            .filter(|en| *en != player_entity && !player_items.contains(en))
            .collect()
    }

    fn goto_next_level(&mut self) {
        // remove all entities except player and player items
        self.entities_to_remove_on_level_change()
            .iter()
            .for_each(|en| {
                self.ecs.delete_entity(*en).unwrap_or_else(|er| {
                    panic!("Failed to delete entity {:?}: {:?}", en, er);
                });
            });

        // Build a new map and place the player
        let (worldmap, current_depth) = {
            let mut worldmap_resource = self.ecs.write_resource::<Map>();
            let current_depth = worldmap_resource.depth;
            *worldmap_resource = new_map_rooms_and_corridors(self, current_depth + 1);
            (worldmap_resource.clone(), current_depth) // TODO: do we have to clone?
        };

        // Spawn monsters and items
        worldmap.rooms.iter().skip(1).for_each(|room| {
            spawn_room(&mut self.ecs, room, current_depth + 1);
        });

        // Place the player and update related resources; set viewshed to dirty
        let player_pos_new = worldmap.rooms[0].center();
        let mut positions = self.ecs.write_storage::<Position>();
        let player_entity = get_player_unwrap(&self.ecs, PLAYER_NAME);
        if let Some(player_pos) = positions.get_mut(player_entity) {
            *player_pos = player_pos_new;
        }
        let mut viewsheds = self.ecs.write_storage::<Viewshed>();
        if let Some(player_vs) = viewsheds.get_mut(player_entity) {
            player_vs.dirty = true;
        }

        // Notify the player and give them some health
        let mut gamelog = self.ecs.fetch_mut::<gamelog::GameLog>();
        gamelog
            .entries
            .push("You descend to the next level, and take a moment to heal.".to_string());
        let mut combat_stats = self.ecs.write_storage::<CombatStats>();
        if let Some(player_stats) = combat_stats.get_mut(player_entity) {
            player_stats.hp = u16::min(
                player_stats.max_hp,
                player_stats.hp + (2 * player_stats.max_hp) / 5,
            );
        }
    }
}

fn remove_or_drop(state: &State, ctx: &mut BTerm, newrunstate: &mut RunState, mode: InventoryMode) {
    let result = gui::show_inventory(state, ctx, mode.clone());
    match result.0 {
        gui::ItemMenuResult::Cancel => *newrunstate = RunState::AwaitingInput,
        gui::ItemMenuResult::NoResponse => {}
        gui::ItemMenuResult::Selected => {
            let item_entity = result
                .1
                .unwrap_or_else(|| panic!("Item selected but not found!"));
            let item_name = state
                .ecs
                .read_storage::<Name>()
                .get(item_entity)
                .unwrap()
                .name
                .clone();
            match mode {
                InventoryMode::Drop => {
                    let mut intent = state.ecs.write_storage::<EventWantsToDropItem>();
                    intent
                        .insert(
                            get_player_unwrap(&state.ecs, PLAYER_NAME),
                            EventWantsToDropItem { item: item_entity },
                        )
                        .unwrap_or_else(|er| {
                            panic!("Tried to drop {} but failed!: {}", item_name, er)
                        });
                }
                InventoryMode::Unequip => {
                    let mut intent = state.ecs.write_storage::<EventWantsToRemoveItem>();
                    intent
                        .insert(
                            get_player_unwrap(&state.ecs, PLAYER_NAME),
                            EventWantsToRemoveItem { item: item_entity },
                        )
                        .unwrap_or_else(|er| {
                            panic!("Tried to remove {} but failed!: {}", item_name, er)
                        });
                }
                InventoryMode::Use => panic!("mode is Use for removeOrDropItem"),
            };
            *newrunstate = RunState::PlayerTurn;
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        // This might be a good candidate for mutual TCO, someday
        // Also, look into bracket/resource handling patterns
        let mut newrunstate = {
            let runstate = self.ecs.fetch::<RunState>();
            *runstate
        };

        // FIXME: appears runstate is being possibly set to AwaitingInput before
        // going into `tick`
        /*
        tick runstate: PlayerTurn
        tick runstate: MonsterTurn
        tick runstate: AwaitingInput
        new runstate: GameOver
        tick runstate: AwaitingInput
        tick runstate: AwaitingInput
                */
        ctx.cls();

        newrunstate = match newrunstate {
            RunState::MainMenu { .. } => newrunstate,
            _ => {
                draw_map(&self.ecs, ctx);
                draw_ui(&self.ecs, ctx, &self.display);

                let game_over_opt = delete_the_dead(&mut self.ecs);

                {
                    // draw renderables
                    let positions = self.ecs.read_storage::<Position>();
                    let renderables = self.ecs.read_storage::<Renderable>();
                    let map = self.ecs.fetch::<Map>();

                    (&positions, &renderables)
                        .join()
                        .filter(|(pos, _)| map.visible_tiles[pos.idx(self.display.width)])
                        .sorted_by(|aa, bb| (aa.1.render_order).cmp(&bb.1.render_order))
                        .for_each(|(pos, render)| {
                            ctx.set(pos.xx, pos.yy, render.fg, render.bg, render.glyph);
                        });
                }
                game_over_opt.unwrap_or(newrunstate)
            }
        };
        match newrunstate {
            RunState::PreRun => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => newrunstate = player_input(self, ctx),
            RunState::PlayerTurn => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::AwaitingInput;
            }
            // TODO: consider abstracting the next 3 into a single function ... but probably not worth it
            RunState::ShowInventory => {
                let result = show_inventory(self, ctx, InventoryMode::Use);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result
                            .1
                            .unwrap_or_else(|| panic!("Item selected but not found!"));

                        let is_ranged = self.ecs.read_storage::<Range>();
                        let is_item_ranged = is_ranged.get(item_entity);
                        if let Some(is_item_ranged) = is_item_ranged {
                            newrunstate = RunState::ShowTargeting {
                                range: is_item_ranged.range.0,
                                item: item_entity,
                            };
                        } else {
                            let mut intent = self.ecs.write_storage::<EventWantsToUseItem>();
                            let item_name = self
                                .ecs
                                .read_storage::<Name>()
                                .get(item_entity)
                                .unwrap()
                                .name
                                .clone();
                            intent
                                .insert(
                                    get_player_unwrap(&self.ecs, PLAYER_NAME),
                                    EventWantsToUseItem {
                                        item: item_entity,
                                        target: None,
                                    },
                                )
                                .unwrap_or_else(|er| {
                                    panic!("Tried to use {} but failed!: {}", item_name, er)
                                });
                            newrunstate = RunState::PlayerTurn;
                        }
                    }
                }
            }
            RunState::ShowDropItem => {
                remove_or_drop(self, ctx, &mut newrunstate, InventoryMode::Drop)
            }
            RunState::ShowRemoveItem => {
                remove_or_drop(self, ctx, &mut newrunstate, InventoryMode::Unequip)
            }
            RunState::ShowTargeting { range, item } => {
                let result = gui::ranged_target(self, ctx, range);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let mut intent = self.ecs.write_storage::<EventWantsToUseItem>();
                        let item_name = self
                            .ecs
                            .read_storage::<Name>()
                            .get(item)
                            .unwrap()
                            .name
                            .clone();
                        intent
                            .insert(
                                get_player_unwrap(&self.ecs, PLAYER_NAME),
                                EventWantsToUseItem {
                                    item,
                                    target: result.1.map(|p| p.into()),
                                },
                            )
                            .unwrap_or_else(|er| {
                                panic!("Tried to use {} but failed!: {}", item_name, er)
                            });
                        newrunstate = RunState::PlayerTurn;
                    }
                }
            }
            RunState::MainMenu { .. } => {
                let result = gui::main_menu(self, ctx);
                match result.status {
                    gui::MainMenuStatus::NoSelection => {
                        newrunstate = RunState::MainMenu {
                            menu_selection: result.highlighted,
                        };
                    }
                    gui::MainMenuStatus::Selected => match result.highlighted {
                        gui::MainMenuSelection::NewGame => {
                            delete_state(&mut self.ecs);
                            (*self, _) = init_state(false, Some(ctx));
                            newrunstate = RunState::PreRun
                        }
                        gui::MainMenuSelection::SaveGame => newrunstate = RunState::SaveGame,
                        gui::MainMenuSelection::ResumeGame => newrunstate = RunState::PreRun,
                        gui::MainMenuSelection::LoadGame => {
                            if saveload_system::does_save_exist() {
                                saveload_system::load_game(&mut self.ecs);
                                newrunstate = RunState::AwaitingInput;
                            } else {
                                let mut gamelog = self.ecs.fetch_mut::<gamelog::GameLog>();
                                gamelog
                                    .entries
                                    .push("No save game to load. Starting new game!".to_string());
                                newrunstate = RunState::PreRun
                            }
                        }
                        gui::MainMenuSelection::KeyBindings => {
                            newrunstate = RunState::KeyBindingsMenu;
                        }
                        gui::MainMenuSelection::Quit => ctx.quit(),
                    },
                }
            }
            RunState::KeyBindingsMenu => match gui::show_keybindings(self, ctx) {
                true => newrunstate = RunState::KeyBindingsMenu,
                false => {
                    newrunstate = RunState::MainMenu {
                        menu_selection: gui::MainMenuSelection::KeyBindings,
                    }
                }
            },
            RunState::SaveGame => {
                saveload_system::save_game(&mut self.ecs);
                newrunstate = RunState::MainMenu {
                    menu_selection: gui::MainMenuSelection::SaveGame,
                }
            }
            RunState::NextLevel => {
                self.goto_next_level();
                newrunstate = RunState::PreRun;
            }
            RunState::GameOver => {
                let result = gui::game_over(ctx);
                match result {
                    gui::GameOverResult::NoSelection => {}
                    gui::GameOverResult::QuitToMenu => {
                        delete_state(&mut self.ecs);
                        newrunstate = RunState::MainMenu {
                            menu_selection: gui::MainMenuSelection::NewGame,
                        }
                    }
                }
            }
        }
        let mut runstate = self.ecs.fetch_mut::<RunState>();
        *runstate = newrunstate;
    }
}

pub fn delete_state(ecs: &mut World) {
    // Delete everything
    let to_delete: Vec<Entity> = ecs.entities().join().collect();
    to_delete.iter().for_each(|entity| {
        ecs.delete_entity(*entity)
            .unwrap_or_else(|er| panic!("Unable to delete entity with id {}: {}", entity.id(), er))
    });
}

pub fn init_state(test_ecs: bool, ctxt_opt: Option<&BTerm>) -> (State, Option<BTerm>) {
    let (mut gs, opt_ctxt) = if test_ecs {
        (
            State {
                ecs: World::new(),
                display: DisplayState::default(),
            },
            None,
        )
    } else {
        use bracket_lib::prelude::BTermBuilder;
        let context_opt = if ctxt_opt.is_some() {
            None
        } else {
            let mut ctxt = BTermBuilder::simple80x50()
                .with_title("Rusty Rogue")
                .build()
                .unwrap(); // TODO: better error handling from software tools
            ctxt.with_post_scanlines(true);
            // ^ gives a retro "scanlines and screen burn" effect
            Some(ctxt)
        };
        let display_state = if let Some(ctxt) = context_opt.as_ref() {
            calc_display_state(ctxt)
        } else if let Some(ctxt) = ctxt_opt {
            calc_display_state(ctxt)
        } else {
            DisplayState::default()
        };

        (
            State {
                ecs: World::new(),
                display: display_state,
            },
            context_opt,
        )
    };

    execute_with_type_list!(register_individually!(gs.ecs));

    // register markers
    gs.ecs.register::<SimpleMarker<SerializeMe>>();

    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());
    gs.ecs.insert(RunState::MainMenu {
        menu_selection: MainMenuSelection::NewGame,
    });
    gs.ecs.insert(gamelog::GameLog {
        entries: vec!["Welcome to Rusty Rogue!".to_string()],
    });
    gs.ecs.insert(RandomNumberGenerator::new());

    let map = new_map_rooms_and_corridors(&gs, 1);

    let player_posn = map.rooms.first().unwrap().center();
    gs.ecs.insert(map);
    populate_rooms(&mut gs.ecs);

    // FIXME: unit discard warning?
    spawner::player(&mut gs, player_posn);
    gs.ecs.insert(player_posn);

    (gs, opt_ctxt)
}

// TODO: start using RustIO here;
// https://github.com/politrons/FunctionalRust/blob/main/src/features/rust_io.rs#L478
fn main() {
    {
        // Globals
        let default_keys = KeyBindings::_make_default();
        DEFAULT_KEY_BINDINGS.set(default_keys).unwrap();
    }
    if let (gs, Some(context)) = init_state(false, None) {
        bracket_lib::prelude::main_loop(context, gs).unwrap()
    } else {
        console::log("init_state called as if in test mode, exiting");
    }
}

fn populate_rooms(ecs: &mut World) -> Vec<Entity> {
    let rooms = {
        let map = ecs.read_resource::<Map>();
        map.rooms.clone()
    };
    rooms
        .iter()
        .skip(1)
        .flat_map(|room| spawn_room(ecs, room, 1))
        .collect()
}
