#[cfg(test)]
mod tests {
    use crate::systems::world_gen::{TerrainMap, TerrainChanges};
    use crate::systems::pathfinding_cache::PathfindingCache;
    use crate::tests::create_test_ground_configs;
    use std::time::Instant;

    fn create_test_terrain() -> TerrainMap {
        let mut terrain_map = TerrainMap::new(10, 10, 32.0);
        let ground_configs = create_test_ground_configs();
        let grass_type = *ground_configs.terrain_mapping.get("grass").unwrap_or(&2);
        let stone_type = *ground_configs.terrain_mapping.get("stone").unwrap_or(&3);
        
        // Create simple test layout:
        // [G G G G G G G G G G]
        // [G G G G G G G G G G]
        // [G G S S S S S G G G] 
        // [G G G G G G G G G G]
        // [G G G G G G G G G G]
        // [G G G G G G G G G G]
        // [G G G G G G G G G G]
        // [G G G G G G G G G G]
        // [G G G G G G G G G G]
        // [G G G G G G G G G G]
        
        for x in 0..10 {
            for y in 0..10 {
                if y == 2 && x >= 2 && x <= 6 {
                    terrain_map.set_tile(x, y, stone_type); // Horizontal barrier
                } else {
                    terrain_map.set_tile(x, y, grass_type);
                }
            }
        }
        
        terrain_map
    }
    
    fn create_test_terrain_with_configs() -> (TerrainMap, crate::systems::world_gen::GroundConfigs) {
        (create_test_terrain(), create_test_ground_configs())
    }
    
    // Helper function for tests to call pathfinding with ground configs
    fn find_path_cached_test(terrain_map: &TerrainMap, start: (f32, f32), goal: (f32, f32), size: f32, cache: &mut PathfindingCache) -> Option<Vec<(f32, f32)>> {
        let ground_configs = create_test_ground_configs();
        terrain_map.find_path_for_size_cached(start, goal, size, &ground_configs, cache)
    }

    #[test]
    fn test_cache_basic_hit_miss_behavior() {
        let terrain_map = create_test_terrain();
        let mut cache = PathfindingCache::new();
        
        let start = terrain_map.tile_to_world_coords(1, 1);
        let goal = terrain_map.tile_to_world_coords(8, 8);
        let size = 1.0;
        
        // First call should be a cache miss
        assert_eq!(cache.stats.path_cache_hits, 0);
        assert_eq!(cache.stats.path_cache_misses, 0);
        
        let path1 = find_path_cached_test(&terrain_map, start, goal, size, &mut cache);
        assert!(path1.is_some(), "Should find a path");
        assert_eq!(cache.stats.path_cache_hits, 0);
        assert_eq!(cache.stats.path_cache_misses, 1);
        
        // Second call with same parameters should be a cache hit
        let path2 = find_path_cached_test(&terrain_map, start, goal, size, &mut cache);
        assert!(path2.is_some(), "Should find cached path");
        assert_eq!(cache.stats.path_cache_hits, 1);
        assert_eq!(cache.stats.path_cache_misses, 1);
        
        // Paths should be identical
        assert_eq!(path1, path2, "Cached path should match original");
    }

    #[test]
    fn test_cache_size_quantization() {
        let terrain_map = create_test_terrain();
        let mut cache = PathfindingCache::new();
        
        let start = terrain_map.tile_to_world_coords(1, 1);
        let goal = terrain_map.tile_to_world_coords(8, 8);
        
        // First path with size 1.0
        let _path1 = find_path_cached_test(&terrain_map, start, goal, 1.0, &mut cache);
        assert_eq!(cache.stats.path_cache_misses, 1);
        
        // Very similar size should hit cache due to quantization
        let _path2 = find_path_cached_test(&terrain_map, start, goal, 1.01, &mut cache);
        assert_eq!(cache.stats.path_cache_hits, 1, "Similar sizes should hit cache");
        
        // Significantly different size should miss cache
        let _path3 = find_path_cached_test(&terrain_map, start, goal, 2.0, &mut cache);
        assert_eq!(cache.stats.path_cache_misses, 2, "Different size should miss cache");
    }

