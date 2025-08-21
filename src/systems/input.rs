use bevy::prelude::*;
use crate::resources::GameConfig;
use crate::systems::world_gen::{TerrainMap, TerrainType, TerrainChanges};
use crate::systems::pawn::{Pawn, Size};
use crate::systems::debug_display::DebugDisplayState;
use crate::systems::async_pathfinding::{PathfindingRequest, PathfindingPriority};

pub fn handle_player_input(
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera>>,
    config: Res<GameConfig>,
    mut terrain_map: ResMut<TerrainMap>,
    mut terrain_changes: ResMut<TerrainChanges>,
    debug_state: Res<DebugDisplayState>,
    mut commands: Commands,
    player_query: Query<(Entity, &Transform, &Pawn, &Size), With<Pawn>>,
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

                        // Use pathfinding to find route to target for players only
                        for (entity, transform, pawn, size) in player_query.iter() {
                            if pawn.pawn_type == "player" {
                                let player_pos = (transform.translation.x, transform.translation.y);
                                let goal_pos = (snapped_x, snapped_y);

                                // Request critical priority pathfinding for player input
                                commands.entity(entity).insert(
                                    PathfindingRequest::new(player_pos, goal_pos, size.value)
                                        .with_priority(PathfindingPriority::Critical)
                                );
                                
                                println!("Pathfinding requested to {:?}", target_pos);
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Debug terrain editing with middle mouse click
    if mouse_input.just_pressed(MouseButton::Middle) && debug_state.enabled {
        if let Ok(window) = windows.get_single() {
            if let Some(cursor_position) = window.cursor_position() {
                if let Ok((camera, camera_transform)) = camera_query.get_single() {
                    // Convert screen coordinates to world coordinates
                    if let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
                        // Check if current tile is passable and toggle between stone and dirt
                        let current_terrain = terrain_map.get_terrain_at_world_pos(world_position.x, world_position.y);
                        
                        if let Some(terrain_type) = current_terrain {
                            let new_terrain = if terrain_type.is_passable() {
                                TerrainType::Stone // Set to stone if currently passable
                            } else {
                                TerrainType::Dirt  // Set to dirt if currently impassable
                            };
                            
                            if terrain_map.set_tile_at_world_pos(world_position.x, world_position.y, new_terrain, &mut terrain_changes) {
                                println!("Debug: Changed tile at ({:.1}, {:.1}) from {:?} to {:?}", 
                                    world_position.x, world_position.y, terrain_type, new_terrain);
                            }
                        }
                    }
                }
            }
        }
    }
}