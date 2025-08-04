use bevy::prelude::*;
use crate::systems::world_gen::TerrainMap;
use crate::systems::pawn_config::{PawnConfig, PawnType};
use crate::resources::GameConfig;

#[derive(Component)]
pub struct Pawn {
    pub pawn_type: PawnType,
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
        Health::new(pawn_def.max_health),
        Endurance::new(pawn_def.max_endurance),
    )).id()
}

pub fn move_pawn_to_target(
    time: Res<Time>,
    pawn_config: Res<PawnConfig>,
    config: Res<GameConfig>,
    mut pawn_query: Query<(&mut Transform, &mut PawnTarget, &Pawn, &mut Endurance)>,
) {
    for (mut transform, mut target, pawn, mut endurance) in pawn_query.iter_mut() {
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
                    target.reset();
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
    mut pawn_query: Query<(&mut Health, &mut Endurance), With<Pawn>>,
) {
    for (mut health, mut endurance) in pawn_query.iter_mut() {
        if endurance.current <= 0.0 {
            // Update health loss timer
            endurance.health_loss_timer += time.delta_secs();
            
            // Check if it's time to lose health
            if endurance.health_loss_timer >= config.health_loss_interval {
                health.current = (health.current - 1.0).max(0.0);
                endurance.health_loss_timer = 0.0; // Reset timer
                
                if health.current <= 0.0 {
                    println!("Pawn has died from exhaustion!");
                }
            }
        } else {
            // Reset health loss timer if endurance is above 0
            endurance.health_loss_timer = 0.0;
        }
    }
}