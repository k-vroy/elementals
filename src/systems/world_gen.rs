use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use crate::components::TerrainLayer;
use crate::resources::GameConfig;
use noise::{NoiseFn, Perlin, Simplex};
use pathfinding::prelude::astar;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GroundConfig {
    pub sprite: String,
    pub passable: bool,
    pub height_min: f32,
    pub height_max: f32,
}

#[derive(Debug, Clone, Resource)]
pub struct GroundConfigs {
    pub configs: HashMap<String, GroundConfig>,
    pub terrain_mapping: HashMap<String, usize>, // Maps config names to terrain type indices
}

impl GroundConfigs {
    pub fn load_from_yaml(yaml_content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let configs: HashMap<String, GroundConfig> = serde_yaml::from_str(yaml_content)?;
        
        // Create terrain mapping with deterministic order
        let mut sorted_names: Vec<_> = configs.keys().collect();
        sorted_names.sort();
        let terrain_mapping = sorted_names
            .iter()
            .enumerate()
            .map(|(i, name)| ((*name).clone(), i))
            .collect();
            
        Ok(Self {
            configs,
            terrain_mapping,
        })
    }
    
    pub fn get_terrain_type_for_height(&self, height: f32) -> Option<usize> {
        // Find the terrain type that matches the height range
        for (name, config) in &self.configs {
            if height >= config.height_min && height <= config.height_max {
                return self.terrain_mapping.get(name).copied();
            }
        }
        None
    }
    
    pub fn is_passable(&self, terrain_type: usize) -> bool {
        // Find the config by terrain type index
        for (name, config) in &self.configs {
            if let Some(&index) = self.terrain_mapping.get(name) {
                if index == terrain_type {
                    return config.passable;
                }
            }
        }
        false // Default to impassable if not found
    }
}

pub type TerrainType = usize;

#[derive(Resource, Clone)]
pub struct TerrainMap {
    pub width: u32,
    pub height: u32,
    pub tile_size: f32,
    pub tiles: Vec<Vec<TerrainType>>,
}

impl TerrainMap {
    pub fn new(width: u32, height: u32, tile_size: f32) -> Self {
        Self {
            width,
            height,
            tile_size,
            tiles: vec![vec![0; height as usize]; width as usize], // Default to first terrain type
        }
    }

    pub fn set_tile(&mut self, x: u32, y: u32, terrain_type: TerrainType) {
        if x < self.width && y < self.height {
            self.tiles[x as usize][y as usize] = terrain_type;
        }
    }

    pub fn get_terrain_at_world_pos(&self, world_x: f32, world_y: f32) -> Option<TerrainType> {
        // Convert world coordinates to tile coordinates
        // The tilemap is centered at (0,0), so we need to offset by half the map size
        let half_width = (self.width as f32 * self.tile_size) / 2.0;
        let half_height = (self.height as f32 * self.tile_size) / 2.0;
        
        let tile_x = ((world_x + half_width) / self.tile_size).floor() as i32;
        let tile_y = ((world_y + half_height) / self.tile_size).floor() as i32;

        if tile_x >= 0 && tile_x < self.width as i32 && tile_y >= 0 && tile_y < self.height as i32 {
            Some(self.tiles[tile_x as usize][tile_y as usize])
        } else {
            None
        }
    }

    pub fn is_passable_at_world_pos(&self, world_x: f32, world_y: f32, ground_configs: &GroundConfigs) -> bool {
        self.get_terrain_at_world_pos(world_x, world_y)
            .map(|terrain| ground_configs.is_passable(terrain))
            .unwrap_or(false) // If out of bounds, consider impassable
    }

