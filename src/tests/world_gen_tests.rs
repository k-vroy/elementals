use crate::systems::world_gen::{TerrainMap, TerrainType};
use crate::tests::create_test_terrain_map;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_generates_with_correct_dimensions() {
        let width = 20;
        let height = 15;
        let tile_size = 32.0;
        
        let terrain_map = TerrainMap::new(width, height, tile_size);
        
        assert_eq!(terrain_map.width, width);
        assert_eq!(terrain_map.height, height);
        assert_eq!(terrain_map.tile_size, tile_size);
        assert_eq!(terrain_map.tiles.len(), width as usize);
        assert_eq!(terrain_map.tiles[0].len(), height as usize);
    }

    #[test]
    fn test_all_terrain_types_present() {
        let terrain_map = create_test_terrain_map(10, 10, 32.0);
        
        let mut found_grass = false;
        let mut found_water = false;
        let mut found_stone = false;
        
        for x in 0..terrain_map.width {
            for y in 0..terrain_map.height {
                match terrain_map.tiles[x as usize][y as usize] {
                    TerrainType::Grass => found_grass = true,
                    TerrainType::Water => found_water = true,
                    TerrainType::Stone => found_stone = true,
                    _ => {}
                }
            }
        }
        
        assert!(found_grass, "Map should contain grass tiles");
        assert!(found_water, "Map should contain water tiles");
        assert!(found_stone, "Map should contain stone tiles");
    }

    #[test]
    fn test_terrain_passability_rules() {
        assert!(TerrainType::Grass.is_passable(), "Grass should be passable");
        assert!(TerrainType::Dirt.is_passable(), "Dirt should be passable");
        assert!(!TerrainType::Stone.is_passable(), "Stone should be impassable");
        assert!(!TerrainType::Water.is_passable(), "Water should be impassable");
    }

    #[test]
    fn test_tile_setting_and_getting() {
        let mut terrain_map = TerrainMap::new(5, 5, 32.0);
        
        // Set different terrain types
        terrain_map.set_tile(0, 0, TerrainType::Grass);
        terrain_map.set_tile(1, 1, TerrainType::Water);
        terrain_map.set_tile(2, 2, TerrainType::Stone);
        terrain_map.set_tile(3, 3, TerrainType::Dirt);
        
        // Verify they were set correctly
        assert!(matches!(terrain_map.tiles[0][0], TerrainType::Grass));
        assert!(matches!(terrain_map.tiles[1][1], TerrainType::Water));
        assert!(matches!(terrain_map.tiles[2][2], TerrainType::Stone));
        assert!(matches!(terrain_map.tiles[3][3], TerrainType::Dirt));
    }

    #[test]
    fn test_out_of_bounds_tile_setting() {
        let mut terrain_map = TerrainMap::new(3, 3, 32.0);
        
        // These should not panic or corrupt memory
        terrain_map.set_tile(10, 10, TerrainType::Stone);
        terrain_map.set_tile(u32::MAX, u32::MAX, TerrainType::Water);
        
        // Original tiles should be unchanged
        for x in 0..3 {
            for y in 0..3 {
                assert!(matches!(terrain_map.tiles[x][y], TerrainType::Grass));
            }
        }
    }

    #[test]
    fn test_world_position_terrain_lookup() {
        let mut terrain_map = TerrainMap::new(5, 5, 32.0);
        
        // Set a specific tile to water
        terrain_map.set_tile(2, 2, TerrainType::Water);
        
        // Get world coordinates for that tile
        let (world_x, world_y) = terrain_map.tile_to_world_coords(2, 2);
        
        // Look up terrain at those world coordinates
        let terrain = terrain_map.get_terrain_at_world_pos(world_x, world_y);
        assert!(matches!(terrain, Some(TerrainType::Water)));
    }

    #[test]
    fn test_coordinate_system_consistency() {
        let terrain_map = TerrainMap::new(6, 4, 16.0);
        
        // Test that coordinate conversion is consistent
        for tile_x in 0..6 {
            for tile_y in 0..4 {
                let (world_x, world_y) = terrain_map.tile_to_world_coords(tile_x as i32, tile_y as i32);
                let tile_coords = terrain_map.world_to_tile_coords(world_x, world_y);
                
                assert_eq!(tile_coords, Some((tile_x as i32, tile_y as i32)),
                          "Coordinate conversion should be bidirectional for tile ({}, {})", tile_x, tile_y);
            }
        }
    }

    #[test]
    fn test_map_center_coordinates() {
        let terrain_map = TerrainMap::new(4, 4, 10.0);
        
        // For a 4x4 map with 10.0 tile size, world should span -20 to +20 in both axes
        // Center should be at (0, 0)
        let center_tile_coords = terrain_map.world_to_tile_coords(0.0, 0.0);
        assert!(center_tile_coords.is_some(), "Center of world should map to valid tile coordinates");
        
        // Test corner coordinates
        let (world_x, world_y) = terrain_map.tile_to_world_coords(0, 0);
        assert!(world_x < 0.0 && world_y < 0.0, "First tile should be in negative world coordinates");
        
        let (world_x, world_y) = terrain_map.tile_to_world_coords(3, 3);
        assert!(world_x > 0.0 && world_y > 0.0, "Last tile should be in positive world coordinates");
    }

    #[test]
    fn test_passable_tile_finder() {
        // Create a map with mostly impassable terrain
        let mut terrain_map = TerrainMap::new(5, 5, 32.0);
        
        // Fill with stone (impassable)
        for x in 0..5 {
            for y in 0..5 {
                terrain_map.set_tile(x, y, TerrainType::Stone);
            }
        }
        
        // Make one tile passable
        terrain_map.set_tile(3, 3, TerrainType::Grass);
        
        let passable_pos = terrain_map.find_nearest_passable_tile((0.0, 0.0));
        assert!(passable_pos.is_some(), "Should find the one passable tile");
        
        let (px, py) = passable_pos.unwrap();
        let tile_coords = terrain_map.world_to_tile_coords(px, py).unwrap();
        assert_eq!(tile_coords, (3, 3), "Should find the grass tile at (3,3)");
    }

    #[test]
    fn test_no_passable_tile_available() {
        // Create a map with no passable terrain
        let mut terrain_map = TerrainMap::new(3, 3, 32.0);
        
        // Fill entirely with stone
        for x in 0..3 {
            for y in 0..3 {
                terrain_map.set_tile(x, y, TerrainType::Stone);
            }
        }
        
        let passable_pos = terrain_map.find_nearest_passable_tile((0.0, 0.0));
        assert!(passable_pos.is_none(), "Should return None when no passable tiles exist");
    }

    #[test]
    fn test_tile_size_affects_world_coordinates() {
        let terrain_map_small = TerrainMap::new(2, 2, 10.0);
        let terrain_map_large = TerrainMap::new(2, 2, 50.0);
        
        let (small_x, small_y) = terrain_map_small.tile_to_world_coords(1, 1);
        let (large_x, large_y) = terrain_map_large.tile_to_world_coords(1, 1);
        
        // Larger tile size should result in larger world coordinates
        assert!(large_x.abs() > small_x.abs(), "Larger tile size should result in larger world coordinates");
        assert!(large_y.abs() > small_y.abs(), "Larger tile size should result in larger world coordinates");
    }

    #[test]
    fn test_border_tile_handling() {
        let terrain_map = create_test_terrain_map(5, 5, 32.0);
        
        // Test all border tiles exist and are accessible
        let border_positions = vec![
            (0, 0), (0, 4), (4, 0), (4, 4), // corners
            (0, 2), (4, 2), (2, 0), (2, 4), // edge midpoints
        ];
        
        for (tile_x, tile_y) in border_positions {
            let (world_x, world_y) = terrain_map.tile_to_world_coords(tile_x, tile_y);
            let terrain = terrain_map.get_terrain_at_world_pos(world_x, world_y);
            assert!(terrain.is_some(), "Border tile ({}, {}) should be accessible", tile_x, tile_y);
        }
    }

    #[test]
    fn test_terrain_map_default_initialization() {
        let terrain_map = TerrainMap::new(3, 3, 16.0);
        
        // All tiles should default to grass
        for x in 0..3 {
            for y in 0..3 {
                assert!(matches!(terrain_map.tiles[x][y], TerrainType::Grass),
                       "Default terrain should be grass for tile ({}, {})", x, y);
            }
        }
    }
}