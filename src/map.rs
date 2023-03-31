use rltk::{ RGB, Rltk, RandomNumberGenerator, BaseMap, Algorithm2D, Point };
use crate::Viewshed;

use super::{Rect,Player};
use std::cmp::{max, min};
use specs::prelude::*;

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall, Floor
}

#[derive(Default)]
pub struct Map {
    pub tiles : Vec<TileType>,
    pub rooms : Vec<Rect>,
    pub width : i32,
    pub height : i32,
    pub revealed_tiles : Vec<bool>,
    pub visible_tiles : Vec<bool>
}

impl Map {
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    fn apply_room_to_map(&mut self, room : &Rect) {
        for y in room.y1 +1 ..= room.y2 {
            for x in room.x1 + 1 ..= room.x2 {
                let idx = self.xy_idx(x, y);
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    fn apply_horizontal_tunnel(&mut self, x1:i32, x2:i32, y:i32) {
        for x in min(x1,x2) ..= max(x1,x2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < self.width as usize * self.height as usize {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    fn apply_vertical_tunnel(&mut self, y1:i32, y2:i32, x:i32) {
        for y in min(y1,y2) ..= max(y1,y2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < self.width as usize * self.height as usize {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    /// Makes a new map using the algorithm from http://rogueliketutorials.com/tutorials/tcod/part-3/
    /// This gives a handful of random rooms and corridors joining them together.
    pub fn new_map_rooms_and_corridors() -> Map {
        let mut map = Map{
            tiles : vec![TileType::Wall; 80*50],
            rooms : Vec::new(),
            width : 80,
            height: 50,
            revealed_tiles : vec![false; 80*50],
            visible_tiles : vec![false; 80*50]
        };

        const MAX_ROOMS : i32 = 30;
        const MIN_SIZE : i32 = 6;
        const MAX_SIZE : i32 = 10;

        let mut rng = RandomNumberGenerator::new();

        for _i in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, map.width - w - 1) - 1;
            let y = rng.roll_dice(1, map.height - h - 1) - 1;
            let new_room = Rect::new(x, y, w, h);
            let mut ok = true;
            for other_room in map.rooms.iter() {
                if new_room.intersect(other_room) { ok = false }
            }
            if ok {
                map.apply_room_to_map(&new_room);

                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = map.rooms[map.rooms.len()-1].center();
                    if rng.range(0,2) == 1 {
                        map.apply_horizontal_tunnel(prev_x, new_x, prev_y);
                        map.apply_vertical_tunnel(prev_y, new_y, new_x);
                    } else {
                        map.apply_vertical_tunnel(prev_y, new_y, prev_x);
                        map.apply_horizontal_tunnel(prev_x, new_x, new_y);
                    }
                }

                map.rooms.push(new_room);
            }
        }

        map
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx:usize) -> bool {
        self.tiles[idx] == TileType::Wall
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }

    fn point2d_to_index(&self, pt: Point) -> usize {
        let bounds = self.dimensions();
        ((pt.y * bounds.x) + pt.x)
            .try_into()
            .expect("Not a valid usize. Did something go negative?")
    }

    fn index_to_point2d(&self, idx: usize) -> Point {
        let bounds = self.dimensions();
        let w: usize = bounds
            .x
            .try_into()
            .expect("Not a valid usize. Did something go negative?");
        Point::new(idx % w, idx / w)
    }

    fn in_bounds(&self, pos: Point) -> bool {
        let bounds = self.dimensions();
        pos.x >= 0 && pos.x < bounds.x && pos.y >= 0 && pos.y < bounds.y
    }
}
pub fn draw_map(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();
    let mut y = 0;
    let mut x = 0;
    for (idx, tile) in map.tiles.iter().enumerate() {
        // rendera beroende på typ
        if map.revealed_tiles[idx] {
            match tile {
                TileType::Floor => {
                    ctx.set(
                        x, 
                        y, 
                        RGB::from_f32(0.5, 0.5, 0.5), 
                        RGB::from_f32(0., 0., 0.), rltk::to_cp437('.'));
                }
                TileType::Wall => {
                    ctx.set(
                        x, 
                        y, 
                        RGB::from_f32(0.0, 1.0, 0.0), 
                        RGB::from_f32(0., 0., 0.), rltk::to_cp437('#'));
                }
            }
        }
        // flytta coords
        x += 1;
        if x > 79 {
            x = 0;
            y += 1;
        }

    }
}
pub fn draw_map_old(ecs: &World, ctx : &mut Rltk) {
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let mut players = ecs.write_storage::<Player>();
    let map = ecs.fetch::<Map>();

    for(_player, viewshed) in (&mut players, &mut viewsheds).join() {
        let mut y = 0;
        let mut x = 0;
        // rendera tile beroende på typ
       for tile in map.tiles.iter() {
            let pt = Point::new(x,y);
            if viewshed.visible_tiles.contains(&pt){
                match tile {
                    TileType::Floor => {
                        ctx.set(
                            x,
                            y,
                            RGB::from_f32(0.5, 0.5, 0.5),
                            RGB::from_f32(0., 0., 0.), rltk::to_cp437('.')
                        )
                    }
                    TileType::Wall => {
                        ctx.set(
                            x, 
                            y, 
                            RGB::from_f32(0.0, 1.0, 0.0), 
                            RGB::from_f32(0., 0., 0.), rltk::to_cp437('#')
                        )
                    }
                }
            }
            // flytta coords
            x += 1;
            if x > 79 {
                x = 0;
                y  += 1;
            }
        }
    }
}
