use std::time::Duration;
use std::f32::consts::PI;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_asset_loader::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::Rng;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_egui::egui::Color32;

use crate::skybox::{RotateSkyboxEvent, SkyboxPlugin};

mod orbitcamera;
mod gamedebug;
mod skybox;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameStates {
    #[default]
    Loading,
    Running
}

#[derive(AssetCollection, Resource)]
struct GameAssets {
    #[asset(path = "models/fighter.glb#Scene0")]
    fighter_scene: Handle<Scene>,
    #[asset(path = "models/planet.glb#Scene0")]
    planet_scene: Handle<Scene>,
    #[asset(path = "models/planet1.glb#Scene0")]
    planet_down_scene: Handle<Scene>,
    #[asset(path = "models/fighter2.glb#Scene0")]
    opponent_1_scene: Handle<Scene>,
    #[asset(path = "models/stonea.glb#Scene0")]
    opponent_2_scene: Handle<Scene>,
    #[asset(path = "models/tower.glb#Scene0")]
    tower_scene: Handle<Scene>,
    #[asset(path = "textures/tile01.png")]
    tile_1_texture: Handle<Image>,
    #[asset(path = "textures/tile02.png")]
    tile_2_texture: Handle<Image>,
    #[asset(path = "textures/tile03.png")]
    tile_3_texture: Handle<Image>,
    #[asset(path = "textures/tile04.png")]
    tile_4_texture: Handle<Image>,
    #[asset(path = "textures/tile05.png")]
    tile_5_texture: Handle<Image>,
    #[asset(path = "textures/tile06.png")]
    tile_6_texture: Handle<Image>,
    #[asset(path = "textures/tile07.png")]
    tile_7_texture: Handle<Image>,
    #[asset(path = "textures/tile08.png")]
    tile_8_texture: Handle<Image>
}

const SHIP_POSTION: Vec3 = Vec3::new(0.0, 0.0, -25.0);

#[derive(Resource)]
struct Level{
    value: usize
}

#[derive(Resource)]
struct SpanTimer(Timer);

#[derive(Component)]
struct Despawnable{
    min:f32,
    max:f32
}

#[derive(Component)]
struct Planet;

#[derive(Component)]
struct Ship{
    shields: f32,
    hits: i32
}

#[derive(Component)]
struct LaserGun{
    positions: Vec<Vec3>,
    color: Color,
    player: bool,
    fire: bool,
    cooldown: f32,
    std_cooldown: f32
}

#[derive(Component)]
struct Opponent;

#[derive(Component)]
struct Laser {
    player: bool
}

#[derive(Component)]
struct EffectTime {
    timer: Timer
}

#[derive(Event)]
struct CreateEffectEvent(Vec3);

fn main() {
    App::new()
        //add config resources
        .insert_resource(Msaa::Sample4)
        .insert_resource(ClearColor(Color::BLACK))

        .init_state::<GameStates>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "bevy fourth".to_string(),
                resolution: WindowResolution::new(920.0,  640.0),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_loading_state(
            LoadingState::new(GameStates::Loading)
                .continue_to_state(GameStates::Running)
        )
        .add_collection_to_loading_state::<_, GameAssets>(GameStates::Loading)
        .insert_resource(Level{value:1})
        .insert_resource(SpanTimer(Timer::from_seconds(2.0,TimerMode::Repeating)))
        .add_event::<CreateEffectEvent>()
        //.insert_resource(Score::default())
        //bevy itself

        //.add_plugin(EguiPlugin)
        .add_plugins((RapierPhysicsPlugin::<NoUserData>::default(),EguiPlugin,SkyboxPlugin))
        .add_systems(OnEnter(GameStates::Running), (setup_camera, setup))
        .add_systems(   Update, (move_ship, laser_player,laser_opponent,
                                                    spawn_laser, collision, create_effect,
                                                    remove_effect,create_ui, change_level,
                                 spawn_opponent,despawn_all).run_if(in_state(GameStates::Running)))
        .run();
}

