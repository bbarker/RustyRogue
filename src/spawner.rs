use bracket_lib::{random::RandomNumberGenerator, terminal::*};
use itertools::Itertools;
use specs::{
    prelude::*,
    saveload::{MarkedBuilder, SimpleMarker},
};

use crate::util::*;
use crate::{components::*, equipment::*, map::Map, random_table::*, rect::Rect, State};
use EquipmentType::*;
use MeleeWeaponType::*;
use WeaponType::*;

const INIT_MAX_SPAWN: u16 = 5;

type SimpleSpawner<'a> = dyn CloneableFnAB<&'a mut World, Position, Entity> + 'a;

struct WorldEntityData {
    name: String,
    renderable: Renderable,
}

fn base_renderable_entity(
    ecs: &mut World,
    pos_opt: Option<Position>,
    base_data: WorldEntityData,
) -> EntityBuilder {
    let ethereal_entity = ecs
        .create_entity()
        .with(base_data.renderable)
        .with(Name {
            name: base_data.name,
        })
        .marked::<SimpleMarker<SerializeMe>>();
    if let Some(pos) = pos_opt {
        ethereal_entity.with(pos)
    } else {
        ethereal_entity
    }
}

fn sentient_entity(
    ecs: &mut World,
    pos: Position,
    base_data: WorldEntityData,
    view_range_opt: Option<ViewRange>,
) -> EntityBuilder {
    base_renderable_entity(ecs, Some(pos), base_data).with(Viewshed {
        visible_tiles: Vec::new(),
        range: view_range_opt.unwrap_or(ViewRange(8)),
        dirty: true,
    })
}

fn combat_entity(
    ecs: &mut World,
    pos: Position,
    base_data: WorldEntityData,
    view_range_opt: Option<ViewRange>,
    combat_stats: CombatStats,
) -> EntityBuilder {
    sentient_entity(ecs, pos, base_data, view_range_opt).with(combat_stats)
}

fn non_blocking_entity(
    ecs: &mut World,
    pos: Position,
    base_data: WorldEntityData,
) -> EntityBuilder {
    base_renderable_entity(ecs, Some(pos), base_data)
}

fn consumable_entity(ecs: &mut World, pos: Position, base_data: WorldEntityData) -> EntityBuilder {
    non_blocking_entity(ecs, pos, base_data)
        .with(Item::Consumable)
        .with(Consumable {})
}

fn equippable_entity(
    ecs: &mut World,
    pos: Position,
    base_data: WorldEntityData,
    item: Equipment,
) -> EntityBuilder {
    non_blocking_entity(ecs, pos, base_data).with(Item::Equippable(item))
}

pub fn dagger_at_level(map_depth: i32, ecs: &mut World, pos: Position) -> Entity {
    let (dagger_material, dagger_quality) = {
        let rng = &mut ecs.write_resource::<RandomNumberGenerator>();
        (
            random_blade_material(rng, map_depth),
            random_quality(rng, map_depth),
        )
    };
    let eq_item = Equipment::new(
        ONE_HANDED,
        Weapon(Melee(Dagger)),
        dagger_material.clone(),
        dagger_quality,
    );
    equippable_entity(
        ecs,
        pos,
        WorldEntityData {
            name: eq_item.name(),
            renderable: Renderable {
                glyph: bracket_lib::prelude::to_cp437('/'),
                fg: RGB::named(dagger_material.color()),
                bg: RGB::named(BLACK),
                render_order: RenderOrder::First,
            },
        },
        eq_item,
    )
    .build()
}

pub fn dagger<'a>(map_depth: i32) -> Box<SimpleSpawner<'a>> {
    Box::new(move |ecs, pos| dagger_at_level(map_depth, ecs, pos))
}

pub fn sword_at_level(map_depth: i32, ecs: &mut World, pos: Position) -> Entity {
    let (sword_material, sword_quality) = {
        let rng = &mut ecs.write_resource::<RandomNumberGenerator>();
        (
            random_blade_material(rng, map_depth),
            random_quality(rng, map_depth),
        )
    };
    let eq_item = Equipment::new(
        ONE_HANDED,
        Weapon(Melee(Sword)),
        sword_material.clone(),
        sword_quality,
    );
    equippable_entity(
        ecs,
        pos,
        WorldEntityData {
            name: eq_item.name(),
            renderable: Renderable {
                glyph: bracket_lib::prelude::to_cp437('│'),
                fg: RGB::named(sword_material.color()),
                bg: RGB::named(BLACK),
                render_order: RenderOrder::First,
            },
        },
        eq_item,
    )
    .build()
}

