use bevy::prelude::*;
use enum_derive::EnumDisplay;
use serde::{Deserialize, Serialize};

use std::{collections::HashMap, convert::Infallible, fmt::Display};

use crate::components::*;
// `NoError` alias is deprecated in specs ... but specs_derive needs it
pub type NoError = Infallible;

#[derive(Eq, PartialEq, Hash, /* ConvertSaveload, */ Clone, Debug)]
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

#[derive(Eq, PartialEq, Hash, /* ConvertSaveload, */ Clone, Debug)]
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
    pub enum ItemMaterial {
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

impl ItemMaterial {
    // Note: We may want to override these bonuses for specific equipment types
    // or for specific interactions (e.g. silver vs. undead)
    pub fn bonus(&self) -> i16 {
        match self {
            ItemMaterial::Wood => 0,
            ItemMaterial::Stone => 1,
            ItemMaterial::Copper => 1,
            ItemMaterial::Tin => 0,
            ItemMaterial::Bronze => 2,
            ItemMaterial::Iron => 3,
            ItemMaterial::Steel => 4,
            ItemMaterial::Silver => 3,
            ItemMaterial::Gold => 4,
            ItemMaterial::Platinum => 5,
            ItemMaterial::Titanium => 5,
            ItemMaterial::DamascusSteel => 6,
            ItemMaterial::Diamond => 6,
        }
    }

    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            ItemMaterial::Wood => (102, 51, 0),
            ItemMaterial::Stone => (105, 105, 105),
            ItemMaterial::Copper => (184, 115, 51),
            ItemMaterial::Tin => (211, 212, 213),
            ItemMaterial::Bronze => (205, 127, 50),
            ItemMaterial::Iron => (67, 70, 75),
            ItemMaterial::Steel => (203, 205, 205),
            ItemMaterial::Silver => (192, 192, 192),
            ItemMaterial::Gold => (255, 215, 0),
            ItemMaterial::Platinum => (229, 228, 226),
            ItemMaterial::Titanium => (135, 134, 129),
            ItemMaterial::DamascusSteel => (100, 100, 110),
            ItemMaterial::Diamond => (185, 242, 255),
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

#[derive(Eq, PartialEq, Hash, Component, /* ConvertSaveload, */ Clone, Debug)]
pub struct Equipment {
    pub equipment_type: EquipmentType,
    pub allowed_slots: EquipSlotAllowed,
    pub material: ItemMaterial,
    pub quality: u8,
    pub special_power_modifier: i16,
    pub special_defense_modifier: i16,
}

impl Equipment {
    pub fn new(
        slot: EquipSlotAllowed,
        equipment_type: EquipmentType,
        material: ItemMaterial,
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

pub fn get_equipped_items(
    query: Query<(Entity, &Item, &Equipped)>,
    owner: Entity,
) -> EntityEquipmentMap {
    let mut equipped_items = HashMap::new();

    // Iterate over entities with Item and Equipped components
    query.iter().for_each(|(entity, item, equipped)| {
        if equipped.owner == owner {
            if let Item::Equippable(equipment) = item {
                equipped_items.insert(equipped.slot.clone(), (equipment.clone(), entity));
                if let Some(extra_slot) = equipped.slot_extra.clone() {
                    equipped_items.insert(extra_slot, (equipment.clone(), entity));
                }
            }
        }
    });

    equipped_items
}
