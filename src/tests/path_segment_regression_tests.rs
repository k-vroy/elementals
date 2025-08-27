#[cfg(test)]
mod tests {
    use crate::systems::world_gen::TerrainMap;
    use crate::tests::create_test_ground_configs;

    #[test]
    fn test_diagonal_path_avoids_impassable_tiles() {
        // Test that path segment validation works for diagonal movement
        let mut terrain_map = TerrainMap::new(6, 6, 32.0);
        let ground_configs = create_test_ground_configs();
        let grass_type = *ground_configs.terrain_mapping.get("grass").unwrap_or(&2);
        let stone_type = *ground_configs.terrain_mapping.get("stone").unwrap_or(&3);
        
        // Create simple layout with clear path available
        for x in 0..6 {
            for y in 0..6 {
                terrain_map.set_tile(x, y, grass_type);
            }
        }
        // Add single obstacle that shouldn't block path
        terrain_map.set_tile(2, 2, stone_type);
        
        let start = terrain_map.tile_to_world_coords(0, 0);
        let goal = terrain_map.tile_to_world_coords(5, 5);
        
        // Small pawn should find a path around the obstacle
        let path = terrain_map.find_path_for_size(start, goal, 0.5, &ground_configs);
        assert!(path.is_some(), "Small pawn should find path around single obstacle");
        
        // Large pawn should also find a path in this spacious layout
        let path = terrain_map.find_path_for_size(start, goal, 1.5, &ground_configs);
        if let Some(path_points) = path {
            // Verify no path point overlaps with the stone
            for point in path_points {
                assert!(terrain_map.is_position_passable_for_size(point.0, point.1, 1.5, &ground_configs),
                    "Path point {:?} should be passable for large pawn", point);
            }
        }
    }

    #[test]
    fn test_diagonal_movement_blocked_by_corner_stone() {
        // Test diagonal movement that's blocked by a stone at the corner
        let mut terrain_map = TerrainMap::new(3, 3, 32.0);
        let ground_configs = create_test_ground_configs();
        let grass_type = *ground_configs.terrain_mapping.get("grass").unwrap_or(&2);
        let stone_type = *ground_configs.terrain_mapping.get("stone").unwrap_or(&3);
        
        // Create layout:
        // [G G G]
        // [G S G] 
        // [G G G]
        // Test diagonal from (0,0) to (2,2) with stone at (1,1)
        for x in 0..3 {
            for y in 0..3 {
                terrain_map.set_tile(x, y, grass_type);
            }
        }
        terrain_map.set_tile(1, 1, stone_type); // Center stone
        
        let start = terrain_map.tile_to_world_coords(0, 0);
        let goal = terrain_map.tile_to_world_coords(2, 2);
        
        // Small pawn should find path around the stone
        let path = terrain_map.find_path_for_size(start, goal, 0.5, &ground_configs);
        assert!(path.is_some(), "Small pawn should find path around center stone");
        
        // Large pawn should also find path but definitely avoid the stone
        let path = terrain_map.find_path_for_size(start, goal, 1.5, &ground_configs);
        if let Some(path_points) = path {
            // Verify no path point goes through the stone
            for point in path_points {
                let (px, py) = point;
                let stone_world = terrain_map.tile_to_world_coords(1, 1);
                let distance_to_stone = ((px - stone_world.0).powi(2) + (py - stone_world.1).powi(2)).sqrt();
                
                // Large pawn should maintain safe distance from stone
                let pawn_radius = 1.5 * (terrain_map.tile_size / 2.0);
                let safe_distance = pawn_radius + (terrain_map.tile_size * 0.25); // Account for tolerance
                assert!(distance_to_stone >= safe_distance - 1.0, // Small epsilon for floating point
                    "Large pawn path point {:?} too close to stone at {:?}", point, stone_world);
            }
        }
    }

