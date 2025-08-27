use bevy::prelude::*;
use rand::prelude::*;
use crate::systems::pawn::{Pawn, PawnTarget, CurrentBehavior, Health, Endurance, Size};
use crate::systems::pawn_config::PawnConfig;
use crate::systems::world_gen::{TerrainMap, GroundConfigs};
use crate::systems::async_pathfinding::{PathfindingRequest, PathfindingPriority, request_pathfinding};
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

#[derive(Component)]
pub struct HuntSoloAI {
    pub target_entity: Option<Entity>,
    pub last_attack_time: f32,
    pub search_timer: f32,
}

impl HuntSoloAI {
    pub fn new() -> Self {
        Self {
            target_entity: None,
            last_attack_time: 0.0,
            search_timer: 0.0,
        }
    }
}

pub fn wandering_ai_system(
    time: Res<Time>,
    terrain_map: Res<TerrainMap>,
    ground_configs: Res<GroundConfigs>,
    pawn_config: Res<PawnConfig>,
    config: Res<GameConfig>,
    mut commands: Commands,
    mut wandering_query: Query<(Entity, &Transform, &Pawn, &Size, &CurrentBehavior, &mut WanderingAI), (With<Pawn>, Without<PawnTarget>, Without<PathfindingRequest>)>,
) {
    let mut rng = rand::thread_rng();
    
    for (entity, transform, pawn, size, current_behavior, mut ai) in wandering_query.iter_mut() {
        // Get wandering config for this pawn's current behavior
        let wandering_config = match pawn_config.get_wandering_config(&pawn.pawn_type, &current_behavior.state) {
            Some(config) => config,
            None => continue, // Skip pawns without wandering behavior for current state
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
                
                // Check if target is potentially passable (quick check)
                if terrain_map.is_position_passable_for_size(target_pos.0, target_pos.1, size.value, &ground_configs) {
                    // Request async pathfinding
                    request_pathfinding(&mut commands, entity, current_pos, target_pos, size.value);
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
    wandering_query: Query<(Entity, &Pawn, &CurrentBehavior), (With<Pawn>, Without<WanderingAI>)>,
) {
    for (entity, pawn, current_behavior) in wandering_query.iter() {
        // Check if this pawn has wandering behavior configured for its current state
        if let Some(wandering_config) = pawn_config.get_wandering_config(&pawn.pawn_type, &current_behavior.state) {
            let mut ai = WanderingAI::new();
            ai.schedule_next_move(wandering_config.move_interval_min, wandering_config.move_interval_max);
            commands.entity(entity).insert(ai);
        }
    }
}

// System to add HuntSoloAI component to pawns with hunt_solo behavior
pub fn setup_hunt_solo_ai(
    mut commands: Commands,
    pawn_config: Res<PawnConfig>,
    hunt_query: Query<(Entity, &Pawn, &CurrentBehavior), (With<Pawn>, Without<HuntSoloAI>)>,
) {
    for (entity, pawn, current_behavior) in hunt_query.iter() {
        // Check if this pawn has hunt_solo behavior configured for its current state
        if let Some(behavior_config) = pawn_config.get_behaviour_config(&pawn.pawn_type, &current_behavior.state) {
            if matches!(behavior_config, crate::systems::pawn_config::BehaviourConfig::Simple(crate::systems::pawn_config::BehaviourType::HuntSolo)) {
                commands.entity(entity).insert(HuntSoloAI::new());
            }
        }
    }
}

pub fn hunt_solo_ai_system(
    time: Res<Time>,
    pawn_config: Res<PawnConfig>,
    config: Res<GameConfig>,
    mut commands: Commands,
    mut hunter_query: Query<(Entity, &Transform, &Pawn, &Size, &CurrentBehavior, &mut HuntSoloAI, &mut Endurance, Option<&PawnTarget>), (With<Pawn>, Without<PathfindingRequest>)>,
    mut prey_query: Query<(Entity, &Transform, &Pawn, &mut Health), (With<Pawn>, Without<HuntSoloAI>)>,
) {
    for (hunter_entity, hunter_transform, hunter_pawn, hunter_size, current_behavior, mut hunt_ai, mut hunter_endurance, current_target) in hunter_query.iter_mut() {
        // Only process if in hunt_solo behavior state
        if let Some(behavior_config) = pawn_config.get_behaviour_config(&hunter_pawn.pawn_type, &current_behavior.state) {
            if !matches!(behavior_config, crate::systems::pawn_config::BehaviourConfig::Simple(crate::systems::pawn_config::BehaviourType::HuntSolo)) {
                continue;
            }
        } else {
            continue;
        }

        let hunter_def = match pawn_config.get_pawn_definition(&hunter_pawn.pawn_type) {
            Some(def) => def,
            None => continue,
        };

        // Update attack timer
        hunt_ai.last_attack_time += time.delta_secs();
        hunt_ai.search_timer += time.delta_secs();

        // Check if current target is still valid
        if let Some(target_entity) = hunt_ai.target_entity {
            if let Ok((_, target_transform, target_pawn, mut target_health)) = prey_query.get_mut(target_entity) {
                // Check distance to target
                let distance = hunter_transform.translation.distance(target_transform.translation);
                let reach_distance = hunter_def.reach as f32 * config.tile_size;

                // If within reach, attack
                if distance <= reach_distance {
                    let attack_interval = 1.0 / hunter_def.attack_speed;
                    if hunt_ai.last_attack_time >= attack_interval {
                        // Calculate damage
                        let target_def = pawn_config.get_pawn_definition(&target_pawn.pawn_type).unwrap();
                        let damage = (hunter_def.strength as f32 - target_def.defence as f32).max(0.0);
                        
                        target_health.current = (target_health.current - damage).max(0.0);
                        hunt_ai.last_attack_time = 0.0;
                        
                        println!("{} attacks {} for {} damage (health: {:.1})", 
                                hunter_pawn.pawn_type, target_pawn.pawn_type, damage, target_health.current);

                        // Check if target died
                        if target_health.current <= 0.0 {
                            // Add target's max health to hunter's endurance
                            hunter_endurance.current = (hunter_endurance.current + target_def.max_health as f32).min(hunter_endurance.max);
                            println!("{} gained {} endurance from killing {}", 
                                    hunter_pawn.pawn_type, target_def.max_health, target_pawn.pawn_type);
                            hunt_ai.target_entity = None;
                        }
                    }
                    continue; // Don't move if attacking
                } else {
                    // Move towards target - only create new path if hunter doesn't have one
                    let needs_new_path = match current_target {
                        Some(pawn_target) => {
                            // Check if we need a new path (target position changed or path is empty)
                            let target_pos_vec = Vec3::new(target_transform.translation.x, target_transform.translation.y, 100.0);
                            pawn_target.target_position.distance(target_pos_vec) > 5.0 || pawn_target.path.is_empty()
                        },
                        None => true, // No current target, need new path
                    };

                    if needs_new_path {
                        let current_pos = (hunter_transform.translation.x, hunter_transform.translation.y);
                        let target_pos = (target_transform.translation.x, target_transform.translation.y);
                        
                        // Request high-priority async pathfinding for hunting
                        commands.entity(hunter_entity).insert(
                            PathfindingRequest::new(current_pos, target_pos, hunter_size.value)
                                .with_priority(PathfindingPriority::High)
                        );
                    }
                    continue;
                }
            } else {
                // Target no longer exists, clear it
                hunt_ai.target_entity = None;
            }
        }

        // Search for new target every 2 seconds
        if hunt_ai.search_timer >= 2.0 {
            hunt_ai.search_timer = 0.0;
            
            let mut closest_target: Option<(Entity, f32)> = None;
            let hunter_pos = hunter_transform.translation;

            for (prey_entity, prey_transform, prey_pawn, prey_health) in prey_query.iter() {
                // Skip dead prey
                if prey_health.current <= 0.0 {
                    continue;
                }
                
                // Check if hunter can eat this prey
                if pawn_config.can_eat_by_tags(&hunter_pawn.pawn_type, &prey_pawn.pawn_type) {
                    let distance = hunter_pos.distance(prey_transform.translation);
                    
                    if let Some((_, closest_dist)) = closest_target {
                        if distance < closest_dist {
                            closest_target = Some((prey_entity, distance));
                        }
                    } else {
                        closest_target = Some((prey_entity, distance));
                    }
                }
            }

            if let Some((target_entity, _)) = closest_target {
                hunt_ai.target_entity = Some(target_entity);
            }
        }
    }
}