    pub fn world_to_tile_coords(&self, world_x: f32, world_y: f32) -> Option<(i32, i32)> {
        let half_width = (self.width as f32 * self.tile_size) / 2.0;
        let half_height = (self.height as f32 * self.tile_size) / 2.0;
        
        let tile_x = ((world_x + half_width) / self.tile_size).floor() as i32;
        let tile_y = ((world_y + half_height) / self.tile_size).floor() as i32;

        if tile_x >= 0 && tile_x < self.width as i32 && tile_y >= 0 && tile_y < self.height as i32 {
            Some((tile_x, tile_y))
        } else {
            None
        }
    }

    pub fn tile_to_world_coords(&self, tile_x: i32, tile_y: i32) -> (f32, f32) {
        let half_width = (self.width as f32 * self.tile_size) / 2.0;
        let half_height = (self.height as f32 * self.tile_size) / 2.0;
        
        let world_x = (tile_x as f32 * self.tile_size) - half_width + (self.tile_size / 2.0);
        let world_y = (tile_y as f32 * self.tile_size) - half_height + (self.tile_size / 2.0);
        
        (world_x, world_y)
    }

    pub fn is_tile_passable(&self, tile_x: i32, tile_y: i32, ground_configs: &GroundConfigs) -> bool {
        if tile_x >= 0 && tile_x < self.width as i32 && tile_y >= 0 && tile_y < self.height as i32 {
            ground_configs.is_passable(self.tiles[tile_x as usize][tile_y as usize])
        } else {
            false // Out of bounds is impassable
        }
    }

    pub fn find_nearest_passable_tile(&self, start_world: (f32, f32), ground_configs: &GroundConfigs) -> Option<(f32, f32)> {
        // First check if the starting position is already passable
        if let Some((start_tile_x, start_tile_y)) = self.world_to_tile_coords(start_world.0, start_world.1) {
            if self.is_tile_passable(start_tile_x, start_tile_y, ground_configs) {
                return Some(self.tile_to_world_coords(start_tile_x, start_tile_y));
            }
        }

        // Search outward in expanding squares for nearest passable tile
        let center_tile = self.world_to_tile_coords(start_world.0, start_world.1)?;
        
        for radius in 1i32..=20 { // Search up to 20 tiles away
            for dx in -radius..=radius {
                for dy in -radius..=radius {
                    // Only check tiles on the perimeter of current radius
                    if dx.abs() != radius && dy.abs() != radius {
                        continue;
                    }
                    
                    let tile_x = center_tile.0 + dx;
                    let tile_y = center_tile.1 + dy;
                    
                    if self.is_tile_passable(tile_x, tile_y, ground_configs) {
                        return Some(self.tile_to_world_coords(tile_x, tile_y));
                    }
                }
            }
        }
        
        None // No passable tile found within reasonable distance
    }

    pub fn set_tile_at_world_pos(&mut self, world_x: f32, world_y: f32, terrain_type: TerrainType, terrain_changes: &mut TerrainChanges) -> bool {
        if let Some((tile_x, tile_y)) = self.world_to_tile_coords(world_x, world_y) {
            if tile_x >= 0 && tile_x < self.width as i32 && tile_y >= 0 && tile_y < self.height as i32 {
                self.tiles[tile_x as usize][tile_y as usize] = terrain_type;
                terrain_changes.add_change(tile_x as u32, tile_y as u32, terrain_type);
                return true;
            }
        }
        false
    }

    fn is_position_passable_for_size_with_cache(&self, world_x: f32, world_y: f32, size: f32, ground_configs: &GroundConfigs, cache: &mut Option<&mut crate::systems::pathfinding_cache::PathfindingCache>) -> bool {
        if let Some(cache) = cache {
            // Try cache first
            if let Some(tile_coords) = self.world_to_tile_coords(world_x, world_y) {
                if let Some(cached_result) = cache.get_passability(tile_coords.0, tile_coords.1, size) {
                    return cached_result;
                }
                
                // Not cached, compute and cache
                let result = self.is_position_passable_for_size(world_x, world_y, size, ground_configs);
                cache.cache_passability(tile_coords.0, tile_coords.1, size, result);
                result
            } else {
                false
            }
        } else {
            // No cache available, compute directly
            self.is_position_passable_for_size(world_x, world_y, size, ground_configs)
        }
    }

