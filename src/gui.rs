use std::cmp::{max, min};

use bracket_lib::{
    prelude::{BTerm, RGB},
    terminal::{BLACK, MAGENTA, RED, WHITE, YELLOW},
};
use specs::prelude::*;

use crate::{
    components::{CombatStats, Name, Player, Position, Positionable},
    display_state::DisplayState,
    gamelog::GameLog,
    map::Map,
    PsnU,
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
