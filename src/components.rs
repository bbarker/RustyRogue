use specs::prelude::*;
use specs_derive::Component;

use bracket_lib::prelude::{FontCharType, RGB};

use crate::PsnU;

#[derive(Component)]
pub struct LeftMover {}

#[derive(Component, Debug)]
pub struct Player {}

#[derive(Component, Clone)]
pub struct Position {
    pub xx: PsnU,
    pub yy: PsnU,
}

#[derive(Component)]
pub struct Renderable {
    pub glyph: FontCharType,
    pub fg: RGB,
    pub bg: RGB,
}