    #[test]
    fn test_cache_different_start_goal_combinations() {
        let terrain_map = create_test_terrain();
        let mut cache = PathfindingCache::new();
        
        let start1 = terrain_map.tile_to_world_coords(1, 1);
        let goal1 = terrain_map.tile_to_world_coords(8, 8);
        let start2 = terrain_map.tile_to_world_coords(1, 4);
        let goal2 = terrain_map.tile_to_world_coords(8, 4);
        let size = 1.0;
        
        // Cache different path combinations
        let _path1 = find_path_cached_test(&terrain_map, start1, goal1, size, &mut cache);
        let _path2 = find_path_cached_test(&terrain_map, start2, goal2, size, &mut cache);
        assert_eq!(cache.stats.path_cache_misses, 2, "Different paths should miss cache");
        
        // Repeat same combinations should hit cache
        let _path1_repeat = find_path_cached_test(&terrain_map, start1, goal1, size, &mut cache);
        let _path2_repeat = find_path_cached_test(&terrain_map, start2, goal2, size, &mut cache);
        assert_eq!(cache.stats.path_cache_hits, 2, "Same paths should hit cache");
    }

    #[test]
    fn test_passability_cache() {
        let terrain_map = create_test_terrain();
        let mut cache = PathfindingCache::new();
        
        let passable_pos = terrain_map.tile_to_world_coords(1, 1); // On grass
        let size = 1.0;
        
        // First checks should miss cache
        assert_eq!(cache.stats.passability_cache_hits, 0);
        let result1 = find_path_cached_test(&terrain_map, passable_pos, passable_pos, size, &mut cache);
        assert!(result1.is_some());
        
        // Check passability cache stats
        assert!(cache.stats.passability_cache_misses > 0, "Should have passability cache misses");
        
        // Subsequent checks of same positions should hit cache
        find_path_cached_test(&terrain_map, passable_pos, passable_pos, size, &mut cache);
        
        // Should have some cache hits now (though exact count depends on implementation details)
        assert!(cache.stats.passability_cache_hits > 0, "Should have passability cache hits");
    }

    #[test]
    fn test_terrain_change_invalidation() {
        let mut terrain_map = create_test_terrain();
        let mut cache = PathfindingCache::new();
        let mut terrain_changes = TerrainChanges::default();
        let ground_configs = create_test_ground_configs();
        let stone_type = *ground_configs.terrain_mapping.get("stone").unwrap_or(&3);
        
        let start = terrain_map.tile_to_world_coords(1, 1);
        let goal = terrain_map.tile_to_world_coords(8, 4);
        let size = 1.0;
        
        // Cache a path
        let path1 = find_path_cached_test(&terrain_map, start, goal, size, &mut cache);
        assert!(path1.is_some());
        assert_eq!(cache.stats.path_cache_misses, 1);
        
        // Verify cache hit
        let path2 = find_path_cached_test(&terrain_map, start, goal, size, &mut cache);
        assert_eq!(path1, path2);
        assert_eq!(cache.stats.path_cache_hits, 1);
        
        // Change terrain that affects the path
        terrain_map.set_tile_at_world_pos(
            terrain_map.tile_to_world_coords(4, 4).0,
            terrain_map.tile_to_world_coords(4, 4).1,
            stone_type,
            &mut terrain_changes
        );
        
        // Invalidate cache based on terrain changes
        cache.invalidate_from_terrain_changes(&terrain_changes);
        assert_eq!(cache.stats.terrain_invalidations, 1);
        
        // Next pathfinding should be a cache miss due to invalidation
        let path3 = find_path_cached_test(&terrain_map, start, goal, size, &mut cache);
        assert_eq!(cache.stats.path_cache_misses, 2, "Should miss cache after terrain change");
        
        // Path might be different due to new terrain
        // (We can't guarantee exact path comparison due to A* potentially finding different valid paths)
        assert!(path3.is_some(), "Should still find a path after terrain change");
        
        terrain_changes.clear();
    }

