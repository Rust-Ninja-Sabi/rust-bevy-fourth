use bevy::prelude::*;

#[derive(Component)]
pub struct Despawnable {
    pub min: f32,
    pub max: f32,
}

#[derive(Component)]
pub struct Planet;

#[derive(PartialEq)]
pub enum WinOrLostState {
    Win,
    Lost,
    Neutral,
}

#[derive(Component)]
pub struct Ship {
    pub shields: f32,
    pub hits: i32,
    pub win_or_lost: WinOrLostState,
}

#[derive(Component)]
pub struct LaserGun {
    pub positions: Vec<Vec3>,
    pub color: Color,
    pub player: bool,
    pub fire: bool,
    pub cooldown: f32,
    pub std_cooldown: f32,
}

#[derive(Component)]
pub struct Opponent {
    pub max_hits: i32,
}

#[derive(Component)]
pub struct Laser {
    pub player: bool,
}

#[derive(Component)]
pub struct EffectTime {
    pub timer: Timer,
}