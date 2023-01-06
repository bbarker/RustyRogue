use std::{fs::File, path::Path};

use specs::{prelude::*, saveload::*, World, WorldExt};

use crate::components::*;

const SAVE_FILE: &str = "savegame.json";

macro_rules! serialize_individually {
  ($ecs:expr, $ser:expr, $data:expr, $( $type:ty),*) => {
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
        serialize_individually!(
            ecs,
            serializer,
            data,
            AreaOfEffect,
            BlocksTile,
            CombatStats,
            Confusion,
            Consumable,
            EventIncomingDamage,
            EventWantsToDropItem,
            EventWantsToMelee,
            EventWantsToPickupItem,
            EventWantsToUseItem,
            InBackpack,
            InflictsDamage,
            Item,
            Monster,
            Name,
            Player,
            Position,
            ProvidesHealing,
            Ranged,
            Renderable,
            SerializationHelper,
            Viewshed
        );
    }

    ecs.delete_entity(save_helper)
        .unwrap_or_else(|_| panic!("Unable to delete serialization helper entity"));
}

pub fn does_save_exist() -> bool {
    Path::new(SAVE_FILE).exists()
}