fn setup_camera(
    mut commands: Commands
) {
    commands.
        spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 2.0, 0.0),
            ..Default::default()
        })
        .insert(Name::new("MainCamera"));

    commands.spawn(Camera3dBundle{
        camera: Camera{
            is_active:false,
            ..default()
        },
        ..default()
    })
        .insert(Name::new("OrbitCamera"));
}

fn setup(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
) {

    //light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 11.6, -15.1),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
            ..default()
        },
        ..default()
    });

    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.2,
    });

    //ship

    commands.spawn(SceneBundle {
        scene: game_assets.fighter_scene.clone(),
        transform:Transform {
            translation: SHIP_POSTION.clone(),
            scale: Vec3::new(1.0,1.0,1.0),
            //rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
            ..default()
        },
        ..Default::default()
    })
    .insert(RigidBody::KinematicVelocityBased)
    .insert(Velocity {
        linvel: Vec3::new(0.0, 0.0, 0.0),
        ..default()
    })
    .insert(Collider::cuboid(3.0,
                             1.0,
                             3.0))
    .insert(ActiveEvents::COLLISION_EVENTS)
    .insert(GravityScale(0.0))
    .insert(Name::new("Ship"))
    .insert(Ship{
        shields: 1.0,
        hits: 10
    })
    .insert(LaserGun{
        positions: vec!(
            Vec3::new(-1.0,0.0,0.0),
            Vec3::new(1.0,0.0,0.0)
        ),
        player: true,
        color: Color::LIME_GREEN,
        fire: false,
        std_cooldown: 0.2,
        cooldown:0.0,
    });

    //planet

    commands.spawn(SceneBundle {
        scene: game_assets.planet_scene.clone(),
        transform:Transform {
            translation: Vec3::new(-80.0,0.0,-320.0),
            scale: Vec3::new(16.0,16.0,16.0),
            //rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
            ..default()
        },
        ..Default::default()
    })
        .insert(Planet{})
        .insert(Name::new("Planet"));

    //planet down

    commands.spawn(SceneBundle {
        scene: game_assets.planet_down_scene.clone(),
        transform:Transform {
            translation: Vec3::new(0.0,-180.0,-146.0),
            scale: Vec3::new(128.0,128.0,128.0),
            //rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
            ..default()
        },
        ..Default::default()
    })
        .insert(Planet{})
        .insert(Name::new("Planet down"));

}


const MAXSPEED:f32 = 30.0;
const ACCELERATION:f32 = 0.75;

fn move_ship(
    keyboard_input:Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut Transform),With<Ship>>
){
    let (mut velo, mut transform) = query.single_mut();

    let horizontal = if keyboard_input.pressed(KeyCode::ArrowLeft) {
        -1.
    } else if keyboard_input.pressed(KeyCode::ArrowRight) {
        1.
    } else {
        0.0
    };
    let vertical:f32 = if keyboard_input.pressed(KeyCode::ArrowDown) {
        -1.
    } else if keyboard_input.pressed(KeyCode::ArrowUp) {
        1.
    } else {
        0.0
    };

    velo.linvel.x  = velo.linvel.x.lerp(horizontal * MAXSPEED, ACCELERATION);
    velo.linvel.y = velo.linvel.y.lerp(vertical * MAXSPEED, ACCELERATION);

    transform.rotation = Quat::from_euler( EulerRot::YXZ,
                                           (-velo.linvel.y / 2.0).to_radians(), //1.5*std::f32::consts::PI, //
                                           -(velo.linvel.y / 2.0).to_radians(),
                                           (velo.linvel.x * -1.0).to_radians());

    if transform.translation.x < -15.0
    {
        velo.linvel.x = 0.0;
        transform.translation.x = -15.0
    }
    if transform.translation.x > 15.0
    {
        velo.linvel.x = 0.0;
        transform.translation.x = 15.0
    }
    if transform.translation.y < -8.0
    {
        velo.linvel.y = 0.0;
        transform.translation.y = -8.0
    }
    if transform.translation.y > 10.0
    {
        velo.linvel.y = 0.0;
        transform.translation.y = 10.0
    }
}

const SPAWN_POS:Vec3 = Vec3::new(0.0,0.0,-300.0);