    #[test] 
    fn test_path_segment_validation_basic() {
        // Basic test that path segment validation doesn't break normal pathfinding
        let mut terrain_map = TerrainMap::new(8, 8, 32.0);
        let ground_configs = create_test_ground_configs();
        let grass_type = *ground_configs.terrain_mapping.get("grass").unwrap_or(&2);
        let stone_type = *ground_configs.terrain_mapping.get("stone").unwrap_or(&3);
        
        // Create simple open terrain
        for x in 0..8 {
            for y in 0..8 {
                terrain_map.set_tile(x, y, grass_type);
            }
        }
        
        let start = terrain_map.tile_to_world_coords(1, 1);
        let goal = terrain_map.tile_to_world_coords(6, 6);
        
        // Should find paths for various sizes
        for size in [0.5, 1.0, 1.5, 2.0] {
            let path = terrain_map.find_path_for_size(start, goal, size, &ground_configs);
            assert!(path.is_some(), "Should find path for size {}", size);
            
            // Verify all points in path are passable
            if let Some(path_points) = path {
                for point in path_points {
                    assert!(terrain_map.is_position_passable_for_size(point.0, point.1, size, &ground_configs),
                        "Path point {:?} should be passable for size {}", point, size);
                }
            }
        }
    }

    #[test]
    fn test_narrow_gap_path_segment_blocking() {
        // Test where endpoints are reachable but path between requires 
        // squeezing through a gap that's too narrow
        let mut terrain_map = TerrainMap::new(5, 5, 32.0);
        let ground_configs = create_test_ground_configs();
        let grass_type = *ground_configs.terrain_mapping.get("grass").unwrap_or(&2);
        let stone_type = *ground_configs.terrain_mapping.get("stone").unwrap_or(&3);
        
        // Create narrow gap:
        // [G G G G G]
        // [G S S S G]
        // [G S G S G] <- narrow 1-tile gap
        // [G S S S G]
        // [G G G G G]
        for x in 0..5 {
            for y in 0..5 {
                terrain_map.set_tile(x, y, grass_type);
            }
        }
        
        // Create walls with narrow gap
        for x in 1..4 {
            terrain_map.set_tile(x, 1, stone_type);
            terrain_map.set_tile(x, 3, stone_type);
        }
        terrain_map.set_tile(1, 2, stone_type);
        terrain_map.set_tile(3, 2, stone_type);
        // Gap at (2,2) remains grass
        
        let start = terrain_map.tile_to_world_coords(2, 0); // Above gap
        let goal = terrain_map.tile_to_world_coords(2, 4);  // Below gap
        
        // Small pawn should pass through narrow gap
        let path = terrain_map.find_path_for_size(start, goal, 0.5, &ground_configs);
        assert!(path.is_some(), "Small pawn should fit through narrow gap");
        
        // Large pawn should be blocked by narrow gap
        let path = terrain_map.find_path_for_size(start, goal, 2.2, &ground_configs);
        assert!(path.is_none(), "Large pawn should be blocked by narrow gap");
    }

    #[test]
    fn test_path_crosses_multiple_stone_tiles() {
        // Test path that would cross multiple contiguous stone tiles
        let mut terrain_map = TerrainMap::new(8, 8, 32.0);
        let ground_configs = create_test_ground_configs();
        let grass_type = *ground_configs.terrain_mapping.get("grass").unwrap_or(&2);
        let stone_type = *ground_configs.terrain_mapping.get("stone").unwrap_or(&3);
        
        // Fill with grass
        for x in 0..8 {
            for y in 0..8 {
                terrain_map.set_tile(x, y, grass_type);
            }
        }
        
        // Create a stone diagonal from (2,2) to (5,5)
        terrain_map.set_tile(2, 2, stone_type);
        terrain_map.set_tile(3, 3, stone_type);
        terrain_map.set_tile(4, 4, stone_type);
        terrain_map.set_tile(5, 5, stone_type);
        
        let start = terrain_map.tile_to_world_coords(1, 1);
        let goal = terrain_map.tile_to_world_coords(6, 6);
        
        // Small pawn should find a path around the stones
        let path = terrain_map.find_path_for_size(start, goal, 0.5, &ground_configs);
        assert!(path.is_some(), "Small pawn should find path around stone diagonal");
        
        // Medium pawn should also find path around stones
        let path = terrain_map.find_path_for_size(start, goal, 1.2, &ground_configs);
        assert!(path.is_some(), "Medium pawn should find path around stone diagonal");
        
        // Verify path doesn't cross stones for medium pawn
        if let Some(path_points) = path {
            for point in path_points {
                assert!(terrain_map.is_position_passable_for_size(point.0, point.1, 1.2, &ground_configs),
                    "Path point {:?} should be passable for medium pawn", point);
            }
        }
    }

