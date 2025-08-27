use bevy::prelude::*;
use crate::systems::world_gen::{TerrainMap, GroundConfigs};
use crate::systems::pawn_config::{PawnConfig, PawnType, BehaviourConfig, BehaviourType};
use crate::resources::GameConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SpriteInfo {
    name: String,
    index: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TilesetIndex {
    tileset_name: String,
    tile_size: u32,
    tiles_per_row: u32,
    total_tiles: u32,
    sprites: Vec<SpriteInfo>,
}

#[derive(Resource)]
pub struct TilesetManager {
    tilesets: HashMap<String, TilesetIndex>,
    atlases: HashMap<String, Handle<TextureAtlasLayout>>,
}

impl Default for TilesetManager {
    fn default() -> Self {
        Self {
            tilesets: HashMap::new(),
            atlases: HashMap::new(),
        }
    }
}

impl TilesetManager {
    pub fn load_tileset(&mut self, tileset_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let yaml_path = format!("assets/tilesets/{}.yaml", tileset_name);
        let yaml_content = std::fs::read_to_string(&yaml_path)?;
        let tileset_index: TilesetIndex = serde_yaml::from_str(&yaml_content)?;
        self.tilesets.insert(tileset_name.to_string(), tileset_index);
        Ok(())
    }
    
    pub fn get_sprite_index(&self, tileset_name: &str, sprite_name: &str) -> Option<u32> {
        self.tilesets.get(tileset_name)?
            .sprites.iter()
            .find(|sprite| sprite.name == sprite_name)
            .map(|sprite| sprite.index)
    }
    
    pub fn create_atlas_layout(&self, tileset_name: &str, texture_atlas_layouts: &mut Assets<TextureAtlasLayout>) -> Option<Handle<TextureAtlasLayout>> {
        let tileset = self.tilesets.get(tileset_name)?;
        
        // Create atlas layout based on tileset configuration
        let layout = TextureAtlasLayout::from_grid(
            UVec2::new(tileset.tile_size, tileset.tile_size),
            tileset.tiles_per_row,
            (tileset.total_tiles + tileset.tiles_per_row - 1) / tileset.tiles_per_row,
            None,
            None
        );
        
        Some(texture_atlas_layouts.add(layout))
    }
}

#[derive(Component)]
pub struct Pawn {
    pub pawn_type: PawnType,
}

#[derive(Component)]
pub struct Size {
    pub value: f32,
}

#[derive(Component)]
pub struct CurrentBehavior {
    pub state: String,
}

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: u32) -> Self {
        Self {
            current: max as f32,
            max: max as f32,
        }
    }
}

#[derive(Component)]  
pub struct Endurance {
    pub current: f32,
    pub max: f32,
    pub health_loss_timer: f32,
}

impl Endurance {
    pub fn new(max: u32) -> Self {  
        Self {
            current: max as f32,
            max: max as f32,
            health_loss_timer: 0.0,
        }
    }
}

impl Pawn {
    pub fn new(pawn_type: PawnType) -> Self {
        Self { pawn_type }
    }
}

#[derive(Component)]
pub struct PawnTarget {
    pub target_position: Vec3,
    pub path: Vec<Vec3>,
    pub current_waypoint_index: usize,
}

impl PawnTarget {
    pub fn new(target_position: Vec3) -> Self {
        Self {
            target_position,
            path: vec![target_position],
            current_waypoint_index: 0,
        }
    }

    pub fn set_path(&mut self, path: Vec<(f32, f32)>) {
        if !path.is_empty() {
            self.path = path
                .into_iter()
                .map(|(x, y)| Vec3::new(x, y, 100.0))
                .collect();
            self.current_waypoint_index = 0;
            self.target_position = *self.path.last().unwrap();
        }
    }

    pub fn get_current_waypoint(&self) -> Option<Vec3> {
        self.path.get(self.current_waypoint_index).copied()
    }

    pub fn advance_waypoint(&mut self) {
        if self.current_waypoint_index < self.path.len() - 1 {
            self.current_waypoint_index += 1;
        }
    }

