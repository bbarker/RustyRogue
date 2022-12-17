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
pub struct EventIncomingDamage {
    pub amount: Vec<u16>,
}

impl EventIncomingDamage {
    pub fn new_damage(store: &mut WriteStorage<EventIncomingDamage>, victim: Entity, amount: u16) {
        if let Some(dmg) = store.get_mut(victim) {
            dmg.amount.push(amount);
        } else {
            store
                .insert(
                    victim,
                    EventIncomingDamage {
                        amount: vec![amount],
                    },
                )
                .expect("Unable to insert damage event");
        }
    }
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct EventWantsToMelee {
    pub target: Entity,
}

#[derive(Component, Debug)]
pub struct EventWantsToPickupItem {
    pub collected_by: Entity,
    pub item: Entity,
}

#[derive(Component, Debug)]
pub struct InBackpack {
    pub owner: Entity,
}

#[derive(Component, Debug)]
pub struct Item {}

#[derive(Component, Debug)]
pub struct Monster {}

#[derive(Component, Debug)]
pub struct Name {
    pub name: String,
}

#[derive(Component, Debug, Clone)]
pub struct Player {}

pub trait IsPlayer {
    fn from(self) -> Player;
}

impl<T> IsPlayer for &T
where
    T: IsPlayer + Clone,
{
    fn from(self) -> Player {
        self.clone().from()
    }
}

impl IsPlayer for Player {
    fn from(self) -> Player {
        self
    }
}

#[derive(Component, Clone, Copy, PartialEq)]
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

impl<T> Positionable for &T
where
    T: Positionable + Clone,
{
    fn from(self) -> Position {
        (*self).clone().from()
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

impl Positionable for (i32, i32) {
    fn from(self) -> Position {
        Position {
            xx: self.0.try_into().unwrap(),
            yy: self.1.try_into().unwrap(),
        }
    }
}

#[derive(Component, Debug)]
pub struct Potion {
    pub heal_amount: u16,
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
