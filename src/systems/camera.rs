use bevy::prelude::*;
use bevy::input::mouse::{MouseWheel, MouseScrollUnit, MouseMotion};
use crate::resources::GameConfig;

#[derive(Component)]
pub struct CameraController;

#[derive(Resource, Default)]
pub struct MouseDragState {
    pub is_dragging: bool,
    pub last_position: Vec2,
}

pub fn camera_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    config: Res<GameConfig>,
    mut query: Query<&mut Transform, (With<Camera>, With<CameraController>)>,
) {
    for mut transform in &mut query {
        let mut direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
            direction.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
            direction.y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
            direction.x += 1.0;
        }

        if direction.length() > 0.0 {
            direction = direction.normalize();
            transform.translation += direction * config.camera_speed * time.delta_secs();
        }
    }
}

pub fn camera_zoom(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    config: Res<GameConfig>,
    mut scroll_events: EventReader<MouseWheel>,
    mut camera_query: Query<(&mut Transform, &mut OrthographicProjection), With<Camera>>,
    windows: Query<&Window>,
) {
    if let Ok((mut camera_transform, mut projection)) = camera_query.get_single_mut() {
        let zoom_speed = 2.0;

        // Keyboard zoom (centered)
        if keyboard_input.pressed(KeyCode::Equal) || keyboard_input.pressed(KeyCode::NumpadAdd) {
            projection.scale *= 1.0 - zoom_speed * time.delta_secs();
            projection.scale = projection.scale.max(config.zoom_min);
        }
        if keyboard_input.pressed(KeyCode::Minus) || keyboard_input.pressed(KeyCode::NumpadSubtract) {
            projection.scale *= 1.0 + zoom_speed * time.delta_secs();
            projection.scale = projection.scale.min(config.zoom_max);
        }

        // Mouse wheel zoom (zoom towards cursor)
        for scroll in scroll_events.read() {
            if let Ok(window) = windows.get_single() {
                if let Some(cursor_position) = window.cursor_position() {
                    let zoom_factor = match scroll.unit {
                        MouseScrollUnit::Line => 0.1,
                        MouseScrollUnit::Pixel => 0.001,
                    };
                    
                    let old_scale = projection.scale;
                    projection.scale *= 1.0 - scroll.y * zoom_factor;
                    projection.scale = projection.scale.clamp(config.zoom_min, config.zoom_max);
                    
                    // Calculate zoom towards cursor
                    if projection.scale != old_scale {
                        // Convert cursor position to world coordinates before zoom
                        let window_size = Vec2::new(window.width(), window.height());
                        let cursor_ndc = (cursor_position / window_size) * 2.0 - Vec2::ONE;
                        let cursor_ndc = Vec2::new(cursor_ndc.x, -cursor_ndc.y); // Flip Y
                        
                        // Calculate world position of cursor before zoom
                        let cursor_world_before = camera_transform.translation.truncate() 
                            + cursor_ndc * window_size * old_scale * 0.5;
                        
                        // Calculate world position of cursor after zoom
                        let cursor_world_after = camera_transform.translation.truncate() 
                            + cursor_ndc * window_size * projection.scale * 0.5;
                        
                        // Move camera to keep cursor at same world position
                        let offset = cursor_world_before - cursor_world_after;
                        camera_transform.translation += offset.extend(0.0);
                    }
                }
            }
        }
    }
}

pub fn mouse_camera_pan(
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut drag_state: ResMut<MouseDragState>,
    config: Res<GameConfig>,
    mut camera_query: Query<&mut Transform, (With<Camera>, With<CameraController>)>,
    projection_query: Query<&OrthographicProjection, With<Camera>>,
) {
    // Handle mouse button press/release for drag state
    if mouse_input.just_pressed(MouseButton::Middle) {
        drag_state.is_dragging = true;
    }
    
    if mouse_input.just_released(MouseButton::Middle) {
        drag_state.is_dragging = false;
    }

    // Handle mouse movement for camera panning
    if drag_state.is_dragging {
        // Accumulate all mouse motion for this frame
        let mut total_delta = Vec2::ZERO;
        for motion in mouse_motion.read() {
            total_delta += motion.delta;
        }

        // Only move camera if there was actual mouse movement
        if total_delta.length() > 0.0 {
            if let (Ok(mut camera_transform), Ok(projection)) = 
                (camera_query.get_single_mut(), projection_query.get_single()) {
                
                // Scale the movement by the camera's zoom level and mouse sensitivity
                let movement_scale = projection.scale * config.mouse_sensitivity;
                
                // Invert the movement so dragging feels natural
                let movement = Vec3::new(
                    -total_delta.x * movement_scale,
                    total_delta.y * movement_scale,
                    0.0
                );
                
                camera_transform.translation += movement;
            }
        }
    }
}