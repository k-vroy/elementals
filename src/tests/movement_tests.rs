use bevy::prelude::*;
use crate::systems::world_gen::{TerrainMap, TerrainType};
use crate::systems::player::PlayerTarget;
use crate::tests::create_test_terrain_map;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_spawns_on_passable_terrain() {
        let terrain_map = create_test_terrain_map(10, 10, 32.0);
        
        // Test that spawn position finder works
        let spawn_pos = terrain_map.find_nearest_passable_tile((0.0, 0.0));
        assert!(spawn_pos.is_some(), "Should find a passable spawn position");
        
        let (x, y) = spawn_pos.unwrap();
        let (tile_x, tile_y) = terrain_map.world_to_tile_coords(x, y).unwrap();
        assert!(terrain_map.is_tile_passable(tile_x, tile_y), "Spawn position should be passable");
    }

    #[test]
    fn test_player_cannot_move_through_stone() {
        let terrain_map = create_test_terrain_map(10, 10, 32.0);
        
        // Stone is at (5, 5) in our test map
        let stone_world_pos = terrain_map.tile_to_world_coords(5, 5);
        assert!(!terrain_map.is_passable_at_world_pos(stone_world_pos.0, stone_world_pos.1), 
                "Stone tiles should be impassable");
    }

    #[test]
    fn test_player_cannot_move_through_water() {
        let terrain_map = create_test_terrain_map(10, 10, 32.0);
        
        // Water is in left third of map
        let water_world_pos = terrain_map.tile_to_world_coords(2, 5);
        assert!(!terrain_map.is_passable_at_world_pos(water_world_pos.0, water_world_pos.1), 
                "Water tiles should be impassable");
    }

    #[test]
    fn test_pathfinding_around_obstacles() {
        let terrain_map = create_test_terrain_map(10, 10, 32.0);
        
        // Use tile coordinates and convert to world coordinates for more predictable testing
        let start_tile = (8, 8);
        let end_tile = (8, 0); // Move to a known passable location (grass border)
        
        let start = terrain_map.tile_to_world_coords(start_tile.0, start_tile.1);
        let end = terrain_map.tile_to_world_coords(end_tile.0, end_tile.1);
        
        // Verify start and end are both passable
        assert!(terrain_map.is_passable_at_world_pos(start.0, start.1), "Start should be passable");
        assert!(terrain_map.is_passable_at_world_pos(end.0, end.1), "End should be passable");
        
        let path = terrain_map.find_path(start, end);
        assert!(path.is_some(), "Should find a path between passable locations");
        
        let path = path.unwrap();
        assert!(path.len() >= 2, "Path should have at least start and end points");
        
        // Verify all path points are on passable terrain
        for &(x, y) in &path {
            assert!(terrain_map.is_passable_at_world_pos(x, y), 
                    "All path points should be on passable terrain at ({}, {})", x, y);
        }
    }

    #[test]
    fn test_pathfinding_fails_when_no_path_exists() {
        // Create a map where destination is completely surrounded by impassable terrain
        let mut terrain_map = TerrainMap::new(5, 5, 32.0);
        
        // Fill with grass
        for x in 0..5 {
            for y in 0..5 {
                terrain_map.set_tile(x, y, TerrainType::Grass);
            }
        }
        
        // Surround center completely with stone
        for x in 1..4 {
            for y in 1..4 {
                terrain_map.set_tile(x, y, TerrainType::Stone);
            }
        }
        
        // Make center passable but unreachable
        terrain_map.set_tile(2, 2, TerrainType::Grass);
        
        let start = terrain_map.tile_to_world_coords(0, 0);
        let end = terrain_map.tile_to_world_coords(2, 2); // Surrounded center
        
        // Verify start is passable but end is unreachable (surrounded by stone)
        assert!(terrain_map.is_passable_at_world_pos(start.0, start.1), "Start should be passable");
        assert!(terrain_map.is_passable_at_world_pos(end.0, end.1), "End should be passable");
        
        let path = terrain_map.find_path(start, end);
        assert!(path.is_none(), "Should not find path when destination is unreachable");
    }

    #[test]
    fn test_player_target_creation() {
        let target_pos = Vec3::new(100.0, 200.0, 0.0);
        let move_speed = 150.0;
        
        let player_target = PlayerTarget::new(target_pos, move_speed);
        
        assert_eq!(player_target.target_position, target_pos);
        assert_eq!(player_target.move_speed, move_speed);
        assert_eq!(player_target.path.len(), 1);
        assert_eq!(player_target.path[0], target_pos);
        assert_eq!(player_target.current_waypoint_index, 0);
    }

    #[test]
    fn test_player_target_path_setting() {
        let mut player_target = PlayerTarget::new(Vec3::ZERO, 100.0);
        
        let path = vec![(10.0, 20.0), (30.0, 40.0), (50.0, 60.0)];
        player_target.set_path(path.clone());
        
        assert_eq!(player_target.path.len(), 3);
        assert_eq!(player_target.current_waypoint_index, 0);
        
        // Check that target position is set to last waypoint
        let expected_target = Vec3::new(50.0, 60.0, 100.0);
        assert_eq!(player_target.target_position, expected_target);
    }

    #[test]
    fn test_player_target_waypoint_advancement() {
        let mut player_target = PlayerTarget::new(Vec3::ZERO, 100.0);
        let path = vec![(10.0, 20.0), (30.0, 40.0), (50.0, 60.0)];
        player_target.set_path(path);
        
        // Initially at first waypoint
        assert_eq!(player_target.current_waypoint_index, 0);
        assert!(!player_target.is_at_destination());
        
        // Advance waypoint
        player_target.advance_waypoint();
        assert_eq!(player_target.current_waypoint_index, 1);
        assert!(!player_target.is_at_destination());
        
        // Advance to last waypoint
        player_target.advance_waypoint();
        assert_eq!(player_target.current_waypoint_index, 2);
        assert!(player_target.is_at_destination());
        
        // Advancing past end shouldn't change index
        player_target.advance_waypoint();
        assert_eq!(player_target.current_waypoint_index, 2);
    }

    #[test]
    fn test_player_target_reset() {
        let mut player_target = PlayerTarget::new(Vec3::new(100.0, 100.0, 0.0), 150.0);
        let path = vec![(10.0, 20.0), (30.0, 40.0)];
        player_target.set_path(path);
        player_target.advance_waypoint();
        
        // Before reset
        assert!(!player_target.path.is_empty());
        assert_ne!(player_target.current_waypoint_index, 0);
        assert_ne!(player_target.target_position, Vec3::ZERO);
        
        // After reset
        player_target.reset();
        assert!(player_target.path.is_empty());
        assert_eq!(player_target.current_waypoint_index, 0);
        assert_eq!(player_target.target_position, Vec3::ZERO);
    }

    #[test]
    fn test_coordinate_conversion_accuracy() {
        let terrain_map = create_test_terrain_map(10, 10, 32.0);
        
        // Test round-trip conversion
        for tile_x in 0..10 {
            for tile_y in 0..10 {
                let (world_x, world_y) = terrain_map.tile_to_world_coords(tile_x, tile_y);
                let converted_back = terrain_map.world_to_tile_coords(world_x, world_y);
                
                assert_eq!(converted_back, Some((tile_x, tile_y)), 
                          "Coordinate conversion should be accurate for tile ({}, {})", tile_x, tile_y);
            }
        }
    }

    #[test]
    fn test_map_boundary_handling() {
        let terrain_map = create_test_terrain_map(5, 5, 32.0);
        
        // Test out-of-bounds coordinates
        assert_eq!(terrain_map.world_to_tile_coords(-1000.0, -1000.0), None);
        assert_eq!(terrain_map.world_to_tile_coords(1000.0, 1000.0), None);
        
        // Test out-of-bounds tile access
        assert!(!terrain_map.is_tile_passable(-1, -1));
        assert!(!terrain_map.is_tile_passable(10, 10));
        
        // Test out-of-bounds world position access
        assert!(!terrain_map.is_passable_at_world_pos(-1000.0, -1000.0));
        assert_eq!(terrain_map.get_terrain_at_world_pos(-1000.0, -1000.0), None);
    }
}