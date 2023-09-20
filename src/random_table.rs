use bracket_lib::random::RandomNumberGenerator;

use std::marker::PhantomData;

#[derive(Clone, Copy)]
pub struct RandomEntry<'a, T> {
    pub spawner: T,
    pub weight: u16,
    phantom: PhantomData<&'a T>,
}

#[derive(Clone)]
pub struct RandomTable<'a, T> {
    pub entries: Vec<(RandomEntry<'a, T>, u16)>,
    pub total_weight: u16,
}

impl<'a, T> RandomTable<'a, T>
where
    T: Clone,
{
    pub fn new(init: T, init_weight: u16) -> RandomTable<'a, T> {
        RandomTable {
            entries: vec![(
                RandomEntry {
                    spawner: init,
                    weight: init_weight,
                    phantom: PhantomData,
                },
                init_weight,
            )],
            total_weight: init_weight,
        }
    }

    pub fn add(mut self, spawn: T, weight: u16) -> Self {
        let new_total_weight = self.total_weight + weight;
        if weight > 0 {
            self.entries.push((
                RandomEntry {
                    spawner: spawn,
                    weight,
                    phantom: PhantomData,
                },
                new_total_weight,
            ))
        }
        RandomTable {
            entries: self.entries,
            total_weight: new_total_weight,
        }
    }

    pub fn roll(&self, rng: &mut RandomNumberGenerator) -> T {
        let roll = rng.range(0, self.total_weight);

        // TODO: if we get a lot of items, may want to consider a search
        self.entries
            .iter()
            .find(|(_, weight)| roll < *weight)
            .unwrap()
            .0
            .spawner
            .clone()
    }
}
