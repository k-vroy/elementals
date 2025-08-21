use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};
use std::time::{Duration, Instant};
use crate::systems::world_gen::{TerrainChanges, TerrainMap};

/// High-performance pathfinding cache with event-driven invalidation
#[derive(Resource)]
pub struct PathfindingCache {
    // Path cache with spatial invalidation
    path_cache: HashMap<PathCacheKey, CachedPathResult>,
    // Passability cache for expensive position checks
    passability_cache: HashMap<PassabilityCacheKey, CachedPassability>,
    // Current terrain version - incremented when terrain changes
    pub terrain_version: u64,
    // Spatial index for efficient cache invalidation
    spatial_index: HashMap<(u32, u32), Vec<PathCacheKey>>, // tile -> affected cache keys
    // Performance metrics
    pub stats: CacheStats,
}

#[derive(Hash, PartialEq, Eq, Clone)]
struct PathCacheKey {
    start_tile: (i32, i32),
    goal_tile: (i32, i32),
    size_tier: u8, // Quantized size to reduce cache fragmentation
}

#[derive(Hash, PartialEq, Eq, Clone)]
struct PassabilityCacheKey {
    tile_x: i32,
    tile_y: i32,
    size_tier: u8,
}

struct CachedPathResult {
    path: Option<Vec<(f32, f32)>>,
    terrain_version: u64, // Version when computed
    last_accessed: Instant,
    // Store tiles this path crosses for invalidation
    affected_tiles: HashSet<(u32, u32)>,
}

struct CachedPassability {
    is_passable: bool,
    terrain_version: u64,
    last_accessed: Instant,
}

#[derive(Default)]
pub struct CacheStats {
    pub path_cache_hits: u64,
    pub path_cache_misses: u64,
    pub passability_cache_hits: u64,
    pub passability_cache_misses: u64,
    pub terrain_invalidations: u64,
    pub cache_size: usize,
}

impl PathfindingCache {
    pub fn new() -> Self {
        Self {
            path_cache: HashMap::with_capacity(512),
            passability_cache: HashMap::with_capacity(1024),
            terrain_version: 1,
            spatial_index: HashMap::new(),
            stats: CacheStats::default(),
        }
    }

    /// Update cache based on terrain changes - called when terrain is modified
    pub fn invalidate_from_terrain_changes(&mut self, terrain_changes: &TerrainChanges) {
        if terrain_changes.changed_tiles.is_empty() {
            return;
        }

        // Increment version for new computations
        self.terrain_version += 1;
        self.stats.terrain_invalidations += 1;

        let mut keys_to_remove = HashSet::new();

        // Invalidate paths that cross changed tiles
        for (x, y, _terrain_type) in &terrain_changes.changed_tiles {
            // Remove passability cache entries for this tile and nearby tiles
            self.invalidate_passability_around_tile(*x as i32, *y as i32);

            // Find and invalidate paths that cross this tile
            if let Some(affected_keys) = self.spatial_index.get(&(*x, *y)) {
                for key in affected_keys {
                    keys_to_remove.insert(key.clone());
                }
            }
        }

        // Remove invalidated path cache entries
        for key in keys_to_remove {
            if let Some(cached_path) = self.path_cache.remove(&key) {
                // Clean up spatial index
                for tile in &cached_path.affected_tiles {
                    if let Some(keys) = self.spatial_index.get_mut(tile) {
                        keys.retain(|k| k != &key);
                        if keys.is_empty() {
                            self.spatial_index.remove(tile);
                        }
                    }
                }
            }
        }

        self.update_stats();
    }

    fn invalidate_passability_around_tile(&mut self, center_x: i32, center_y: i32) {
        // Remove passability cache in a radius around the changed tile
        // (since large pawns can be affected by changes in nearby tiles)
        const INVALIDATION_RADIUS: i32 = 3;
        
        self.passability_cache.retain(|key, _| {
            let dx = (key.tile_x - center_x).abs();
            let dy = (key.tile_y - center_y).abs();
            dx > INVALIDATION_RADIUS || dy > INVALIDATION_RADIUS
        });
    }

    /// Get cached path if valid - returns cloned result to avoid lifetime issues
    pub fn get_path(&mut self, start: (i32, i32), goal: (i32, i32), size: f32) -> Option<Option<Vec<(f32, f32)>>> {
        let key = PathCacheKey {
            start_tile: start,
            goal_tile: goal,
            size_tier: self.quantize_size(size),
        };

        // Check if entry exists and is valid
        let should_remove = if let Some(cached) = self.path_cache.get(&key) {
            if cached.terrain_version == self.terrain_version {
                self.stats.path_cache_hits += 1;
                // Update access time in a separate call to avoid borrowing issues
                let result = cached.path.clone();
                // Update last accessed time
                if let Some(cached_mut) = self.path_cache.get_mut(&key) {
                    cached_mut.last_accessed = Instant::now();
                }
                return Some(result);
            } else {
                true // Mark for removal
            }
        } else {
            false
        };

        // Remove stale entry if needed
        if should_remove {
            if let Some(old_entry) = self.path_cache.remove(&key) {
                self.cleanup_spatial_index(&key, &old_entry.affected_tiles);
            }
        }

        self.stats.path_cache_misses += 1;
        None
    }

