use std::cmp::{max, min};

use bracket_lib::{
    prelude::{BTerm, RGB},
    terminal::{BLACK, RED, WHITE, YELLOW},
};
use specs::prelude::*;

use crate::{
    components::{CombatStats, Player},
    display_state::DisplayState,
    gamelog::GameLog,
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
