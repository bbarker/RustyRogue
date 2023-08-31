use bracket_lib::random::RandomNumberGenerator;

#[derive(Clone, Copy)]
pub struct RandomEntry<T> {
    pub spawner: T,
    pub weight: u16,
}

#[derive(Clone)]
pub struct RandomTable<T> {
    pub entries: Vec<(RandomEntry<T>, u16)>,
    pub total_weight: u16,
}

impl<T> RandomTable<T>
where
    T: Clone,
{
    pub fn new() -> Self {
        RandomTable {
            entries: Vec::new(),
            total_weight: 0,
        }
    }

    pub fn add(mut self, spawn: T, weight: u16) -> Self {
        let new_total_weight = self.total_weight + weight;
        if weight > 0 {
            self.entries.push((
                RandomEntry {
                    spawner: spawn,
                    weight,
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
