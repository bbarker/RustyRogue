use specs::{prelude::*, World};

use crate::components::Position;

// Keep this defiend in spawner.rs?
pub type SimpleSpawner = fn(&mut World, Position) -> Entity;

#[derive(Clone, Copy)]
pub struct RandomEntry {
    pub spawner: SimpleSpawner,
    pub weight: u16,
}

#[derive(Clone)]
pub struct RandomTable {
    pub entries: Vec<(RandomEntry, u16)>,
    pub total_weight: u16,
}

impl RandomTable {
    pub fn new() -> RandomTable {
        RandomTable {
            entries: Vec::new(),
            total_weight: 0,
        }
    }

    pub fn add(mut self, spawn: SimpleSpawner, weight: u16) -> RandomTable {
        let new_total_weight = self.total_weight + weight;
        self.entries.push((
            RandomEntry {
                spawner: spawn,
                weight,
            },
            new_total_weight,
        ));
        RandomTable {
            entries: self.entries,
            total_weight: new_total_weight,
        }
    }
}
