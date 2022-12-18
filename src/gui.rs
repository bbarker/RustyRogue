use std::cmp::{max, min};

use bracket_lib::{
    prelude::{BTerm, RGB},
    terminal::{to_cp437, FontCharType, VirtualKeyCode, BLACK, MAGENTA, RED, WHITE, YELLOW},
};
use specs::prelude::*;

use crate::{
    components::{CombatStats, InBackpack, Name, Player, Position, Positionable},
    display_state::DisplayState,
    gamelog::GameLog,
    map::Map,
    player::{get_player_unwrap, PLAYER_NAME},
    PsnU, State,
};

pub const PANEL_HEIGHT: usize = 7;
pub const PANEL_HEIGHT_SAFE: usize = max(PANEL_HEIGHT, 1);
pub const PANEL_HEIGHT_INTERIOR: usize = min(PANEL_HEIGHT, PANEL_HEIGHT - 2);

fn panel_top(display_state: &DisplayState) -> PsnU {
    display_state.height - (PANEL_HEIGHT as PsnU)
}

pub fn draw_ui(ecs: &World, ctx: &mut BTerm, display_state: &DisplayState) {
    if PANEL_HEIGHT > 0 {
        ctx.draw_box(
            0,
            panel_top(display_state),
            display_state.width - 1,
            PANEL_HEIGHT_SAFE - 1,
            RGB::named(WHITE),
            RGB::named(BLACK),
        );
        draw_health_bar(ecs, ctx, display_state);
        draw_log(ecs, ctx, display_state);
    }

    let mouse_pos = ctx.mouse_pos();
    ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(MAGENTA));
    draw_tooltips(ecs, ctx)
}

fn draw_log(ecs: &World, ctx: &mut BTerm, display_state: &DisplayState) {
    let log = ecs.fetch::<GameLog>();
    log.entries
        .iter()
        .rev()
        .take(PANEL_HEIGHT_INTERIOR)
        .enumerate()
        .for_each(|(line_num, msg)| {
            ctx.print(2, panel_top(display_state) + 1 + line_num as PsnU, msg);
        })
}

fn draw_health_bar(ecs: &World, ctx: &mut BTerm, display_state: &DisplayState) {
    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();
    let bar_start_x = 28;
    let bar_end_x = display_state.width - bar_start_x - 1;

    (&combat_stats, &players)
        .join()
        .for_each(|(stats, _player)| {
            let health = format!("HP: {} / {}", stats.hp, stats.max_hp);
            ctx.print_color(
                12,
                panel_top(display_state),
                RGB::named(YELLOW),
                RGB::named(BLACK),
                &health,
            );
            ctx.draw_bar_horizontal(
                bar_start_x,
                panel_top(display_state),
                bar_end_x,
                stats.hp,
                stats.max_hp,
                RGB::named(RED),
                RGB::named(BLACK),
            )
        })
}

fn draw_tooltips(ecs: &World, ctx: &mut BTerm) {
    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();

    let mouse_pos = ctx.mouse_pos();
    if mouse_pos.0 < (map.width() as i32) && mouse_pos.1 < (map.height() as i32) {
        let tooltip = (&names, &positions)
            .join()
            .filter(|(_name, pos)| (**pos) == mouse_pos.from())
            .map(|(name, _pos)| name.name.to_string())
            .collect::<Vec<String>>();

        if !tooltip.is_empty() {
            let width = 1 + tooltip.iter().map(|line| line.len()).max().unwrap_or(0);
            let height = 1 + tooltip.len();

            let (tooltip_x, tooltip_y) = if (mouse_pos.0 as usize + width + 1) > map.width() {
                (mouse_pos.0 - width as i32, mouse_pos.1)
            } else {
                (mouse_pos.0 + 1, mouse_pos.1)
            };
            ctx.draw_box(
                tooltip_x,
                tooltip_y,
                width,
                height,
                RGB::named(WHITE),
                RGB::named(BLACK),
            );
            tooltip.iter().enumerate().for_each(|(ii, line)| {
                ctx.print_color(
                    tooltip_x + 1,
                    tooltip_y + 1 + ii as i32,
                    RGB::named(WHITE),
                    RGB::named(BLACK),
                    line,
                )
            })
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum ItemMenuResult {
    Cancel,
    NoResponse,
    // Selected,
}

pub fn show_inventory(gs: &mut State, ctx: &mut BTerm) -> ItemMenuResult {
    const ESCAPE_MSG: &str = "ESCAPE to cancel";
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();

    let player_entity = get_player_unwrap(&gs.ecs, PLAYER_NAME);

    let inventory = (&backpack, &names)
        .join()
        .filter(|item| item.0.owner == player_entity);

    let (inventory_size, max_item_name_length) = inventory
        .fold((0, 0), |(size, max_length), (_item, name)| {
            (size + 1, max_length.max(name.name.len()))
        });
    let box_width = max(max_item_name_length, ESCAPE_MSG.len()) + 3 + 1;

    // (start at: mid height - half of item size):
    let x_init = 15;
    let y_init = (gs.display.height - inventory_size as PsnU) / 2;
    let y_box_init = (y_init - 2).clamp(0, y_init);
    ctx.draw_box(
        x_init,
        y_box_init,
        box_width,
        inventory_size + 3,
        RGB::named(WHITE),
        RGB::named(BLACK),
    );
    ctx.print_color(
        x_init + 3,
        y_box_init,
        RGB::named(YELLOW),
        RGB::named(BLACK),
        "Inventory",
    );
    ctx.print_color(
        x_init + 3,
        y_init + inventory_size as PsnU + 1,
        RGB::named(YELLOW),
        RGB::named(BLACK),
        ESCAPE_MSG,
    );

    (&backpack, &names)
        .join()
        .enumerate()
        .for_each(|(jj, (item, name))| {
            if item.owner == player_entity {
                ctx.set(
                    x_init + 1,
                    y_init + jj as PsnU,
                    RGB::named(WHITE),
                    RGB::named(BLACK),
                    to_cp437('('),
                );
                ctx.set(
                    // assign the item a letter in the menu
                    x_init + 2,
                    y_init + jj as PsnU,
                    RGB::named(WHITE),
                    RGB::named(BLACK),
                    97 + jj as FontCharType,
                );
                ctx.set(
                    x_init + 3,
                    y_init + jj as PsnU,
                    RGB::named(WHITE),
                    RGB::named(BLACK),
                    to_cp437(')'),
                );
                ctx.print_color(
                    x_init + 4,
                    y_init + jj as PsnU,
                    RGB::named(WHITE),
                    RGB::named(BLACK),
                    &name.name,
                )
            }
        });

    if let Some(key) = ctx.key {
        if VirtualKeyCode::Escape == key {
            // TODO: match
            ItemMenuResult::Cancel
        } else {
            ItemMenuResult::NoResponse
        }
    } else {
        ItemMenuResult::NoResponse
    }
}