pub fn sword<'a>(map_depth: i32) -> Box<SimpleSpawner<'a>> {
    Box::new(move |ecs, pos| sword_at_level(map_depth, ecs, pos))
}

pub fn random_quality(rng: &mut RandomNumberGenerator, map_depth: i32) -> u8 {
    let roll = rng.range(0, 100);
    let weighted_roll = roll + 3 * map_depth;
    match weighted_roll {
        0..=50 => 0,
        51..=70 => 1,
        71..=82 => 2,
        83..=93 => 3,
        94..=98 => 4,
        _ => 5,
    }
}

pub fn random_blade_material(rng: &mut RandomNumberGenerator, map_depth: i32) -> Material {
    let roll = rng.range(0, 100);
    let weighted_roll = roll + 3 * map_depth;
    match weighted_roll {
        0..=40 => Material::Wood,
        41..=50 => Material::Stone,
        51..=56 => Material::Copper,
        57..=60 => Material::Bronze,
        61..=70 => Material::Iron,
        71..=75 => Material::Steel,
        94..=98 => Material::Titanium,
        _ => Material::DamascusSteel,
    }
}

pub fn random_shield_material(rng: &mut RandomNumberGenerator, map_depth: i32) -> Material {
    fn depth_table<'a>(map_depth: i32) -> RandomTable<'a, Material> {
        let map_depth_u16 = <u32 as TryInto<u16>>::try_into(map_depth.unsigned_abs()).unwrap();
        RandomTable::<'a, Material>::new(Material::Wood, 40_u16.saturating_sub(3 * map_depth_u16))
            .add(Material::Copper, 20_u16.saturating_sub(2 * map_depth_u16))
            .add(Material::Bronze, 20_u16.saturating_sub(map_depth_u16))
            .add(Material::Iron, 10_u16.saturating_add(map_depth_u16))
            .add(Material::Steel, 5_u16.saturating_add(2 * map_depth_u16))
    }

    let mat_table = depth_table(map_depth);
    mat_table.roll(rng)
}

pub fn shield_at_level(map_depth: i32, ecs: &mut World, pos: Position) -> Entity {
    let (shield_material, shield_quality) = {
        let rng = &mut ecs.write_resource::<RandomNumberGenerator>();
        (
            random_shield_material(rng, map_depth),
            random_quality(rng, map_depth),
        )
    };
    let eq_item = Equipment::new(OFF_HAND, Shield, shield_material.clone(), shield_quality);
    equippable_entity(
        ecs,
        pos,
        WorldEntityData {
            name: eq_item.name(),
            renderable: Renderable {
                glyph: bracket_lib::prelude::to_cp437('◙'),
                fg: RGB::named(shield_material.color()),
                bg: RGB::named(BLACK),
                render_order: RenderOrder::First,
            },
        },
        eq_item,
    )
    .build()
}

pub fn shield<'a>(map_depth: i32) -> Box<SimpleSpawner<'a>> {
    Box::new(move |ecs, pos| shield_at_level(map_depth, ecs, pos))
}

fn ranged_consumable_entity(
    ecs: &mut World,
    pos: Position,
    base_data: WorldEntityData,
    range: AbilityRange,
) -> EntityBuilder {
    consumable_entity(ecs, pos, base_data).with(Range { range })
}

pub fn player(gs: &mut State, position: Position) -> Entity {
    combat_entity(
        &mut gs.ecs,
        position,
        WorldEntityData {
            name: "Player".into(),
            renderable: Renderable {
                glyph: bracket_lib::prelude::to_cp437('@'),
                fg: RGB::named(YELLOW),
                bg: RGB::named(BLACK),
                render_order: RenderOrder::Last,
            },
        },
        Some(ViewRange(8)),
        CombatStats {
            max_hp: 30,
            hp: 30,
            defense: 2,
            power: 5,
        },
    )
    .with(Player {})
    // Note that player should not have BlocksTile; this appears to interfere with
    // the pathing algorithm used by mobs.
    .build()
}