    #[test]
    fn test_terrain_invalidation_basic() {
        let terrain_map = create_test_terrain();
        let mut cache = PathfindingCache::new();
        let mut terrain_changes = TerrainChanges::default();
        let ground_configs = create_test_ground_configs();
        let stone_type = *ground_configs.terrain_mapping.get("stone").unwrap_or(&3);
        
        let start = terrain_map.tile_to_world_coords(1, 1);
        let goal = terrain_map.tile_to_world_coords(8, 8);
        let size = 1.0;
        
        // Cache a path
        let path1 = find_path_cached_test(&terrain_map, start, goal, size, &mut cache);
        assert!(path1.is_some());
        assert_eq!(cache.stats.path_cache_misses, 1);
        
        // Verify it's cached
        let path2 = find_path_cached_test(&terrain_map, start, goal, size, &mut cache);
        assert_eq!(path1, path2);
        assert_eq!(cache.stats.path_cache_hits, 1);
        
        // Make a terrain change somewhere
        terrain_changes.add_change(5, 5, stone_type);
        cache.invalidate_from_terrain_changes(&terrain_changes);
        
        // The invalidation system should work (exact behavior depends on spatial indexing)
        // Just verify that the cache system continues to function
        let path3 = find_path_cached_test(&terrain_map, start, goal, size, &mut cache);
        assert!(path3.is_some(), "Should still be able to find paths after invalidation");
        
        terrain_changes.clear();
    }

    #[test]
    fn test_passability_cache_basic_operation() {
        let terrain_map = create_test_terrain();
        let mut cache = PathfindingCache::new();
        
        let start = terrain_map.tile_to_world_coords(1, 1);
        let goal = terrain_map.tile_to_world_coords(8, 8);
        let size = 1.0;
        
        // First pathfinding should miss path cache and use passability cache
        let path1 = find_path_cached_test(&terrain_map, start, goal, size, &mut cache);
        assert!(path1.is_some());
        let initial_passability_checks = cache.stats.passability_cache_hits + cache.stats.passability_cache_misses;
        
        // Make a different pathfinding request that should reuse some passability cache entries
        let goal2 = terrain_map.tile_to_world_coords(7, 7);
        let path2 = find_path_cached_test(&terrain_map, start, goal2, size, &mut cache);
        assert!(path2.is_some(), "Should find path for similar request");
        
        // Should have some passability cache activity from the second request
        let final_passability_checks = cache.stats.passability_cache_hits + cache.stats.passability_cache_misses;
        assert!(final_passability_checks > initial_passability_checks,
               "Should have more passability cache activity from similar pathfinding request");
    }

    #[test]
    fn test_cache_hit_ratio_calculation() {
        let terrain_map = create_test_terrain();
        let mut cache = PathfindingCache::new();
        
        let start = terrain_map.tile_to_world_coords(1, 1);
        let goal = terrain_map.tile_to_world_coords(8, 8);
        let size = 1.0;
        
        // Initially no requests
        assert_eq!(cache.get_hit_ratio(), 0.0);
        
        // First call - miss
        let _path1 = find_path_cached_test(&terrain_map, start, goal, size, &mut cache);
        assert_eq!(cache.get_hit_ratio(), 0.0); // 0 hits / 1 total = 0%
        
        // Second call - hit
        let _path2 = find_path_cached_test(&terrain_map, start, goal, size, &mut cache);
        assert_eq!(cache.get_hit_ratio(), 0.5); // 1 hit / 2 total = 50%
        
        // Third call - hit
        let _path3 = find_path_cached_test(&terrain_map, start, goal, size, &mut cache);
        assert!((cache.get_hit_ratio() - 0.6666667).abs() < 0.001); // 2 hits / 3 total ≈ 66.7%
    }

