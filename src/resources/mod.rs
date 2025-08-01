use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Resource)]
pub struct GameConfig {
    pub tile_size: f32,
    pub map_width: u32,
    pub map_height: u32,
    pub camera_speed: f32,
    pub zoom_min: f32,
    pub zoom_max: f32,
    pub mouse_sensitivity: f32,
    pub window_title: String,
    pub target_fps: u32,
    pub show_fps: bool,
}

#[derive(Deserialize, Serialize)]
struct Settings {
    world: WorldSettings,
    camera: CameraSettings,
    game: GameSettings,
}

#[derive(Deserialize, Serialize)]
struct WorldSettings {
    map_width: u32,
    map_height: u32,
    tile_size: f32,
}

#[derive(Deserialize, Serialize)]
struct CameraSettings {
    movement_speed: f32,
    zoom_min: f32,
    zoom_max: f32,
    mouse_sensitivity: f32,
}

#[derive(Deserialize, Serialize)]
struct GameSettings {
    window_title: String,
    target_fps: u32,
    show_fps: bool,
}

impl GameConfig {
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let settings: Settings = serde_yaml::from_str(&content)?;
        
        Ok(GameConfig {
            tile_size: settings.world.tile_size,
            map_width: settings.world.map_width,
            map_height: settings.world.map_height,
            camera_speed: settings.camera.movement_speed,
            zoom_min: settings.camera.zoom_min,
            zoom_max: settings.camera.zoom_max,
            mouse_sensitivity: settings.camera.mouse_sensitivity,
            window_title: settings.game.window_title,
            target_fps: settings.game.target_fps,
            show_fps: settings.game.show_fps,
        })
    }

    pub fn default() -> Self {
        Self {
            tile_size: 16.0,
            map_width: 32,
            map_height: 32,
            camera_speed: 200.0,
            zoom_min: 0.1,
            zoom_max: 10.0,
            mouse_sensitivity: 1.0,
            window_title: "Elementals RPG".to_string(),
            target_fps: 60,
            show_fps: false, // Disabled by default in code
        }
    }
}