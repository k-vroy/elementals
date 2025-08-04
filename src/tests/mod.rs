pub mod movement_tests;
pub mod world_gen_tests;
pub mod pawn_tests;

use bevy::prelude::*;
use crate::systems::world_gen::{TerrainMap, TerrainType};

// Test utilities
pub fn create_test_terrain_map(width: u32, height: u32, tile_size: f32) -> TerrainMap {
    let mut terrain_map = TerrainMap::new(width, height, tile_size);
    
    // Create a simple test pattern:
    // - Grass on edges
    // - Some water in middle-left
    // - Some stone obstacles
    for x in 0..width {
        for y in 0..height {
            let terrain = if x == 0 || x == width - 1 || y == 0 || y == height - 1 {
                TerrainType::Grass
            } else if x < width / 3 && y >= height / 3 && y < 2 * height / 3 {
                TerrainType::Water
            } else if x == width / 2 && y == height / 2 {
                TerrainType::Stone
            } else {
                TerrainType::Grass
            };
            terrain_map.set_tile(x, y, terrain);
        }
    }
    
    terrain_map
}

pub fn setup_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TransformPlugin)
        .add_plugins(HierarchyPlugin);
    app
}