fn spawn_opponent(
    mut commands: Commands,
    time:Res<Time>,
    mut spawn_timer: ResMut<SpanTimer>,
    level: Res<Level>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_assets: Res<GameAssets>
){
    if spawn_timer.0.tick(time.delta()).just_finished() {

        let mut rng = rand::thread_rng();

        match level.value  {
            1 => {
                commands.spawn(SceneBundle {
                    scene: game_assets.opponent_1_scene.clone(),
                    transform: Transform {
                        translation: SPAWN_POS.clone() + Vec3::new(rng.gen_range(-15.0..15.0),
                                                                   rng.gen_range(-10.0..10.0),
                                                                   0.0),
                        ..default()
                    },
                    ..Default::default()
                })
                    .insert(RigidBody::Dynamic)
                    .insert(Velocity {
                        linvel: Vec3::new(0.0, 0.0, rng.gen_range(40.0..80.0)),
                        ..default()
                    })
                    .insert(Collider::cuboid(3.0, 3.0, 3.0))
                    .insert(ActiveEvents::COLLISION_EVENTS)
                    .insert(GravityScale(0.0))
                    .insert(Despawnable {
                        min: -1000.0,
                        max: 0.0
                    })
                    .insert(LaserGun {
                        positions: vec!(
                            Vec3::new(0.0, 0.0, 5.0)
                        ),
                        player: false,
                        color: Color::MIDNIGHT_BLUE,
                        fire: false,
                        cooldown: 0.0,
                        std_cooldown: rng.gen_range(0.4..2.0)
                    })
                    .insert(Name::new("Opponent"))
                    .insert(Opponent {});
            },
            2 => {
                let scene = game_assets.opponent_2_scene.clone();
                let factor = rng.gen_range(4.0..=28.0);
                let scale = Vec3::new(factor, factor, factor);

                let collider = Collider::ball(0.5);


                commands.spawn(SceneBundle {
                    scene: scene,
                    transform: Transform {
                        translation: SPAWN_POS.clone() + Vec3::new(rng.gen_range(-15.0..15.0),
                                                                   rng.gen_range(-10.0..10.0),
                                                                   0.0),
                        scale: scale,
                        ..default()
                    },
                    ..Default::default()
                })
                    .insert(RigidBody::Dynamic)
                    .insert(Velocity {
                        linvel: Vec3::new(0.0, 0.0, rng.gen_range(40.0..80.0)),
                        ..default()
                    })
                    .insert(collider)
                    .insert(ActiveEvents::COLLISION_EVENTS)
                    .insert(GravityScale(0.0))
                    .insert(Despawnable {
                        min: -1000.0,
                        max: 0.0
                    })
                    .insert(Name::new("Opponent"))
                    .insert(Opponent {});
            },
            3 => {
                let platform_length = 10.0;
                let platform_start = Vec3::new(0.0, -10.0, -240.0);
                let tiles_y_up = 10.0;
                let tiles_y_half_up = 5.0;
                let tiles_y_half_down = -5.0;
                let tiles_y_down = -10.0;
                let platform_tiles_y = vec![tiles_y_up, tiles_y_up, tiles_y_up,
                                            tiles_y_up, tiles_y_up, tiles_y_half_up,
                                            tiles_y_half_down, tiles_y_down, tiles_y_down,
                                            tiles_y_down,tiles_y_half_down,tiles_y_half_up,
                                            tiles_y_up, tiles_y_up, tiles_y_up,
                                            tiles_y_up, tiles_y_up];
                let mut y = 0;
                let platform_tiles_x = vec![-6.0*platform_length,-5.0*platform_length,-4.0*platform_length,
                                            -3.0*platform_length,-2.0*platform_length,-1.5*platform_length,
                                            -1.5*platform_length,-1.0*platform_length,0.0*platform_length,
                                            1.0* platform_length, 1.5* platform_length,1.5* platform_length,
                                            2.0* platform_length,3.0* platform_length,4.0* platform_length,
                                            5.0* platform_length,6.0* platform_length];
                let platform_tiles_rotate = vec![0.0,0.0,0.0,
                                                 0.0,0.0,PI*0.5,
                                                 PI*0.5,0.0,0.0,
                                                 0.0,PI*-0.5,PI*-0.5,
                                                 0.0,0.0,0.0,
                                                 0.0,0.0];

                let rnd_texture = rng.gen_range(1..=8);
                let texture_handle = match rnd_texture {
                     1 => game_assets.tile_1_texture.clone(),
                     2 => game_assets.tile_2_texture.clone(),
                     3 => game_assets.tile_3_texture.clone(),
                     4 => game_assets.tile_4_texture.clone(),
                     5 => game_assets.tile_5_texture.clone(),
                     6 => game_assets.tile_6_texture.clone(),
                     7 => game_assets.tile_7_texture.clone(),
                    _ => game_assets.tile_8_texture.clone()
                };

                let material_handle =
                    materials.add(StandardMaterial {
                    base_color_texture: Some(texture_handle),
                    ..Default::default()
                });

                for x in platform_tiles_x {
                    commands
                        .spawn(PbrBundle {
                            mesh: meshes.add(Mesh::from(shape::Box::new(platform_length,
                                                                        0.1, platform_length))),
                            material: material_handle.clone(),
                            transform: Transform {
                                translation: Vec3::new(platform_start.x+x,
                                                       platform_start.y+platform_tiles_y[y],
                                                       platform_start.z),
                                rotation: Quat::from_rotation_z(platform_tiles_rotate[y]),
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .insert(RigidBody::KinematicVelocityBased)
                        .insert(Velocity {
                            linvel: Vec3::new(0.0, 0.0, 24.0),
                            ..default()
                        })
                        .insert(Collider::cuboid(platform_length, 0.1, platform_length))
                        .insert(ActiveEvents::COLLISION_EVENTS)
                        .insert(GravityScale(0.0))
                        .insert(Despawnable {
                            min: -1000.0,
                            max: 0.0
                        });
                    y+=1;
                }
            }
            _ => {}
        }



    }
}

fn despawn_all(
    mut commands: Commands,
    mut query: Query<(Entity,&Transform, &Despawnable)>,
) {
    for (e, transform, limits) in query.iter_mut(){
        if transform.translation.z >= limits.max || transform.translation.z <= limits.min {
            commands.entity(e).despawn_recursive();
        }
    }
}

fn laser_player(
    keyboard_input:Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut LaserGun,With<Ship>>
){
    let mut laser_gun = query.single_mut();
    if keyboard_input.pressed(KeyCode::Space) {
        laser_gun.fire = true;
    } else {
        laser_gun.fire = false;
    }
}

fn laser_opponent(
    mut query: Query<( &Transform, &mut LaserGun), With<Opponent>>,
    level: Res<Level>
){
    if level.value == 1 {
        for (transfrom, mut laser_gun) in query.iter_mut() {
            if transfrom.translation.z.abs() < 200.0 {
                laser_gun.fire = true;
            } else {
                laser_gun.fire = false;
            }
        }
    }
}

fn spawn_laser(
    mut commands: Commands,
    time:Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&Transform,&mut LaserGun)>
)
{
    for (transform, mut laser_gun) in query.iter_mut() {
        if laser_gun.fire {
            if laser_gun.cooldown <= 0.0 {
                laser_gun.cooldown = laser_gun.std_cooldown;
                for gun in &laser_gun.positions {
                    let linvel = if laser_gun.player {
                        transform.forward() * 600.0
                    } else {
                        transform.back() * 600.0
                    };
                    commands.spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Box::new(0.2, 0.2, 3.2))),
                        material: materials.add(StandardMaterial {
                            base_color: laser_gun.color.clone(),
                            emissive: laser_gun.color.clone(),
                            ..Default::default()
                        }),
                        transform: Transform {
                            translation: transform.translation.clone() + gun.clone(),
                            rotation: transform.rotation.clone(),
                            scale: Vec3::new(1.0, 1.0, 1.0),
                            ..default()
                        },
                        ..Default::default()
                    })
                        //.insert(Speed { value: 10.0 })
                        .insert(RigidBody::KinematicVelocityBased)
                        .insert(Sleeping::disabled())
                        .insert(Collider::cuboid(0.2 / 2.0,
                                                 0.2 / 2.0,
                                                 3.2 / 2.0))
                        .insert(Velocity {
                            linvel: linvel,
                            ..Default::default()
                        })
                        .insert(GravityScale(0.0))
                        .insert(Despawnable {
                            min: -1000.0,
                            max: 0.0
                        })
                        .insert(Name::new("Laser"))
                        .insert(Laser{
                            player: laser_gun.player
                        });
                }
            } else {
                laser_gun.cooldown -= time.delta_seconds();
            }
        }
    }
}

