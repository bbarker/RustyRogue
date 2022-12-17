use bracket_lib::terminal::{BTerm, VirtualKeyCode};
use specs::*;

use crate::{
    components::{CombatStats, EventWantsToMelee, Player, Position, Viewshed},
    gamelog,
    gui::PANEL_HEIGHT,
    map::Map,
    PsnU, RunState, State,
};

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

            _ => RunState::AwaitingInput,
        },
    }
}
