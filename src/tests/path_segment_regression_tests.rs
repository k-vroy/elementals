#[cfg(test)]
mod tests {
    use crate::systems::world_gen::{TerrainMap, TerrainType};

    #[test]
    fn test_diagonal_path_crosses_impassable_tile() {
        // Test case where a diagonal path would pass through an impassable tile
        // even though both endpoints are passable
        let mut terrain_map = TerrainMap::new(4, 4, 32.0);
        
        // Create layout:
        // [G G G G]
        // [G S G G] 
        // [G G S G]
        // [G G G G]
        // Diagonal from (0,0) to (3,3) would pass through stone tiles
        for x in 0..4 {
            for y in 0..4 {
                terrain_map.set_tile(x, y, TerrainType::Grass);
            }
        }
        terrain_map.set_tile(1, 1, TerrainType::Stone);
        terrain_map.set_tile(2, 2, TerrainType::Stone);
        
        let start = terrain_map.tile_to_world_coords(0, 0);
        let goal = terrain_map.tile_to_world_coords(3, 3);
        
        // Small pawn should still find a path (can navigate around)
        let path = terrain_map.find_path_for_size(start, goal, 0.5);
        assert!(path.is_some(), "Small pawn should find alternate path around obstacles");
        
        // Large pawn should be blocked from diagonal path through stones
        let path = terrain_map.find_path_for_size(start, goal, 2.0);
        assert!(path.is_some(), "Large pawn should find path but avoid crossing stones");
        
        // Verify the path doesn't go through stone tiles for large pawn
        if let Some(path_points) = path {
            for point in path_points {
                assert!(terrain_map.is_position_passable_for_size(point.0, point.1, 2.0),
                    "Path point {:?} should be passable for large pawn", point);
            }
        }
    }

