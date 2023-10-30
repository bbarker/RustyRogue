use bevy::prelude::*;

#[derive(Resource)]
pub struct GameLog {
    pub entries: Vec<String>,
}
