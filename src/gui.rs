use bracket_lib::{
    prelude::{BTerm, RGB},
    terminal::{BLACK, WHITE},
};
use specs::prelude::*;

use crate::{display_state::DisplayState, PsnU};

pub const PANEL_HEIGHT: usize = 7;

pub fn draw_ui(ecs: &World, ctx: &mut BTerm, display_state: &DisplayState) {
    ctx.draw_box(
        0,
        display_state.height - (PANEL_HEIGHT as PsnU),
        display_state.width - 1,
        PANEL_HEIGHT - 1,
        RGB::named(WHITE),
        RGB::named(BLACK),
    );
}
