use bevy::prelude::*;
use crate::systems::world_gen::TerrainMap;
use crate::systems::pawn_config::{PawnConfig, PawnType};

#[derive(Component)]
pub struct Pawn {
    pub pawn_type: PawnType,
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
    pawn_config: &Res<PawnConfig>,
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

    let pawn_def = pawn_config.get_pawn_definition(&pawn.pawn_type)
        .expect("Pawn definition not found in config");

    commands.spawn((
        Sprite::from_image(asset_server.load(&pawn_def.sprite)),
        Transform::from_translation(Vec3::new(position.0, position.1, 100.0)),
        pawn,
    )).id()
}

pub fn move_pawn_to_target(
    time: Res<Time>,
    pawn_config: Res<PawnConfig>,
    mut pawn_query: Query<(&mut Transform, &mut PawnTarget, &Pawn)>,
) {
    for (mut transform, mut target, pawn) in pawn_query.iter_mut() {
        if let Some(current_waypoint) = target.get_current_waypoint() {
            let distance = transform.translation.distance(current_waypoint);
            
            if distance > 2.0 { // Close enough threshold for waypoints
                let pawn_def = pawn_config.get_pawn_definition(&pawn.pawn_type)
                    .expect("Pawn definition not found in config");
                
                let direction = (current_waypoint - transform.translation).normalize();
                let movement = direction * pawn_def.move_speed * time.delta_secs();
                
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
                    println!("{} reached destination: {:?}", pawn.pawn_type, target.target_position);
                    target.reset();
                } else {
                    target.advance_waypoint();
                }
            }
        }
    }
}