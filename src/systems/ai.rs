use bevy::prelude::*;
use rand::prelude::*;
use crate::systems::pawn::{Pawn, PawnTarget, PawnType};
use crate::systems::world_gen::TerrainMap;

#[derive(Component)]
pub struct WolfAI {
    pub next_move_time: f32,
    pub move_interval_min: f32,
    pub move_interval_max: f32,
    pub move_range: f32,
}

impl WolfAI {
    pub fn new() -> Self {
        Self {
            next_move_time: 0.0,
            move_interval_min: 3.0, // Minimum 3 seconds between moves
            move_interval_max: 8.0, // Maximum 8 seconds between moves
            move_range: 128.0,      // Maximum distance to move (4 tiles * 32 tile_size)
        }
    }

    pub fn schedule_next_move(&mut self) {
        let mut rng = rand::thread_rng();
        let interval = rng.gen_range(self.move_interval_min..=self.move_interval_max);
        self.next_move_time = interval;
    }
}

pub fn wolf_ai_system(
    time: Res<Time>,
    terrain_map: Res<TerrainMap>,
    mut commands: Commands,
    mut wolf_query: Query<(Entity, &Transform, &Pawn, &mut WolfAI), With<Pawn>>,
) {
    let mut rng = rand::thread_rng();
    
    for (entity, transform, pawn, mut wolf_ai) in wolf_query.iter_mut() {
        if pawn.pawn_type != PawnType::Wolf {
            continue;
        }

        // Update timer
        wolf_ai.next_move_time -= time.delta_secs();
        
        // Time to move?
        if wolf_ai.next_move_time <= 0.0 {
            let current_pos = (transform.translation.x, transform.translation.y);
            
            // Try to find a random nearby passable location
            let mut attempts = 0;
            while attempts < 10 {
                attempts += 1;
                
                // Generate random offset within move range
                let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                let distance = rng.gen_range(32.0..wolf_ai.move_range);
                
                let target_x = current_pos.0 + angle.cos() * distance;
                let target_y = current_pos.1 + angle.sin() * distance;
                let target_pos = (target_x, target_y);
                
                // Check if target is passable and find a path
                if let Some(path) = terrain_map.find_path(current_pos, target_pos) {
                    // Create target and set path
                    let mut pawn_target = PawnTarget::new(Vec3::new(target_x, target_y, 100.0));
                    pawn_target.set_path(path);
                    
                    // Add target component to wolf
                    commands.entity(entity).insert(pawn_target);
                    break;
                }
            }
            
            // Schedule next move regardless of whether we found a path
            wolf_ai.schedule_next_move();
        }
    }
}

// System to add WolfAI component to newly spawned wolves
pub fn setup_wolf_ai(
    mut commands: Commands,
    wolf_query: Query<(Entity, &Pawn), (With<Pawn>, Without<WolfAI>)>,
) {
    for (entity, pawn) in wolf_query.iter() {
        if pawn.pawn_type == PawnType::Wolf {
            let mut wolf_ai = WolfAI::new();
            wolf_ai.schedule_next_move(); // Schedule first move
            commands.entity(entity).insert(wolf_ai);
        }
    }
}