    pub fn is_position_passable_for_size(&self, world_x: f32, world_y: f32, size: f32, ground_configs: &GroundConfigs) -> bool {
        // Convert position to tile coordinates
        let center_tile = match self.world_to_tile_coords(world_x, world_y) {
            Some(tile) => tile,
            None => return false,
        };
        
        // First check: the tile the pawn is centered on must be passable
        if !self.is_tile_passable(center_tile.0, center_tile.1, ground_configs) {
            return false;
        }
        
        let half_tile = self.tile_size / 2.0;
        let radius = size * half_tile;
        let radius_in_tiles = (radius / self.tile_size).ceil() as i32;
        
        // Check all tiles within radius with proper edge-based collision detection
        for dx in -radius_in_tiles..=radius_in_tiles {
            for dy in -radius_in_tiles..=radius_in_tiles {
                let tile_x = center_tile.0 + dx;
                let tile_y = center_tile.1 + dy;
                
                // Skip if this tile is passable
                if self.is_tile_passable(tile_x, tile_y, ground_configs) {
                    continue;
                }
                
                // Calculate distance from pawn center to nearest point on the impassable tile
                let tile_world = self.tile_to_world_coords(tile_x, tile_y);
                
                // Find the closest point on the tile to the pawn center
                let closest_x = (world_x.max(tile_world.0 - half_tile)).min(tile_world.0 + half_tile);
                let closest_y = (world_y.max(tile_world.1 - half_tile)).min(tile_world.1 + half_tile);
                
                // Calculate distance from pawn center to closest point on tile
                let distance = ((closest_x - world_x).powi(2) + (closest_y - world_y).powi(2)).sqrt();
                
                // Use a more generous tolerance to allow access to adjacent tiles
                // Allow pawns to get close to impassable tiles as long as they don't significantly overlap
                let tolerance = self.tile_size * 0.25; // 25% tolerance for better playability
                
                // Check if pawn's radius overlaps with this impassable tile (with tolerance)
                if distance < radius - tolerance {
                    return false;
                }
            }
        }
        
        true
    }

    pub fn find_path(&self, start_world: (f32, f32), goal_world: (f32, f32), ground_configs: &GroundConfigs) -> Option<Vec<(f32, f32)>> {
        // Convert world coordinates to tile coordinates
        let start_tile = self.world_to_tile_coords(start_world.0, start_world.1)?;
        let goal_tile = self.world_to_tile_coords(goal_world.0, goal_world.1)?;

        // Check if goal is passable
        if !self.is_tile_passable(goal_tile.0, goal_tile.1, ground_configs) {
            return None; // Can't path to impassable tile
        }

        // Capture ground_configs for use in closure
        let ground_configs = ground_configs;

        // A* pathfinding
        let result = astar(
            &start_tile,
            |&(x, y)| {
                // Generate neighbors (8-directional movement with diagonal support)
                let neighbors = vec![
                    (x + 1, y),     // Right
                    (x - 1, y),     // Left
                    (x, y + 1),     // Up
                    (x, y - 1),     // Down
                    (x + 1, y + 1), // Up-Right (diagonal)
                    (x + 1, y - 1), // Down-Right (diagonal)
                    (x - 1, y + 1), // Up-Left (diagonal)
                    (x - 1, y - 1), // Down-Left (diagonal)
                ];
                
                neighbors
                    .into_iter()
                    .filter(|&(nx, ny)| self.is_tile_passable(nx, ny, ground_configs))
                    .map(|pos| {
                        // Diagonal moves cost more (approximately sqrt(2) ≈ 1.414)
                        let cost = if pos.0 != x && pos.1 != y { 14 } else { 10 };
                        (pos, cost)
                    })
                    .collect::<Vec<_>>()
            },
            |&(x, y)| {
                // Heuristic: Diagonal distance (Chebyshev distance) for 8-directional movement
                let dx = (x - goal_tile.0).abs();
                let dy = (y - goal_tile.1).abs();
                (dx.max(dy) * 10 + (dx.min(dy) * 4)) as u32 // 10 for straight, 14 for diagonal
            },
            |&pos| pos == goal_tile,
        );

        // Convert path back to world coordinates
        if let Some((path, _cost)) = result {
            let world_path = path
                .into_iter()
                .map(|(tx, ty)| self.tile_to_world_coords(tx, ty))
                .collect();
            Some(world_path)
        } else {
            None // No path found
        }
    }

