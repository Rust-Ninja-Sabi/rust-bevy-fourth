use bevy::prelude::*;
use bevy::time::FixedTimestep;
use bevy_rapier3d::prelude::*;
use rand::Rng;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_egui::egui::{Color32, Stroke};
use bevy_egui::egui::color_picker::color_edit_button_hsva;
use lerp::Lerp;

use gamedebug::GameDebugPlugin;

mod orbitcamera;
mod gamedebug;

const SHIP_POSTION: Vec3 = Vec3::new(0.0, 0.0, -25.0);

#[derive(Component)]
struct Despawnable{
    min:f32,
    max:f32
}

#[derive(Component)]
struct Ship{
    shields: f32,
    hits: usize
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

struct CreateEffectEvent(Vec3);

fn main() {
    App::new()
        //add config resources
        .insert_resource(Msaa {samples: 4})
        .insert_resource(WindowDescriptor{
            title: "bevy fourth".to_string(),
            width: 920.0,
            height: 640.0,
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::BLACK))
        .add_event::<CreateEffectEvent>()
        //.insert_resource(Score::default())
        //bevy itself
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(EguiPlugin)
        .add_plugin(GameDebugPlugin)
        .add_startup_system(setup_camera)
        .add_startup_system(setup)
        //.add_state(GameState::GameStart)
        .add_system(move_ship)
        .add_system(laser_player)
        .add_system(laser_opponent)
        .add_system(spawn_laser)
        .add_system(collision)
        .add_system(create_effect)
        .add_system(remove_effect)
        .add_system(create_ui)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(2.0))
                .with_system(spawn_opponent)
                .with_system(despawn_all)
        )
        .run();
}

fn setup_camera(
    mut commands: Commands
) {
    commands.
        spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 2.0, 0.0),
            ..Default::default()
        })
        .insert(UiCameraConfig {
            show_ui: true,
            ..default()
        })
        .insert(Name::new("MainCamera"));

    commands.spawn_bundle(Camera3dBundle{
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {

    //light
    commands.spawn_bundle(DirectionalLightBundle {
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

    commands.spawn_bundle(SceneBundle {
        scene: asset_server.load("models/fighter.glb#Scene0"),
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
        hits: 0
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

    commands.spawn_bundle(SceneBundle {
        scene: asset_server.load("models/planet.glb#Scene0"),
        transform:Transform {
            translation: Vec3::new(-80.0,0.0,-320.0),
            scale: Vec3::new(16.0,16.0,16.0),
            //rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
            ..default()
        },
        ..Default::default()
    })
        .insert(Name::new("Planet"));

    //planet down

    commands.spawn_bundle(SceneBundle {
        scene: asset_server.load("models/planet1.glb#Scene0"),
        transform:Transform {
            translation: Vec3::new(0.0,-180.0,-146.0),
            scale: Vec3::new(128.0,128.0,128.0),
            //rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
            ..default()
        },
        ..Default::default()
    })
        .insert(Name::new("Planet down"));

    //sky
    let store_texture_handle = asset_server.load("images/skybox_front1.png");
    let store_aspect = 1.0;

    let store_quad_width = 640.0;
    let store_quad_handle = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
        store_quad_width,
        store_quad_width * store_aspect,
    ))));

    let store_material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(store_texture_handle.clone()),
        unlit: true,
        ..Default::default()
    });

    commands.spawn_bundle(PbrBundle {
        mesh: store_quad_handle.clone(),
        material: store_material_handle,
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, -500.0),
            rotation: Quat::from_rotation_x(0.0),
            ..Default::default()
        },
        ..Default::default()
    });
}


const MAXSPEED:f32 = 30.0;
const ACCELERATION:f32 = 0.75;

fn move_ship(
    keyboard_input:Res<Input<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut Transform),With<Ship>>
){
    let (mut velo, mut transform) = query.single_mut();

    let horizontal = if keyboard_input.pressed(KeyCode::Left) {
        -1.
    } else if keyboard_input.pressed(KeyCode::Right) {
        1.
    } else {
        0.0
    };
    let vertical:f32 = if keyboard_input.pressed(KeyCode::Down) {
        -1.
    } else if keyboard_input.pressed(KeyCode::Up) {
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
    if transform.translation.y < -10.0
    {
        velo.linvel.y = 0.0;
        transform.translation.y = -10.0
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
    asset_server: Res<AssetServer>
){
    let mut rng = rand::thread_rng();

    commands.spawn_bundle(SceneBundle {
        scene: asset_server.load("models/fighter2.glb#Scene0"),
        transform:Transform {
            translation: SPAWN_POS.clone()+Vec3::new(rng.gen_range(-15.0..15.0),
                                                     rng.gen_range(-10.0..10.0),
                                                     0.0),
            scale: Vec3::new(1.0,1.0,1.0),
            ..default()
        },
        ..Default::default()
    })
        .insert(RigidBody::Dynamic)
        .insert(Velocity {
            linvel: Vec3::new(0.0, 0.0, rng.gen_range(40.0..80.0)),
            ..default()
        })
        .insert(Collider::cuboid(3.0,
                                 3.0,
                                 3.0))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(GravityScale(0.0))
        .insert(Despawnable{
            min: -1000.0,
            max: 0.0
        })
        .insert(LaserGun{
            positions: vec!(
                Vec3::new(0.0,0.0,5.0)
            ),
            player: false,
            color: Color::MIDNIGHT_BLUE,
            fire: false,
            cooldown:0.0,
            std_cooldown: rng.gen_range(0.4..2.0)
        })
        .insert(Name::new("Opponent"))
        .insert(Opponent{});
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
    keyboard_input:Res<Input<KeyCode>>,
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
    mut query: Query<( &Transform, &mut LaserGun), With<Opponent>>
){
    for (transfrom, mut laser_gun) in query.iter_mut() {
        if transfrom.translation.z.abs() < 200.0 {
            laser_gun.fire = true;
        } else {
            laser_gun.fire = false;
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
                    commands.spawn_bundle(PbrBundle {
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
    for e in collision_events.iter(){
        //println!("Collision");
        for (entity_opponent, mut opponent_transform, _opponent) in query_opponent.iter_mut() {
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
                                        ship.hits += 1;
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
    for event in event_create_effect.iter() {
        let pos = event.0;
        for x in -2..2 {
            for y in 0..2 {
                for z in -2..2 {
                    commands
                        .spawn_bundle(PbrBundle {
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
                            timer: Timer::from_seconds(EFFECT_TIME,false)
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
    Color::from([rng.gen_range(0.0..1.0),
        rng.gen_range(0.0..1.0),
        rng.gen_range(0.0..1.0)])
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

fn create_ui(
    mut egui_context: ResMut<EguiContext>,
    mut query: Query<(&Ship)>
) {
    let ship = query.single();

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
                ui.label("Hits:");
                ui.text_edit_singleline( &mut format!("{}",ship.hits).as_str());
        });
    });
}




