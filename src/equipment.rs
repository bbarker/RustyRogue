// TODO: how to structure in item "slot"? Let's keep reading and see how it is used.
// The book has this in the Equippable component

// TODO: think about when we want to use the ECS, vs when we want to use ADTs
// ECS makes it easy to query - but an ADT makes it easy to compare values, or
// calculate properties for multiple componenents of the same item; e.g., item value
//
// An approach that might work is to define an item type as an ADT, and then
// interpret (build) values of the ADT into ECS components. We can then create a
// reference back to the original item type for any of the relevant components.

use serde::{Deserialize, Serialize};
use specs::{
    prelude::*,
    saveload::{ConvertSaveload, Marker},
};

use specs_derive::*;

use std::convert::Infallible;
// `NoError` alias is deprecated in specs ... but specs_derive needs it
pub type NoError = Infallible;

#[derive(Clone, Debug, Deserialize, Serialize)]

pub enum Slot {
    Head,
    Neck,
    Torso,
    Ring,
    Hand,
    Feet,
    MainHand,
    OffHand,
}

pub enum AllowedSlot {
    SingleSlot(Slot),
    Either(Slot, Slot),
    Both(Slot, Slot),
}

const TWO_HANDED: AllowedSlot = AllowedSlot::Both(Slot::MainHand, Slot::OffHand);
const ONE_HANDED: AllowedSlot = AllowedSlot::Both(Slot::MainHand, Slot::OffHand);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Equipment {
    Weapon,
    Shield,
}