    fn is_path_segment_clear(&self, from_world: (f32, f32), to_world: (f32, f32), size: f32, ground_configs: &GroundConfigs) -> bool {
        self.is_path_segment_clear_with_cache(from_world, to_world, size, ground_configs, &mut None)
    }

    fn is_path_segment_clear_with_cache(&self, from_world: (f32, f32), to_world: (f32, f32), size: f32, ground_configs: &GroundConfigs, cache: &mut Option<&mut crate::systems::pathfinding_cache::PathfindingCache>) -> bool {
        // Sample points along the path to ensure the entire segment is clear
        let dx = to_world.0 - from_world.0;
        let dy = to_world.1 - from_world.1;
        let distance = (dx * dx + dy * dy).sqrt();
        
        // If it's a very short distance, just check the endpoints
        if distance < self.tile_size * 0.1 {
            return self.is_position_passable_for_size_with_cache(to_world.0, to_world.1, size, ground_configs, cache);
        }
        
        // Sample at intervals smaller than the pawn's radius to ensure coverage
        let half_tile = self.tile_size / 2.0;
        let radius = size * half_tile;
        
        // For very small pawns, use a minimum sample interval to avoid infinite sampling
        let min_sample_interval = self.tile_size * 0.25; // Minimum quarter-tile intervals
        let sample_interval = if radius < min_sample_interval {
            min_sample_interval
        } else {
            radius * 0.5 // Sample at half the radius for good coverage
        };
        
        let num_samples = (distance / sample_interval).ceil() as usize;
        
        // Cap the number of samples to prevent excessive computation
        let num_samples = num_samples.min(50); // Reasonable maximum
        
        // Ensure we always check at least the destination
        let num_samples = if num_samples == 0 { 1 } else { num_samples };
        
        // Check each sample point along the path
        for i in 0..=num_samples {
            let t = if num_samples == 0 { 1.0 } else { i as f32 / num_samples as f32 };
            let sample_x = from_world.0 + dx * t;
            let sample_y = from_world.1 + dy * t;
            
            if !self.is_position_passable_for_size_with_cache(sample_x, sample_y, size, ground_configs, cache) {
                return false;
            }
        }
        
        true
    }

    /// Cached version of pathfinding - use this for performance
    pub fn find_path_for_size_cached(&self, start_world: (f32, f32), goal_world: (f32, f32), size: f32, ground_configs: &GroundConfigs, cache: &mut crate::systems::pathfinding_cache::PathfindingCache) -> Option<Vec<(f32, f32)>> {
        // Convert world coordinates to tile coordinates
        let start_tile = self.world_to_tile_coords(start_world.0, start_world.1)?;
        let goal_tile = self.world_to_tile_coords(goal_world.0, goal_world.1)?;

        // Check cache first
        if let Some(cached_result) = cache.get_path(start_tile, goal_tile, size) {
            return cached_result.clone();
        }

        // Compute path if not cached
        let result = self.find_path_for_size_internal(start_world, goal_world, size, ground_configs, Some(cache));
        
        // Cache the result
        cache.cache_path(start_tile, goal_tile, size, result.clone(), self);
        
        result
    }