pub fn health_potion(ecs: &mut World, position: Position) -> Entity {
    consumable_entity(
        ecs,
        position,
        WorldEntityData {
            name: "Health Potion".into(),
            renderable: Renderable {
                glyph: bracket_lib::prelude::to_cp437('¡'),
                fg: RGB::named(RED),
                bg: RGB::named(BLACK),
                render_order: RenderOrder::First,
            },
        },
    )
    .with(ProvidesHealing { heal_amount: 8 })
    .build()
}

pub fn fireball_scroll(ecs: &mut World, position: Position) -> Entity {
    ranged_consumable_entity(
        ecs,
        position,
        WorldEntityData {
            name: "Fireball Scroll".into(),
            renderable: Renderable {
                glyph: bracket_lib::prelude::to_cp437(')'),
                fg: RGB::named(ORANGE),
                bg: RGB::named(BLACK),
                render_order: RenderOrder::First,
            },
        },
        AbilityRange(6),
    )
    .with(InflictsDamage { damage: 20 })
    .with(AreaOfEffect { radius: 3 })
    .build()
}

pub fn magic_missile_scroll(ecs: &mut World, position: Position) -> Entity {
    ranged_consumable_entity(
        ecs,
        position,
        WorldEntityData {
            name: "Magic Missile Scroll".into(),
            renderable: Renderable {
                glyph: bracket_lib::prelude::to_cp437(')'),
                fg: RGB::named(CYAN),
                bg: RGB::named(BLACK),
                render_order: RenderOrder::First,
            },
        },
        AbilityRange(6),
    )
    .with(InflictsDamage { damage: 8 })
    .build()
}

pub fn confusion_scroll(ecs: &mut World, position: Position) -> Entity {
    let rand_turns = {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        rng.range(1, 7)
    };
    let steps: Vec<(i8, i8)> = {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        (0..rand_turns)
            .map(|_| (rng.range(-1, 2), rng.range(-1, 2)))
            .collect()
    };

    ranged_consumable_entity(
        ecs,
        position,
        WorldEntityData {
            name: "Confusion Scroll".into(),
            renderable: Renderable {
                glyph: bracket_lib::prelude::to_cp437(')'),
                fg: RGB::named(PINK),
                bg: RGB::named(BLACK),
                render_order: RenderOrder::First,
            },
        },
        AbilityRange(6),
    )
    .with(Confusion {
        step_sequence: steps,
    })
    .build()
}

pub fn room_table<'a, 'b>(map_depth: i32) -> RandomTable<'a, Box<SimpleSpawner<'b>>> {
    RandomTable::<'a, Box<SimpleSpawner<'b>>>::new(
        Box::new(health_potion) as Box<SimpleSpawner<'b>>,
        30,
    )
    .add(Box::new(fireball_scroll), 30)
    .add(Box::new(magic_missile_scroll), 40)
    .add(Box::new(confusion_scroll), 30)
    .add(
        Box::new(random_monster),
        120 + 2 * map_depth.unsigned_abs() as u16,
    )
    .add(dagger(map_depth), 10)
    .add(sword(map_depth), 5)
    .add(shield(map_depth), 10)
}

pub fn random_item(ecs: &mut World, position: Position) -> Entity {
    let map_depth = ecs.fetch::<Map>().depth;
    let spawn_table = room_table(map_depth);

    let random_spawner = spawn_table.roll(&mut ecs.write_resource::<RandomNumberGenerator>());
    // TODO: if we get a lot of items, may want to consider a search
    random_spawner(ecs, position)
}

