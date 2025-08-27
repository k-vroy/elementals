#[cfg(test)]
mod tests {
    use crate::systems::world_gen::{TerrainMap, TerrainChanges};
    use crate::tests::create_test_ground_configs;

    #[test]
    fn test_terrain_modification_tracking() {
        let mut terrain_map = TerrainMap::new(5, 5, 32.0);
        let mut terrain_changes = TerrainChanges::default();
        
        // Initially empty changes
        assert!(terrain_changes.changed_tiles.is_empty());
        
        let ground_configs = create_test_ground_configs();
        let stone_type = *ground_configs.terrain_mapping.get("stone").unwrap_or(&3);
        
        // Set a tile and verify it's tracked
        let success = terrain_map.set_tile_at_world_pos(0.0, 0.0, stone_type, &mut terrain_changes);
        assert!(success);
        assert_eq!(terrain_changes.changed_tiles.len(), 1);
        
        // Verify the change is recorded correctly
        let (_x, _y, terrain_type) = terrain_changes.changed_tiles[0];
        assert_eq!(terrain_type, stone_type);
        
        // Clear changes and verify
        terrain_changes.clear();
        assert!(terrain_changes.changed_tiles.is_empty());
    }
    
    #[test]
    fn test_terrain_passability_toggle_logic() {
        // Test the logic used in debug terrain editing
        let ground_configs = create_test_ground_configs();
        let grass_type = *ground_configs.terrain_mapping.get("grass").unwrap_or(&2);
        let dirt_type = *ground_configs.terrain_mapping.get("dirt").unwrap_or(&1);
        let stone_type = *ground_configs.terrain_mapping.get("stone").unwrap_or(&3);
        let water_type = *ground_configs.terrain_mapping.get("water").unwrap_or(&0);
        
        // Passable terrain should become stone
        assert!(ground_configs.is_passable(grass_type));
        assert!(ground_configs.is_passable(dirt_type));
        
        // Impassable terrain should become dirt  
        assert!(!ground_configs.is_passable(stone_type));
        assert!(!ground_configs.is_passable(water_type));
        
        // Verify the toggle logic
        let passable_terrain = grass_type;
        let impassable_terrain = stone_type;
        
        let new_terrain_from_passable = if ground_configs.is_passable(passable_terrain) {
            stone_type
        } else {
            dirt_type
        };
        
        let new_terrain_from_impassable = if ground_configs.is_passable(impassable_terrain) {
            stone_type
        } else {
            dirt_type
        };
        
        assert_eq!(new_terrain_from_passable, stone_type);
        assert_eq!(new_terrain_from_impassable, dirt_type);
    }
}