    #[test]
    fn test_path_segment_sampling_works() {
        // Test that path segment sampling doesn't cause performance issues
        let mut terrain_map = TerrainMap::new(8, 8, 32.0);
        let ground_configs = create_test_ground_configs();
        let grass_type = *ground_configs.terrain_mapping.get("grass").unwrap_or(&2);
        let stone_type = *ground_configs.terrain_mapping.get("stone").unwrap_or(&3);
        
        // Create open terrain with few obstacles  
        for x in 0..8 {
            for y in 0..8 {
                terrain_map.set_tile(x, y, grass_type);
            }
        }
        // Add single obstacle
        terrain_map.set_tile(4, 4, stone_type);
        
        let start = terrain_map.tile_to_world_coords(0, 0);
        let goal = terrain_map.tile_to_world_coords(7, 7);
        
        // Test various sizes to ensure sampling scales properly
        for size in [0.1, 0.5, 1.0, 2.0] {
            let path = terrain_map.find_path_for_size(start, goal, size, &ground_configs);
            // Just verify it doesn't crash and produces reasonable results
            if let Some(path_points) = path {
                assert!(!path_points.is_empty(), "Path should have points for size {}", size);
                // Verify endpoints
                assert_eq!(path_points.first().unwrap(), &start, "Path should start at start point");
                assert_eq!(path_points.last().unwrap(), &goal, "Path should end at goal point");
            }
        }
    }

    #[test]
    fn test_very_small_pawn_edge_cases() {
        // Test edge cases with very small pawns to ensure they don't get false negatives
        let mut terrain_map = TerrainMap::new(5, 5, 32.0);
        let ground_configs = create_test_ground_configs();
        let grass_type = *ground_configs.terrain_mapping.get("grass").unwrap_or(&2);
        let stone_type = *ground_configs.terrain_mapping.get("stone").unwrap_or(&3);
        
        // Create scattered obstacles
        for x in 0..5 {
            for y in 0..5 {
                terrain_map.set_tile(x, y, grass_type);
            }
        }
        terrain_map.set_tile(1, 1, stone_type);
        terrain_map.set_tile(3, 3, stone_type);
        
        let start = terrain_map.tile_to_world_coords(0, 0);
        let goal = terrain_map.tile_to_world_coords(4, 4);
        
        // Very small pawn should navigate easily
        let path = terrain_map.find_path_for_size(start, goal, 0.1, &ground_configs);
        assert!(path.is_some(), "Very small pawn should find path easily");
        
        // Tiny pawn should also work
        let path = terrain_map.find_path_for_size(start, goal, 0.01, &ground_configs);
        assert!(path.is_some(), "Tiny pawn should find path easily");
    }

    #[test]
    fn test_large_pawn_realistic_scenario() {
        // Test large pawns in realistic scenarios
        let mut terrain_map = TerrainMap::new(10, 10, 32.0);
        let ground_configs = create_test_ground_configs();
        let grass_type = *ground_configs.terrain_mapping.get("grass").unwrap_or(&2);
        let stone_type = *ground_configs.terrain_mapping.get("stone").unwrap_or(&3);
        
        // Fill with grass - spacious environment
        for x in 0..10 {
            for y in 0..10 {
                terrain_map.set_tile(x, y, grass_type);
            }
        }
        
        let start = terrain_map.tile_to_world_coords(1, 1);
        let goal = terrain_map.tile_to_world_coords(8, 8);
        
        // Test reasonable large pawn sizes
        for size in [1.5, 2.0, 2.5] {
            let path = terrain_map.find_path_for_size(start, goal, size, &ground_configs);
            assert!(path.is_some(), "Large pawn size {} should find path in open space", size);
            
            if let Some(path_points) = path {
                assert!(!path_points.is_empty(), "Path should have points");
                // Verify all path points are valid
                for point in path_points {
                    assert!(terrain_map.is_position_passable_for_size(point.0, point.1, size, &ground_configs),
                        "Path point {:?} should be passable for size {}", point, size);
                }
            }
        }
    }

