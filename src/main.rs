#![feature(const_cmp)]
#![feature(const_trait_impl)]
// #![feature(derive_const)] // need nightly for this

#[macro_use]
extern crate macro_attr;
#[macro_use]
extern crate enum_derive;

use bracket_lib::{
    prelude::{BTerm, GameState},
    random::RandomNumberGenerator,
};
use inventory_system::{ItemCollectionSystem, ItemDropSystem, ItemUseSystem};
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
    MainMenu {
        menu_selection: gui::MainMenuSelection,
    },
    SaveGame,
    NextLevel,
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
        let mut potions = ItemUseSystem {};
        potions.run_now(&self.ecs);
        let mut drop_items = ItemDropSystem {};
        drop_items.run_now(&self.ecs);

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

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        // This might be a good candidate for mutual TCO, someday
        // Also, look into bracket/resource handling patterns
        let mut newrunstate = {
            let runstate = self.ecs.fetch::<RunState>();
            *runstate
        };

        ctx.cls();

        match newrunstate {
            RunState::MainMenu { .. } => {}
            _ => {
                draw_map(&self.ecs, ctx);
                draw_ui(&self.ecs, ctx, &self.display);

                delete_the_dead(&mut self.ecs);

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
            }
        }

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
                let result = show_inventory(self, ctx, "Inventory: Use Item");
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result
                            .1
                            .unwrap_or_else(|| panic!("Item selected but not found!"));

                        let is_ranged = self.ecs.read_storage::<Ranged>();
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
                            let mut gamelog = self.ecs.fetch_mut::<gamelog::GameLog>();
                            gamelog.entries.push(format!("You use the {}.", item_name));
                            newrunstate = RunState::PlayerTurn;
                        }
                    }
                }
            }
            RunState::ShowDropItem => {
                let result = gui::show_inventory(self, ctx, "Inventory: Drop Item");
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result
                            .1
                            .unwrap_or_else(|| panic!("Item selected but not found!"));
                        let mut intent = self.ecs.write_storage::<EventWantsToDropItem>();
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
                                EventWantsToDropItem { item: item_entity },
                            )
                            .unwrap_or_else(|er| {
                                panic!("Tried to drop {} but failed!: {}", item_name, er)
                            });

                        let mut gamelog = self.ecs.fetch_mut::<gamelog::GameLog>();
                        gamelog.entries.push(format!("You drop the {}.", item_name));
                        newrunstate = RunState::PlayerTurn;
                    }
                }
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
                        let mut gamelog = self.ecs.fetch_mut::<gamelog::GameLog>();
                        gamelog.entries.push(format!("You use the {}.", item_name));
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
                        gui::MainMenuSelection::NewGame => newrunstate = RunState::PreRun,
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
                        gui::MainMenuSelection::Quit => ctx.quit(),
                    },
                }
            }
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
        }
        let mut runstate = self.ecs.fetch_mut::<RunState>();
        *runstate = newrunstate;
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
    // register components
    gs.ecs.register::<AreaOfEffect>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<Confusion>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<EventIncomingDamage>();
    gs.ecs.register::<EventWantsToUseItem>();
    gs.ecs.register::<EventWantsToDropItem>();
    gs.ecs.register::<EventWantsToMelee>();
    gs.ecs.register::<EventWantsToPickupItem>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<InflictsDamage>();
    gs.ecs.register::<Item>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Position>();
    gs.ecs.register::<ProvidesHealing>();
    gs.ecs.register::<Ranged>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<SerializationHelper>();
    gs.ecs.register::<Viewshed>();
    // register makers
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
        .flat_map(|room| spawn_room(ecs, room, 1))
        .collect()
}
