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

#[derive(Eq, PartialEq, Hash, Serialize, Deserialize, Clone, Debug)]
pub enum Infix {
    None,
    //
    Short,
    Broad,
    Long,
    Bastard,
    Great,
    //
    Buckler,
    Round,
    Heater,
    Kite,
    Tower,
}
impl Display for Infix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Infix::None => write!(f, ""),
            //
            Infix::Short => write!(f, "Short"),
            Infix::Broad => write!(f, "Broad"),
            Infix::Long => write!(f, "Long"),
            Infix::Bastard => write!(f, "Bastard"),
            Infix::Great => write!(f, "Great"),
            //
            Infix::Buckler => write!(f, "Buckler"),
            Infix::Round => write!(f, "Round"),
            Infix::Heater => write!(f, "Heater"),
            Infix::Kite => write!(f, "Kite"),
            Infix::Tower => write!(f, "Tower"),
        }
    }
}

macro_attr! {
    #[derive(Eq, PartialEq, Hash, Serialize, Deserialize, Clone, Debug, EnumDisplay!)]
    pub enum Material {
        Wood,
        Stone,
        Copper,
        Tin,
        Bronze,
        Iron,
        Steel,
        Silver,
        Gold,
        Platinum,
        Titanium,
        DamascusSteel,
        Diamond,
    }
}

impl Material {
    // Note: We may want to override these bonuses for specific equipment types
    // or for specific interactions (e.g. silver vs. undead)
    pub fn bonus(&self) -> i16 {
        match self {
            Material::Wood => 0,
            Material::Stone => 1,
            Material::Copper => 1,
            Material::Tin => 0,
            Material::Bronze => 2,
            Material::Iron => 3,
            Material::Steel => 4,
            Material::Silver => 3,
            Material::Gold => 4,
            Material::Platinum => 5,
            Material::Titanium => 5,
            Material::DamascusSteel => 6,
            Material::Diamond => 6,
        }
    }

    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            Material::Wood => (102, 51, 0),
            Material::Stone => (105, 105, 105),
            Material::Copper => (184, 115, 51),
            Material::Tin => (211, 212, 213),
            Material::Bronze => (205, 127, 50),
            Material::Iron => (67, 70, 75),
            Material::Steel => (203, 205, 205),
            Material::Silver => (192, 192, 192),
            Material::Gold => (255, 215, 0),
            Material::Platinum => (229, 228, 226),
            Material::Titanium => (135, 134, 129),
            Material::DamascusSteel => (100, 100, 110),
            Material::Diamond => (185, 242, 255),
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
    pub quality: u8,
    pub special_power_modifier: i16,
    pub special_defense_modifier: i16,
}

impl Equipment {
    pub fn new(
        slot: EquipSlotAllowed,
        equipment_type: EquipmentType,
        material: Material,
        quality: u8,
    ) -> Self {
        Equipment {
            allowed_slots: slot,
            equipment_type,
            material,
            quality,
            special_power_modifier: 0,
            special_defense_modifier: 0,
        }
    }

    fn sword_infix(quality: u8) -> Infix {
        match quality {
            0 => Infix::Short,
            1 => Infix::Broad,
            2 => Infix::Long,
            3 => Infix::Bastard,
            _ => Infix::Great,
        }
    }

    fn shield_infix(quality: u8) -> Infix {
        match quality {
            0 => Infix::Buckler,
            1 => Infix::Round,
            2 => Infix::Heater,
            3 => Infix::Kite,
            _ => Infix::Tower,
        }
    }

    fn infix(&self) -> Infix {
        match &self.equipment_type {
            EquipmentType::Weapon(weapon_type) => match weapon_type {
                WeaponType::Melee(melee_weapon_type) => match melee_weapon_type {
                    MeleeWeaponType::Sword => Self::sword_infix(self.quality),
                    _ => Infix::None,
                },
                WeaponType::Ranged(_, _) => Infix::None,
            },
            EquipmentType::Shield => Self::shield_infix(self.quality),
            _ => Infix::None,
        }
    }

    pub fn name(&self) -> String {
        let infix = match self.infix() {
            Infix::None => " ".to_string(),
            ifx => format!(" {} ", ifx),
        };
        format!("{}{}{}", self.material, infix, self.equipment_type)
    }

    pub fn power_bonus(&self) -> i16 {
        let derived_bonus = match self.equipment_type {
            EquipmentType::Weapon(_) => self.equipment_type.bonus() + self.material.bonus(),
            _ => 0,
        };
        derived_bonus + self.special_power_modifier
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

pub type EntityEquipmentMap = HashMap<EquipSlot, (Equipment, Entity)>;

pub fn get_equipped_items<I: Join, E: Join>(
    entities: &Entities,
    items: I,
    equipped: E,
    owner: Entity,
) -> EntityEquipmentMap
where
    I::Type: IsItem,
    E::Type: IsEquipped,
{
    let mut equipped_items = HashMap::new();
    // Get all Equipped items and join with Items and filter those by the owner
    (entities, items, equipped)
        .join()
        .map(|(ent, item, eqpd)| (ent, item.from(), eqpd.from()))
        .filter(|(_, _, eqpd)| eqpd.owner == owner)
        .filter_map(|(ent, item, eqpd)| match item {
            Item::Equippable(equipment) => Some(((equipment, ent), eqpd)),
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
