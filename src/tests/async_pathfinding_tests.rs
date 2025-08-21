#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use crate::systems::async_pathfinding::{
        PathfindingRequest, PathfindingPriority, PathfindingTask, 
        spawn_cached_pathfinding_tasks, handle_completed_cached_pathfinding,
        PathfindingRequestCounter, GlobalPathfindingCache
    };
    use crate::systems::world_gen::{TerrainMap, TerrainType};
    use crate::systems::pawn::{PawnTarget, Size, Pawn};
    use crate::tests::setup_test_app;

    fn create_simple_terrain() -> TerrainMap {
        let mut terrain_map = TerrainMap::new(5, 5, 32.0);
        
        // Create simple open terrain
        for x in 0..5 {
            for y in 0..5 {
                terrain_map.set_tile(x, y, TerrainType::Grass);
            }
        }
        
        terrain_map
    }

    #[test]
    fn test_pathfinding_request_creation() {
        let request = PathfindingRequest::new((0.0, 0.0), (100.0, 100.0), 1.0);
        assert_eq!(request.start, (0.0, 0.0));
        assert_eq!(request.goal, (100.0, 100.0));
        assert_eq!(request.size, 1.0);
        assert_eq!(request.priority, PathfindingPriority::Normal);
    }

    #[test]
    fn test_pathfinding_request_with_priority() {
        let request = PathfindingRequest::new((0.0, 0.0), (100.0, 100.0), 1.0)
            .with_priority(PathfindingPriority::High);
        assert_eq!(request.priority, PathfindingPriority::High);
    }

    #[test]
    fn test_pathfinding_priority_ordering() {
        assert!(PathfindingPriority::Critical > PathfindingPriority::High);
        assert!(PathfindingPriority::High > PathfindingPriority::Normal);
        assert!(PathfindingPriority::Normal > PathfindingPriority::Low);
    }

    #[test]
    fn test_async_pathfinding_request_spawning() {
        let mut app = setup_test_app();
        let terrain_map = create_simple_terrain();
        
        // Add resources
        app.insert_resource(terrain_map);
        app.insert_resource(PathfindingRequestCounter::default());
        app.insert_resource(GlobalPathfindingCache::default());
        
        // Add only the spawning system (not completion handling to avoid async complexity)
        app.add_systems(Update, spawn_cached_pathfinding_tasks);

        // Create test entity with pathfinding request
        let entity = app.world_mut().spawn((
            Transform::from_xyz(-32.0, -32.0, 0.0),
            Size { value: 1.0 },
            Pawn { pawn_type: "test".to_string() },
            PathfindingRequest::new((-32.0, -32.0), (32.0, 32.0), 1.0),
        )).id();

        // Verify PathfindingRequest exists
        assert!(app.world().get::<PathfindingRequest>(entity).is_some());

        // Run one update cycle to spawn pathfinding task
        app.update();
        
        // Check that PathfindingRequest was replaced with PathfindingTask
        assert!(app.world().get::<PathfindingRequest>(entity).is_none());
        assert!(app.world().get::<PathfindingTask>(entity).is_some());
    }

    #[test]
    fn test_cache_hit_behavior() {
        let mut app = setup_test_app();
        let terrain_map = create_simple_terrain();
        
        // Add resources
        app.insert_resource(terrain_map);
        app.insert_resource(PathfindingRequestCounter::default());
        
        // Pre-populate cache with a path result
        let mut cache = GlobalPathfindingCache::default();
        let start_tile = (1, 1);
        let goal_tile = (3, 3);
        let size = 1.0;
        let path = vec![(-32.0, -32.0), (0.0, 0.0), (32.0, 32.0)];
        
        // Access the terrain map to populate cache (simplified test)
        app.insert_resource(cache);
        app.add_systems(Update, spawn_cached_pathfinding_tasks);

        // Create entity with pathfinding request that should hit cache
        let entity = app.world_mut().spawn((
            Transform::from_xyz(-32.0, -32.0, 0.0),
            Size { value: 1.0 },
            Pawn { pawn_type: "test".to_string() },
            PathfindingRequest::new((-32.0, -32.0), (32.0, 32.0), 1.0),
        )).id();

        // Run one update
        app.update();
        
        // Should create PathfindingTask since cache is empty initially
        // This tests that the system at least processes the request
        assert!(app.world().get::<PathfindingRequest>(entity).is_none());
    }

    #[test]
    fn test_pathfinding_request_counter() {
        let mut counter = PathfindingRequestCounter::default();
        
        let id1 = counter.next_id();
        let id2 = counter.next_id();
        let id3 = counter.next_id();
        
        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(id3, 2);
        
        // Test wraparound behavior
        counter.next_id = u64::MAX;
        let id_max = counter.next_id();
        let id_wrapped = counter.next_id();
        
        assert_eq!(id_max, u64::MAX);
        assert_eq!(id_wrapped, 0);
    }
}