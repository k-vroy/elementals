pub mod movement_tests;
pub mod world_gen_tests;
pub mod pawn_tests;
pub mod hunt_solo_tests;
pub mod debug_terrain_tests;
pub mod size_pathfinding_tests;
pub mod path_segment_regression_tests;
pub mod pathfinding_cache_tests;
pub mod async_pathfinding_tests;

use bevy::prelude::*;
use crate::systems::world_gen::{TerrainMap, GroundConfigs};

// Test utilities
pub fn create_test_ground_configs() -> GroundConfigs {
    let yaml_content = r#"
water:
  sprite: "tileset::grounds::water"
  passable: false
  height_min: 0.0
  height_max: 0.15
dirt:
  sprite: "tileset::grounds::dirt"
  passable: true
  height_min: 0.15
  height_max: 0.3
grass:
  sprite: "tileset::grounds::grass"
  passable: true
  height_min: 0.3
  height_max: 0.7
stone:
  sprite: "tileset::grounds::stone"
  passable: false
  height_min: 0.7
  height_max: 1.0
"#;
    GroundConfigs::load_from_yaml(yaml_content).expect("Failed to load test ground configs")
}

pub fn create_test_terrain_map(width: u32, height: u32, tile_size: f32) -> TerrainMap {
    let mut terrain_map = TerrainMap::new(width, height, tile_size);
    let ground_configs = create_test_ground_configs();
    
    // Create a simple test pattern:
    // - Grass on edges
    // - Some water in middle-left
    // - Some stone obstacles
    for x in 0..width {
        for y in 0..height {
            let terrain = if x == 0 || x == width - 1 || y == 0 || y == height - 1 {
                *ground_configs.terrain_mapping.get("grass").unwrap_or(&2) // Default to grass
            } else if x < width / 3 && y >= height / 3 && y < 2 * height / 3 {
                *ground_configs.terrain_mapping.get("water").unwrap_or(&0) // Default to water
            } else if x == width / 2 && y == height / 2 {
                *ground_configs.terrain_mapping.get("stone").unwrap_or(&3)
            } else {
                *ground_configs.terrain_mapping.get("grass").unwrap_or(&2) // Default to grass
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