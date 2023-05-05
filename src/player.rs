use std::collections::HashMap;
use std::sync::Arc;

use bracket_lib::terminal::{BTerm, VirtualKeyCode};
use itertools::Itertools;
use once_cell::sync::OnceCell;
use specs::{world::EntitiesRes, *};

use crate::{
    components::{
        CombatStats, EventWantsToMelee, EventWantsToPickupItem, IsPlayer, Item, Monster, Name,
        Player, Position, Positionable, Viewshed,
    },
    gamelog,
    gui::MainMenuSelection::*,
    map::{Map, TileType},
    RunState, State,
};

// TODO: add this to a sub-state "Option<ClientState>" in State
pub const PLAYER_NAME: &str = "Player";

pub fn try_move_player(delta_x: i32, delta_y: i32, gs: &mut State) -> RunState {
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
        let try_pos = &gs.ecs.fetch::<Map>().dest_from_delta(pos, delta_x, delta_y);
        let combat_stats = gs.ecs.read_storage::<CombatStats>();
        let mut map = gs.ecs.fetch_mut::<Map>();
        let destination_ix = map.pos_idx(try_pos);
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
            map.move_blocker(pos, try_pos);
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

// TODO: we can build two maps from a list of PlayerInputEntry values
// - one will have keys that are the key codes, so we can immediately lookup
// the action. The other will have the name of the action as the key, so we
// can see what keys are bound for an action.
//
// But what if we want to change the key bindings - do we have to entirely rebuild
// both maps from this list? May be better to start with the name-keyed map, and
// if a change is made, update the entries in the other map accordingly:

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
enum PlayerAction {
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
}

pub trait PlayerActionFnT: FnMut(&mut State) -> RunState + Send + Sync + 'static {}

impl<F> PlayerActionFnT for F where F: FnMut(&mut State) -> RunState + Send + Sync + 'static {}

pub type PlayerActionFn = Arc<dyn PlayerActionFnT>;

impl std::fmt::Debug for dyn PlayerActionFnT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<PlayerActionFn>")
    }
}

#[derive(Clone, Debug)]
struct ActionAndKeys {
    key_codes: Vec<VirtualKeyCode>,
    action: PlayerActionFn,
}

#[derive(Clone, Debug)]
struct ActionAndId {
    id: PlayerAction,
    action: PlayerActionFn,
}

#[derive(Debug)]
pub struct KeyBindings {
    action_by_id: HashMap<PlayerAction, ActionAndKeys>,
    action_by_key: HashMap<VirtualKeyCode, ActionAndId>,
}

pub static DEFAULT_KEY_BINDINGS: OnceCell<KeyBindings> = OnceCell::new();