    #[test]
    fn test_cache_cleanup_expired_entries() {
        let terrain_map = create_test_terrain();
        let mut cache = PathfindingCache::new();
        
        let start = terrain_map.tile_to_world_coords(1, 1);
        let goal = terrain_map.tile_to_world_coords(8, 8);
        let size = 1.0;
        
        // Cache a path
        let _path1 = find_path_cached_test(&terrain_map, start, goal, size, &mut cache);
        assert!(cache.stats.cache_size > 0);
        
        // Verify it's cached
        let _path2 = find_path_cached_test(&terrain_map, start, goal, size, &mut cache);
        assert_eq!(cache.stats.path_cache_hits, 1);
        
        // Manually run cleanup (in real game this would be called periodically)
        cache.cleanup_expired_entries();
        
        // Since we just accessed the cache, entries shouldn't be expired yet
        let _path3 = find_path_cached_test(&terrain_map, start, goal, size, &mut cache);
        assert_eq!(cache.stats.path_cache_hits, 2, "Recently accessed entries should not be cleaned up");
    }

    #[test]
    fn test_cache_performance_with_multiple_sizes() {
        let terrain_map = create_test_terrain();
        let mut cache = PathfindingCache::new();
        
        let start = terrain_map.tile_to_world_coords(1, 1);
        let goal = terrain_map.tile_to_world_coords(8, 8);
        
        let sizes = vec![0.5, 1.0, 1.5, 2.0, 2.5];
        
        // Cache paths for different sizes
        for size in &sizes {
            let _path = find_path_cached_test(&terrain_map, start, goal, *size, &mut cache);
        }
        assert_eq!(cache.stats.path_cache_misses, sizes.len() as u64);
        
        // Verify all sizes are independently cached
        for size in &sizes {
            let _path = find_path_cached_test(&terrain_map, start, goal, *size, &mut cache);
        }
        assert_eq!(cache.stats.path_cache_hits, sizes.len() as u64);
        
        // Cache should maintain separate entries for each size
        assert!(cache.stats.cache_size >= sizes.len(), "Should cache different sizes separately");
    }

    #[test]
    fn test_no_path_caching() {
        let mut terrain_map = create_test_terrain();
        let mut cache = PathfindingCache::new();
        let ground_configs = create_test_ground_configs();
        let stone_type = *ground_configs.terrain_mapping.get("stone").unwrap_or(&3);
        
        // Create completely blocked terrain
        for x in 0..10 {
            for y in 3..7 {
                terrain_map.set_tile(x, y, stone_type);
            }
        }
        
        let start = terrain_map.tile_to_world_coords(1, 1); // Above barrier
        let goal = terrain_map.tile_to_world_coords(1, 8);  // Below barrier
        let size = 2.0; // Large size that can't fit through
        
        // Should return None (no path)
        let path1 = find_path_cached_test(&terrain_map, start, goal, size, &mut cache);
        assert!(path1.is_none(), "Should not find path through complete barrier");
        assert_eq!(cache.stats.path_cache_misses, 1);
        
        // Should cache the "no path" result
        let path2 = find_path_cached_test(&terrain_map, start, goal, size, &mut cache);
        assert!(path2.is_none(), "Should return cached no-path result");
        assert_eq!(cache.stats.path_cache_hits, 1, "Should cache 'no path' results");
    }

    #[test]
    fn test_performance_benchmark_cache_vs_nocache() {
        let terrain_map = create_test_terrain();
        let mut cache = PathfindingCache::new();
        
        let start = terrain_map.tile_to_world_coords(1, 1);
        let goal = terrain_map.tile_to_world_coords(8, 8);
        let size = 1.0;
        
        // Benchmark uncached pathfinding (first call)
        let start_time = Instant::now();
        let _path1 = find_path_cached_test(&terrain_map, start, goal, size, &mut cache);
        let uncached_duration = start_time.elapsed();
        
        // Benchmark cached pathfinding (second call)
        let start_time = Instant::now();
        let _path2 = find_path_cached_test(&terrain_map, start, goal, size, &mut cache);
        let cached_duration = start_time.elapsed();
        
        // Cached call should be significantly faster (at least 3x faster)
        println!("Uncached: {:?}, Cached: {:?}", uncached_duration, cached_duration);
        assert!(cached_duration < uncached_duration / 3, 
               "Cached pathfinding should be at least 3x faster than uncached");
        
        // Verify cache stats
        assert_eq!(cache.stats.path_cache_hits, 1);
        assert_eq!(cache.stats.path_cache_misses, 1);
    }