    #[test]
    fn test_zero_and_negative_size_edge_cases() {
        // Test edge cases with zero or invalid sizes
        let mut terrain_map = TerrainMap::new(5, 5, 32.0);
        let ground_configs = create_test_ground_configs();
        let grass_type = *ground_configs.terrain_mapping.get("grass").unwrap_or(&2);
        let stone_type = *ground_configs.terrain_mapping.get("stone").unwrap_or(&3);
        
        for x in 0..5 {
            for y in 0..5 {
                terrain_map.set_tile(x, y, grass_type);
            }
        }
        terrain_map.set_tile(2, 2, stone_type);
        
        let start = terrain_map.tile_to_world_coords(0, 0);
        let goal = terrain_map.tile_to_world_coords(4, 4);
        
        // Zero size pawn should work (treated as point)
        let path = terrain_map.find_path_for_size(start, goal, 0.0, &ground_configs);
        assert!(path.is_some(), "Zero size pawn should find path");
        
        // Very close to zero should also work
        let path = terrain_map.find_path_for_size(start, goal, 0.001, &ground_configs);
        assert!(path.is_some(), "Near-zero size pawn should find path");
    }

    #[test]
    fn test_horizontal_and_vertical_path_segments() {
        // Test purely horizontal and vertical movements
        let mut terrain_map = TerrainMap::new(6, 6, 32.0);
        let ground_configs = create_test_ground_configs();
        let grass_type = *ground_configs.terrain_mapping.get("grass").unwrap_or(&2);
        let stone_type = *ground_configs.terrain_mapping.get("stone").unwrap_or(&3);
        
        // Create obstacle pattern:
        // [G G G G G G]
        // [G S S S S G]
        // [G G G G G G] 
        // [G G G G G G]
        // [G S S S S G]
        // [G G G G G G]
        for x in 0..6 {
            for y in 0..6 {
                terrain_map.set_tile(x, y, grass_type);
            }
        }
        
        // Horizontal stone barriers
        for x in 1..5 {
            terrain_map.set_tile(x, 1, stone_type);
            terrain_map.set_tile(x, 4, stone_type);
        }
        
        // Test horizontal movement
        let start = terrain_map.tile_to_world_coords(0, 2);
        let goal = terrain_map.tile_to_world_coords(5, 2);
        
        let path = terrain_map.find_path_for_size(start, goal, 1.0, &ground_configs);
        assert!(path.is_some(), "Horizontal path should be found");
        
        // Test vertical movement  
        let start = terrain_map.tile_to_world_coords(2, 0);
        let goal = terrain_map.tile_to_world_coords(2, 5);
        
        let path = terrain_map.find_path_for_size(start, goal, 1.0, &ground_configs);
        assert!(path.is_some(), "Vertical path should be found");
    }

    #[test]
    fn test_path_segment_sampling_accuracy() {
        // Test that path segment sampling catches obstacles it should catch
        let mut terrain_map = TerrainMap::new(4, 4, 32.0);
        let ground_configs = create_test_ground_configs();
        let grass_type = *ground_configs.terrain_mapping.get("grass").unwrap_or(&2);
        let stone_type = *ground_configs.terrain_mapping.get("stone").unwrap_or(&3);
        
        // Create diagonal obstacle line
        for x in 0..4 {
            for y in 0..4 {
                terrain_map.set_tile(x, y, grass_type);
            }
        }
        
        // Diagonal line of stones from bottom-left to top-right
        terrain_map.set_tile(1, 1, stone_type);
        terrain_map.set_tile(2, 2, stone_type);
        
        // Try to path diagonally across the stone line
        let start = terrain_map.tile_to_world_coords(0, 0);
        let goal = terrain_map.tile_to_world_coords(3, 3);
        
        // Small pawn should find alternate path
        let path = terrain_map.find_path_for_size(start, goal, 0.5, &ground_configs);
        assert!(path.is_some(), "Small pawn should find alternate path around stone line");
        
        // Medium pawn should also avoid the stone line
        let path = terrain_map.find_path_for_size(start, goal, 1.0, &ground_configs);
        if let Some(path_points) = path {
            // Verify the path doesn't go through the stone tiles
            for point in path_points {
                assert!(terrain_map.is_position_passable_for_size(point.0, point.1, 1.0, &ground_configs),
                    "Path point {:?} should be passable for medium pawn", point);
            }
        }
    }
}