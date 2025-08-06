use bevy::prelude::*;
use crate::systems::pawn::{Pawn, Health, Endurance, CurrentBehavior, PawnTarget};

#[derive(Resource)]
pub struct DebugDisplayState {
    pub enabled: bool,
}

impl Default for DebugDisplayState {
    fn default() -> Self {
        Self {
            enabled: false,
        }
    }
}

#[derive(Component)]
pub struct DebugText {
    pub pawn_entity: Entity,
}

#[derive(Component)]
pub struct WaypointLine {
    pub pawn_entity: Entity,
    pub line_segments: Vec<Entity>,
}

pub fn toggle_debug_display(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut debug_state: ResMut<DebugDisplayState>,
) {
    if keyboard_input.just_pressed(KeyCode::F12) {
        debug_state.enabled = !debug_state.enabled;
        println!("Debug display: {}", if debug_state.enabled { "ON" } else { "OFF" });
    }
}

pub fn manage_debug_text_entities(
    mut commands: Commands,
    debug_state: Res<DebugDisplayState>,
    pawn_query: Query<Entity, (With<Pawn>, With<Health>, With<Endurance>, With<CurrentBehavior>)>,
    debug_text_query: Query<(Entity, &DebugText)>,
) {
    if debug_state.enabled {
        // Create debug text entities for pawns that don't have them
        for pawn_entity in pawn_query.iter() {
            let has_debug_text = debug_text_query.iter().any(|(_, debug_text)| {
                debug_text.pawn_entity == pawn_entity
            });
            
            if !has_debug_text {
                commands.spawn((
                    Text2d::new(""),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Transform::from_translation(Vec3::new(0.0, 20.0, 200.0)),
                    DebugText {
                        pawn_entity,
                    },
                ));
            }
        }
    } else {
        // Remove all debug text entities when disabled
        for (debug_entity, _) in debug_text_query.iter() {
            commands.entity(debug_entity).despawn();
        }
    }
}

pub fn update_debug_text(
    debug_state: Res<DebugDisplayState>,
    pawn_query: Query<(&Transform, &Health, &Endurance, &CurrentBehavior), With<Pawn>>,
    mut debug_text_query: Query<(&mut Transform, &mut Text2d, &DebugText), Without<Pawn>>,
) {
    if !debug_state.enabled {
        return;
    }

    for (mut debug_transform, mut debug_text, debug_component) in debug_text_query.iter_mut() {
        if let Ok((pawn_transform, health, endurance, behavior)) = pawn_query.get(debug_component.pawn_entity) {
            // Position debug text above the pawn
            debug_transform.translation.x = pawn_transform.translation.x;
            debug_transform.translation.y = pawn_transform.translation.y + 20.0;
            debug_transform.translation.z = 200.0; // High z to render on top
            
            // Update text content with behavior
            debug_text.0 = format!(
                "H:{:.0}/{:.0} E:{:.0}/{:.0}\n{}",
                health.current,
                health.max,
                endurance.current,
                endurance.max,
                behavior.state
            );
        } else {
            // Pawn no longer exists, mark for removal
            debug_text.0 = String::new();
        }
    }
}

pub fn cleanup_orphaned_debug_text(
    mut commands: Commands,
    pawn_query: Query<Entity, With<Pawn>>,
    debug_text_query: Query<(Entity, &DebugText)>,
) {
    let existing_pawns: std::collections::HashSet<Entity> = pawn_query.iter().collect();
    
    for (debug_entity, debug_text) in debug_text_query.iter() {
        if !existing_pawns.contains(&debug_text.pawn_entity) {
            commands.entity(debug_entity).despawn();
        }
    }
}

pub fn manage_waypoint_lines(
    mut commands: Commands,
    debug_state: Res<DebugDisplayState>,
    pawn_query: Query<Entity, (With<Pawn>, With<PawnTarget>)>,
    waypoint_line_query: Query<(Entity, &WaypointLine)>,
) {
    if debug_state.enabled {
        // Create waypoint lines for pawns with targets that don't have them
        for pawn_entity in pawn_query.iter() {
            let has_waypoint_line = waypoint_line_query.iter().any(|(_, waypoint_line)| {
                waypoint_line.pawn_entity == pawn_entity
            });
            
            if !has_waypoint_line {
                commands.spawn(WaypointLine {
                    pawn_entity,
                    line_segments: Vec::new(),
                });
            }
        }
    } else {
        // Remove all waypoint lines when disabled
        for (waypoint_line_entity, waypoint_line) in waypoint_line_query.iter() {
            // Clean up all line segment entities
            for &segment_entity in &waypoint_line.line_segments {
                if let Some(mut entity_commands) = commands.get_entity(segment_entity) {
                    entity_commands.despawn();
                }
            }
            // Remove the waypoint line entity itself
            commands.entity(waypoint_line_entity).despawn();
        }
    }
}

