// use/using
// rltk är roguelike-engine
use rltk::{Rltk, GameState, RGB};
// specs är ECS
use specs::prelude::*;
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

// implementation av structen state. 
// objekt har couplead funktionalitet med data men här är det funktioner
// som kan agera mot data
impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls(); // rensa skärm
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
    // skapa en entity med position och renderable
    // senare läggs nog en "tag" till som säger "player"
    gs.ecs.create_entity()
        .with(Position{x:40, y:40})
        .with(Renderable{
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .build();
    // skapar 10 "monster"
    for i in 0..=10 {
        gs.ecs
        .create_entity()
        .with(Position{x: i * 7, y: 20})
        .with(Renderable{
            glyph: rltk::to_cp437('r'),
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
        })
        .build();
    }
    // skapar fönster och sköter all magi med rendering
    let context = RltkBuilder::simple80x50()
        .with_title("Rustlike")
        .build()?;
    // starta game-loopen med rendering-data/context och gamestate / world
    rltk::main_loop(context, gs)
}