    pub fn is_at_destination(&self) -> bool {
        self.current_waypoint_index >= self.path.len() - 1
    }

    pub fn reset(&mut self) {
        self.path.clear();
        self.current_waypoint_index = 0;
        self.target_position = Vec3::ZERO;
    }
}

pub fn spawn_pawn(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    terrain_map: &Res<TerrainMap>,
    ground_configs: &Res<GroundConfigs>,
    pawn_config: &Res<PawnConfig>,
    tileset_manager: &mut ResMut<TilesetManager>,
    texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
    pawn: Pawn,
    spawn_position: Option<(f32, f32)>,
) -> Entity {
    let position = if let Some(pos) = spawn_position {
        pos
    } else {
        // Find a passable spawn position
        let initial_center = (0.0, 0.0);
        if let Some(passable_pos) = terrain_map.find_nearest_passable_tile(initial_center, ground_configs) {
            passable_pos
        } else {
            (0.0, 0.0) // Fallback
        }
    };

    let pawn_def = pawn_config.get_pawn_definition(&pawn.pawn_type)
        .expect("Pawn definition not found in config");

    // Parse sprite reference - check if it's a tileset reference or direct file
    let sprite_bundle = if pawn_def.sprite.starts_with("tileset::") {
        // Parse tileset reference: "tileset::tileset_name::sprite_name"
        let parts: Vec<&str> = pawn_def.sprite.split("::").collect();
        if parts.len() == 3 && parts[0] == "tileset" {
            let tileset_name = parts[1];
            let sprite_name = parts[2];
            
            // Load tileset if not already loaded
            if !tileset_manager.tilesets.contains_key(tileset_name) {
                if let Err(e) = tileset_manager.load_tileset(tileset_name) {
                    eprintln!("Failed to load tileset {}: {}", tileset_name, e);
                    // Fallback to direct sprite loading
                    Sprite::from_image(asset_server.load(&pawn_def.sprite))
                } else {
                    // Create sprite with atlas
                    let texture_handle = asset_server.load(format!("tilesets/{}.png", tileset_name));
                    let atlas_layout = tileset_manager.create_atlas_layout(tileset_name, texture_atlas_layouts)
                        .expect("Failed to create atlas layout");
                    
                    let sprite_index = tileset_manager.get_sprite_index(tileset_name, sprite_name)
                        .unwrap_or(0);
                    
                    Sprite::from_atlas_image(texture_handle, TextureAtlas {
                        layout: atlas_layout,
                        index: sprite_index as usize,
                    })
                }
            } else {
                // Tileset already loaded, just get the sprite
                let texture_handle = asset_server.load(format!("tilesets/{}.png", tileset_name));
                let atlas_layout = tileset_manager.create_atlas_layout(tileset_name, texture_atlas_layouts)
                    .expect("Failed to create atlas layout");
                
                let sprite_index = tileset_manager.get_sprite_index(tileset_name, sprite_name)
                    .unwrap_or(0);
                
                Sprite::from_atlas_image(texture_handle, TextureAtlas {
                    layout: atlas_layout,
                    index: sprite_index as usize,
                })
            }
        } else {
            // Invalid tileset format, fallback to direct loading
            Sprite::from_image(asset_server.load(&pawn_def.sprite))
        }
    } else {
        // Direct sprite file
        Sprite::from_image(asset_server.load(&pawn_def.sprite))
    };

    commands.spawn((
        sprite_bundle,
        Transform::from_translation(Vec3::new(position.0, position.1, 100.0)),
        pawn,
        Size { value: pawn_def.size },
        Health::new(pawn_def.max_health),
        Endurance::new(pawn_def.max_endurance),
        CurrentBehavior { state: "idle".to_string() },
    )).id()
}