    /// Original pathfinding method (kept for compatibility)
    pub fn find_path_for_size(&self, start_world: (f32, f32), goal_world: (f32, f32), size: f32, ground_configs: &GroundConfigs) -> Option<Vec<(f32, f32)>> {
        self.find_path_for_size_internal(start_world, goal_world, size, ground_configs, None)
    }

    fn find_path_for_size_internal(&self, start_world: (f32, f32), goal_world: (f32, f32), size: f32, ground_configs: &GroundConfigs, mut cache: Option<&mut crate::systems::pathfinding_cache::PathfindingCache>) -> Option<Vec<(f32, f32)>> {
        // Convert world coordinates to tile coordinates
        let start_tile = self.world_to_tile_coords(start_world.0, start_world.1)?;
        let goal_tile = self.world_to_tile_coords(goal_world.0, goal_world.1)?;

        // Check if start is passable for this size
        if !self.is_position_passable_for_size_with_cache(start_world.0, start_world.1, size, ground_configs, &mut cache) {
            return None; // Can't start from impassable position
        }

        // Check if goal is passable for this size
        if !self.is_position_passable_for_size_with_cache(goal_world.0, goal_world.1, size, ground_configs, &mut cache) {
            return None; // Can't path to position that's impassable for this size
        }

        // A* pathfinding with size awareness  
        // Note: We can't pass mutable cache into the closure, so we'll use the uncached version in A*
        let result = astar(
            &start_tile,
            |&(x, y)| {
                // Generate neighbors (8-directional movement with diagonal support)
                let neighbors = vec![
                    (x + 1, y),     // Right
                    (x - 1, y),     // Left
                    (x, y + 1),     // Up
                    (x, y - 1),     // Down
                    (x + 1, y + 1), // Up-Right (diagonal)
                    (x + 1, y - 1), // Down-Right (diagonal)
                    (x - 1, y + 1), // Up-Left (diagonal)
                    (x - 1, y - 1), // Down-Left (diagonal)
                ];
                
                neighbors
                    .into_iter()
                    .filter(|&(nx, ny)| {
                        // Check if destination position is passable for the given size
                        let to_world = self.tile_to_world_coords(nx, ny);
                        if !self.is_position_passable_for_size(to_world.0, to_world.1, size, ground_configs) {
                            return false;
                        }
                        
                        // Check if the entire path segment from current position to neighbor is clear
                        let from_world = self.tile_to_world_coords(x, y);
                        self.is_path_segment_clear(from_world, to_world, size, ground_configs)
                    })
                    .map(|pos| {
                        // Diagonal moves cost more (approximately sqrt(2) ≈ 1.414)
                        let cost = if pos.0 != x && pos.1 != y { 14 } else { 10 };
                        (pos, cost)
                    })
                    .collect::<Vec<_>>()
            },
            |&(x, y)| {
                // Heuristic: Diagonal distance (Chebyshev distance) for 8-directional movement
                let dx = (x - goal_tile.0).abs();
                let dy = (y - goal_tile.1).abs();
                (dx.max(dy) * 10 + (dx.min(dy) * 4)) as u32 // 10 for straight, 14 for diagonal
            },
            |&pos| pos == goal_tile,
        );

        // Convert path back to world coordinates
        if let Some((path, _cost)) = result {
            let world_path = path
                .into_iter()
                .map(|(tx, ty)| self.tile_to_world_coords(tx, ty))
                .collect();
            Some(world_path)
        } else {
            None // No path found
        }
    }
}

#[derive(Resource, Default)]
pub struct TerrainChanges {
    pub changed_tiles: Vec<(u32, u32, TerrainType)>, // (x, y, new_terrain_type)
}

impl TerrainChanges {
    pub fn add_change(&mut self, x: u32, y: u32, terrain_type: TerrainType) {
        self.changed_tiles.push((x, y, terrain_type));
    }
    
