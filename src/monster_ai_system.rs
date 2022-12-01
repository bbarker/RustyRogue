use crate::components::{Name, PlayerPosition};

use super::{Monster, Viewshed};
use bracket_lib::prelude::console;
use specs::prelude::*;

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    type SystemData = (
        ReadExpect<'a, PlayerPosition>,
        ReadStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Name>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (ppos, viewsheds, monsters, names) = data;

        (&viewsheds, &monsters, &names)
            .join()
            .for_each(|(viewshed, _monster, name)| {
                if viewshed.visible_tiles.contains(&ppos.pos().to_point()) {
                    console::log(format!("{} sees player", name.name));
                }
            });
    }
}
