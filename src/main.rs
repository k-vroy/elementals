use bevy::prelude::*;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

mod systems;
mod components;
mod resources;

#[cfg(test)]
mod tests;

use resources::GameConfig;
use systems::world_gen::generate_world;
use systems::camera::{CameraController, MouseDragState, camera_movement, camera_zoom, mouse_camera_pan};
use systems::fps_counter::{setup_fps_counter, update_fps_counter};
use systems::spawn::spawn_all_pawns;
use systems::input::handle_player_input;
use systems::pawn::{move_pawn_to_target, endurance_health_loss_system, pawn_death_system, endurance_behavior_switching_system, TilesetManager};
use systems::pawn_config::PawnConfig;
use systems::ai::{wandering_ai_system, setup_wandering_ai, hunt_solo_ai_system, setup_hunt_solo_ai};
use systems::water_shader::WaterShaderPlugin;

fn main() {
    // Load settings from YAML file, fall back to defaults if file doesn't exist
    let config = GameConfig::load_from_file("settings.yaml")
        .unwrap_or_else(|e| {
            eprintln!("Warning: Could not load settings.yaml ({}), using defaults", e);
            GameConfig::default()
        });

    // Load pawn configuration from YAML file
    let pawn_config = PawnConfig::load_from_file("pawns.yaml")
        .expect("Failed to load pawns.yaml configuration file");

    let mut app = App::new();
    
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(bevy_ecs_tilemap::TilemapPlugin)
        .add_plugins(WaterShaderPlugin)
        .insert_resource(MouseDragState::default())
        .insert_resource(TilesetManager::default())
        .insert_resource(pawn_config)
        .add_systems(Startup, (
            setup_camera,
            generate_world,
            spawn_all_pawns.after(generate_world),
        ))
        .add_systems(Update, (
            camera_movement, 
            camera_zoom, 
            mouse_camera_pan,
            handle_player_input,
            move_pawn_to_target,
            setup_wandering_ai,
            wandering_ai_system,
            setup_hunt_solo_ai,
            hunt_solo_ai_system,
            endurance_health_loss_system,
            endurance_behavior_switching_system.after(endurance_health_loss_system),
            pawn_death_system,
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
