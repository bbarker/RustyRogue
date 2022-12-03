use bracket_lib::{
    prelude::{FontCharType, RGB},
    terminal::Point,
};
use serde::{Deserialize, Serialize};
use specs::{
    prelude::*,
    saveload::{ConvertSaveload, Marker},
    Entity,
};

use crate::PsnU;
use specs_derive::{Component, ConvertSaveload};
use std::convert::Infallible;

// `NoError` alias is deprecated in specs ... but specs_derive needs it
pub type NoError = Infallible;

#[derive(Component, Debug)]
pub struct BlocksTile {}

#[derive(Component, Debug)]
pub struct CombatStats {
    pub max_hp: u16,
    pub hp: u16,
    pub defense: u16,
    pub power: u16,
}

#[derive(Component, Debug)]
pub struct Monster {}

#[derive(Component, Debug)]
pub struct Name {
    pub name: String,
}

#[derive(Component, Debug)]
pub struct Player {}

#[derive(Component, Clone, Copy)]
pub struct Position {
    pub xx: PsnU,
    pub yy: PsnU,
}

impl From<Position> for Point {
    fn from(pos: Position) -> Self {
        Point::new(pos.xx, pos.yy)
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

#[derive(Clone, Copy)]
pub struct PlayerPosition(Position);

impl PlayerPosition {
    pub fn new(pos: Position) -> Self {
        Self(pos)
    }
    pub fn set(&mut self, pos: Position) {
        self.0 = pos;
    }
    pub fn pos(&self) -> Position {
        self.0
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

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct EventWantsToMelee {
    pub target: Entity,
}
