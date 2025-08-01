use bevy::prelude::*;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

mod systems;
mod components;
mod resources;

use resources::GameConfig;
use systems::world_gen::generate_world;
use systems::camera::{CameraController, MouseDragState, camera_movement, camera_zoom, mouse_camera_pan};
use systems::fps_counter::{setup_fps_counter, update_fps_counter};
use systems::player::{spawn_player, handle_player_movement_input, move_player_to_target};

fn main() {
    // Load settings from YAML file, fall back to defaults if file doesn't exist
    let config = GameConfig::load_from_file("settings.yaml")
        .unwrap_or_else(|e| {
            eprintln!("Warning: Could not load settings.yaml ({}), using defaults", e);
            GameConfig::default()
        });

    let mut app = App::new();
    
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(bevy_ecs_tilemap::TilemapPlugin)
        .insert_resource(MouseDragState::default())
        .add_systems(Startup, (setup_camera, generate_world, spawn_player))
        .add_systems(Update, (
            camera_movement, 
            camera_zoom, 
            mouse_camera_pan,
            handle_player_movement_input,
            move_player_to_target,
        ));

    // Conditionally add FPS counter based on settings
    if config.show_fps {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_systems(Startup, setup_fps_counter)
            .add_systems(Update, update_fps_counter);
    }

    app.insert_resource(config)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        CameraController,
    ));
}
