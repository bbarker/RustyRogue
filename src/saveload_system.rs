use std::{
    fs::{self, File},
    path::Path,
};

use specs::{prelude::*, saveload::*, World, WorldExt};

use crate::components::*;

const SAVE_FILE: &str = "savegame.json";

// see https://users.rust-lang.org/t/how-to-store-a-list-tuple-of-types-that-can-be-uses-as-arguments-in-another-macro/87891
// credit to Michael F. Bryan for this approach
macro_rules! execute_with_type_list {
  ($name:ident!($($arg:tt)*)) => {
      $name!(
        $($arg)*,
        AreaOfEffect,
        BlocksTile,
        CombatStats,
        Confusion,
        Consumable,
        DefenseBonus,
        Equipped,
        EventIncomingDamage,
        EventWantsToDropItem,
        EventWantsToMelee,
        EventWantsToPickupItem,
        EventWantsToUseItem,
        InBackpack,
        InflictsDamage,
        Item,
        MeleePowerBonus,
        Monster,
        Name,
        Player,
        Position,
        ProvidesHealing,
        Range,
        Renderable,
        SerializationHelper,
        Viewshed,
      )
  }
}

macro_rules! serialize_individually {
  ($ecs:expr, $ser:expr, $data:expr, $( $type:ty),*, $(,)?) => {
      $(
      SerializeComponents::<NoError, SimpleMarker<SerializeMe>>::serialize(
          &( $ecs.read_storage::<$type>(), ),
          &$data.0,
          &$data.1,
          &mut $ser,
      )
      .unwrap();
      )*
  };
}

#[cfg(target_arch = "wasm32")]
pub fn save_game(_ecs: &mut World) {
    console.log("Saving is not supported on the web");
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_game(ecs: &mut World) {
    //let map_data = serde_json::to_string(&*ecs.fetch::<Map>()).unwrap();
    //println!("map data:\n{}", map_data);

    let mapcopy = ecs.get_mut::<super::map::Map>().unwrap().clone();
    let save_helper = ecs
        .create_entity()
        .with(SerializationHelper { map: mapcopy })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    {
        let data = (
            ecs.entities(),
            ecs.read_storage::<SimpleMarker<SerializeMe>>(),
        );

        let writer = File::create(SAVE_FILE).unwrap();
        let mut serializer = serde_json::Serializer::new(writer);
        execute_with_type_list!(serialize_individually!(ecs, serializer, data));
    }

    ecs.delete_entity(save_helper)
        .unwrap_or_else(|_| panic!("Unable to delete serialization helper entity"));
}

pub fn does_save_exist() -> bool {
    Path::new(SAVE_FILE).exists()
}

// loading

macro_rules! deserialize_individually {
  ($ecs:expr, $de_ser:expr, $data:expr, $( $type:ty),* $(,)?) => {
      $(
      DeserializeComponents::<NoError, _>::deserialize(
          &mut ( &mut $ecs.write_storage::<$type>(), ),
          &$data.0, // entities
          &mut $data.1, // marker
          &mut $data.2, // allocater
          &mut $de_ser,
      )
      .unwrap();
      )*
  };
}

pub fn load_game(ecs: &mut World) {
    // Delete everything
    let to_delete: Vec<Entity> = ecs.entities().join().collect();
    to_delete.iter().for_each(|entity| {
        ecs.delete_entity(*entity)
            .unwrap_or_else(|er| panic!("Unable to delete entity with id {}: {}", entity.id(), er))
    });
    let save_file_contents = fs::read_to_string(SAVE_FILE)
        .unwrap_or_else(|_| panic!("Unable to read file {}", SAVE_FILE));
    let mut de_ser = serde_json::Deserializer::from_str(&save_file_contents);
    {
        let mut de_ser_reqs = (
            ecs.entities(),
            ecs.write_storage::<SimpleMarker<SerializeMe>>(),
            ecs.write_resource::<SimpleMarkerAllocator<SerializeMe>>(),
        );
        execute_with_type_list!(deserialize_individually!(ecs, de_ser, de_ser_reqs));
    }

    let ser_helper_vec: Vec<Entity> = {
        let entities = ecs.entities();
        let helper = ecs.read_storage::<SerializationHelper>();
        (&entities, &helper)
            .join()
            .map(|(ent, help)| {
                // load the map
                let mut worldmap = ecs.write_resource::<super::map::Map>();
                *worldmap = help.map.clone();
                worldmap.tile_content = vec![Vec::new(); worldmap.tile_count()];
                ent
            })
            .collect()
    };
    // Delete serialization helper, so we don't keep an extra copy of it (and its contents)
    // each time we save.
    ser_helper_vec.into_iter().for_each(|help| {
        ecs.delete_entity(help)
            .unwrap_or_else(|er| panic!("Unable to delete helper: {}", er))
    });
}
