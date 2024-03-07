use std::sync::Arc;

use bracket_lib::terminal::{BTerm, VirtualKeyCode};
use indexmap::IndexMap;
use itertools::Itertools;
use once_cell::sync::OnceCell;
//use specs::{world::EntitiesRes, *};
use bevy::{ecs::system::RunSystemOnce, prelude::*};

use crate::{
    components::{
        CombatStats, EventWantsToMelee, EventWantsToPickupItem, Item, Monster, Player, Position,
        Viewshed,
    },
    gamelog::GameLog,
    gui::MainMenuSelection::*,
    map::{Map, TileType},
    RunState, State,
};

// TODO: add this to a sub-state "Option<ClientState>" in State
pub const PLAYER_NAME: &str = "Player";

pub fn try_move_player_system(
    In(dx_dy): In<(i32, i32)>,
    mut commands: Commands,
    mut log: ResMut<GameLog>,
    mut map: ResMut<Map>,
    mut query: Query<(Entity, &mut Position, &mut Viewshed), With<Player>>, // need mutable?
    combat_stats: Query<&CombatStats>,
) -> RunState {
    let (delta_x, delta_y) = dx_dy;
    if let Some((entity, mut pos, mut viewshed)) = query.iter_mut().next() {
        let try_pos = map.dest_from_delta(&*pos, delta_x, delta_y);
        let destination_ix = map.pos_idx(try_pos);
        let combat = map.tile_content[destination_ix]
            .iter()
            .filter(|potential_target| **potential_target != entity)
            .any(|potential_target| {
                if let Ok(_c_stats) = combat_stats.get(*potential_target) {
                    log.entries
                        .push("I stab thee with righteous fury!".to_string());
                    commands.entity(entity).insert(EventWantsToMelee {
                        target: *potential_target,
                    });
                    true
                } else {
                    false
                }
            });
        if !combat && !map.blocked[destination_ix] {
            map.move_blocker(&mut pos, &try_pos);
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

pub fn try_move_player(ecs: &mut World, delta_x: i32, delta_y: i32) -> RunState {
    ecs.run_system_once_with((delta_x, delta_y), try_move_player_system)
}

macro_attr! {
    #[derive(PartialEq, Eq, Clone, Copy, Debug, Hash, EnumDisplay!)]
    pub enum PlayerAction {
        ShowInventory,
        ShowDropItem,
        Escape,
        ShowRemoveItem,
        Left,
        Right,
        Up,
        Down,
        UpLeft,
        UpRight,
        DownLeft,
        DownRight,
        Rest,
        Grab,
    }
}

macro_attr! {
    #[derive(PartialEq, Eq, Clone, Copy, Debug, Hash, EnumDisplay!)]
    pub enum ContextKeys {
        Shift
    }

}

impl ContextKeys {
    pub fn display_vec(keys: &Vec<ContextKeys>) -> String {
        if keys.is_empty() {
            "".to_string()
        } else {
            keys.iter().map(|k| k.to_string()).join(" + ") + " + "
        }
    }
}

type Keys = (VirtualKeyCode, Vec<ContextKeys>);

pub fn display_key_combo(keys: &Keys) -> String {
    let (key, context_keys) = keys;
    format!("{}{:?}", ContextKeys::display_vec(context_keys), key)
}

pub trait PlayerActionFnT: Fn(&mut State) -> RunState + Send + Sync + 'static {}

impl<F> PlayerActionFnT for F where F: Fn(&mut State) -> RunState + Send + Sync + 'static {}

pub type PlayerActionFn = Arc<dyn PlayerActionFnT>;

impl std::fmt::Debug for dyn PlayerActionFnT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<PlayerActionFn>")
    }
}

#[derive(Clone, Debug)]
pub struct ActionAndKeys {
    pub key_codes: Vec<Keys>,
    pub action: PlayerActionFn,
}

#[derive(Clone, Debug)]
pub struct ActionAndId {
    pub id: PlayerAction,
    pub action: PlayerActionFn,
}

#[derive(Debug)]
pub struct KeyBindings {
    pub action_by_id: IndexMap<PlayerAction, ActionAndKeys>,
    pub action_by_key: IndexMap<Keys, ActionAndId>,
}

pub static DEFAULT_KEY_BINDINGS: OnceCell<KeyBindings> = OnceCell::new();

impl KeyBindings {
    pub fn default() -> &'static KeyBindings {
        DEFAULT_KEY_BINDINGS
            .get()
            .expect("DEFAULT_KEY_BINDINGS not initialized")
    }

    pub fn _make_default() -> KeyBindings {
        let action_by_id: IndexMap<PlayerAction, ActionAndKeys> = [
            (
                PlayerAction::ShowInventory,
                ActionAndKeys {
                    key_codes: vec![(VirtualKeyCode::I, vec![])],
                    action: Arc::new(|_| RunState::ShowInventory),
                },
            ),
            (
                PlayerAction::ShowDropItem,
                ActionAndKeys {
                    key_codes: vec![(VirtualKeyCode::D, vec![ContextKeys::Shift])],
                    action: Arc::new(|_| RunState::ShowDropItem),
                },
            ),
            (
                PlayerAction::Escape,
                ActionAndKeys {
                    key_codes: vec![(VirtualKeyCode::Escape, vec![])],
                    action: Arc::new(|_| RunState::MainMenu {
                        menu_selection: SaveGame,
                    }),
                },
            ),
            (
                PlayerAction::ShowRemoveItem,
                ActionAndKeys {
                    key_codes: vec![(VirtualKeyCode::R, vec![])],
                    action: Arc::new(|_| RunState::ShowRemoveItem),
                },
            ),
            (
                PlayerAction::Left,
                ActionAndKeys {
                    key_codes: vec![
                        (VirtualKeyCode::Left, vec![]),
                        (VirtualKeyCode::A, vec![]),
                        (VirtualKeyCode::Numpad4, vec![]),
                    ],
                    action: Arc::new(|gs| try_move_player(&mut gs.ecs, -1, 0)),
                },
            ),
            (
                PlayerAction::Right,
                ActionAndKeys {
                    key_codes: vec![
                        (VirtualKeyCode::Right, vec![]),
                        (VirtualKeyCode::D, vec![]),
                        (VirtualKeyCode::Numpad6, vec![]),
                    ],
                    action: Arc::new(|gs| try_move_player(&mut gs.ecs, 1, 0)),
                },
            ),
            (
                PlayerAction::Up,
                ActionAndKeys {
                    key_codes: vec![
                        (VirtualKeyCode::Up, vec![]),
                        (VirtualKeyCode::W, vec![]),
                        (VirtualKeyCode::Numpad8, vec![]),
                    ],
                    action: Arc::new(|gs| try_move_player(&mut gs.ecs, 0, -1)),
                },
            ),
            (
                PlayerAction::Down,
                ActionAndKeys {
                    key_codes: vec![
                        (VirtualKeyCode::Down, vec![]),
                        (VirtualKeyCode::S, vec![]),
                        (VirtualKeyCode::Numpad2, vec![]),
                    ],
                    action: Arc::new(|gs| try_move_player(&mut gs.ecs, 0, 1)),
                },
            ),
            (
                PlayerAction::UpLeft,
                ActionAndKeys {
                    key_codes: vec![(VirtualKeyCode::Numpad7, vec![])],
                    action: Arc::new(|gs| try_move_player(&mut gs.ecs, -1, -1)),
                },
            ),
            (
                PlayerAction::UpRight,
                ActionAndKeys {
                    key_codes: vec![(VirtualKeyCode::Numpad9, vec![])],
                    action: Arc::new(|gs| try_move_player(&mut gs.ecs, 1, -1)),
                },
            ),
            (
                PlayerAction::DownLeft,
                ActionAndKeys {
                    key_codes: vec![(VirtualKeyCode::Numpad1, vec![])],
                    action: Arc::new(|gs| try_move_player(&mut gs.ecs, -1, 1)),
                },
            ),
            (
                PlayerAction::DownRight,
                ActionAndKeys {
                    key_codes: vec![(VirtualKeyCode::Numpad3, vec![])],
                    action: Arc::new(|gs| try_move_player(&mut gs.ecs, 1, 1)),
                },
            ),
            (
                PlayerAction::Rest,
                ActionAndKeys {
                    key_codes: vec![
                        (VirtualKeyCode::Numpad5, vec![]),
                        (VirtualKeyCode::Space, vec![]),
                    ],
                    action: Arc::new(|gs| skip_turn(&mut gs.ecs)),
                },
            ),
            (
                PlayerAction::Grab,
                ActionAndKeys {
                    key_codes: vec![(VirtualKeyCode::G, vec![])],
                    action: Arc::new(|gs| interact(&mut gs.ecs)),
                },
            ),
        ]
        .iter()
        .cloned()
        .collect();

        let action_by_key: IndexMap<Keys, ActionAndId> = action_by_id
            .iter()
            .flat_map(|(id, action_and_keys)| {
                action_and_keys.key_codes.iter().map(|key_code| {
                    (
                        key_code.clone(),
                        ActionAndId {
                            id: *id,
                            action: action_and_keys.action.clone(),
                        },
                    )
                })
            })
            .collect();

        KeyBindings {
            action_by_id,
            action_by_key,
        }
    }
}