pub fn random_monster(ecs: &mut World, position: Position) -> Entity {
    let map_depth = ecs.fetch::<Map>().depth;
    let pos_ix = {
        let map = ecs.read_resource::<Map>();
        map.pos_idx(position)
    };
    {
        let mut map = ecs.write_resource::<Map>();
        map.blocked[pos_ix] = true;
    }

    let roll = {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        rng.range(0, 100) // TODO: refactor monsters as an ADT?
                          // Some possibilities listed here: https://stackoverflow.com/questions/41637978/how-to-get-the-number-of-elements-variants-in-an-enum-as-a-constant-value
    };
    match roll + 2 * map_depth {
        0..=60 => goblin(ecs, position),
        61..=80 => orc(ecs, position),
        81..=95 => troll(ecs, position),
        _ => tarrasque(ecs, position),
    }
}

struct MonsterModifiers {
    pub damage: u16,
    pub defense: u16,
}

fn goblin(ecs: &mut World, position: Position) -> Entity {
    monster(
        ecs,
        position,
        bracket_lib::prelude::to_cp437('g'),
        "Goblin",
        RGB::named(RED),
        MonsterModifiers {
            damage: 0,
            defense: 0,
        },
    )
}

fn orc(ecs: &mut World, position: Position) -> Entity {
    monster(
        ecs,
        position,
        bracket_lib::prelude::to_cp437('o'),
        "Orc",
        RGB::named(GREEN),
        MonsterModifiers {
            damage: 0,
            defense: 1,
        },
    )
}

fn troll(ecs: &mut World, position: Position) -> Entity {
    monster(
        ecs,
        position,
        bracket_lib::prelude::to_cp437('t'),
        "Troll",
        RGB::named(BLUE),
        MonsterModifiers {
            damage: 1,
            defense: 1,
        },
    )
}

fn tarrasque(ecs: &mut World, position: Position) -> Entity {
    monster(
        ecs,
        position,
        bracket_lib::prelude::to_cp437('T'),
        "Tarrasque",
        RGB::named(YELLOW),
        MonsterModifiers {
            damage: 2,
            defense: 1,
        },
    )
}

fn monster<S: ToString>(
    ecs: &mut World,
    posn: Position,
    glyph: FontCharType,
    name: S,
    fg: RGB,
    mods: MonsterModifiers,
) -> Entity {
    combat_entity(
        ecs,
        posn,
        WorldEntityData {
            name: name.to_string(),
            renderable: Renderable {
                glyph,
                fg,
                bg: RGB::named(BLACK),
                render_order: RenderOrder::Second,
            },
        },
        Some(ViewRange(8)),
        CombatStats {
            max_hp: 16,
            hp: 16,
            defense: 1 + mods.defense,
            power: 4 + mods.damage,
        },
    )
    .with(Monster {})
    .with(BlocksTile {})
    .build()
}

pub fn spawn_room(ecs: &mut World, room: &Rect, map_depth: i32) -> Vec<Entity> {
    let num_entities = {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        rng.roll_dice(1, INIT_MAX_SPAWN as i32) + (map_depth.abs() - 1)
    }
    .try_into()
    .unwrap();
    spawn_in_room(ecs, room, num_entities, random_item)
}

/// Fills a room with monsters and items
pub fn spawn_in_room(
    ecs: &mut World,
    room: &Rect,
    num_indices: u16,
    spawn_fn: fn(&mut World, Position) -> Entity,
) -> Vec<Entity> {
    let fill_indices: Vec<usize> = {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let map = &mut ecs.fetch_mut::<Map>();

        let mut free_room_indices: Vec<usize> = (room.x1..=room.x2)
            .cartesian_product(room.y1..=room.y2)
            .map(|(x, y)| map.xy_idx(x, y))
            .filter(|ix| !map.blocked[*ix] && map.tile_content[*ix].is_empty())
            .collect();

        (0..num_indices)
            .map(|_| {
                let idx = rng.range(0, free_room_indices.len());
                free_room_indices.remove(idx)
            })
            .collect()
    };

    fill_indices
        .iter()
        .map(|ix| {
            let pos = {
                let map = &mut ecs.fetch_mut::<Map>();
                map.idx_to_pos(*ix)
            };
            let entity = spawn_fn(
                ecs,
                Position {
                    xx: pos.xx,
                    yy: pos.yy,
                },
            );
            let map = &mut ecs.fetch_mut::<Map>();
            map.tile_content[*ix].push(entity);
            // TODO: ^ make our own entity wrapper to avoid having to remember to do this
            entity
        })
        .collect()
}