    #[test]
    fn test_diagonal_movement_blocked_by_corner_stone() {
        // Test diagonal movement that's blocked by a stone at the corner
        let mut terrain_map = TerrainMap::new(3, 3, 32.0);
        
        // Create layout:
        // [G G G]
        // [G S G] 
        // [G G G]
        // Test diagonal from (0,0) to (2,2) with stone at (1,1)
        for x in 0..3 {
            for y in 0..3 {
                terrain_map.set_tile(x, y, TerrainType::Grass);
            }
        }
        terrain_map.set_tile(1, 1, TerrainType::Stone); // Center stone
        
        let start = terrain_map.tile_to_world_coords(0, 0);
        let goal = terrain_map.tile_to_world_coords(2, 2);
        
        // Small pawn should find path around the stone
        let path = terrain_map.find_path_for_size(start, goal, 0.5);
        assert!(path.is_some(), "Small pawn should find path around center stone");
        
        // Large pawn should also find path but definitely avoid the stone
        let path = terrain_map.find_path_for_size(start, goal, 1.5);
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
    fn test_path_segment_validation_prevents_stone_crossing() {
        // Specific test for path segment validation - create scenario where
        // waypoints are passable but path between them crosses impassable tiles
        let mut terrain_map = TerrainMap::new(7, 3, 32.0);
        
        // Create layout with stone barrier in middle:
        // [G G G S S S G]
        // [G G G S S S G] 
        // [G G G S S S G]
        for x in 0..7 {
            for y in 0..3 {
                if x >= 3 && x <= 5 {
                    terrain_map.set_tile(x, y, TerrainType::Stone);
                } else {
                    terrain_map.set_tile(x, y, TerrainType::Grass);
                }
            }
        }
        
        let start = terrain_map.tile_to_world_coords(1, 1);
        let goal = terrain_map.tile_to_world_coords(6, 1);
        
        // Small pawn should find path around the barrier
        let path = terrain_map.find_path_for_size(start, goal, 0.5);
        assert!(path.is_some(), "Small pawn should find path around stone barrier");
        
        // Large pawn should also find path but cannot cross through stones
        let path = terrain_map.find_path_for_size(start, goal, 1.8);
        assert!(path.is_some(), "Large pawn should find path around barrier");
        
        // Verify that the path doesn't cross the stone barrier
        if let Some(path_points) = path {
            for point in path_points {
                assert!(terrain_map.is_position_passable_for_size(point.0, point.1, 1.8),
                    "Path point {:?} should be passable for large pawn", point);
            }
        }
    }

    #[test]
    fn test_narrow_gap_path_segment_blocking() {
        // Test where endpoints are reachable but path between requires 
        // squeezing through a gap that's too narrow
        let mut terrain_map = TerrainMap::new(5, 5, 32.0);
        
        // Create narrow gap:
        // [G G G G G]
        // [G S S S G]
        // [G S G S G] <- narrow 1-tile gap
        // [G S S S G]
        // [G G G G G]
        for x in 0..5 {
            for y in 0..5 {
                terrain_map.set_tile(x, y, TerrainType::Grass);
            }
        }
        
        // Create walls with narrow gap
        for x in 1..4 {
            terrain_map.set_tile(x, 1, TerrainType::Stone);
            terrain_map.set_tile(x, 3, TerrainType::Stone);
        }
        terrain_map.set_tile(1, 2, TerrainType::Stone);
        terrain_map.set_tile(3, 2, TerrainType::Stone);
        // Gap at (2,2) remains grass
        
        let start = terrain_map.tile_to_world_coords(2, 0); // Above gap
        let goal = terrain_map.tile_to_world_coords(2, 4);  // Below gap
        
        // Small pawn should pass through narrow gap
        let path = terrain_map.find_path_for_size(start, goal, 0.5);
        assert!(path.is_some(), "Small pawn should fit through narrow gap");
        
        // Large pawn should be blocked by narrow gap
        let path = terrain_map.find_path_for_size(start, goal, 2.2);
        assert!(path.is_none(), "Large pawn should be blocked by narrow gap");
    }

    #[test]
    fn test_path_crosses_multiple_stone_tiles() {
        // Test path that would cross multiple contiguous stone tiles
        let mut terrain_map = TerrainMap::new(8, 8, 32.0);
        
        // Fill with grass
        for x in 0..8 {
            for y in 0..8 {
                terrain_map.set_tile(x, y, TerrainType::Grass);
            }
        }
        
        // Create a stone diagonal from (2,2) to (5,5)
        terrain_map.set_tile(2, 2, TerrainType::Stone);
        terrain_map.set_tile(3, 3, TerrainType::Stone);
        terrain_map.set_tile(4, 4, TerrainType::Stone);
        terrain_map.set_tile(5, 5, TerrainType::Stone);
        
        let start = terrain_map.tile_to_world_coords(1, 1);
        let goal = terrain_map.tile_to_world_coords(6, 6);
        
        // Small pawn should find a path around the stones
        let path = terrain_map.find_path_for_size(start, goal, 0.5);
        assert!(path.is_some(), "Small pawn should find path around stone diagonal");
        
        // Medium pawn should also find path around stones
        let path = terrain_map.find_path_for_size(start, goal, 1.2);
        assert!(path.is_some(), "Medium pawn should find path around stone diagonal");
        
        // Verify path doesn't cross stones for medium pawn
        if let Some(path_points) = path {
            for point in path_points {
                assert!(terrain_map.is_position_passable_for_size(point.0, point.1, 1.2),
                    "Path point {:?} should be passable for medium pawn", point);
            }
        }
    }

    #[test]
    fn test_long_path_segment_validation() {
        // Test very long path segments to ensure sampling works correctly
        let mut terrain_map = TerrainMap::new(10, 3, 32.0);
        
        // Create long corridor with obstacle in middle:
        // [G G G G S G G G G G]
        // [G G G G S G G G G G] 
        // [G G G G S G G G G G]
        for x in 0..10 {
            for y in 0..3 {
                if x == 4 {
                    terrain_map.set_tile(x, y, TerrainType::Stone);
                } else {
                    terrain_map.set_tile(x, y, TerrainType::Grass);
                }
            }
        }
        
        let start = terrain_map.tile_to_world_coords(0, 1);
        let goal = terrain_map.tile_to_world_coords(9, 1);
        
        // Small pawn should find path around the stone column
        let path = terrain_map.find_path_for_size(start, goal, 0.5);
        assert!(path.is_some(), "Small pawn should find path around stone column");
        
        // Large pawn should also navigate around but with more constraints
        let path = terrain_map.find_path_for_size(start, goal, 1.5);
        assert!(path.is_some(), "Large pawn should find path around stone column");
        
        // Verify path doesn't cross the stone column
        if let Some(path_points) = path {
            for point in path_points {
                let (px, _py) = point;
                // Should not be too close to x=4 where the stone column is
                let stone_world_x = terrain_map.tile_to_world_coords(4, 1).0;
                let distance_to_stone_column = (px - stone_world_x).abs();
                let pawn_radius = 1.5 * (terrain_map.tile_size / 2.0);
                let min_safe_distance = pawn_radius + (terrain_map.tile_size * 0.25);
                
                assert!(distance_to_stone_column >= min_safe_distance - 2.0, // Small epsilon
                    "Large pawn path point x={} too close to stone column at x={}", px, stone_world_x);
            }
        }
    }

    #[test]
    fn test_very_small_pawn_edge_cases() {
        // Test edge cases with very small pawns to ensure they don't get false negatives
        let mut terrain_map = TerrainMap::new(5, 5, 32.0);
        
        // Create scattered obstacles
        for x in 0..5 {
            for y in 0..5 {
                terrain_map.set_tile(x, y, TerrainType::Grass);
            }
        }
        terrain_map.set_tile(1, 1, TerrainType::Stone);
        terrain_map.set_tile(3, 3, TerrainType::Stone);
        
        let start = terrain_map.tile_to_world_coords(0, 0);
        let goal = terrain_map.tile_to_world_coords(4, 4);
        
        // Very small pawn should navigate easily
        let path = terrain_map.find_path_for_size(start, goal, 0.1);
        assert!(path.is_some(), "Very small pawn should find path easily");
        
        // Tiny pawn should also work
        let path = terrain_map.find_path_for_size(start, goal, 0.01);
        assert!(path.is_some(), "Tiny pawn should find path easily");
    }

    #[test]
    fn test_very_large_pawn_edge_cases() {
        // Test edge cases with very large pawns
        let mut terrain_map = TerrainMap::new(15, 15, 32.0); // Large map for large pawns
        
        // Fill mostly with grass, few scattered stones
        for x in 0..15 {
            for y in 0..15 {
                terrain_map.set_tile(x, y, TerrainType::Grass);
            }
        }
        
        // Add some scattered obstacles
        terrain_map.set_tile(5, 5, TerrainType::Stone);
        terrain_map.set_tile(10, 10, TerrainType::Stone);
        
        let start = terrain_map.tile_to_world_coords(2, 2);
        let goal = terrain_map.tile_to_world_coords(12, 12);
        
        // Very large pawn should still find path in spacious area
        let path = terrain_map.find_path_for_size(start, goal, 5.0);
        assert!(path.is_some(), "Very large pawn should find path in spacious area");
        
        // Extremely large pawn should still work if there's enough space
        let path = terrain_map.find_path_for_size(start, goal, 8.0);
        if let Some(path_points) = path {
            // Verify all path points are passable for the huge pawn
            for point in path_points {
                assert!(terrain_map.is_position_passable_for_size(point.0, point.1, 8.0),
                    "Path point {:?} should be passable for huge pawn", point);
            }
        }
    }

    #[test]
    fn test_zero_and_negative_size_edge_cases() {
        // Test edge cases with zero or invalid sizes
        let mut terrain_map = TerrainMap::new(5, 5, 32.0);
        
        for x in 0..5 {
            for y in 0..5 {
                terrain_map.set_tile(x, y, TerrainType::Grass);
            }
        }
        terrain_map.set_tile(2, 2, TerrainType::Stone);
        
        let start = terrain_map.tile_to_world_coords(0, 0);
        let goal = terrain_map.tile_to_world_coords(4, 4);
        
        // Zero size pawn should work (treated as point)
        let path = terrain_map.find_path_for_size(start, goal, 0.0);
        assert!(path.is_some(), "Zero size pawn should find path");
        
        // Very close to zero should also work
        let path = terrain_map.find_path_for_size(start, goal, 0.001);
        assert!(path.is_some(), "Near-zero size pawn should find path");
    }

    #[test]
    fn test_horizontal_and_vertical_path_segments() {
        // Test purely horizontal and vertical movements
        let mut terrain_map = TerrainMap::new(6, 6, 32.0);
        
        // Create obstacle pattern:
        // [G G G G G G]
        // [G S S S S G]
        // [G G G G G G] 
        // [G G G G G G]
        // [G S S S S G]
        // [G G G G G G]
        for x in 0..6 {
            for y in 0..6 {
                terrain_map.set_tile(x, y, TerrainType::Grass);
            }
        }
        
        // Horizontal stone barriers
        for x in 1..5 {
            terrain_map.set_tile(x, 1, TerrainType::Stone);
            terrain_map.set_tile(x, 4, TerrainType::Stone);
        }
        
        // Test horizontal movement
        let start = terrain_map.tile_to_world_coords(0, 2);
        let goal = terrain_map.tile_to_world_coords(5, 2);
        
        let path = terrain_map.find_path_for_size(start, goal, 1.0);
        assert!(path.is_some(), "Horizontal path should be found");
        
        // Test vertical movement  
        let start = terrain_map.tile_to_world_coords(2, 0);
        let goal = terrain_map.tile_to_world_coords(2, 5);
        
        let path = terrain_map.find_path_for_size(start, goal, 1.0);
        assert!(path.is_some(), "Vertical path should be found");
    }

    #[test]
    fn test_path_segment_sampling_accuracy() {
        // Test that path segment sampling catches obstacles it should catch
        let mut terrain_map = TerrainMap::new(4, 4, 32.0);
        
        // Create diagonal obstacle line
        for x in 0..4 {
            for y in 0..4 {
                terrain_map.set_tile(x, y, TerrainType::Grass);
            }
        }
        
        // Diagonal line of stones from bottom-left to top-right
        terrain_map.set_tile(1, 1, TerrainType::Stone);
        terrain_map.set_tile(2, 2, TerrainType::Stone);
        
        // Try to path diagonally across the stone line
        let start = terrain_map.tile_to_world_coords(0, 0);
        let goal = terrain_map.tile_to_world_coords(3, 3);
        
        // Small pawn should find alternate path
        let path = terrain_map.find_path_for_size(start, goal, 0.5);
        assert!(path.is_some(), "Small pawn should find alternate path around stone line");
        
        // Medium pawn should also avoid the stone line
        let path = terrain_map.find_path_for_size(start, goal, 1.0);
        if let Some(path_points) = path {
            // Verify the path doesn't go through the stone tiles
            for point in path_points {
                assert!(terrain_map.is_position_passable_for_size(point.0, point.1, 1.0),
                    "Path point {:?} should be passable for medium pawn", point);
            }
        }
    }
}