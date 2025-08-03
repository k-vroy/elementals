use bevy::prelude::*;
use crate::systems::world_gen::TerrainMap;

#[derive(Component)]
pub struct Pawn {
    pub pawn_type: PawnType,
    pub move_speed: f32,
    pub sprite_path: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PawnType {
    Player,
    Wolf,
}

impl Pawn {
    pub fn new_player() -> Self {
        Self {
            pawn_type: PawnType::Player,
            move_speed: 150.0,
            sprite_path: "player.png".to_string(),
        }
    }

    pub fn new_wolf() -> Self {
        Self {
            pawn_type: PawnType::Wolf,
            move_speed: 120.0,
            sprite_path: "wolf.png".to_string(),
        }
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
    pawn: Pawn,
    spawn_position: Option<(f32, f32)>,
) -> Entity {
    let position = if let Some(pos) = spawn_position {
        pos
    } else {
        // Find a passable spawn position
        let initial_center = (0.0, 0.0);
        if let Some(passable_pos) = terrain_map.find_nearest_passable_tile(initial_center) {
            passable_pos
        } else {
            (0.0, 0.0) // Fallback
        }
    };

    commands.spawn((
        Sprite::from_image(asset_server.load(&pawn.sprite_path)),
        Transform::from_translation(Vec3::new(position.0, position.1, 100.0)),
        pawn,
    )).id()
}

pub fn move_pawn_to_target(
    time: Res<Time>,
    mut pawn_query: Query<(&mut Transform, &mut PawnTarget, &Pawn)>,
) {
    for (mut transform, mut target, pawn) in pawn_query.iter_mut() {
        if let Some(current_waypoint) = target.get_current_waypoint() {
            let distance = transform.translation.distance(current_waypoint);
            
            if distance > 2.0 { // Close enough threshold for waypoints
                let direction = (current_waypoint - transform.translation).normalize();
                let movement = direction * pawn.move_speed * time.delta_secs();
                
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
                    match pawn.pawn_type {
                        PawnType::Player => println!("Player reached destination: {:?}", target.target_position),
                        PawnType::Wolf => println!("Wolf reached destination: {:?}", target.target_position),
                    }
                    target.reset();
                } else {
                    target.advance_waypoint();
                }
            }
        }
    }
}