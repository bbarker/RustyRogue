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

#[derive(Component, Clone, Copy)]
pub struct Position {
    pub xx: PsnU,
    pub yy: PsnU,
}

impl Position {
    pub fn to_point(&self) -> Point {
        Point::new(self.xx, self.yy)
    }
}

pub fn xy_idx(width: PsnU, xx: PsnU, yy: PsnU) -> usize {
    ((yy * width) + xx).try_into().unwrap()
}

pub trait Positionable {
    fn from(self) -> Position;

    fn idx(self, width: PsnU) -> usize
    where
        Self: Sized,
    {
        let pos = self.from();
        xy_idx(width, pos.xx, pos.yy)
    }
}

impl Positionable for Position {
    fn from(self) -> Position {
        self
    }
}

impl Positionable for Point {
    fn from(self) -> Position {
        Position {
            xx: self.x.try_into().unwrap(),
            yy: self.y.try_into().unwrap(),
        }
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
    pub dirty: bool,
}