    pub fn clear(&mut self) {
        self.changed_tiles.clear();
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ObjectType {
    Tree = 0,
    Rock = 1,
    Wall = 2,
    Chest = 3,
}

#[derive(Debug, Clone, Copy)]
pub enum DecoType {
    Flower = 0,
    Mushroom = 1,
    SmallRock = 2,
    Bush = 3,
}

pub struct TerrainNoise {
    elevation: Perlin,
    moisture: Perlin,
    temperature: Simplex,
    seed: u32,
}

impl TerrainNoise {
    pub fn new(seed: u32) -> Self {
        Self {
            elevation: Perlin::new(seed),
            moisture: Perlin::new(seed.wrapping_add(1000)),
            temperature: Simplex::new(seed.wrapping_add(2000)),
            seed,
        }
    }

    pub fn get_terrain_type(&self, x: f64, y: f64, ground_configs: &GroundConfigs) -> usize {
        let scale = 0.05; // Controls noise frequency
        
        let elevation = self.elevation.get([x * scale, y * scale]);

        // Normalize elevation to 0-1 range
        let height = (elevation + 1.0) * 0.5;

        // Use ground configs to determine terrain type based on height
        ground_configs.get_terrain_type_for_height(height as f32)
            .unwrap_or(0) // Default to first terrain type if no match found
    }
}

pub fn generate_world(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: Res<GameConfig>,
) {
    // Load ground configuration from YAML
    let grounds_yaml = std::fs::read_to_string("grounds.yaml")
        .expect("Failed to read grounds.yaml file");
    let ground_configs = GroundConfigs::load_from_yaml(&grounds_yaml)
        .expect("Failed to parse grounds.yaml");
    let map_size = TilemapSize { 
        x: config.map_width, 
        y: config.map_height 
    };
    let tile_size = TilemapTileSize { 
        x: config.tile_size, 
        y: config.tile_size 
    };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    // Create and populate terrain map
    let mut terrain_map = TerrainMap::new(config.map_width, config.map_height, config.tile_size);
    
    // Generate ground layer and populate terrain map
    generate_ground_layer(&mut commands, &asset_server, &map_size, &tile_size, &grid_size, &map_type, &mut terrain_map, &ground_configs);
    
    // Insert the populated terrain map and ground configs as resources
    commands.insert_resource(terrain_map);
    commands.insert_resource(ground_configs);
    
    // Generate objects layer
    // generate_objects_layer(&mut commands, &asset_server, &map_size, &tile_size, &grid_size, &map_type);
    
    // Generate decoration layer
    // generate_decoration_layer(&mut commands, &asset_server, &map_size, &tile_size, &grid_size, &map_type);
}

fn generate_ground_layer(
    commands: &mut Commands,
    asset_server: &AssetServer,
    map_size: &TilemapSize,
    tile_size: &TilemapTileSize,
    grid_size: &TilemapGridSize,
    map_type: &TilemapType,
    terrain_map: &mut TerrainMap,
    ground_configs: &GroundConfigs,
) {
    let texture_handle: Handle<Image> = asset_server.load("ground_tileset.png");
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(*map_size);
    let mut rng = rand::thread_rng();
    
    // Create noise generator with random seed
    let seed: u32 = rng.next_u32();
    let noise = TerrainNoise::new(seed);

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            
            // Use noise-based terrain generation with ground configs
            let terrain_type = if x == 0 || y == 0 || x == map_size.x - 1 || y == map_size.y - 1 {
                // Find water terrain type from configs (or default to first)
                ground_configs.terrain_mapping.get("water").copied().unwrap_or(0)
            } else {
                noise.get_terrain_type(x as f64, y as f64, ground_configs)
            };

            // Store terrain type in the terrain map
            terrain_map.set_tile(x, y, terrain_type);

            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: TileTextureIndex(terrain_type as u32),
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size: *grid_size,
        map_type: *map_type,
        size: *map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size: *tile_size,
        transform: get_tilemap_center_transform(map_size, grid_size, map_type, 0.0),
        ..Default::default()
    })
    .insert(TerrainLayer {
        layer_id: 0,
        z_index: 0.0,
    });
}

