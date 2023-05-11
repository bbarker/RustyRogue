use enum_derive::EnumDisplay;
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

#[derive(Eq, PartialEq, Hash, Component, ConvertSaveload, Clone, Debug)]
pub struct Equipment {
    pub equipment_type: EquipmentType,
    pub allowed_slots: EquipSlotAllowed,
    pub material: Material,
    pub special_melee_modifier: i16,
    pub special_defense_modifier: i16,
}

impl Equipment {
    pub fn new(slot: EquipSlotAllowed, equipment_type: EquipmentType, material: Material) -> Self {
        Equipment {
            allowed_slots: slot,
            equipment_type,
            material,
            special_melee_modifier: 0,
            special_defense_modifier: 0,
        }
    }

    pub fn name(&self) -> String {
        format!("{} {}", self.material, self.equipment_type)
    }

    pub fn melee_bonus(&self) -> i16 {
        let derived_bonus = match self.equipment_type {
            EquipmentType::Weapon(_) => self.equipment_type.bonus() + self.material.bonus(),
            _ => 0,
        };
        derived_bonus + self.special_melee_modifier
    }

    pub fn defense_bonus(&self) -> i16 {
        let derived_bonus = match self.equipment_type {
            EquipmentType::Armor | EquipmentType::Shield => {
                self.equipment_type.bonus() + self.material.bonus()
            }
            _ => 0,
        };
        derived_bonus + self.special_defense_modifier
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