    /// Cache a computed path with spatial indexing
    pub fn cache_path(&mut self, start: (i32, i32), goal: (i32, i32), size: f32, path: Option<Vec<(f32, f32)>>, terrain_map: &TerrainMap) {
        let key = PathCacheKey {
            start_tile: start,
            goal_tile: goal,
            size_tier: self.quantize_size(size),
        };

        // Determine which tiles this path affects
        let affected_tiles = if let Some(ref path_points) = path {
            self.get_affected_tiles(path_points, size, terrain_map)
        } else {
            HashSet::new()
        };

        // Update spatial index
        for tile in &affected_tiles {
            self.spatial_index
                .entry(*tile)
                .or_insert_with(Vec::new)
                .push(key.clone());
        }

        let cached_result = CachedPathResult {
            path,
            terrain_version: self.terrain_version,
            last_accessed: Instant::now(),
            affected_tiles,
        };

        self.path_cache.insert(key, cached_result);
        self.update_stats();
    }

    /// Get cached passability result
    pub fn get_passability(&mut self, tile_x: i32, tile_y: i32, size: f32) -> Option<bool> {
        let key = PassabilityCacheKey {
            tile_x,
            tile_y,
            size_tier: self.quantize_size(size),
        };

        if let Some(cached) = self.passability_cache.get_mut(&key) {
            if cached.terrain_version == self.terrain_version {
                cached.last_accessed = Instant::now();
                self.stats.passability_cache_hits += 1;
                return Some(cached.is_passable);
            } else {
                // Remove stale entry
                self.passability_cache.remove(&key);
            }
        }

        self.stats.passability_cache_misses += 1;
        None
    }

    /// Cache passability result
    pub fn cache_passability(&mut self, tile_x: i32, tile_y: i32, size: f32, is_passable: bool) {
        let key = PassabilityCacheKey {
            tile_x,
            tile_y,
            size_tier: self.quantize_size(size),
        };

        let cached = CachedPassability {
            is_passable,
            terrain_version: self.terrain_version,
            last_accessed: Instant::now(),
        };

        self.passability_cache.insert(key, cached);
    }

    /// Periodic cleanup of old cache entries
    pub fn cleanup_expired_entries(&mut self) {
        let expiry_time = Duration::from_secs(30); // Keep entries for 30 seconds
        let now = Instant::now();

        // Clean up old path cache entries
        let mut expired_keys = Vec::new();
        self.path_cache.retain(|key, cached| {
            if now.duration_since(cached.last_accessed) > expiry_time {
                expired_keys.push((key.clone(), cached.affected_tiles.clone()));
                false
            } else {
                true
            }
        });

        // Clean up spatial index for expired entries
        for (key, affected_tiles) in expired_keys {
            self.cleanup_spatial_index(&key, &affected_tiles);
        }

        // Clean up old passability cache entries
        self.passability_cache.retain(|_, cached| {
            now.duration_since(cached.last_accessed) <= expiry_time
        });

        self.update_stats();
    }

    fn quantize_size(&self, size: f32) -> u8 {
        // Quantize to reduce cache fragmentation while maintaining accuracy
        (size * 8.0).round().min(255.0) as u8
    }

    fn get_affected_tiles(&self, path: &[(f32, f32)], size: f32, terrain_map: &TerrainMap) -> HashSet<(u32, u32)> {
        let mut affected = HashSet::new();
        let radius_in_tiles = ((size * terrain_map.tile_size / 2.0) / terrain_map.tile_size).ceil() as i32;

        for point in path {
            if let Some((center_x, center_y)) = terrain_map.world_to_tile_coords(point.0, point.1) {
                // Add tiles in radius around this path point
                for dx in -radius_in_tiles..=radius_in_tiles {
                    for dy in -radius_in_tiles..=radius_in_tiles {
                        let tile_x = center_x + dx;
                        let tile_y = center_y + dy;
                        if tile_x >= 0 && tile_y >= 0 {
                            affected.insert((tile_x as u32, tile_y as u32));
                        }
                    }
                }
            }
        }

        affected
    }

    fn cleanup_spatial_index(&mut self, key: &PathCacheKey, affected_tiles: &HashSet<(u32, u32)>) {
        for tile in affected_tiles {
            if let Some(keys) = self.spatial_index.get_mut(tile) {
                keys.retain(|k| k != key);
                if keys.is_empty() {
                    self.spatial_index.remove(tile);
                }
            }
        }
    }

    fn update_stats(&mut self) {
        self.stats.cache_size = self.path_cache.len() + self.passability_cache.len();
    }

    /// Get cache hit ratio for performance monitoring
    pub fn get_hit_ratio(&self) -> f32 {
        let total_requests = self.stats.path_cache_hits + self.stats.path_cache_misses;
        if total_requests > 0 {
            self.stats.path_cache_hits as f32 / total_requests as f32
        } else {
            0.0
        }
    }
}

impl Default for PathfindingCache {
    fn default() -> Self {
        Self::new()
    }
}

/// System to update pathfinding cache when terrain changes
pub fn update_pathfinding_cache(
    mut cache: ResMut<PathfindingCache>,
    terrain_changes: Res<TerrainChanges>,
) {
    if terrain_changes.is_changed() {
        cache.invalidate_from_terrain_changes(&terrain_changes);
    }
}

/// System to periodically clean up expired cache entries
pub fn cleanup_pathfinding_cache(
    mut cache: ResMut<PathfindingCache>,
) {
    // Run cleanup every 5 seconds
    static mut LAST_CLEANUP: Option<Instant> = None;
    
    unsafe {
        let now = Instant::now();
        if LAST_CLEANUP.map_or(true, |last| now.duration_since(last) >= Duration::from_secs(5)) {
            cache.cleanup_expired_entries();
            LAST_CLEANUP = Some(now);
        }
    }
}