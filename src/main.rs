use bevy::prelude::*;
use bevy::time::FixedTimestep;
use bevy_rapier3d::prelude::*;
use rand::Rng;
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
    guns: Vec<Vec3>,
    cooldown: f32
}

#[derive(Component)]
struct Opponent;

#[derive(Component)]
struct Laser;

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
        .add_plugin(GameDebugPlugin)
        .add_startup_system(setup_camera)
        .add_startup_system(setup)
        //.add_state(GameState::GameStart)
        .add_system(move_ship)
        .add_system(spawn_laser)
        .add_system(collision)
        .add_system(create_effect)
        .add_system(remove_effect)
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
    .insert(RigidBody::Dynamic)
    .insert(Velocity {
        linvel: Vec3::new(0.0, 0.0, 0.0),
        ..default()
    })
    .insert(Name::new("Ship"))
    .insert(Ship{
        guns: vec!(
            Vec3::new(-1.0,0.0,0.0),
            Vec3::new(1.0,0.0,0.0)
        ),
        cooldown:0.0
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

const COOLDOWN:f32=0.2;

fn spawn_laser(
    mut commands: Commands,
    time:Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    keyboard_input:Res<Input<KeyCode>>,
    mut query: Query<(&Transform,&mut Ship)>
)
{
    if keyboard_input.pressed(KeyCode::Space) {
        let (ship_transform, mut ship) = query.single_mut();

        if ship.cooldown <= 0.0 {
            ship.cooldown = COOLDOWN;
            for gun in &ship.guns {
                commands.spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Box::new(0.2, 0.2, 3.2))),
                    material: materials.add(StandardMaterial {
                        base_color: Color::LIME_GREEN,
                        emissive: Color::LIME_GREEN,
                        ..Default::default()
                    }),
                    transform: Transform {
                        translation: ship_transform.translation.clone() + gun.clone(),
                        rotation: ship_transform.rotation.clone(),
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
                        linvel: ship_transform.forward() * 600.0,
                        ..Default::default()
                    })
                    .insert(GravityScale(0.0))
                    .insert(Despawnable {
                        min: -1000.0,
                        max: 0.0
                    })
                    .insert(Name::new("Laser"))
                    .insert(Laser);
            }
        } else {
            ship.cooldown -= time.delta_seconds();
        }
    }
}

fn collision(
    mut collision_events: EventReader<CollisionEvent>,
    mut query_opponent: Query<(Entity,&mut Transform, &Opponent), Without<Laser>>,
    query_laser: Query<(Entity, &Transform), With<Laser>>,
    mut event_create_effect: EventWriter<CreateEffectEvent>,
    mut commands: Commands
){
    for e in collision_events.iter(){
        //println!("Collision");
        for (entity_opponent, mut opponent_transform, _opponent) in query_opponent.iter_mut() {
            match e {
                CollisionEvent::Started(e1, e2, _) => {
                    if e1 == &entity_opponent || e2 == &entity_opponent{
                        for (entity_laser, _) in query_laser.iter() {
                            if e1 == &entity_laser || e2 == &entity_laser {
                                event_create_effect.send(CreateEffectEvent(Vec3::from(opponent_transform.translation)));
                                commands.entity(entity_laser).despawn_recursive();
                                commands.entity(entity_opponent).despawn_recursive();

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


