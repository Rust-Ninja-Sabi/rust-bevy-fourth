use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    #[asset(path = "models/fighter.glb#Scene0")]
    pub fighter_scene: Handle<Scene>,
    #[asset(path = "models/planet.glb#Scene0")]
    pub planet_scene: Handle<Scene>,
    #[asset(path = "models/planet1.glb#Scene0")]
    pub planet_down_scene: Handle<Scene>,
    #[asset(path = "models/fighter2.glb#Scene0")]
    pub opponent_1_scene: Handle<Scene>,
    #[asset(path = "models/stonea.glb#Scene0")]
    pub opponent_2_scene: Handle<Scene>,
    #[asset(path = "models/tower.glb#Scene0")]
    pub tower_scene: Handle<Scene>,
    #[asset(path = "textures/tile01.png")]
    pub tile_1_texture: Handle<Image>,
    #[asset(path = "textures/tile02.png")]
    pub tile_2_texture: Handle<Image>,
    #[asset(path = "textures/tile03.png")]
    pub tile_3_texture: Handle<Image>,
    #[asset(path = "textures/tile04.png")]
    pub tile_4_texture: Handle<Image>,
    #[asset(path = "textures/tile05.png")]
    pub tile_5_texture: Handle<Image>,
    #[asset(path = "textures/tile06.png")]
    pub tile_6_texture: Handle<Image>,
    #[asset(path = "textures/tile07.png")]
    pub tile_7_texture: Handle<Image>,
    #[asset(path = "textures/tile08.png")]
    pub tile_8_texture: Handle<Image>,
}

#[derive(Resource)]
pub struct Level {
    pub value: usize,
}

impl Default for Level {
    fn default() -> Self {
        Self { value: 1 }
    }
}

#[derive(Resource)]
pub struct SpawnTimer(pub Timer);

impl Default for SpawnTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(2.0, TimerMode::Repeating))
    }
}