fn collision(
    mut collision_events: EventReader<CollisionEvent>,
    mut query_opponent: Query<(Entity,&mut Transform, &Opponent), Without<Laser>>,
    query_laser: Query<(Entity, &Transform, &Laser)>,
    mut query_ship: Query<(Entity, &mut Ship)>,
    mut event_create_effect: EventWriter<CreateEffectEvent>,
    mut commands: Commands
){
    let (entity_ship, mut ship) = query_ship.single_mut();
    for e in collision_events.read(){
        //println!("Collision");
        for (entity_opponent, opponent_transform, _opponent) in query_opponent.iter_mut() {
            match e {
                CollisionEvent::Started(e1, e2, _) => {
                    if e1 == &entity_opponent || e2 == &entity_opponent{
                        if e1 == &entity_ship || e2 == &entity_ship {
                            // Ship -- Opponent
                            ship.shields -= 0.10;
                            event_create_effect.send(CreateEffectEvent(Vec3::from(opponent_transform.translation)));
                            commands.entity(entity_opponent).despawn_recursive();

                        } else {
                            for (entity_laser, _, laser) in query_laser.iter() {
                                if e1 == &entity_laser || e2 == &entity_laser {
                                    if laser.player {
                                        // Laser -- Opponent
                                        ship.hits -= 1;
                                        event_create_effect.send(CreateEffectEvent(Vec3::from(opponent_transform.translation)));
                                        commands.entity(entity_laser).despawn_recursive();
                                        commands.entity(entity_opponent).despawn_recursive();
                                    }
                                }
                            }
                        }
                    } else {
                        if e1 == &entity_ship || e2 == &entity_ship {
                            for (entity_laser, _, laser) in query_laser.iter() {
                                if e1 == &entity_laser || e2 == &entity_laser {
                                    if ! laser.player {
                                        // Laser -- Ship
                                        ship.shields -= 0.05;
                                        commands.entity(entity_laser).despawn_recursive();
                                    }
                                }
                            }
                        }
                    }
                }
                CollisionEvent::Stopped(_, _, _) => {}
            }
        }
    }
}

