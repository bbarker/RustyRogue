use bracket_lib::{
    prelude::{FontCharType, RGB},
    terminal::Point,
};
use specs::{
    prelude::*,
    saveload::{ConvertSaveload, Marker},
    Entity,
};

use crate::{
    equipment::{EntityEquipmentMap, EquipSlot, EquipSlotAllowed, Equipment},
    map::Map,
    PsnU,
};
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

#[derive(Eq, PartialEq, Hash, Clone, Component, ConvertSaveload, Debug)]
pub struct Equipped {
    pub owner: Entity,
    pub slot: EquipSlot,
    pub slot_extra: Option<EquipSlot>, // 2H weapons, etc.
}

impl Equipped {
    pub fn new(
        owner: Entity,
        equip_map: &EntityEquipmentMap,
        slot_allowed: &EquipSlotAllowed,
    ) -> Self {
        match slot_allowed {
            EquipSlotAllowed::SingleSlot(slot) => Equipped {
                owner,
                slot: slot.clone(),
                slot_extra: None,
            },
            EquipSlotAllowed::Both(slot1, slot2) => Equipped {
                owner,
                slot: slot1.clone(),
                slot_extra: Some(slot2.clone()),
            },
            EquipSlotAllowed::Either(slot1, slot2) => {
                // We assume new items are generally better, so preferentally equip it in
                // the primary slot (slot1) to create a convention
                let slot = if equip_map.get(slot1).is_some() {
                    if equip_map.get(slot2).is_some() {
                        slot1
                    } else {
                        slot2
                    }
                } else {
                    slot1
                };

                Equipped {
                    owner,
                    slot: slot.clone(),
                    slot_extra: None,
                }
            }
        }
    }
}

pub trait IsEquipped {
    fn from(self) -> Equipped;
}

impl<T> IsEquipped for &T
where
    T: IsEquipped + Clone,
{
    fn from(self) -> Equipped {
        self.clone().from()
    }
}

impl IsEquipped for Equipped {
    fn from(self) -> Equipped {
        self
    }
}

pub trait HasOwner {
    fn owner(&self) -> Entity;
    fn into_has_owner(self) -> Box<dyn HasOwner>;
}

impl HasOwner for InBackpack {
    fn owner(&self) -> Entity {
        self.owner
    }
    fn into_has_owner(self) -> Box<dyn HasOwner> {
        Box::new(self)
    }
}

impl HasOwner for Equipped {
    fn owner(&self) -> Entity {
        self.owner
    }
    fn into_has_owner(self) -> Box<dyn HasOwner> {
        Box::new(self)
    }
}

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

#[derive(Component, ConvertSaveload, Debug, Clone)]
pub struct EventWantsToRemoveItem {
    pub item: Entity,
}

#[derive(Clone, Component, ConvertSaveload, Debug)]
pub struct InBackpack {
    pub owner: Entity,
}

pub trait IsInBackpack {
    fn from(self) -> InBackpack;
}

impl<T> IsInBackpack for &T
where
    T: IsInBackpack + Clone,
{
    fn from(self) -> InBackpack {
        self.clone().from()
    }
}

impl IsInBackpack for InBackpack {
    fn from(self) -> InBackpack {
        self
    }
}

#[derive(Component, ConvertSaveload, Debug)]
pub struct InflictsDamage {
    pub damage: u16,
}

#[derive(Eq, PartialEq, Hash, Component, ConvertSaveload, Clone, Debug)]
pub enum Item {
    Consumable,
    Equippable(Equipment), // Note: In book this is a component
}

impl Item {
    pub fn equip_opt(&self) -> Option<&Equipment> {
        match self {
            Item::Equippable(eqp) => Some(eqp),
            _ => None,
        }
    }
}

pub trait IsItem {
    fn from(self) -> Item;
}

impl<T> IsItem for &T
where
    T: IsItem + Clone,
{
    fn from(self) -> Item {
        self.clone().from()
    }
}

impl IsItem for Item {
    fn from(self) -> Item {
        self
    }
}

#[derive(Component, Deserialize, Serialize, Clone, Debug)]
pub struct Monster {}

#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct Name {
    pub name: String,
}

const DEBUG_NAME: &str = "<no name for entity>";

pub fn debug_name() -> Name {
    Name {
        name: DEBUG_NAME.to_string(),
    }
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

#[derive(Eq, PartialEq, Hash, Copy, Clone, ConvertSaveload, Debug)]
pub struct AbilityRange(pub u16);

#[derive(Eq, PartialEq, Hash, Component, Clone, ConvertSaveload, Debug)]
pub struct Range {
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

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct WantsToUnequipItem {
    pub item: Entity,
}

// see https://users.rust-lang.org/t/how-to-store-a-list-tuple-of-types-that-can-be-uses-as-arguments-in-another-macro/87891
// credit to Michael F. Bryan for this approach
#[macro_export]
macro_rules! execute_with_type_list {
    ($name:ident!($($arg:tt)*)) => {
        $name!(
          $($arg)*,
          AreaOfEffect,
          BlocksTile,
          CombatStats,
          Confusion,
          Consumable,
          Equipped,
          EventIncomingDamage,
          EventWantsToDropItem,
          EventWantsToMelee,
          EventWantsToPickupItem,
          EventWantsToRemoveItem,
          EventWantsToUseItem,
          InBackpack,
          InflictsDamage,
          Item,
          Monster,
          Name,
          Player,
          Position,
          ProvidesHealing,
          Range,
          Renderable,
          SerializationHelper,
          Viewshed,
          WantsToUnequipItem,
        )
    }
  }

#[macro_export]
macro_rules! register_individually {
    ($ecs:expr, $( $type:ty),*, $(,)?) => {
        $(
        $ecs.register::<$type>();
        )*
    };
  }