    #[test]
    fn test_cache_memory_scaling() {
        let terrain_map = create_test_terrain();
        let mut cache = PathfindingCache::new();
        
        // Generate many different path requests
        let mut path_count = 0;
        for start_x in [1, 3, 5] {
            for start_y in [1, 4, 7] {
                for goal_x in [6, 8] {
                    for goal_y in [1, 4, 7] {
                        let start = terrain_map.tile_to_world_coords(start_x, start_y);
                        let goal = terrain_map.tile_to_world_coords(goal_x, goal_y);
                        let _path = find_path_cached_test(&terrain_map, start, goal, 1.0, &mut cache);
                        path_count += 1;
                    }
                }
            }
        }
        
        // Verify all paths were cached
        assert_eq!(cache.stats.path_cache_misses, path_count);
        assert!(cache.stats.cache_size > 0, "Cache should contain entries");
        
        // Test cache hit ratio by repeating some requests
        let start = terrain_map.tile_to_world_coords(1, 1);
        let goal = terrain_map.tile_to_world_coords(8, 8);
        for _ in 0..5 {
            let _path = find_path_cached_test(&terrain_map, start, goal, 1.0, &mut cache);
        }
        
        // Should have reasonable hit ratio (not necessarily > 0.5 due to many different initial paths)
        let hit_ratio = cache.get_hit_ratio();
        assert!(hit_ratio >= 0.0 && hit_ratio <= 1.0, "Hit ratio should be valid percentage: {}", hit_ratio);
        assert!(cache.stats.path_cache_hits > 0, "Should have some cache hits from repeated requests");
    }

    #[test]
    fn test_cache_with_multiple_paths() {
        let terrain_map = create_test_terrain();
        let mut cache = PathfindingCache::new();
        
        // Create multiple different paths
        let paths = vec![
            (terrain_map.tile_to_world_coords(1, 1), terrain_map.tile_to_world_coords(8, 1)),
            (terrain_map.tile_to_world_coords(1, 4), terrain_map.tile_to_world_coords(8, 4)),
            (terrain_map.tile_to_world_coords(1, 7), terrain_map.tile_to_world_coords(8, 7)),
        ];
        
        // Cache all paths
        for (start, goal) in &paths {
            let path = find_path_cached_test(&terrain_map, *start, *goal, 1.0, &mut cache);
            assert!(path.is_some(), "Should find path for {:?} to {:?}", start, goal);
        }
        assert_eq!(cache.stats.path_cache_misses, 3);
        
        // Verify all are cached  
        for (start, goal) in &paths {
            let path = find_path_cached_test(&terrain_map, *start, *goal, 1.0, &mut cache);
            assert!(path.is_some(), "Should find cached path for {:?} to {:?}", start, goal);
        }
        assert_eq!(cache.stats.path_cache_hits, 3, "All paths should be cached");
    }

