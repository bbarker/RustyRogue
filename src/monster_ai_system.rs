use crate::components::PlayerPosition;

use super::{Monster, Viewshed};
use bracket_lib::prelude::console;
use specs::prelude::*;

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    type SystemData = (
        ReadStorage<'a, Viewshed>,
        ReadExpect<'a, PlayerPosition>,
        ReadStorage<'a, Monster>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (viewshed, ppos, monster) = data;

        (&viewshed, &monster)
            .join()
            .for_each(|(viewshed, _monster)| {
                if viewshed.visible_tiles.contains(&ppos.pos().to_point()) {
                    console::log(&format!("Monster sees player"));
                }
            });
    }
}
