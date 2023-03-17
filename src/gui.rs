use std::cmp::{max, min};

use bracket_lib::{
    prelude::{BTerm, RGB},
    terminal::{
        letter_to_option, to_cp437, DistanceAlg, FontCharType, Point, VirtualKeyCode, BLACK, BLUE,
        CYAN, MAGENTA, RED, WHITE, YELLOW,
    },
};
use itertools::FoldWhile::{Continue, Done};
use itertools::Itertools;
use specs::prelude::*;

use crate::{
    components::{
        CombatStats, Equipped, InBackpack, Name, Player, Position, Positionable, Viewshed,
    },
    display_state::DisplayState,
    gamelog::GameLog,
    map::Map,
    player::{get_player_pos_unwrap, get_player_unwrap, PLAYER_NAME},
    util::*,
    PsnU, RunState, State,
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
        let map = ecs.fetch::<Map>();
        let depth = format!("Depth: {}", map.depth);
        ctx.print_color(
            2,
            panel_top(display_state),
            RGB::named(YELLOW),
            RGB::named(BLACK),
            &depth,
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
    Selected,
}

pub fn owned_items(ecs: &World, owner: Entity) -> Vec<(Entity, String)> {
    let entities = ecs.entities();
    let backpack = ecs.read_storage::<InBackpack>();
    let equipped = ecs.read_storage::<Equipped>();
    let names = ecs.read_storage::<Name>();

    (&entities, &backpack, &names)
        .join()
        .filter(|(_e, bpack, _n)| bpack.owner == owner)
        .map(|(ent, _b, name)| (ent, name.name.to_string()))
        .chain(
            (&entities, &equipped, &names)
                .join()
                .filter(|(_e, equip, _n)| equip.owner == owner)
                .map(|(ent, _b, name)| (ent, name.name.to_string())),
        )
        .collect::<Vec<(Entity, String)>>()
}

pub fn show_inventory(
    gs: &mut State,
    ctx: &mut BTerm,
    title: impl Into<String>,
) -> (ItemMenuResult, Option<Entity>) {
    const ESCAPE_MSG: &str = "ESCAPE to cancel";
    let title_str = title.into();
    let entities = gs.ecs.entities();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();

    let player_entity = get_player_unwrap(&gs.ecs, PLAYER_NAME);

    let inventory = owned_items(&gs.ecs, player_entity);

    let (inventory_size, max_item_name_length) = inventory
        .iter()
        .fold((0, 0), |(size, max_length), (_item, name)| {
            (size + 1, max_length.max(name.len()))
        });
    let box_width = max(max(max_item_name_length, ESCAPE_MSG.len()), title_str.len()) + 4;

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
        title_str,
    );
    ctx.print_color(
        x_init + 3,
        y_init + inventory_size as PsnU + 1,
        RGB::named(YELLOW),
        RGB::named(BLACK),
        ESCAPE_MSG,
    );

    let useable: Vec<Entity> = (&entities, &backpack, &names)
        .join()
        .enumerate()
        .filter_map(|(jj, (entity, item, name))| {
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
                );
                Some(entity)
            } else {
                None
            }
        })
        .collect();

    if let Some(key) = ctx.key {
        match key {
            VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                if let Some(selection) = letter_to_option(key).clamp_opt(0, inventory_size - 1) {
                    (ItemMenuResult::Selected, Some(useable[selection as usize]))
                } else {
                    (ItemMenuResult::NoResponse, None)
                }
            }
        }
    } else {
        (ItemMenuResult::NoResponse, None)
    }
}