fn generate_objects_layer(
    commands: &mut Commands,
    asset_server: &AssetServer,
    map_size: &TilemapSize,
    tile_size: &TilemapTileSize,
    grid_size: &TilemapGridSize,
    map_type: &TilemapType,
) {
    let texture_handle: Handle<Image> = asset_server.load("objects_tileset.png");
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(*map_size);
    let mut rng = rand::thread_rng();

    for x in 1..map_size.x - 1 { // Skip borders
        for y in 1..map_size.y - 1 {
            let tile_pos = TilePos { x, y };
            
            // Sparse object placement
            if rng.gen_ratio(1, 8) {
                let object_type = match rng.gen_range(0..4) {
                    0 => ObjectType::Tree,
                    1 => ObjectType::Rock,
                    2 => ObjectType::Wall,
                    _ => ObjectType::Chest,
                };

                let tile_entity = commands
                    .spawn(TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(tilemap_entity),
                        texture_index: TileTextureIndex(object_type as u32),
                        ..Default::default()
                    })
                    .id();
                tile_storage.set(&tile_pos, tile_entity);
            }
        }
    }

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size: *grid_size,
        map_type: *map_type,
        size: *map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size: *tile_size,
        transform: get_tilemap_center_transform(map_size, grid_size, map_type, 1.0),
        ..Default::default()
    })
    .insert(TerrainLayer {
        layer_id: 1,
        z_index: 1.0,
    });
}

fn generate_decoration_layer(
    commands: &mut Commands,
    asset_server: &AssetServer,
    map_size: &TilemapSize,
    tile_size: &TilemapTileSize,
    grid_size: &TilemapGridSize,
    map_type: &TilemapType,
) {
    let texture_handle: Handle<Image> = asset_server.load("decoration_tileset.png");
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(*map_size);
    let mut rng = rand::thread_rng();

    for x in 1..map_size.x - 1 { // Skip borders
        for y in 1..map_size.y - 1 {
            let tile_pos = TilePos { x, y };
            
            // Very sparse decoration placement
            if rng.gen_ratio(1, 15) {
                let deco_type = match rng.gen_range(0..4) {
                    0 => DecoType::Flower,
                    1 => DecoType::Mushroom,
                    2 => DecoType::SmallRock,
                    _ => DecoType::Bush,
                };

                let tile_entity = commands
                    .spawn(TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(tilemap_entity),
                        texture_index: TileTextureIndex(deco_type as u32),
                        ..Default::default()
                    })
                    .id();
                tile_storage.set(&tile_pos, tile_entity);
            }
        }
    }

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size: *grid_size,
        map_type: *map_type,
        size: *map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size: *tile_size,
        transform: get_tilemap_center_transform(map_size, grid_size, map_type, 2.0),
        ..Default::default()
    })
    .insert(TerrainLayer {
        layer_id: 2,
        z_index: 2.0,
    });
}

pub fn update_terrain_visuals(
    mut terrain_changes: ResMut<TerrainChanges>,
    mut tile_query: Query<&mut TileTextureIndex>,
    tile_storage_query: Query<&TileStorage, With<TerrainLayer>>,
) {
    if terrain_changes.changed_tiles.is_empty() {
        return;
    }
    
    // Find the terrain layer tile storage
    if let Ok(tile_storage) = tile_storage_query.get_single() {
        for (x, y, terrain_type) in terrain_changes.changed_tiles.drain(..) {
            let tile_pos = TilePos { x, y };
            
            if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                if let Ok(mut texture_index) = tile_query.get_mut(tile_entity) {
                    texture_index.0 = terrain_type as u32;
                }
            }
        }
    }
    
    terrain_changes.clear();
}