impl KeyBindings {
    pub fn default() -> &'static KeyBindings {
        DEFAULT_KEY_BINDINGS
            .get()
            .expect("DEFAULT_KEY_BINDINGS not initialized")
    }

    // TODO: wrap State in OnceCell so we can have a static reference to it
    // Update: does not seem to work well as State needs to be &mut
    // Alternatively may need to try FnMut(gs: &mut State) -> RunState
    pub fn _make_default() -> KeyBindings {
        let action_by_id: HashMap<PlayerAction, ActionAndKeys> = [
            (
                PlayerAction::ShowInventory,
                ActionAndKeys {
                    key_codes: vec![VirtualKeyCode::I],
                    action: Arc::new(|_| RunState::ShowInventory),
                },
            ),
            (
                PlayerAction::ShowDropItem,
                ActionAndKeys {
                    key_codes: vec![VirtualKeyCode::D],
                    action: Arc::new(|_| RunState::ShowDropItem),
                },
            ),
            (
                PlayerAction::Escape,
                ActionAndKeys {
                    key_codes: vec![VirtualKeyCode::Escape],
                    action: Arc::new(|_| RunState::MainMenu {
                        menu_selection: SaveGame,
                    }),
                },
            ),
            (
                PlayerAction::ShowRemoveItem,
                ActionAndKeys {
                    key_codes: vec![VirtualKeyCode::R],
                    action: Arc::new(|_| RunState::ShowRemoveItem),
                },
            ),
            (
                PlayerAction::Left,
                ActionAndKeys {
                    key_codes: vec![
                        VirtualKeyCode::Left,
                        VirtualKeyCode::A,
                        VirtualKeyCode::Numpad4,
                    ],
                    action: Arc::new(|gs| try_move_player(-1, 0, gs)),
                },
            ),
        ]
        .iter()
        .cloned()
        .collect();

        let action_by_key: HashMap<VirtualKeyCode, ActionAndId> = action_by_id
            .iter()
            .flat_map(|(id, action_and_keys)| {
                action_and_keys.key_codes.iter().map(|key_code| {
                    (
                        *key_code,
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

pub fn player_input(gs: &mut State, ctx: &mut BTerm) -> RunState {
    match ctx.key {
        None => RunState::AwaitingInput,
        Some(key) => match key {
            // Menus
            VirtualKeyCode::I => RunState::ShowInventory,
            VirtualKeyCode::D if ctx.shift => RunState::ShowDropItem,
            VirtualKeyCode::Escape => RunState::MainMenu {
                menu_selection: SaveGame,
            },
            VirtualKeyCode::R => RunState::ShowRemoveItem,

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
            VirtualKeyCode::Numpad5 | VirtualKeyCode::Space => skip_turn(&mut gs.ecs),

            // Diagonals
            VirtualKeyCode::Numpad7 | VirtualKeyCode::Q => try_move_player(-1, -1, gs),
            VirtualKeyCode::Numpad9 | VirtualKeyCode::E => try_move_player(1, -1, gs),
            VirtualKeyCode::Numpad1 | VirtualKeyCode::Z => try_move_player(-1, 1, gs),
            VirtualKeyCode::Numpad3 | VirtualKeyCode::X => try_move_player(1, 1, gs),

            // Misc Map Actions
            // G was originally for "grab", but we'll turn this into a general interaction
            VirtualKeyCode::G => interact(&mut gs.ecs),
            _ => RunState::AwaitingInput,
        },
    }
}

fn skip_turn(ecs: &mut World) -> RunState {
    let player_entity = get_player_unwrap(ecs, PLAYER_NAME);
    let viewsheds = ecs.read_storage::<Viewshed>();
    let monsters = ecs.read_storage::<Monster>();

    let worldmap_res = ecs.fetch::<Map>();

    let viewshed = viewsheds.get(player_entity).unwrap();
    let can_heal = viewshed.visible_tiles.iter().any(|ix| {
        let some_monster = worldmap_res.tile_content[worldmap_res.pos_idx(ix)]
            .iter()
            .filter_map(|en| monsters.get(*en))
            .next();
        some_monster.is_none()
    });
    if can_heal {
        let mut combat_stats = ecs.write_storage::<CombatStats>();
        let player_stats = combat_stats.get_mut(player_entity).unwrap();
        player_stats.hp = u16::min(player_stats.max_hp, player_stats.hp + 1);
    }
    RunState::PlayerTurn
}

pub fn get_player_entities_with_pos<P: Join, R: Join>(
    entities: &Read<EntitiesRes>,
    players: P,
    positions: R,
) -> Vec<(Entity, Position)>
where
    P::Type: IsPlayer,
    R::Type: Positionable,
{
    (entities, players, positions)
        .join()
        .map(|(ent, _, pos)| (ent, pos.from()))
        .collect::<Vec<_>>()
}

pub fn get_player_no_ecs<P: Join>(
    entities: &Read<EntitiesRes>,
    names: &ReadStorage<Name>,
    players: P,
    player_name: impl Into<String>,
) -> Option<Entity>
where
    P::Type: IsPlayer,
{
    let pname = player_name.into();
    (entities, players, names)
        .join()
        .filter_map(
            |(ent, _, name)| {
                if pname == name.name {
                    Some(ent)
                } else {
                    None
                }
            },
        )
        .next()
}

pub fn get_player(ecs: &World, player_name: impl Into<String>) -> Option<Entity> {
    let entities = ecs.entities();
    let names = ecs.read_storage::<Name>();
    let players = ecs.read_storage::<Player>();

    get_player_no_ecs(&entities, &names, &players, player_name)
}

pub fn get_player_unwrap(ecs: &World, player_name: impl Into<String>) -> Entity {
    let name = player_name.into();
    get_player(ecs, &name).unwrap_or_else(|| panic!("Player {} not found", name))
}

pub fn get_player_pos_unwrap(ecs: &World, player_name: impl Into<String>) -> Position {
    let player_entity = get_player_unwrap(ecs, player_name);
    let positions = ecs.read_storage::<Position>();
    *positions.get(player_entity).unwrap_or_else(|| {
        panic!(
            "Player entity {} does not have a position component",
            player_entity.id()
        )
    })
}

fn interact(ecs: &mut World) -> RunState {
    let player_map_ix = {
        let player_pos = get_player_pos_unwrap(ecs, PLAYER_NAME);
        let map = ecs.fetch::<Map>();
        map.pos_idx(player_pos)
    };
    let map_tiles = {
        let map = ecs.fetch::<Map>();
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
    let players = ecs.read_storage::<Player>();
    let positions = ecs.read_storage::<Position>();
    let items = ecs.read_storage::<Item>();
    let mut gamelog = ecs.write_resource::<gamelog::GameLog>();

    let player_posns = get_player_entities_with_pos(&entities, &players, &positions);

    let player_target_items: Vec<EventWantsToPickupItem> = (&entities, &items, &positions)
        .join()
        .cartesian_product(player_posns)
        .filter_map(|((item_entity, _, pos), player_pos)| {
            if player_pos.1 == *pos {
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
            let mut pickup = ecs.write_storage::<EventWantsToPickupItem>();
            pickup
                .insert(wants_to_pickup.collected_by, wants_to_pickup)
                .unwrap_or_else(|er| panic!("Unable to insert pickup event: {}", er));
        })
    }
    RunState::PlayerTurn
}

fn try_next_level(ecs: &mut World) -> RunState {
    let player_pos = get_player_pos_unwrap(ecs, PLAYER_NAME);
    let map = ecs.fetch::<Map>();
    let player_ix = map.pos_idx(player_pos);
    let mut gamelog = ecs.write_resource::<gamelog::GameLog>();
    if map.tiles[player_ix] == TileType::DownStairs {
        gamelog.entries.push("You descend the stairs.".to_string());
        RunState::NextLevel
    } else {
        gamelog
            .entries
            .push("There is no way down from here.".to_string());
        RunState::AwaitingInput
    }
}
