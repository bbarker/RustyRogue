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

use std::convert::Infallible;

use crate::components::*;
// `NoError` alias is deprecated in specs ... but specs_derive needs it
pub type NoError = Infallible;

#[derive(ConvertSaveload, Clone, Debug)]
pub enum EquipmentType {
    Weapon(WeaponType),
    Shield,
    Armor,
    Accessory,
}

// TODO: each weapon type could have certain modifiers, applied to its base
// stats
#[derive(ConvertSaveload, Clone, Debug)]
pub enum WeaponType {
    Melee(MeleeWeaponType),
    Ranged(RangedWeaponType, Range),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MeleeWeaponType {
    Axe,
    Mace,
    Sword,
    Dagger,
    Staff,
    Polearm,
    Whip,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum RangedWeaponType {
    Bow,
    Crossbow,
    Thrown,
}

// TODO: add combat stats to equipment
#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct Equipment {
    pub equipment_type: EquipmentType,
    pub slot: EquipSlotAllowed,
}

impl Equipment {
    pub fn new(slot: EquipSlotAllowed, equipment_type: EquipmentType) -> Self {
        Equipment {
            slot,
            equipment_type,
        }
    }
}
