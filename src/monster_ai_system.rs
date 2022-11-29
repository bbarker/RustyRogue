use super::{xy_idx, Map, Monster, Player, Position, PsnU, Viewshed};
use bracket_lib::prelude::{console, field_of_view, Point};
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
            .for_each(|(viewshed, pos, _monster)| {
                console::log(&format!("Monster sees player"));
            });
    }
}