pub fn player_input(gs: &mut State, ctx: &BTerm) -> RunState {
    let key_map = &KeyBindings::default().action_by_key;

    let mut ctxt_keys = vec![];
    if ctx.shift {
        ctxt_keys.push(ContextKeys::Shift);
    }
    let keys_opt = ctx.key.map(|key| (key, ctxt_keys));
    match keys_opt {
        None => RunState::AwaitingInput,
        Some(keys) => match key_map.get(&keys) {
            None => RunState::AwaitingInput,
            Some(action_and_id) => (action_and_id.action)(gs),
        },
    }
}

fn skip_turn_system(
    player_names: Query<(Entity, &Name), With<Player>>,
    viewsheds: Query<&Viewshed>,
    monsters: Query<&Monster>,
    mut combat_stats: Query<&mut CombatStats>,
    worldmap: Res<Map>,
) -> RunState {
    let player_entity = get_player_system_unwrap(In(PLAYER_NAME), player_names);

    let viewshed = viewsheds.get(player_entity).unwrap();
    let can_heal = viewshed.visible_tiles.iter().any(|ix| {
        let some_monster = worldmap.tile_content[worldmap.pos_idx(ix)]
            .iter()
            .find_map(|en| monsters.get(*en).ok());
        some_monster.is_none()
    });
    if can_heal {
        // TODO: this will be a good test to see how mutability matters for
        //     : the various parts of a query
        if let Ok(mut player_stats) = combat_stats.get_mut(player_entity) {
            player_stats.hp = u16::min(player_stats.max_hp, player_stats.hp + 1);
        };
    }
    RunState::PlayerTurn
}
//
fn skip_turn(ecs: &mut World) -> RunState {
    ecs.run_system_once(skip_turn_system)
}

