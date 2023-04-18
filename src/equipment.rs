// TODO: how to structure in item "slot"? Let's keep reading and see how it is used.
// The book has this in the Equippable component

// TODO: think about when we want to use the ECS, vs when we want to use ADTs
// ECS makes it easy to query - but an ADT makes it easy to compare values, or
// calculate properties for multiple componenents of the same item; e.g., item value
//
// An approach that might work is to define an item type as an ADT, and then
// interpret (build) values of the ADT into ECS components. We can then create a
// reference back to the original item type for any of the relevant components.
//
// In order to re-use exiting types and values, we can build up Equipment from
// certain componenets, such as the EquipSlot. Since we still need some for of
// reference to the parent from in the ADT from the component; traits won't help
// unless the original value stores a reference to the root - so a trait is useless
// here, except perhap to have a uniform interface across different ADTs.
//
// We can try a reference and see how it goes; i.e. every component that is a node
// in the ADT(type, but also tree) will have a reference to the root.
//
// As I found though, Serde can't serialize references, which throws a significant
// wrench into this plan. We may need to look into a custom serializer:
// 1. Serialize not the components, but the ADT (although the root ADT can already
//    be part of a component - e.g. Equipment is part of Item)
// 2. After deserializing the ADT, populate components that are children of the ADT
// 3. The 'Equipped' component would still need to be serialized separately

use serde::{Deserialize, Serialize};
use specs::{
    prelude::*,
    saveload::{ConvertSaveload, Marker},
};

use specs_derive::*;

use std::{collections::HashMap, convert::Infallible, fmt::Display};

use crate::components::*;
// `NoError` alias is deprecated in specs ... but specs_derive needs it
pub type NoError = Infallible;

#[derive(Eq, PartialEq, Hash, ConvertSaveload, Clone, Debug)]
pub enum EquipmentType {
    Weapon(WeaponType),
    Shield,
    Armor,
    Accessory,
}

impl EquipmentType {
    pub fn bonus(&self) -> i16 {
        match self {
            EquipmentType::Weapon(weapon_type) => weapon_type.bonus(),
            EquipmentType::Shield => 1,
            EquipmentType::Armor => 1,
            EquipmentType::Accessory => 1,
        }
    }
}

impl Display for EquipmentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EquipmentType::Weapon(weapon_type) => write!(f, "{}", weapon_type),
            EquipmentType::Shield => write!(f, "Shield"),
            EquipmentType::Armor => write!(f, "Armor"),
            EquipmentType::Accessory => write!(f, "Accessory"),
        }
    }
}

use enum_derive::EnumDisplay;

// TODO: each weapon type could have certain modifiers, applied to its base
// stats
#[derive(Eq, PartialEq, Hash, ConvertSaveload, Clone, Debug)]
pub enum WeaponType {
    Melee(MeleeWeaponType),
    Ranged(RangedWeaponType, Range),
}

impl WeaponType {
    pub fn bonus(&self) -> i16 {
        match self {
            WeaponType::Melee(weapon_type) => weapon_type.bonus(),
            WeaponType::Ranged(weapon_type, _) => weapon_type.bonus(),
        }
    }
}

impl Display for WeaponType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WeaponType::Melee(weapon_type) => write!(f, "{}", weapon_type),
            WeaponType::Ranged(weapon_type, _) => write!(f, "{}", weapon_type),
        }
    }
}

macro_attr! {
    #[derive(Eq, PartialEq, Hash, Serialize, Deserialize, Clone, Debug, EnumDisplay!)]
    pub enum MeleeWeaponType {
        Axe,
        Mace,
        Sword,
        Dagger,
        Staff,
        Polearm,
        Whip,
    }
}

impl MeleeWeaponType {
    pub fn bonus(&self) -> i16 {
        match self {
            MeleeWeaponType::Axe => 1,
            MeleeWeaponType::Mace => 1,
            MeleeWeaponType::Sword => 1,
            MeleeWeaponType::Dagger => 0,
            MeleeWeaponType::Staff => 0,
            MeleeWeaponType::Polearm => 1,
            MeleeWeaponType::Whip => 1,
        }
    }
}

macro_attr! {
    #[derive(Eq, PartialEq, Hash, Serialize, Deserialize, Clone, Debug, EnumDisplay!)]
    pub enum Material {
        Wood,
        Stone,
        Iron,
        Steel,
        Silver,
        Gold,
        Platinum,
        Diamond,
    }
}