const EFFECT_SIZE:f32=0.1;
const EFFECT_TIME:f32=2.0;

fn create_effect(
    mut commands: Commands,
    mut event_create_effect: EventReader<CreateEffectEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
)
{
    let mut rng = rand::thread_rng();
    for event in event_create_effect.read() {
        let pos = event.0;
        for x in -2..2 {
            for y in 0..2 {
                for z in -2..2 {
                    commands
                        .spawn(PbrBundle {
                            mesh: meshes.add(Mesh::from(shape::Box::new(1.0, 1.0, 1.0))),
                            material: materials.add(StandardMaterial {
                                metallic: 0.5,
                                emissive: random_color().into(),
                                ..Default::default()
                            }),
                            transform: Transform {
                                translation: Vec3::new(x as f32 * EFFECT_SIZE+pos.x,
                                                       y as f32 * EFFECT_SIZE+pos.y,
                                                       z as f32 * EFFECT_SIZE+pos.z),
                                rotation: Quat::from_rotation_x(0.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .insert(RigidBody::KinematicVelocityBased)
                        .insert(Velocity {
                            linvel: Vec3::new(rng.gen_range(-100.0..100.0),
                                              rng.gen_range(-100.0..100.0),
                                              rng.gen_range(-100.0..100.0)),
                            ..Default::default()
                        })
                        .insert(EffectTime{
                            timer: Timer::from_seconds(EFFECT_TIME,TimerMode::Once)
                        })
                        .insert(Sleeping::disabled());
                        //.insert(Collider::cuboid(1.0 / 2.0, 1.0 / 2.0, 1.0 / 2.0));
                }
            }
        }
    }
}

fn random_color()->Color {
    let mut rng = rand::thread_rng();
    Color::rgb(rng.gen_range(0.0..1.0),
        rng.gen_range(0.0..1.0),
        rng.gen_range(0.0..1.0))
}

fn remove_effect(
    mut commands: Commands,
    time:Res<Time>,
    mut query: Query<(Entity, &mut EffectTime)>
)
{
    for (entity, mut timer) in query.iter_mut() {
        timer.timer.tick(time.delta());
        if timer.timer.just_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/*fn ui_example_system(mut contexts: EguiContexts) {
    egui::Window::new("Hello").show(contexts.ctx_mut(), |ui| {
        ui.label("world");
    })*/

fn create_ui(
    mut egui_context: EguiContexts,
    query: Query<&Ship>
) {

    if let Ok(ship) = query.get_single() {
        // do something with the components
        let my_frame = egui::containers::Frame {
            fill: Color32::from_rgba_premultiplied(0, 0, 0, 0),
            ..Default::default()
        };

        egui::CentralPanel::default().frame(my_frame)
            //egui::Window::new("Properties")
            .show(egui_context.ctx_mut(), |ui| {
                let mut style = (*ui.ctx().style()).clone();
                // Redefine text_styles
                style.text_styles = [
                    (egui::TextStyle::Heading, egui::FontId::new(30.0, egui::FontFamily::Proportional)),
                    (egui::TextStyle::Name("Heading2".into()), egui::FontId::new(25.0, egui::FontFamily::Proportional)),
                    (egui::TextStyle::Name("Context".into()), egui::FontId::new(23.0, egui::FontFamily::Proportional)),
                    (egui::TextStyle::Body, egui::FontId::new(24.0, egui::FontFamily::Proportional)),
                    (egui::TextStyle::Monospace, egui::FontId::new(14.0, egui::FontFamily::Proportional)),
                    (egui::TextStyle::Button, egui::FontId::new(24.0, egui::FontFamily::Proportional)),
                    (egui::TextStyle::Small, egui::FontId::new(10.0, egui::FontFamily::Proportional)),
                ].into();
                style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke{
                    color:egui::Color32::WHITE,
                    width: 5.0
                }  ;
                // Mutate global style with above changes
                ui.ctx().set_style(style);
                ui.horizontal(|ui| {
                    ui.add_sized([70.0, 40.0],egui::Label::new("Shield:"));
                    let progress_bar = egui::ProgressBar::new(ship.shields)
                        .show_percentage();
                    ui.add_sized([400.0, 40.0], progress_bar);
                    ui.allocate_space(egui::Vec2::new(20.0, 40.0));
                    //ui.add(progress_bar);
                    ui.label("To Hits:");
                    ui.text_edit_singleline( &mut format!("{}",ship.hits).as_str());
                });
            });
    }

}

const CHANGE_LEVEL_HITS:i32 = 10;

fn change_level(
    mut commands: Commands,
    mut spawn_timer: ResMut<SpanTimer>,
    mut event_rotate_skybox:
    EventWriter<RotateSkyboxEvent>,
    mut level: ResMut<Level>,
    mut query_ship: Query<&mut Ship>,
    query_planet: Query<Entity,With<Planet>>,
    mut query_opponent: Query<(Entity, &Opponent)>,
){
    let mut ship = query_ship.single_mut();
    if  ship.hits <= 0 {
        ship.hits = CHANGE_LEVEL_HITS;
        level.value += 1;
        match level.value {
            2 => {
                //let mut state = fixed_timesteps.get(TIME_LABEL).unwrap();
                //rotate sky
                event_rotate_skybox.send(RotateSkyboxEvent());
                //remove planets
                for e in query_planet.iter(){
                    commands.entity(e).despawn_recursive();
                };
                spawn_timer.0.set_duration(Duration::from_secs_f32(0.1))
            },
            3 => {
                //rotate sky
                event_rotate_skybox.send(RotateSkyboxEvent());
                // despawn all opponents
                for (e, _) in query_opponent.iter_mut(){
                    commands.entity(e).despawn_recursive();
                }
                //platform
                spawn_timer.0.set_duration(Duration::from_secs_f32(0.4));
            }
            _ =>{}
        };
    }
}
