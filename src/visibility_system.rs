use specs::prelude::*;
use super::{Viewshed, Position, Map};
use rltk::{field_of_view, Point};


pub struct VisibilitySystemOld {}

impl <'a> System<'a> for VisibilitySystemOld {
    // ReadExpect är en assert, krashar annars
    type SystemData = (ReadExpect<'a, Map>,
                       WriteStorage<'a, Viewshed>,
                       WriteStorage<'a, Position>);

    fn run(&mut self, data : Self::SystemData) {
        let (map, mut viewshed, pos) = data;

        for (viewshed,pos) in (&mut viewshed, &pos).join() {
            // rensa visible tiles
            viewshed.visible_tiles.clear();
            // arg1 är currentpos, lägg till range
            // TODO: vad fan är &*? referens, dereference och sen unwrap?
            viewshed.visible_tiles = field_of_view(Point::new(pos.x, pos.y), viewshed.range, &*map);
            // filtrera, detta kallas closure (är bara en enkel filter-lambda)
            viewshed.visible_tiles.retain(
                |p| 
                    p.x >= 0 && 
                    p.x < map.width && 
                    p.y >= 0 && 
                    p.y < map.height);
        }
    }
}
