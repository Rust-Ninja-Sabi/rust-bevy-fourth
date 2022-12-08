use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_inspector_egui::{WorldInspectorPlugin,WorldInspectorParams};
use bevy_inspector_egui_rapier::InspectableRapierPlugin;
use crate::orbitcamera::{OrbitCameraPlugin, OrbitCamera};

pub struct GameDebugPlugin;

impl Plugin for GameDebugPlugin {
    fn build(&self, app: &mut App){
        app
            .add_plugin(InspectableRapierPlugin)
            .add_plugin(WorldInspectorPlugin::new())
            .add_plugin(RapierDebugRenderPlugin::default())
            .add_plugin(OrbitCameraPlugin)
            .add_startup_system(setup_debug)
            .add_system(debug);
    }
}



fn setup_debug(
    mut commands: Commands,
    mut world_inspector_params: ResMut<WorldInspectorParams>,
    mut debug_render_context : ResMut<DebugRenderContext>,
) {

    world_inspector_params.enabled = false;
    debug_render_context.enabled = false;

    commands.spawn_bundle(Camera3dBundle{
        camera: Camera{
            is_active:false,
            priority:5,
            ..default()
        },
        ..default()
    })
        .insert(OrbitCamera{
            distance : 28.0,
            ..default()
        })
        .insert(Name::new("OrbitCamera"));
}

fn debug(
    keyboard_input:Res<Input<KeyCode>>,
    mut world_inspector_params: ResMut<WorldInspectorParams>,
    mut debug_render_context : ResMut<DebugRenderContext>,
    mut query: Query<&mut Camera>
)
{
    if keyboard_input.just_pressed(KeyCode::O) {
        for mut camera in query.iter_mut() {
            camera.is_active = ! camera.is_active
        }
    };
    if keyboard_input.just_pressed(KeyCode::D){
        world_inspector_params.enabled = !world_inspector_params.enabled;
    };
    if keyboard_input.just_pressed(KeyCode::P){
        debug_render_context.enabled = !debug_render_context.enabled;
    }
}
