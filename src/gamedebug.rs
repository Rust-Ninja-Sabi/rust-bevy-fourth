use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_egui::EguiPlugin;
use crate::orbitcamera::{OrbitCameraPlugin, OrbitCamera};

pub struct GameDebugPlugin;

#[derive(Resource)]
struct BevyInspector{
    enabled: bool
}

impl Plugin for GameDebugPlugin {
    fn build(&self, app: &mut App){
        app
            .insert_resource(BevyInspector{enabled:false})
            .add_plugins((RapierDebugRenderPlugin::default(), EguiPlugin,
                          OrbitCameraPlugin))
            .add_systems(Startup, setup_debug)
            .add_systems( Update, debug);
            //.add_system(inspector_ui);
    }
}



fn setup_debug(
    mut commands: Commands,
    mut debug_render_context : ResMut<DebugRenderContext>,
) {

    debug_render_context.enabled = false;

    commands.spawn(Camera3dBundle{
        camera: Camera{
            is_active:false,
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
    keyboard_input:Res<ButtonInput<KeyCode>>,
    mut debug_render_context : ResMut<DebugRenderContext>,
    mut bevy_inspector:ResMut<BevyInspector>,
    mut query: Query<&mut Camera>
)
{
    if keyboard_input.just_pressed(KeyCode::KeyO) {
        for mut camera in query.iter_mut() {
            camera.is_active = ! camera.is_active
        }
    };
    if keyboard_input.just_pressed(KeyCode::KeyD){
        bevy_inspector.enabled = !bevy_inspector.enabled;
    };
    if keyboard_input.just_pressed(KeyCode::KeyP){
        debug_render_context.enabled = !debug_render_context.enabled;
    }
}

/*fn inspector_ui(
    world: &mut World
) {
    let egui_context = world.resource_mut::<bevy_egui::EguiContext>().ctx_mut().clone();

    let bevy_inspector = world.get_resource::<BevyInspector>().unwrap();
    if bevy_inspector.enabled {
        Window::new("UI").show(&egui_context, |ui| {
        ScrollArea::vertical().show(ui, |ui| {
            // equivalent to `WorldInspectorPlugin`
            // bevy_inspector_egui::bevy_inspector::ui_for_world(world, ui);

            CollapsingHeader::new("Materials").show(ui, |ui| {
                bevy_inspector_egui::bevy_inspector::ui_for_assets::<StandardMaterial>(world, ui);
            });

            ui.heading("Entities");
            bevy_inspector_egui::bevy_inspector::ui_for_world_entities(world, ui);
        });
    });
    }
}
*/