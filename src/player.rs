use bracket_lib::terminal::{BTerm, VirtualKeyCode};
use itertools::Itertools;
use specs::{world::EntitiesRes, *};

use crate::{
    components::{
        CombatStats, EventWantsToMelee, EventWantsToPickupItem, IsPlayer, Item, Name, Player,
        Position, Positionable, Viewshed,
    },
    gamelog,
    gui::PANEL_HEIGHT,
    map::Map,
    PsnU, RunState, State,
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

pub fn player_input(gs: &mut State, ctx: &mut BTerm) -> RunState {
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

            // Misc Map Actions
            VirtualKeyCode::G => get_item(&mut gs.ecs),

            // Menus
            VirtualKeyCode::I => RunState::ShowInventory,

            _ => RunState::AwaitingInput,
        },
    }
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

fn get_item(ecs: &mut World) -> RunState {
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
                .unwrap_or_else(|_| panic!("Unable to insert pickup event"));
        })
    }
    RunState::PlayerTurn
}
