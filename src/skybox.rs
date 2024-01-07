use bevy::prelude::*;
use std::f32::consts::PI;

pub struct SkyboxPlugin;

#[derive(Component)]
struct Skybox{
    rotate:f32
}

#[derive(Component)]
struct Wall{}

#[derive(Event)]
pub struct RotateSkyboxEvent();

impl Plugin for SkyboxPlugin {
    fn build(&self, app: &mut App){
        app
            .add_systems(PreStartup,setup_skybox)
            .add_event::<RotateSkyboxEvent>()
            .add_systems(Update, (start_rotate, rotate));
    }
}

const SIZE:f32=1000.0; //  640.0;

fn setup_skybox(
    mut commands:Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
){
    let images = vec!["images/skybox_front.png",
                                 "images/skybox_left.png",
                                 "images/skybox_right.png",
                                 "images/skybox_back.png",
                                 "images/skybox_down.png",
                                 "images/skybox_up.png"];
    let distance = SIZE/2.0;
    let translations = vec![Vec3::new(0.0, 0.0, -distance),
                                       Vec3::new(distance, 0.0, 0.0),
                                       Vec3::new(-distance, 0.0, 0.0),
                                       Vec3::new(0.0, 0.0, distance),
                                       Vec3::new(0.0, -distance, 0.0),
                                       Vec3::new(0.0, distance, 0.0),];
    let rotations =vec![ Quat::from_rotation_x(0.0),
                                    Quat::from_euler(EulerRot::XYZ,0.0,-PI/2.0,0.0),
                                    Quat::from_euler(EulerRot::XYZ,0.0,PI/2.0,0.0),
                                    Quat::from_euler(EulerRot::XYZ,PI,0.0,-PI),
                                    Quat::from_euler(EulerRot::XYZ,-PI/2.0,0.0,0.0),
                                    Quat::from_euler(EulerRot::XYZ,PI/2.0,0.0,0.0)];

    let mut children_list:Vec<Entity> = Vec::new();

    for i in 0..images.len() {
        //sky
        let store_texture_handle = asset_server.load(images[i]);
        let store_aspect = 1.0;

        let store_quad_width = SIZE;
        let store_quad_handle = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
            store_quad_width,
            store_quad_width * store_aspect,
        ))));

        let store_material_handle = materials.add(StandardMaterial {
            base_color_texture: Some(store_texture_handle.clone()),
            unlit: true,
            double_sided: true,
            ..Default::default()
        });

        let entity =  commands.spawn(PbrBundle {
            mesh: store_quad_handle.clone(),
            material: store_material_handle,
            transform: Transform {
                translation: translations[i],
                rotation: rotations[i],
                ..Default::default()
            },
            ..Default::default()
        })
            .insert(Wall{})
            .insert(Name::new("Wall"))
            .id();

        children_list.push(entity);
    };

    commands.spawn(
        SpatialBundle {
            transform: Transform::from_translation(Vec3::ZERO),
            visibility:  Visibility::Visible,
            ..Default::default()
        },
    )
        .insert(Skybox{rotate:0.0})
        .insert(Name::new("Skybox"))
        .push_children(&children_list);
}

fn start_rotate(
    mut rotate_events: EventReader<RotateSkyboxEvent>,
    mut query: Query<&mut Skybox>
){
    for _ in rotate_events.iter(){
        for mut skybox in query.iter_mut(){
            skybox.rotate = PI/2.0;
        }
    }
}

fn rotate(
    mut query: Query<(&mut Transform, &mut Skybox)>,
    time:Res<Time>,
){
    for (mut transform, mut skybox) in query.iter_mut(){
        if skybox.rotate > 0.0 {
            let diff = time.delta_seconds() * PI / 2.0* 0.1;
            transform.rotate_y(diff);
            skybox.rotate -= diff;
        }
    }
}
