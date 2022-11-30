use super::{Monster, Position, Viewshed};
use bracket_lib::prelude::console;
use specs::prelude::*;

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    type SystemData = (
        ReadStorage<'a, Viewshed>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Monster>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (viewshed, pos, monster) = data;

        (&viewshed, &pos, &monster)
            .join()
            .for_each(|(_viewshed, _pos, _monster)| {
                console::log(&format!("Monster sees player"));
            });
    }
}