    #[test]
    fn test_cache_with_large_terrain_map() {
        // Test with larger map to verify performance scales
        let mut terrain_map = TerrainMap::new(50, 50, 32.0);
        let ground_configs = create_test_ground_configs();
        let stone_type = *ground_configs.terrain_mapping.get("stone").unwrap_or(&3);
        
        // Create some obstacles
        for x in 10..40 {
            terrain_map.set_tile(x, 25, stone_type); // Horizontal barrier
        }
        for y in 10..20 {
            terrain_map.set_tile(15, y, stone_type); // Vertical barrier
        }
        
        let mut cache = PathfindingCache::new();
        
        // Test long-distance pathfinding
        let start = terrain_map.tile_to_world_coords(5, 5);
        let goal = terrain_map.tile_to_world_coords(45, 45);
        let size = 1.0;
        
        // First call - should compute complex path
        let start_time = Instant::now();
        let path1 = find_path_cached_test(&terrain_map, start, goal, size, &mut cache);
        let first_duration = start_time.elapsed();
        
        assert!(path1.is_some(), "Should find path in large map");
        assert_eq!(cache.stats.path_cache_misses, 1);
        
        // Second call - should use cache
        let start_time = Instant::now();
        let path2 = find_path_cached_test(&terrain_map, start, goal, size, &mut cache);
        let cached_duration = start_time.elapsed();
        
        assert_eq!(path1, path2, "Cached path should match original");
        assert_eq!(cache.stats.path_cache_hits, 1);
        
        // Cache should provide significant speedup for complex paths
        println!("Large map - First: {:?}, Cached: {:?}", first_duration, cached_duration);
        assert!(cached_duration < first_duration / 10, 
               "Cache should provide at least 10x speedup for complex paths");
    }

    #[test]
    fn test_cache_behavior_with_impossible_paths() {
        let mut terrain_map = TerrainMap::new(5, 5, 32.0);
        let ground_configs = create_test_ground_configs();
        let stone_type = *ground_configs.terrain_mapping.get("stone").unwrap_or(&3);
        
        // Create completely separated areas
        for x in 0..5 {
            terrain_map.set_tile(x, 2, stone_type); // Complete horizontal barrier
        }
        
        let mut cache = PathfindingCache::new();
        
        let start = terrain_map.tile_to_world_coords(1, 1); // Above barrier
        let goal = terrain_map.tile_to_world_coords(1, 3);  // Below barrier
        let size = 0.5;
        
        // Should return None quickly and cache the result
        let start_time = Instant::now();
        let path1 = find_path_cached_test(&terrain_map, start, goal, size, &mut cache);
        let first_duration = start_time.elapsed();
        
        assert!(path1.is_none(), "Should not find path through complete barrier");
        assert_eq!(cache.stats.path_cache_misses, 1);
        
        // Second call should be cached and very fast
        let start_time = Instant::now();
        let path2 = find_path_cached_test(&terrain_map, start, goal, size, &mut cache);
        let cached_duration = start_time.elapsed();
        
        assert!(path2.is_none(), "Cached result should also be None");
        assert_eq!(cache.stats.path_cache_hits, 1);
        
        // Even for impossible paths, cache should provide speedup
        assert!(cached_duration < first_duration, "Cached impossible path should be faster");
    }

    #[test]
    fn test_cache_version_management() {
        let terrain_map = create_test_terrain();
        let mut cache = PathfindingCache::new();
        let mut terrain_changes = TerrainChanges::default();
        let ground_configs = create_test_ground_configs();
        let stone_type = *ground_configs.terrain_mapping.get("stone").unwrap_or(&3);
        
        let start = terrain_map.tile_to_world_coords(1, 1);
        let goal = terrain_map.tile_to_world_coords(8, 8);
        let size = 1.0;
        
        // Cache initial path
        let _path1 = find_path_cached_test(&terrain_map, start, goal, size, &mut cache);
        let initial_version = cache.terrain_version;
        
        // Terrain change should increment version
        terrain_changes.add_change(5, 5, stone_type);
        cache.invalidate_from_terrain_changes(&terrain_changes);
        assert!(cache.terrain_version > initial_version, "Terrain version should increment");
        
        // New path should be computed with new version
        let _path2 = find_path_cached_test(&terrain_map, start, goal, size, &mut cache);
        assert_eq!(cache.stats.path_cache_misses, 2, "Should miss cache after version change");
        
        // Subsequent calls should use new cached version
        let _path3 = find_path_cached_test(&terrain_map, start, goal, size, &mut cache);
        assert_eq!(cache.stats.path_cache_hits, 1, "Should hit cache for new version");
        
        terrain_changes.clear();
    }
}