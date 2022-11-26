use super::{Map, Position, Viewshed};
use bracket_lib::prelude::field_of_view;
use specs::prelude::*;

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        ReadExpect<'a, Map>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (map, mut viewshed, pos) = data;

        (&mut viewshed, &pos).join().for_each(|(viewshed, pos)| {
            viewshed.visible_tiles.clear();
            viewshed.visible_tiles = field_of_view(pos.to_point(), viewshed.range, &*map);
            viewshed.visible_tiles.retain(|pt| {
                pt.x >= 0
                    && pt.x < map.width.try_into().unwrap()
                    && pt.y >= 0
                    && pt.y < map.height.try_into().unwrap()
            });
        })
    }
}
