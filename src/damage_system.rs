use super::{CombatStats, Player, SufferDamage};
use rltk::console;
use specs::prelude::*;

/// system för att ta skada, döda osv
pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut stats, mut damage) = data;

        // applya skada
        for (mut stats, damage) in (&mut stats, &damage).join() {
            stats.hp -= damage.amount.iter().sum::<i32>();
        }

        // cleara ut all sufferdamage
        damage.clear();
    }
}

/// ta bort alla med -hp
pub fn delete_the_dead(ecs: &mut World) {
    let mut dead: Vec<Entity> = Vec::new();
    // använder scope för att få borrow checkern att sluta whinea
    // skulle gå att droppa borrow manuellt men den fattar att döda borrows utanför scope
    // varför?
    {
        let combat_stats = ecs.read_storage::<CombatStats>();
        let players = ecs.read_storage::<Player>();
        let entities = ecs.entities();
        for (entity, stats) in (&entities, &combat_stats).join() {
            if stats.hp < 1 {
                let player = players.get(entity);
                match player {
                    None => dead.push(entity), // döda andra entities
                    Some(_) => console::log("Player är död"),
                }
            }
        }
    }

    for victim in dead {
        ecs.delete_entity(victim).expect(&format!(
            "ERROR: gick inte att döda entity_id: {}",
            victim.id()
        ));
    }
}
