#[cfg(not(target_arch = "wasm32"))]
use std::fs::File;
use std::{
    fs::{self},
    path::Path,
};

use bevy::prelude::*;
use bevy::utils::hashbrown::HashMap;
use bevy_serde_macros::*;

use crate::execute_with_type_list;
use crate::{components::*, delete_state};

use serde_json::Value;
const SAVE_FILE: &str = "savegame.json";

#[cfg(target_arch = "wasm32")]
pub fn save_game(_ecs: &mut World) {
    bracket_lib::terminal::console::log("Saving is not supported on the web");
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_game(ecs: &mut World) {
    //let map_data = serde_json::to_string(&*ecs.fetch::<Map>()).unwrap();
    //println!("map data:\n{}", map_data);

    let mapcopy = ecs.get_resource_mut::<super::map::Map>().unwrap().clone();
    let save_helper = ecs
        .spawn((SerializeMe {}, SerializationHelper { map: mapcopy }))
        .id();
    {
        let writer = File::create(SAVE_FILE).unwrap();
        let mut serializer = serde_json::Serializer::new(writer);
        execute_with_type_list!(serialize_individually!(ecs, serializer, SerializeMe));
    }

    // TODO: fix for bevy_ecs
    match ecs.despawn(save_helper) {
        true => (),
        false => eprintln!("Unable to delete serialization helper entity"),
    }
}

pub fn does_save_exist() -> bool {
    Path::new(SAVE_FILE).exists()
}

// loading

pub fn load_game(ecs: &mut World) {
    // Delete everything
    delete_state(ecs);
    let save_file_contents =
        fs::read(SAVE_FILE).unwrap_or_else(|_| panic!("Unable to read file {}", SAVE_FILE));

    let mut entity_map = HashMap::new();
    let mut component_value_map: HashMap<String, Value> =
        serde_json::from_slice(&save_file_contents).unwrap();
    {
        execute_with_type_list!(deserialize_individually!(
            ecs,
            &mut entity_map,
            &mut component_value_map,
            SerializeMe
        ));
    }
    // TODO: continue from here, and rewrite query for bevy
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

#[cfg(test)]
mod tests {
    use bevy::prelude::World;

    fn test_serialization() {
        let mut world = World::new();
    }
}