pub fn move_pawn_to_target(
    time: Res<Time>,
    pawn_config: Res<PawnConfig>,
    config: Res<GameConfig>,
    mut commands: Commands,
    mut pawn_query: Query<(Entity, &mut Transform, &mut PawnTarget, &Pawn, &mut Endurance)>,
) {
    for (entity, mut transform, mut target, pawn, mut endurance) in pawn_query.iter_mut() {
        if let Some(current_waypoint) = target.get_current_waypoint() {
            let distance = transform.translation.distance(current_waypoint);
            
            if distance > 2.0 { // Close enough threshold for waypoints
                let pawn_def = pawn_config.get_pawn_definition(&pawn.pawn_type)
                    .expect("Pawn definition not found in config");
                
                let direction = (current_waypoint - transform.translation).normalize();
                let movement = direction * pawn_def.move_speed * time.delta_secs();
                
                let actual_movement_distance = if movement.length() > distance {
                    // Don't overshoot the waypoint
                    let final_distance = distance;
                    transform.translation = current_waypoint;
                    final_distance
                } else {
                    let move_distance = movement.length();
                    transform.translation += movement;
                    move_distance
                };
                
                // Reduce endurance based on distance moved
                let cells_moved = actual_movement_distance / config.tile_size;
                let endurance_cost = cells_moved * config.endurance_cost_per_cell;
                endurance.current = (endurance.current - endurance_cost).max(0.0);
            } else {
                // Reached current waypoint, advance to next
                transform.translation = current_waypoint;
                
                if target.is_at_destination() {
                    println!("{} reached destination: {:?}", pawn.pawn_type, target.target_position);
                    // Remove PawnTarget component so pawn can get new AI targets
                    commands.entity(entity).remove::<PawnTarget>();
                } else {
                    target.advance_waypoint();
                }
            }
        }
    }
}

pub fn endurance_health_loss_system(
    time: Res<Time>,
    config: Res<GameConfig>,
    mut pawn_query: Query<(Entity, &mut Health, &mut Endurance, &Pawn)>,
) {
    for (_entity, mut health, mut endurance, _pawn) in pawn_query.iter_mut() {
        if endurance.current <= 0.0 {
            // Update health loss timer
            endurance.health_loss_timer += time.delta_secs();
            
            // Check if it's time to lose health
            if endurance.health_loss_timer >= config.health_loss_interval {
                health.current = (health.current - 1.0).max(0.0);
                endurance.health_loss_timer = 0.0; // Reset timer
                
            }
        } else {
            // Reset health loss timer if endurance is above 0
            endurance.health_loss_timer = 0.0;
        }
    }
}

pub fn pawn_death_system(
    mut commands: Commands,
    pawn_query: Query<(Entity, &Health, &Pawn)>,
) {
    for (entity, health, pawn) in pawn_query.iter() {
        if health.current <= 0.0 {
            println!("{} has died!", pawn.pawn_type);
            commands.entity(entity).despawn();
        }
    }
}

pub fn endurance_behavior_switching_system(
    pawn_config: Res<PawnConfig>,
    mut pawn_query: Query<(&Pawn, &Endurance, &mut CurrentBehavior)>,
) {
    for (pawn, endurance, mut current_behavior) in pawn_query.iter_mut() {
        let endurance_percentage = endurance.current / endurance.max;
        
        // Switch to looking_for_food when endurance is 30% or below
        if endurance_percentage <= 0.3 && current_behavior.state != "looking_for_food" {
            // Check if this pawn has a looking_for_food behavior defined
            if let Some(behavior_config) = pawn_config.get_behaviour_config(&pawn.pawn_type, "looking_for_food") {
                if !matches!(behavior_config, BehaviourConfig::Simple(BehaviourType::Null)) {
                    println!("{} switching to looking_for_food behavior (endurance: {:.1}%)", 
                             pawn.pawn_type, endurance_percentage * 100.0);
                    current_behavior.state = "looking_for_food".to_string();
                }
            }
        }
        // Switch back to idle when endurance is above 50% (hysteresis to prevent flapping)
        else if endurance_percentage > 0.5 && current_behavior.state == "looking_for_food" {
            println!("{} switching back to idle behavior (endurance: {:.1}%)", 
                     pawn.pawn_type, endurance_percentage * 100.0);
            current_behavior.state = "idle".to_string();
        }
    }
}