pub fn ranged_target(
    gs: &mut State,
    ctx: &mut BTerm,
    range: u16,
) -> (ItemMenuResult, Option<Position>) {
    let player_entity = get_player_unwrap(&gs.ecs, PLAYER_NAME);
    let player_pos = get_player_pos_unwrap(&gs.ecs, PLAYER_NAME);
    let viewsheds = gs.ecs.read_storage::<Viewshed>();

    ctx.print_color(
        5,
        0,
        RGB::named(YELLOW),
        RGB::named(BLACK),
        "Select Target:",
    );

    let visible_opt = viewsheds.get(player_entity);
    if let Some(visible) = visible_opt {
        // Highlight available target cells
        let available_cells: Vec<Point> = visible
            .visible_tiles
            .iter()
            .filter_map(|pos| {
                let distance = DistanceAlg::Pythagoras.distance2d(*pos, player_pos.into());
                if distance <= range as f32 {
                    ctx.set_bg(pos.x, pos.y, RGB::named(BLUE));
                    Some(*pos)
                } else {
                    None
                }
            })
            .collect();

        // Draw mouse cursor
        let mouse_pos: Point = ctx.mouse_pos().into();
        let valid_target = available_cells
            .iter()
            .fold_while(false, |_, pos| {
                if *pos == mouse_pos {
                    Done(true)
                } else {
                    Continue(false)
                }
            })
            .is_done();
        if valid_target {
            ctx.set_bg(mouse_pos.x, mouse_pos.y, RGB::named(CYAN));
            if ctx.left_click {
                (ItemMenuResult::Selected, Some(mouse_pos.from()))
            } else {
                (ItemMenuResult::NoResponse, None)
            }
        } else {
            ctx.set_bg(mouse_pos.x, mouse_pos.y, RGB::named(RED));
            if ctx.left_click {
                (ItemMenuResult::Cancel, None)
            } else {
                (ItemMenuResult::NoResponse, None)
            }
        }
    } else {
        (ItemMenuResult::Cancel, None)
    }
}

macro_attr! {
    #[derive(PartialEq, Copy, Clone, PrevVariant!, NextVariant!, IterVariants!(MainMenuVariants))]
    pub enum MainMenuSelection {
        NewGame,
        SaveGame,
        ResumeGame,
        LoadGame,
        Quit,
    }
}
//
const MAIN_MENU_FIRST: MainMenuSelection = MainMenuSelection::NewGame;
const MAIN_MENU_LAST: MainMenuSelection = MainMenuSelection::Quit;
//
const fn main_menu_entry_string(selection: MainMenuSelection) -> &'static str {
    match selection {
        MainMenuSelection::NewGame => "New Game",
        MainMenuSelection::SaveGame => "Save Game",
        MainMenuSelection::ResumeGame => "Resume Playing",
        MainMenuSelection::LoadGame => "Load Game",
        MainMenuSelection::Quit => "Quit",
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuStatus {
    NoSelection,
    Selected,
}

pub struct MainMenuResult {
    pub highlighted: MainMenuSelection,
    pub status: MainMenuStatus,
}

pub fn menu_fg_color(selection: MainMenuSelection, current_selection: MainMenuSelection) -> RGB {
    if selection == current_selection {
        RGB::named(MAGENTA)
    } else {
        RGB::named(WHITE)
    }
}

fn draw_main_menu_entry(ctx: &mut BTerm, entry: MainMenuSelection, selection: MainMenuSelection) {
    ctx.print_color_centered(
        24 + entry as i32,
        menu_fg_color(entry, selection),
        RGB::named(BLACK),
        main_menu_entry_string(entry),
    );
}
pub fn main_menu(gs: &mut State, ctx: &mut BTerm) -> MainMenuResult {
    let runstate = gs.ecs.fetch::<RunState>();

    ctx.print_color_centered(15, RGB::named(YELLOW), RGB::named(BLACK), "Rusty Rogue");

    if let RunState::MainMenu {
        menu_selection: selection,
    } = *runstate
    {
        MainMenuSelection::iter_variants().for_each(|entry| {
            draw_main_menu_entry(ctx, entry, selection);
        });

        match ctx.key {
            None => MainMenuResult {
                highlighted: selection,
                status: MainMenuStatus::NoSelection,
            },
            Some(key) => match key {
                VirtualKeyCode::Escape => MainMenuResult {
                    highlighted: MainMenuSelection::ResumeGame,
                    status: MainMenuStatus::Selected,
                },
                VirtualKeyCode::Up => MainMenuResult {
                    highlighted: selection.prev_variant().unwrap_or(MAIN_MENU_LAST),
                    status: MainMenuStatus::NoSelection,
                },
                VirtualKeyCode::Down => MainMenuResult {
                    highlighted: selection.next_variant().unwrap_or(MAIN_MENU_FIRST),
                    status: MainMenuStatus::NoSelection,
                },
                VirtualKeyCode::Return => MainMenuResult {
                    highlighted: selection,
                    status: MainMenuStatus::Selected,
                },
                _ => MainMenuResult {
                    highlighted: selection,
                    status: MainMenuStatus::NoSelection,
                },
            },
        }
    } else {
        MainMenuResult {
            highlighted: MainMenuSelection::NewGame,
            status: MainMenuStatus::NoSelection,
        }
    }
}
