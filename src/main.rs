// use/using
// rltk är roguelike-engine
use rltk::{Rltk, GameState, RGB, VirtualKeyCode};
// specs är ECS
use specs::{prelude::*, world, rayon::iter::Positions};
// derive är macros för att bara skriva #[derive(Component)]
use specs_derive::Component;
// standard library compare
use std::cmp::{max, min};
// all state / data, lite ledsen att det inte hanteras som immutable
struct State {
    ecs: World
}

// komponenter / data som representerar all data för entities
#[derive(Component)]
struct Position {
    x: i32,
    y: i32,
}
#[derive(Component)]
struct Renderable{
    glyph: rltk::FontCharType,
    fg: RGB,
    bg: RGB,
}
#[derive(Component)]
struct LeftMover {}

#[derive(Component, Debug)]
struct Player {}

impl<'a> System<'a> for LeftMover {
    type SystemData = (ReadStorage<'a, LeftMover>, 
                        WriteStorage<'a, Position>);

    fn run(&mut self, (lefty, mut pos) : Self::SystemData) {
        for (_lefty,pos) in (&lefty, &mut pos).join() {
            pos.x -= 1;
            if pos.x < 0 { pos.x = 79; }
        }
    }
}

impl State {
    fn run_systems(&mut self){
        let mut lm = LeftMover{};
        lm.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    
    for (_player, pos) in (&mut players, &mut positions).join() {
        pos.x = min(79, max(0, pos.x + delta_x));
        pos.y = min(49, max(0, pos.y + delta_y));
    }
}

fn player_input(gs: &mut State, ctx: &mut Rltk) {
    // player movement
    match ctx.key {
        None => {} // inget händer
        Some(key) => match key {
            VirtualKeyCode::Left => try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::Right => try_move_player(1, 0, &mut gs.ecs),
            VirtualKeyCode::Up => try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::Down => try_move_player(0, 1, &mut gs.ecs),
            _ => {}
        }
    }
}

// implementation av structen state. 
// objekt har couplead funktionalitet med data men här är det funktioner
// som kan agera mot data
impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls(); // rensa skärm
        player_input(self, ctx);
        self.run_systems();
        // hämta read-only alla entities som har en position-component
        let positions = self.ecs.read_storage::<Position>(); 
        // hämta read-only alla entities som har en renderable-component
        let renderables = self.ecs.read_storage::<Renderable>();
        // hämta alla entities som har både en position och renderable-component
        // (blir senare "renderering-system")
        for (pos, render) in (&positions, &renderables).join() {
            // printa dem till skärmen
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    } 
}

// main med returntype BError
fn main() -> rltk::BError {
    // tror den hjälper till med builder-pattern, aka .with().with().build()
    use rltk::RltkBuilder;
    // skapa en mutable world state, gs = gamestate
    let mut gs = State {
        ecs: World::new()
    };
    // lägg till så systemet vet vad en position / renderable är
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<LeftMover>();
    gs.ecs.register::<Player>();
    // skapa en entity med position och renderable
    // senare läggs nog en "tag" till som säger "player"
    gs.ecs.create_entity()
        .with(Position{x:40, y:40})
        .with(Renderable{
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player{})
        .build();
    // skapar 10 "monster"
    for i in 0..=5 {
        gs.ecs
        .create_entity()
        .with(Position{x: i * 3 + 30, y: 20})
        .with(Renderable{
            glyph: rltk::to_cp437('r'),
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
        })
        .with(LeftMover{})
        .build();
    }
    // skapar fönster och sköter all magi med rendering
    let context = RltkBuilder::simple80x50()
        .with_title("Rustlike")
        .build()?;
    // starta game-loopen med rendering-data/context och gamestate / world
    rltk::main_loop(context, gs)
}