pub fn is_player(query_result: Vec<(Entity, Player)>, entity: Entity) -> bool {
    query_result.iter().any(|(ent, _)| *ent == entity)
}

pub fn get_player_system(
    In(player_name): In<impl Into<String>>,
    query: Query<(Entity, &Name), With<Player>>,
) -> Option<Entity> {
    let pname = player_name.into();
    query
        .iter()
        .filter_map(|(ent, name)| {
            if pname == name.to_string() {
                Some(ent)
            } else {
                None
            }
        })
        .next()
}
//
pub fn get_player(ecs: &mut World, player_name: impl Into<String>) -> Option<Entity> {
    ecs.run_system_once_with(player_name.into(), get_player_system)
}
pub fn get_player_system_unwrap(
    player_name: In<impl Into<String>>,
    query: Query<(Entity, &Name), With<Player>>,
) -> Entity {
    get_player_system(player_name, query).unwrap_or_else(|| panic!("Player not found"))
}
//
pub fn get_player_unwrap(ecs: &mut World, player_name: impl Into<String>) -> Entity {
    ecs.run_system_once_with(player_name.into(), get_player_system_unwrap)
}

pub fn get_player_pos_no_ecs(
    In(player_name): In<impl Into<String>>,
    query: Query<(&Name, &Position), With<Player>>,
) -> Option<Position> {
    let pname = player_name.into();
    query
        .iter()
        .filter_map(|(name, pos)| {
            if pname == name.to_string() {
                Some(*pos)
            } else {
                None
            }
        })
        .next()
}
//
// TODO: if we have more functions like these, make them generic in the component type we are
//     : asking for.
pub fn get_player_pos(ecs: &mut World, player_name: impl Into<String>) -> Option<Position> {
    ecs.run_system_once_with(player_name.into(), get_player_pos_no_ecs)
}
//
pub fn get_player_pos_unwrap(ecs: &mut World, player_name: impl Into<String>) -> Position {
    get_player_pos(ecs, player_name).unwrap_or_else(|| panic!("Player not found"))
}

