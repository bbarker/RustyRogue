use specs::prelude::*;
use specs_derive::Component;

use bracket_lib::{
    prelude::{FontCharType, RGB},
    terminal::Point,
};

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

impl Position {
    pub fn to_point(&self) -> Point {
        Point::new(self.xx, self.yy)
    }
}

#[derive(Component)]
pub struct Renderable {
    pub glyph: FontCharType,
    pub fg: RGB,
    pub bg: RGB,
}

#[derive(Component)]
pub struct Viewshed {
    pub visible_tiles: Vec<Point>,
    pub range: i32,
    // pub dirty: bool,
}
