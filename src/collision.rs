use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::{Ship, Opponent, Laser};
use crate::events::CreateEffectEvent;

pub fn handle_collisions(
    mut collision_events: EventReader<CollisionEvent>,
    mut query_opponent: Query<(Entity, &Transform, &mut Opponent)>,
    query_laser: Query<(Entity, &Transform, &Laser)>,
    mut query_ship: Query<(Entity, &mut Ship)>,
    mut event_create_effect: EventWriter<CreateEffectEvent>,
    mut commands: Commands,
) {
    let (ship_entity, mut ship) = query_ship.single_mut();

    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = collision_event {
            handle_collision(*e1, *e2, &ship_entity, &mut ship, &mut query_opponent, &query_laser, &mut event_create_effect, &mut commands);
        }
    }
}

fn handle_collision(
    e1: Entity,
    e2: Entity,
    ship_entity: &Entity,
    ship: &mut Ship,
    query_opponent: &mut Query<(Entity, &Transform, &mut Opponent)>,
    query_laser: &Query<(Entity, &Transform, &Laser)>,
    event_create_effect: &mut EventWriter<CreateEffectEvent>,
    commands: &mut Commands,
) {
    if let Some((opponent_entity, opponent_transform, mut opponent)) = query_opponent.iter_mut().find(|(e, _, _)| *e == e1 || *e == e2) {
        if e1 == *ship_entity || e2 == *ship_entity {
            handle_ship_opponent_collision(ship, opponent_entity, opponent_transform, event_create_effect, commands);
        } else {
            handle_laser_opponent_collision(e1, e2, &opponent_entity, &mut opponent, opponent_transform, query_laser, ship, event_create_effect, commands);
        }
    } else if e1 == *ship_entity || e2 == *ship_entity {
        handle_laser_ship_collision(e1, e2, ship, query_laser, commands);
    }
}

fn handle_ship_opponent_collision(
    ship: &mut Ship,
    opponent_entity: Entity,
    opponent_transform: &Transform,
    event_create_effect: &mut EventWriter<CreateEffectEvent>,
    commands: &mut Commands,
) {
    ship.shields -= 0.10;
    event_create_effect.send(CreateEffectEvent(opponent_transform.translation));
    commands.entity(opponent_entity).despawn_recursive();
}

fn handle_laser_opponent_collision(
    e1: Entity,
    e2: Entity,
    opponent_entity: &Entity,
    opponent: &mut Opponent,
    opponent_transform: &Transform,
    query_laser: &Query<(Entity, &Transform, &Laser)>,
    ship: &mut Ship,
    event_create_effect: &mut EventWriter<CreateEffectEvent>,
    commands: &mut Commands,
) {
    if let Some((laser_entity, _, laser)) = query_laser.iter().find(|(e, _, _)| *e == e1 || *e == e2) {
        if laser.player {
            opponent.max_hits -= 1;
            if opponent.max_hits <= 0 {
                ship.hits -= 1;
                event_create_effect.send(CreateEffectEvent(opponent_transform.translation));
                commands.entity(laser_entity).despawn_recursive();
                commands.entity(*opponent_entity).despawn_recursive();
            }
        }
    }
}

fn handle_laser_ship_collision(
    e1: Entity,
    e2: Entity,
    ship: &mut Ship,
    query_laser: &Query<(Entity, &Transform, &Laser)>,
    commands: &mut Commands,
) {
    if let Some((laser_entity, _, laser)) = query_laser.iter().find(|(e, _, _)| *e == e1 || *e == e2) {
        if !laser.player {
            ship.shields -= 0.05;
            commands.entity(laser_entity).despawn_recursive();
        }
    }
}