pub fn update_waypoint_lines(
    mut commands: Commands,
    debug_state: Res<DebugDisplayState>,
    pawn_query: Query<(&Transform, &PawnTarget), With<Pawn>>,
    mut waypoint_line_query: Query<(Entity, &mut WaypointLine)>,
) {
    if !debug_state.enabled {
        return;
    }

    for (waypoint_line_entity, mut waypoint_line) in waypoint_line_query.iter_mut() {
        if let Ok((pawn_transform, pawn_target)) = pawn_query.get(waypoint_line.pawn_entity) {
            // Clean up existing line segments
            for &segment_entity in &waypoint_line.line_segments {
                if let Some(mut entity_commands) = commands.get_entity(segment_entity) {
                    entity_commands.despawn();
                }
            }
            waypoint_line.line_segments.clear();

            // Create new line segments for the current path
            if !pawn_target.path.is_empty() {
                let mut previous_point = pawn_transform.translation;
                
                // Draw lines from pawn to first waypoint, then between waypoints
                for (i, &waypoint) in pawn_target.path.iter().enumerate() {
                    let start_pos = previous_point;
                    let end_pos = waypoint;
                    
                    // Calculate line properties
                    let direction = (end_pos - start_pos).normalize_or_zero();
                    let length = start_pos.distance(end_pos);
                    let center = (start_pos + end_pos) * 0.5;
                    let angle = direction.y.atan2(direction.x);
                    
                    // Choose color based on waypoint status
                    let color = if i < pawn_target.current_waypoint_index {
                        Color::srgb(0.3, 0.3, 0.3) // Gray for completed waypoints
                    } else if i == pawn_target.current_waypoint_index {
                        Color::srgb(0.0, 1.0, 0.0) // Green for current waypoint
                    } else {
                        Color::srgb(1.0, 1.0, 0.0) // Yellow for future waypoints
                    };
                    
                    // Create line segment entity
                    let line_entity = commands.spawn((
                        Sprite {
                            color,
                            custom_size: Some(Vec2::new(length.max(1.0), 2.0)), // 2px thick line
                            ..default()
                        },
                        Transform {
                            translation: Vec3::new(center.x, center.y, 150.0), // Below text but above terrain
                            rotation: Quat::from_rotation_z(angle),
                            ..default()
                        },
                    )).id();
                    
                    waypoint_line.line_segments.push(line_entity);
                    previous_point = end_pos;
                }
                
                // Add waypoint markers (small circles at each waypoint)
                for (i, &waypoint) in pawn_target.path.iter().enumerate() {
                    let color = if i < pawn_target.current_waypoint_index {
                        Color::srgb(0.3, 0.3, 0.3) // Gray for completed waypoints
                    } else if i == pawn_target.current_waypoint_index {
                        Color::srgb(0.0, 1.0, 0.0) // Green for current waypoint
                    } else {
                        Color::srgb(1.0, 1.0, 0.0) // Yellow for future waypoints
                    };
                    
                    let marker_entity = commands.spawn((
                        Sprite {
                            color,
                            custom_size: Some(Vec2::new(4.0, 4.0)), // 4px diameter circle
                            ..default()
                        },
                        Transform::from_translation(Vec3::new(waypoint.x, waypoint.y, 160.0)),
                    )).id();
                    
                    waypoint_line.line_segments.push(marker_entity);
                }
            }
        } else {
            // Pawn no longer exists or doesn't have a target, clean up
            for &segment_entity in &waypoint_line.line_segments {
                if let Some(mut entity_commands) = commands.get_entity(segment_entity) {
                    entity_commands.despawn();
                }
            }
            commands.entity(waypoint_line_entity).despawn();
        }
    }
}

pub fn cleanup_orphaned_waypoint_lines(
    mut commands: Commands,
    pawn_query: Query<Entity, (With<Pawn>, With<PawnTarget>)>,
    waypoint_line_query: Query<(Entity, &WaypointLine)>,
) {
    let existing_pawns_with_targets: std::collections::HashSet<Entity> = pawn_query.iter().collect();
    
    for (waypoint_line_entity, waypoint_line) in waypoint_line_query.iter() {
        if !existing_pawns_with_targets.contains(&waypoint_line.pawn_entity) {
            // Clean up all line segment entities
            for &segment_entity in &waypoint_line.line_segments {
                if let Some(mut entity_commands) = commands.get_entity(segment_entity) {
                    entity_commands.despawn();
                }
            }
            // Remove the waypoint line entity itself
            commands.entity(waypoint_line_entity).despawn();
        }
    }
}