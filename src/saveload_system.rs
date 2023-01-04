use specs::World;

use crate::map::Map;

pub fn save_game(ecs: &mut World) {
    let map_data = serde_json::to_string(&*ecs.fetch::<Map>()).unwrap();
    println!("map data:\n{}", map_data);
}
