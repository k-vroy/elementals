use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use crate::components::Player;
use crate::resources::GameConfig;
use crate::systems::world_gen::TerrainMap;

#[derive(Component)]
pub struct PlayerTarget {
    pub target_position: Vec3,
    pub move_speed: f32,
    pub path: Vec<Vec3>,
    pub current_waypoint_index: usize,
}

impl PlayerTarget {
    pub fn new(target_position: Vec3, move_speed: f32) -> Self {
        Self {
            target_position,
            move_speed,
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

pub fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: Res<GameConfig>,
    terrain_map: Res<TerrainMap>,
) {
    // Use the same positioning logic as the tilemap
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

    // Get the tilemap center transform to understand the coordinate system
    let tilemap_center = get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0);
    
    // Try to spawn player at map center, but ensure it's on a passable tile
    let initial_center = (0.0, 0.0);
    let spawn_position = if let Some(passable_pos) = terrain_map.find_nearest_passable_tile(initial_center) {
        passable_pos
    } else {
        // Fallback to center if no passable tile found (shouldn't happen in normal maps)
        initial_center
    };
    
    let player_pos = Vec3::new(spawn_position.0, spawn_position.1, 100.0);

    // Load player texture
    let texture_handle: Handle<Image> = asset_server.load("player.png");

    // Spawn player at center of map
    commands.spawn((
        Player,
        Sprite {
            image: texture_handle,
            color: Color::WHITE,
            ..default()
        },
        Transform::from_translation(player_pos).with_scale(Vec3::splat(1.0)), // Scale up to make more visible
        PlayerTarget::new(player_pos, 200.0),
    ));

    // Debug print to verify spawning
    println!("Player spawned at: {:?}", player_pos);
    if let Some(terrain_type) = terrain_map.get_terrain_at_world_pos(spawn_position.0, spawn_position.1) {
        println!("Player spawn terrain: {:?} (passable: {})", terrain_type, terrain_type.is_passable());
    }
    if spawn_position != initial_center {
        println!("Player moved from center {:?} to nearest passable tile {:?}", initial_center, spawn_position);
    }
}

pub fn handle_player_movement_input(
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera>>,
    config: Res<GameConfig>,
    terrain_map: Res<TerrainMap>,
    mut player_query: Query<(&Transform, &mut PlayerTarget), With<Player>>,
) {
    if mouse_input.just_pressed(MouseButton::Right) {
        if let Ok(window) = windows.get_single() {
            if let Some(cursor_position) = window.cursor_position() {
                if let Ok((camera, camera_transform)) = camera_query.get_single() {
                    // Convert screen coordinates to world coordinates
                    if let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
                        // Snap to tile grid - use floor with offset to get tile center
                        let tile_size = config.tile_size;
                        
                        // Convert to tile coordinates first
                        let half_width = (config.map_width as f32 * tile_size) / 2.0;
                        let half_height = (config.map_height as f32 * tile_size) / 2.0;
                        
                        let tile_x = ((world_position.x + half_width) / tile_size).floor() as i32;
                        let tile_y = ((world_position.y + half_height) / tile_size).floor() as i32;
                        
                        // Convert back to world coordinates at tile center
                        let snapped_x = (tile_x as f32 * tile_size) - half_width + (tile_size / 2.0);
                        let snapped_y = (tile_y as f32 * tile_size) - half_height + (tile_size / 2.0);
                        let target_pos = Vec3::new(snapped_x, snapped_y, 100.0);

                        // Use pathfinding to find route to target
                        for (transform, mut player_target) in player_query.iter_mut() {
                            let player_pos = (transform.translation.x, transform.translation.y);
                            let goal_pos = (snapped_x, snapped_y);

                            if let Some(path) = terrain_map.find_path(player_pos, goal_pos) {
                                player_target.set_path(path.clone());
                                println!("Path found to {:?} with {} waypoints", target_pos, path.len());
                                
                                // Debug: print first few waypoints
                                for (i, waypoint) in path.iter().take(5).enumerate() {
                                    println!("  Waypoint {}: ({:.1}, {:.1})", i, waypoint.0, waypoint.1);
                                }
                                if path.len() > 5 {
                                    println!("  ... and {} more waypoints", path.len() - 5);
                                }
                            } else {
                                // No path found - debug the issue
                                if let Some(terrain_type) = terrain_map.get_terrain_at_world_pos(snapped_x, snapped_y) {
                                    let is_passable = terrain_type.is_passable();
                                    println!("No path to {:?} - target terrain is {:?} (passable: {})", target_pos, terrain_type, is_passable);
                                    
                                    // Additional debug info
                                    if let Some((tile_x, tile_y)) = terrain_map.world_to_tile_coords(snapped_x, snapped_y) {
                                        println!("  World pos: ({:.1}, {:.1}) -> Tile coords: ({}, {})", snapped_x, snapped_y, tile_x, tile_y);
                                        println!("  Player world pos: ({:.1}, {:.1})", player_pos.0, player_pos.1);
                                        if let Some((player_tile_x, player_tile_y)) = terrain_map.world_to_tile_coords(player_pos.0, player_pos.1) {
                                            println!("  Player tile coords: ({}, {})", player_tile_x, player_tile_y);
                                        }
                                    }
                                } else {
                                    println!("No path to {:?} - unreachable or out of bounds", target_pos);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn move_player_to_target(
    time: Res<Time>,
    mut player_query: Query<(&mut Transform, &mut PlayerTarget), With<Player>>,
) {
    for (mut transform, mut target) in player_query.iter_mut() {
        if let Some(current_waypoint) = target.get_current_waypoint() {
            let distance = transform.translation.distance(current_waypoint);
            
            if distance > 2.0 { // Close enough threshold for waypoints
                let direction = (current_waypoint - transform.translation).normalize();
                let movement = direction * target.move_speed * time.delta_secs();
                
                // Don't overshoot the waypoint
                if movement.length() > distance {
                    transform.translation = current_waypoint;
                } else {
                    transform.translation += movement;
                }
            } else {
                // Reached current waypoint, advance to next
                transform.translation = current_waypoint;
                
                if target.is_at_destination() {
                    println!("Player reached destination: {:?}", target.target_position);
                    target.reset();
                } else {
                    target.advance_waypoint();
                }
            }
        }
    }
}