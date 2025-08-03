use bevy::prelude::*;
use crate::resources::GameConfig;
use crate::systems::world_gen::TerrainMap;
use crate::systems::pawn::{Pawn, PawnTarget};

pub fn handle_player_input(
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera>>,
    config: Res<GameConfig>,
    terrain_map: Res<TerrainMap>,
    mut commands: Commands,
    player_query: Query<(Entity, &Transform, &Pawn), With<Pawn>>,
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
                        for (entity, transform, pawn) in player_query.iter() {
                            if pawn.pawn_type == "player" {
                                let player_pos = (transform.translation.x, transform.translation.y);
                                let goal_pos = (snapped_x, snapped_y);

                                if let Some(path) = terrain_map.find_path(player_pos, goal_pos) {
                                    let mut pawn_target = PawnTarget::new(target_pos);
                                    pawn_target.set_path(path.clone());
                                    
                                    // Add or update the PawnTarget component
                                    commands.entity(entity).insert(pawn_target);
                                    
                                    println!("Path found to {:?} with {} waypoints", target_pos, path.len());
                                } else {
                                    println!("No path found to {:?}", target_pos);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}