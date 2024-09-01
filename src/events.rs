use bevy::prelude::Event;
use bevy::math::Vec3;

#[derive(Event)]
pub struct CreateEffectEvent(pub Vec3);