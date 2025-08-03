use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use crate::components::TerrainLayer;
use crate::resources::GameConfig;
use noise::{NoiseFn, Perlin, Simplex};
use pathfinding::prelude::astar;
use rand::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TerrainType {
    Grass = 0,
    Dirt = 1,
    Stone = 2,
    Water = 3,
}

impl TerrainType {
    pub fn is_passable(self) -> bool {
        match self {
            TerrainType::Grass => true,
            TerrainType::Dirt => true,
            TerrainType::Stone => false, // Stone is impassable
            TerrainType::Water => false, // Water is impassable
        }
    }
}

#[derive(Resource)]
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
            tiles: vec![vec![TerrainType::Grass; height as usize]; width as usize],
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

    pub fn is_passable_at_world_pos(&self, world_x: f32, world_y: f32) -> bool {
        self.get_terrain_at_world_pos(world_x, world_y)
            .map(|terrain| terrain.is_passable())
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

    pub fn is_tile_passable(&self, tile_x: i32, tile_y: i32) -> bool {
        if tile_x >= 0 && tile_x < self.width as i32 && tile_y >= 0 && tile_y < self.height as i32 {
            self.tiles[tile_x as usize][tile_y as usize].is_passable()
        } else {
            false // Out of bounds is impassable
        }
    }

    pub fn find_nearest_passable_tile(&self, start_world: (f32, f32)) -> Option<(f32, f32)> {
        // First check if the starting position is already passable
        if let Some((start_tile_x, start_tile_y)) = self.world_to_tile_coords(start_world.0, start_world.1) {
            if self.is_tile_passable(start_tile_x, start_tile_y) {
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
                    
                    if self.is_tile_passable(tile_x, tile_y) {
                        return Some(self.tile_to_world_coords(tile_x, tile_y));
                    }
                }
            }
        }
        
        None // No passable tile found within reasonable distance
    }

    pub fn find_path(&self, start_world: (f32, f32), goal_world: (f32, f32)) -> Option<Vec<(f32, f32)>> {
        // Convert world coordinates to tile coordinates
        let start_tile = self.world_to_tile_coords(start_world.0, start_world.1)?;
        let goal_tile = self.world_to_tile_coords(goal_world.0, goal_world.1)?;

        // Check if goal is passable
        if !self.is_tile_passable(goal_tile.0, goal_tile.1) {
            return None; // Can't path to impassable tile
        }

        // A* pathfinding
        let result = astar(
            &start_tile,
            |&(x, y)| {
                // Generate neighbors (4-directional movement)
                let neighbors = vec![
                    (x + 1, y),     // Right
                    (x - 1, y),     // Left
                    (x, y + 1),     // Up
                    (x, y - 1),     // Down
                ];
                
                neighbors
                    .into_iter()
                    .filter(|&(nx, ny)| self.is_tile_passable(nx, ny))
                    .map(|pos| (pos, 1)) // All moves have cost 1
                    .collect::<Vec<_>>()
            },
            |&(x, y)| {
                // Heuristic: Manhattan distance to goal
                ((x - goal_tile.0).abs() + (y - goal_tile.1).abs()) as u32
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

    pub fn get_terrain_type(&self, x: f64, y: f64) -> TerrainType {
        let scale = 0.05; // Controls noise frequency
        
        let elevation = self.elevation.get([x * scale, y * scale]);
        let moisture = self.moisture.get([x * scale * 0.7, y * scale * 0.7]);
        let temperature = self.temperature.get([x * scale * 1.3, y * scale * 1.3]);

        // Normalize values to 0-1 range
        let elevation = (elevation + 1.0) * 0.5;
        let moisture = (moisture + 1.0) * 0.5;
        let temperature = (temperature + 1.0) * 0.5;

        // Terrain selection based on biome logic
        match (elevation, moisture, temperature) {
            (e, _, _) if e < 0.2 => TerrainType::Water,
            (e, _m, t) if e > 0.8 && t < 0.3 => TerrainType::Stone,
            (_, m, _) if m < 0.3 => TerrainType::Dirt,
            _ => TerrainType::Grass,
        }
    }
}

pub fn generate_world(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: Res<GameConfig>,
) {
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
    generate_ground_layer(&mut commands, &asset_server, &map_size, &tile_size, &grid_size, &map_type, &mut terrain_map);
    
    // Insert the populated terrain map as a resource
    commands.insert_resource(terrain_map);
    
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
            
            // Use noise-based terrain generation
            let terrain_type = if x == 0 || y == 0 || x == map_size.x - 1 || y == map_size.y - 1 {
                TerrainType::Water // Keep water border for now
            } else {
                noise.get_terrain_type(x as f64, y as f64)
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