fn interact(ecs: &mut World) -> RunState {
    let player_map_ix = {
        let player_pos = get_player_pos_unwrap(ecs, PLAYER_NAME);
        let map = ecs.resource::<Map>();
        map.pos_idx(player_pos)
    };
    let map_tiles = {
        let map = ecs.resource::<Map>();
        map.tiles.clone()
    };
    match map_tiles[player_map_ix] {
        TileType::Floor => get_item(ecs),
        TileType::DownStairs => try_next_level(ecs),
        _ => get_item(ecs),
    }
}

pub fn get_item(ecs: &mut World) -> RunState {
    let entities = ecs.entities();
    let mut gamelog = ecs.resource_mut::<GameLog>();

    let player_posns = ecs.query::<(Entity, &Position, With<Player>)>().iter(ecs);

    let player_target_items: Vec<EventWantsToPickupItem> = ecs
        .query_filtered::<(Entity, &Position), With<Item>>() // (&entities, &items, &positions)
        .iter(ecs)
        .cartesian_product(player_posns.collect_vec())
        .filter_map(|((item_entity, pos), player_pos)| {
            if *player_pos.1 == *pos {
                Some(EventWantsToPickupItem {
                    collected_by: player_pos.0,
                    item: item_entity,
                })
            } else {
                None
            }
        })
        .collect();
    if player_target_items.is_empty() {
        gamelog
            .entries
            .push("There is nothing here to pick up.".to_string());
    } else {
        player_target_items.into_iter().for_each(|wants_to_pickup| {
            ecs.entity_mut(wants_to_pickup.collected_by)
                .insert(wants_to_pickup);
        })
    }
    RunState::PlayerTurn
}

// TODO: use one-shot system
fn try_next_level(ecs: &mut World) -> RunState {
    let player_pos = get_player_pos_unwrap(ecs, PLAYER_NAME);
    let map = ecs.resource::<Map>();
    let player_ix = map.pos_idx(player_pos);
    ecs.resource_scope(|ecs, mut gamelog: Mut<GameLog>| {
        // let mut gamelog = ecs.resource_mut::<GameLog>();
        if map.tiles[player_ix] == TileType::DownStairs {
            gamelog.entries.push("You descend the stairs.".to_string());
            RunState::NextLevel
        } else {
            gamelog
                .entries
                .push("There is no way down from here.".to_string());
            RunState::AwaitingInput
        }
    })
}