impl Material {
    pub fn bonus(&self) -> i16 {
        match self {
            Material::Wood => 0,
            Material::Stone => 1,
            Material::Iron => 2,
            Material::Steel => 3,
            Material::Silver => 2,
            Material::Gold => 2,
            Material::Platinum => 4,
            Material::Diamond => 5,
        }
    }
}

macro_attr! {
#[derive(Eq, PartialEq, Hash, Serialize, Deserialize, Clone, Debug, EnumDisplay!)]
    pub enum RangedWeaponType {
        Bow,
        Crossbow,
        Thrown,
    }
}

impl RangedWeaponType {
    pub fn bonus(&self) -> i16 {
        match self {
            RangedWeaponType::Bow => 1,
            RangedWeaponType::Crossbow => 1,
            RangedWeaponType::Thrown => 0,
        }
    }
}

macro_attr! {
#[derive(PartialEq, Eq, Clone, Debug, Hash, Deserialize, Serialize, EnumDisplay!)]
    pub enum EquipSlot {
        Head,
        Neck,
        Torso,
        Ring,
        Hand,
        Feet,
        MainHand,
        OffHand,
    }
}

#[derive(Eq, PartialEq, Hash, Clone, Debug, Deserialize, Serialize)]
pub enum EquipSlotAllowed {
    SingleSlot(EquipSlot),
    Either(EquipSlot, EquipSlot),
    Both(EquipSlot, EquipSlot),
}

pub const TWO_HANDED: EquipSlotAllowed =
    EquipSlotAllowed::Both(EquipSlot::MainHand, EquipSlot::OffHand);
pub const ONE_HANDED: EquipSlotAllowed =
    EquipSlotAllowed::Either(EquipSlot::MainHand, EquipSlot::OffHand);
pub const OFF_HAND: EquipSlotAllowed = EquipSlotAllowed::SingleSlot(EquipSlot::OffHand);

// TODO: add combat stats to equipment
#[derive(Eq, PartialEq, Hash, Component, ConvertSaveload, Clone, Debug)]
pub struct Equipment {
    pub equipment_type: EquipmentType,
    pub allowed_slots: EquipSlotAllowed,
    pub material: Material,
}

impl Equipment {
    pub fn new(slot: EquipSlotAllowed, equipment_type: EquipmentType, material: Material) -> Self {
        Equipment {
            allowed_slots: slot,
            equipment_type,
            material,
        }
    }

    pub fn name(&self) -> String {
        format!("{} {}", self.material, self.equipment_type)
    }

    pub fn bonus(&self) -> i16 {
        self.equipment_type.bonus() + self.material.bonus()
    }

    pub fn is_2h(&self) -> bool {
        self.allowed_slots == TWO_HANDED
    }

    pub fn is_oh_capable(&self) -> bool {
        matches!(
            self.allowed_slots,
            EquipSlotAllowed::SingleSlot(EquipSlot::OffHand)
                | EquipSlotAllowed::Either(EquipSlot::OffHand, _)
                | EquipSlotAllowed::Either(_, EquipSlot::OffHand)
        )
    }
}

// TODO: we would ideally have shared references to a Map that is associated with
// the entity; this Map should probably be a component, since it could be
// associated with various entity types
//
// Then when we attempt to equip an item, we can check which slots are available
// for that entity.
//
// The alternative would be to query all equipment on the entity, and compute the map on the fly
// This would be more flexible, but also more expensive; however it relies on a single source of
// truth, which I like. It also avoids the need for any shared reference.
// We still need an Equipped component in order to associate the item with the entity
// equipping it.

pub type EntityEquipmentMap = HashMap<EquipSlot, Equipment>;

pub fn get_equipped_items<I: Join, E: Join>(
    items: I,
    equipped: E,
    entity: Entity,
) -> EntityEquipmentMap
where
    I::Type: IsItem,
    E::Type: IsEquipped,
{
    let mut equipped_items = HashMap::new();
    // Get all Equipped items and join with Items and filter those by the owner
    (items, equipped)
        .join()
        .map(|(item, eqpd)| (item.from(), eqpd.from()))
        .filter(|(_, eqpd)| eqpd.owner == entity)
        .filter_map(|(item, eqpd)| match item {
            Item::Equippable(equipment) => Some((equipment, eqpd)),
            _ => None,
        })
        .for_each(|(item, eqpd)| {
            equipped_items.insert(eqpd.slot, item.clone());
            if let Some(extra_slot) = eqpd.slot_extra {
                equipped_items.insert(extra_slot, item);
            }
        });

    equipped_items
}
