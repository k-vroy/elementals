#[cfg(test)]
mod tests {
    use crate::systems::world_gen::{TerrainMap, TerrainType, TerrainChanges};

    #[test]
    fn test_terrain_modification_tracking() {
        let mut terrain_map = TerrainMap::new(5, 5, 32.0);
        let mut terrain_changes = TerrainChanges::default();
        
        // Initially empty changes
        assert!(terrain_changes.changed_tiles.is_empty());
        
        // Set a tile and verify it's tracked
        let success = terrain_map.set_tile_at_world_pos(0.0, 0.0, TerrainType::Stone, &mut terrain_changes);
        assert!(success);
        assert_eq!(terrain_changes.changed_tiles.len(), 1);
        
        // Verify the change is recorded correctly
        let (_x, _y, terrain_type) = terrain_changes.changed_tiles[0];
        assert_eq!(terrain_type, TerrainType::Stone);
        
        // Clear changes and verify
        terrain_changes.clear();
        assert!(terrain_changes.changed_tiles.is_empty());
    }
    
    #[test]
    fn test_terrain_passability_toggle_logic() {
        // Test the logic used in debug terrain editing
        
        // Passable terrain should become stone
        assert!(TerrainType::Grass.is_passable());
        assert!(TerrainType::Dirt.is_passable());
        
        // Impassable terrain should become dirt  
        assert!(!TerrainType::Stone.is_passable());
        assert!(!TerrainType::Water.is_passable());
        
        // Verify the toggle logic
        let passable_terrain = TerrainType::Grass;
        let impassable_terrain = TerrainType::Stone;
        
        let new_terrain_from_passable = if passable_terrain.is_passable() {
            TerrainType::Stone
        } else {
            TerrainType::Dirt
        };
        
        let new_terrain_from_impassable = if impassable_terrain.is_passable() {
            TerrainType::Stone
        } else {
            TerrainType::Dirt
        };
        
        assert_eq!(new_terrain_from_passable, TerrainType::Stone);
        assert_eq!(new_terrain_from_impassable, TerrainType::Dirt);
    }
}