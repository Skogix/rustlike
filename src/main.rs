// use/using
// rltk är roguelite-engine
use rltk::{GameState, Rltk, RGB, VirtualKeyCode};
// specs är ECS
use specs::prelude::*;
use specs_derive::*;
// cmp är compare
use std::cmp::{max, min};
// #[derive(x)] är macros
// gör så man slipper definiera components
#[derive(Component)]
struct Position {
    x: i32,
    y: i32,
}
#[derive(Component)]
struct Renderable {
    glyph: rltk::FontCharType,
    fg: RGB,
    bg: RGB,
}
#[derive(Component, Debug)]
struct Player {}
#[derive(PartialEq, Copy, Clone)]
enum TileType {
    Wall, Floor
}
// world-state, inte immutable men får väl stå ut
struct State {
    ecs: World
}
// skapar en unsigned-lista med coords
// bra för cache-missar
pub fn xy_idx(x: i32, y: i32) -> usize {
    (y as usize * 80) + x as usize
}
// skapar en ny map som en vec av tiletypes
fn new_map() -> Vec<TileType> {
    // 80*50 från rltk:s standard-upplösning
    let mut map = vec![TileType::Floor; 80*50];

    // skapa väggar
    for x in 0..80 {
        map[xy_idx(x, 0)] = TileType::Wall;
        map[xy_idx(x, 49)] = TileType::Wall;
    }
    for y in 0..50 {
        map[xy_idx(0, y)] = TileType::Wall;
        map[xy_idx(79, y)] = TileType::Wall;
    }

    // hämtar en rng
    // tydligen så går det emot rusts regler och följer d&d-regler och är inklusiv
    let mut rng = rltk::RandomNumberGenerator::new();
    // sätter randomväggar slumpat över mappen
    for _i in 0..400 {
        let x = rng.roll_dice(1, 79);
        let y = rng.roll_dice(1, 49);
        let idx = xy_idx(x, y);
        if idx != xy_idx(40, 25) {
            map[idx] = TileType::Wall;
        }
    }

    map
}

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    // hämtar mutable kombo av pos och players
    // är bara player som hämtas och dess pos
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    // fetchar immutable map
    let map = ecs.fetch::<Vec<TileType>>();
    // loopar genom resultat men är player och dess pos
    for (_player, pos) in (&mut players, &mut positions).join() {
        // target pos
        let destination_idx = xy_idx(pos.x + delta_x, pos.y + delta_y);
        // om target inte är immovable/wall så move
        if map[destination_idx] != TileType::Wall {
            pos.x = min(79 , max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));
        }
    }
}

fn player_input(gs: &mut State, ctx: &mut Rltk) {
    // movement
    match ctx.key {
        None => {} // inget händer
        Some(key) => match key { // flytta beroende på input
            VirtualKeyCode::Left => try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::Right => try_move_player(1, 0, &mut gs.ecs),
            VirtualKeyCode::Up => try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::Down => try_move_player(0, 1, &mut gs.ecs),
            _ => {} // basecase
        },
    }
}
// render / draw-system för map
// TODO: varför hämtar vi mutable rltk?
fn draw_map(map: &[TileType], ctx : &mut Rltk) {
    let mut y = 0;
    let mut x = 0;
    for tile in map.iter() {
        // rendera tiles beronde på typ
        match tile {
            TileType::Floor => {
                ctx.set(x, y, RGB::from_f32(0.5, 0.5, 0.5), RGB::from_f32(0., 0., 0.), rltk::to_cp437('.'));
            }
            TileType::Wall => {
                ctx.set(x, y, RGB::from_f32(0.0, 1.0, 0.0), RGB::from_f32(0., 0., 0.), rltk::to_cp437('#'));
            }
        }

        // Move the coordinates
        // vi kör något fulhaxx-array som är cache-friendly, så loopa och öka y när x är maxxat
        x += 1;
        if x > 79 {
            x = 0;
            y += 1;
        }
    }
}

// implementation för state
impl GameState for State {
    // vanlig gametick/update
    fn tick(&mut self, ctx : &mut Rltk) {
        ctx.cls(); // clear screen
        player_input(self, ctx); // hämta input som start
        self.run_systems(); // kör system
        let map = self.ecs.fetch::<Vec<TileType>>(); 
        draw_map(&map, ctx); // efter system är körda så draw map 

        // efter map så dra entities med pos/renderable ovanpå
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}
// state påverkas och kör systems
impl State {
    fn run_systems(&mut self) {
        self.ecs.maintain();
    }
}

// main returnar BError men annars vanlig main
fn main() -> rltk::BError {
    // ger oss builder-pattern
    use rltk::RltkBuilder;
    // context / ctx är rendering och os-specifikt
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;
    // inita gamestate
    let mut gs = State {
        ecs: World::new()
    };
    // regga alla components
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    // regga resources
    gs.ecs.insert(new_map());
    // skapa entities
    gs.ecs
        .create_entity()
        .with(Position { x: 40, y: 25 })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player{})
        .build();
    // kör main-loop med context och gamestate
    rltk::main_loop(context, gs)
}
