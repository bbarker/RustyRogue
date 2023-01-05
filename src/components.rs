use bracket_lib::{
    prelude::{FontCharType, RGB},
    terminal::Point,
};
use specs::{
    prelude::*,
    saveload::{ConvertSaveload, Marker},
    Entity,
};

use crate::{map::Map, PsnU};
use serde::{Deserialize, Serialize};
use specs_derive::*;

use std::convert::Infallible;
// `NoError` alias is deprecated in specs ... but specs_derive needs it
pub type NoError = Infallible;

#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct AreaOfEffect {
    pub radius: u16,
}

#[derive(Component, Deserialize, Serialize, Clone, Debug)]
pub struct BlocksTile {}

#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct CombatStats {
    pub max_hp: u16,
    pub hp: u16,
    pub defense: u16,
    pub power: u16,
}

#[derive(Component, ConvertSaveload, Debug, Clone)]
pub struct Confusion {
    pub step_sequence: Vec<(i8, i8)>,
}

#[derive(Component, Deserialize, Serialize, Clone, Debug)]
pub struct Consumable {}

#[derive(Component, ConvertSaveload, Clone, Debug)]
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

#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct EventWantsToUseItem {
    pub item: Entity,
    pub target: Option<Point>,
}

#[derive(Component, ConvertSaveload, Debug, Clone)]
pub struct EventWantsToDropItem {
    pub item: Entity,
}

#[derive(Component, ConvertSaveload, Debug, Clone)]
pub struct EventWantsToMelee {
    pub target: Entity,
}

#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct EventWantsToPickupItem {
    pub collected_by: Entity,
    pub item: Entity,
}

#[derive(Component, ConvertSaveload, Debug)]
pub struct InBackpack {
    pub owner: Entity,
}

#[derive(Component, ConvertSaveload, Debug)]
pub struct InflictsDamage {
    pub damage: u16,
}

#[derive(Component, Deserialize, Serialize, Clone, Debug)]
pub struct Item {}

#[derive(Component, Deserialize, Serialize, Clone, Debug)]
pub struct Monster {}

#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct Name {
    pub name: String,
}

#[derive(Component, Deserialize, Serialize, Clone, Debug)]
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

// FIXME: ConvertSaveload is not working here; maybe check newer releases of the tutorial
#[derive(Component, Clone, Copy, ConvertSaveload, Debug, PartialEq)]
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

#[derive(Component, Clone, ConvertSaveload, Debug)]
pub struct ProvidesHealing {
    pub heal_amount: u16,
}

#[derive(Copy, Clone, ConvertSaveload, Debug)]
pub struct AbilityRange(pub u16);

#[derive(Component, Clone, ConvertSaveload, Debug)]
pub struct Ranged {
    pub range: AbilityRange,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Deserialize, Serialize, Debug)]
pub enum RenderOrder {
    First,
    Second,
    Last,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Renderable {
    pub glyph: FontCharType,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order: RenderOrder,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct SerializationHelper {
    pub map: Map,
}

pub struct SerializeMe;

#[derive(Copy, Clone, ConvertSaveload, Debug)]
pub struct ViewRange(pub i32);
#[derive(Component, Clone, ConvertSaveload)]
pub struct Viewshed {
    pub visible_tiles: Vec<Point>,
    pub range: ViewRange,
    pub dirty: bool,
}
