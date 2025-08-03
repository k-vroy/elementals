use bevy::prelude::*;
use rand::prelude::*;
use crate::systems::pawn::{Pawn, PawnTarget};
use crate::systems::pawn_config::PawnConfig;
use crate::systems::world_gen::TerrainMap;
use crate::resources::GameConfig;

#[derive(Component)]
pub struct WanderingAI {
    pub next_move_time: f32,
}

impl WanderingAI {
    pub fn new() -> Self {
        Self {
            next_move_time: 0.0,
        }
    }

    pub fn schedule_next_move(&mut self, min_interval: f32, max_interval: f32) {
        let mut rng = rand::thread_rng();
        let interval = rng.gen_range(min_interval..=max_interval);
        self.next_move_time = interval;
    }
}

pub fn wandering_ai_system(
    time: Res<Time>,
    terrain_map: Res<TerrainMap>,
    pawn_config: Res<PawnConfig>,
    config: Res<GameConfig>,
    mut commands: Commands,
    mut wandering_query: Query<(Entity, &Transform, &Pawn, &mut WanderingAI), With<Pawn>>,
) {
    let mut rng = rand::thread_rng();
    
    for (entity, transform, pawn, mut ai) in wandering_query.iter_mut() {
        // Get wandering config for this pawn's idle behavior
        let wandering_config = match pawn_config.get_wandering_config(&pawn.pawn_type, "idle") {
            Some(config) => config,
            None => continue, // Skip pawns without wandering behavior
        };

        // Update timer
        ai.next_move_time -= time.delta_secs();
        
        // Time to move?
        if ai.next_move_time <= 0.0 {
            let current_pos = (transform.translation.x, transform.translation.y);
            
            // Try to find a random nearby passable location
            let mut attempts = 0;
            while attempts < 10 {
                attempts += 1;
                
                // Generate random offset within move range (convert cells to pixels)
                let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                let move_range_pixels = wandering_config.move_range as f32 * config.tile_size;
                let min_distance = config.tile_size;
                let max_distance = move_range_pixels.max(min_distance + 1.0);
                let distance = rng.gen_range(min_distance..max_distance);
                
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
            ai.schedule_next_move(wandering_config.move_interval_min, wandering_config.move_interval_max);
        }
    }
}

// System to add WanderingAI component to pawns with wandering behavior
pub fn setup_wandering_ai(
    mut commands: Commands,
    pawn_config: Res<PawnConfig>,
    wandering_query: Query<(Entity, &Pawn), (With<Pawn>, Without<WanderingAI>)>,
) {
    for (entity, pawn) in wandering_query.iter() {
        // Check if this pawn has wandering behavior configured
        if let Some(wandering_config) = pawn_config.get_wandering_config(&pawn.pawn_type, "idle") {
            let mut ai = WanderingAI::new();
            ai.schedule_next_move(wandering_config.move_interval_min, wandering_config.move_interval_max);
            commands.entity(entity).insert(ai);
        }
    }
}