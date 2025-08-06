#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use crate::systems::pawn::{Pawn, Health, Endurance, CurrentBehavior};
    use crate::systems::pawn_config::{PawnConfig, PawnDefinition, PawnBehaviours, PawnEats, BehaviourConfig, BehaviourType};
    use crate::systems::ai::{HuntSoloAI, hunt_solo_ai_system, setup_hunt_solo_ai};
    use crate::resources::GameConfig;
    use crate::tests::{setup_test_app, create_test_terrain_map};

    fn create_test_hunter_config() -> PawnConfig {
        let mut pawns = std::collections::HashMap::new();
        
        // Create hunter (wolf)
        pawns.insert("wolf".to_string(), PawnDefinition {
            sprite: "wolf.png".to_string(),
            tags: vec!["medium".to_string(), "animal".to_string(), "carnivore".to_string()],
            move_speed: 120.0,
            max_health: 110,
            max_endurance: 60,
            strength: 30,
            defence: 10,
            attack_speed: 3.0,
            reach: 1,
            spawn_count: 1,
            behaviours: PawnBehaviours {
                idle: None,
                hunted: None,
                looking_for_food: Some(BehaviourConfig::Simple(BehaviourType::HuntSolo)),
                eat: None,
                controlled: None,
                flee: None,
            },
            eats: PawnEats { pawns: vec!["small".to_string(), "animal".to_string()] },
        });
        
        // Create prey (rabbit)
        pawns.insert("rabbit".to_string(), PawnDefinition {
            sprite: "rabbit.png".to_string(),
            tags: vec!["small".to_string(), "animal".to_string(), "herbivore".to_string()],
            move_speed: 100.0,
            max_health: 25,
            max_endurance: 10,
            strength: 5,
            defence: 5,
            attack_speed: 3.0,
            reach: 1,
            spawn_count: 1,
            behaviours: PawnBehaviours {
                idle: None,
                hunted: None,
                looking_for_food: None,
                eat: None,
                controlled: None,
                flee: None,
            },
            eats: PawnEats { pawns: vec![] },
        });
        
        // Create non-prey (stone golem - not small or animal)
        pawns.insert("golem".to_string(), PawnDefinition {
            sprite: "golem.png".to_string(),
            tags: vec!["large".to_string(), "construct".to_string()],
            move_speed: 50.0,
            max_health: 200,
            max_endurance: 100,
            strength: 40,
            defence: 20,
            attack_speed: 1.0,
            reach: 1,
            spawn_count: 1,
            behaviours: PawnBehaviours {
                idle: None,
                hunted: None,
                looking_for_food: None,
                eat: None,
                controlled: None,
                flee: None,
            },
            eats: PawnEats { pawns: vec![] },
        });
        
        PawnConfig { pawns }
    }

    fn create_test_config() -> GameConfig {
        GameConfig {
            tile_size: 16.0,
            map_width: 32,
            map_height: 32,
            camera_speed: 200.0,
            zoom_min: 0.1,
            zoom_max: 10.0,
            mouse_sensitivity: 1.0,
            window_title: "Test".to_string(),
            target_fps: 60,
            show_fps: false,
            endurance_cost_per_cell: 1.0,
            health_loss_interval: 5.0,
        }
    }

    #[test]
    fn test_hunt_solo_ai_component_creation() {
        let ai = HuntSoloAI::new();
        assert_eq!(ai.target_entity, None);
        assert_eq!(ai.last_attack_time, 0.0);
        assert_eq!(ai.search_timer, 0.0);
    }

    #[test]
    fn test_can_eat_by_tags() {
        let config = create_test_hunter_config();
        
        // Wolf should be able to eat rabbit (has both "small" and "animal" tags)
        assert!(config.can_eat_by_tags(&"wolf".to_string(), &"rabbit".to_string()));
        
        // Wolf should not be able to eat golem (doesn't have required tags)
        assert!(!config.can_eat_by_tags(&"wolf".to_string(), &"golem".to_string()));
        
        // Rabbit should not be able to eat wolf (rabbit has no eats defined, so empty list means can't eat anything)
        assert!(!config.can_eat_by_tags(&"rabbit".to_string(), &"wolf".to_string()));
    }

    #[test]
    fn test_setup_hunt_solo_ai_adds_component() {
        let mut app = setup_test_app();
        let config = create_test_hunter_config();
        
        app.insert_resource(config);
        
        // Spawn a wolf in looking_for_food state (should get HuntSoloAI)
        let hunter_entity = app.world_mut().spawn((
            Pawn::new("wolf".to_string()),
            CurrentBehavior { state: "looking_for_food".to_string() },
            Health::new(110),
            Endurance::new(60),
            Transform::default(),
        )).id();
        
        // Spawn a rabbit in idle state (should not get HuntSoloAI)
        let prey_entity = app.world_mut().spawn((
            Pawn::new("rabbit".to_string()),
            CurrentBehavior { state: "idle".to_string() },
            Health::new(25),
            Endurance::new(10),
            Transform::default(),
        )).id();

        app.add_systems(Update, setup_hunt_solo_ai);
        app.update();

        // Check that hunter got HuntSoloAI component
        assert!(app.world().entity(hunter_entity).get::<HuntSoloAI>().is_some(), 
                "Hunter should have HuntSoloAI component");
        
        // Check that prey did not get HuntSoloAI component
        assert!(app.world().entity(prey_entity).get::<HuntSoloAI>().is_none(), 
                "Prey should not have HuntSoloAI component");
    }

    #[test]
    fn test_hunt_solo_ai_finds_valid_prey() {
        let mut app = setup_test_app();
        let config = create_test_hunter_config();
        let game_config = create_test_config();
        let terrain_map = create_test_terrain_map(10, 10, 16.0);
        
        app.insert_resource(config);
        app.insert_resource(game_config);
        app.insert_resource(terrain_map);
        
        // Spawn hunter at (0, 0)
        let hunter_entity = app.world_mut().spawn((
            Pawn::new("wolf".to_string()),
            CurrentBehavior { state: "looking_for_food".to_string() },
            Health::new(110),
            Endurance::new(60),
            Transform::from_translation(Vec3::new(0.0, 0.0, 100.0)),
            HuntSoloAI::new(),
        )).id();
        
        // Spawn valid prey at (32, 0) - should be found
        let valid_prey_entity = app.world_mut().spawn((
            Pawn::new("rabbit".to_string()),
            CurrentBehavior { state: "idle".to_string() },
            Health::new(25),
            Transform::from_translation(Vec3::new(32.0, 0.0, 100.0)),
        )).id();
        
        // Spawn invalid prey at (16, 0) - should be ignored (closer but invalid)
        let _invalid_prey_entity = app.world_mut().spawn((
            Pawn::new("golem".to_string()),
            CurrentBehavior { state: "idle".to_string() },
            Health::new(200),
            Transform::from_translation(Vec3::new(16.0, 0.0, 100.0)),
        )).id();

        app.add_systems(Update, hunt_solo_ai_system);
        
        // Fast-forward time to trigger search (search happens every 2 seconds)
        // Modify the hunt AI to trigger immediate search
        app.world_mut().entity_mut(hunter_entity).get_mut::<HuntSoloAI>().unwrap().search_timer = 2.0;
        
        app.update();

        // Check that hunter found the valid prey
        let hunt_ai = app.world().entity(hunter_entity).get::<HuntSoloAI>().unwrap();
        assert_eq!(hunt_ai.target_entity, Some(valid_prey_entity), 
                   "Hunter should target the valid prey (rabbit)");
    }

    #[test] 
    fn test_hunt_solo_ai_ignores_dead_prey() {
        let mut app = setup_test_app();
        let config = create_test_hunter_config();
        let game_config = create_test_config();
        let terrain_map = create_test_terrain_map(10, 10, 16.0);
        
        app.insert_resource(config);
        app.insert_resource(game_config);
        app.insert_resource(terrain_map);
        
        // Spawn hunter
        let hunter_entity = app.world_mut().spawn((
            Pawn::new("wolf".to_string()),
            CurrentBehavior { state: "looking_for_food".to_string() },
            Health::new(110),
            Endurance::new(60),
            Transform::from_translation(Vec3::new(0.0, 0.0, 100.0)),
            HuntSoloAI::new(),
        )).id();
        
        // Spawn dead prey
        let _dead_prey_entity = app.world_mut().spawn((
            Pawn::new("rabbit".to_string()),
            CurrentBehavior { state: "idle".to_string() },
            Health { current: 0.0, max: 25.0 }, // Dead
            Transform::from_translation(Vec3::new(16.0, 0.0, 100.0)),
        )).id();

        app.add_systems(Update, hunt_solo_ai_system);
        
        // Fast-forward time to trigger search
        app.world_mut().entity_mut(hunter_entity).get_mut::<HuntSoloAI>().unwrap().search_timer = 2.0;
        
        app.update();

        // Check that hunter found no target
        let hunt_ai = app.world().entity(hunter_entity).get::<HuntSoloAI>().unwrap();
        assert_eq!(hunt_ai.target_entity, None, 
                   "Hunter should not target dead prey");
    }

    #[test]
    fn test_damage_calculation() {
        let config = create_test_hunter_config();
        
        let wolf_def = config.get_pawn_definition("wolf").unwrap();
        let rabbit_def = config.get_pawn_definition("rabbit").unwrap();
        
        // Wolf (30 strength) vs Rabbit (5 defence) = 25 damage
        let expected_damage = (wolf_def.strength as f32 - rabbit_def.defence as f32).max(0.0);
        assert_eq!(expected_damage, 25.0);
        
        // Test case where defence >= strength (no damage)
        let golem_def = config.get_pawn_definition("golem").unwrap();
        let rabbit_vs_golem_damage = (rabbit_def.strength as f32 - golem_def.defence as f32).max(0.0);
        assert_eq!(rabbit_vs_golem_damage, 0.0);
    }

    #[test]
    fn test_attack_timing() {
        let config = create_test_hunter_config();
        let wolf_def = config.get_pawn_definition("wolf").unwrap();
        
        let attack_interval = 1.0 / wolf_def.attack_speed; // 1.0 / 3.0 = 0.333...
        assert!((attack_interval - 0.333333).abs() < 0.001);
    }

    #[test]
    fn test_reach_distance_calculation() {
        let config = create_test_hunter_config();
        let game_config = create_test_config();
        let wolf_def = config.get_pawn_definition("wolf").unwrap();
        
        let reach_distance = wolf_def.reach as f32 * game_config.tile_size; // 1 * 16.0 = 16.0
        assert_eq!(reach_distance, 16.0);
    }

    #[test]
    fn test_endurance_gain_on_kill() {
        let config = create_test_hunter_config();
        let rabbit_def = config.get_pawn_definition("rabbit").unwrap();
        
        let mut hunter_endurance = Endurance::new(50);
        let initial_endurance = hunter_endurance.current;
        
        // Simulate gaining endurance from killing rabbit
        hunter_endurance.current = (hunter_endurance.current + rabbit_def.max_health as f32).min(hunter_endurance.max);
        
        let expected_endurance = (initial_endurance + rabbit_def.max_health as f32).min(hunter_endurance.max);
        assert_eq!(hunter_endurance.current, expected_endurance);
        
        // Test case where endurance would exceed max
        let mut full_endurance_hunter = Endurance::new(60);
        full_endurance_hunter.current = 55.0; // Close to max
        
        full_endurance_hunter.current = (full_endurance_hunter.current + rabbit_def.max_health as f32).min(full_endurance_hunter.max);
        assert_eq!(full_endurance_hunter.current, 60.0); // Should be capped at max
    }

    #[test]
    fn test_closest_target_selection() {
        // This is a logic test for the target selection algorithm
        let hunter_pos = Vec3::new(0.0, 0.0, 100.0);
        let prey1_pos = Vec3::new(32.0, 0.0, 100.0);  // Distance: 32
        let prey2_pos = Vec3::new(16.0, 0.0, 100.0);  // Distance: 16 (closer)
        
        let distance1 = hunter_pos.distance(prey1_pos);
        let distance2 = hunter_pos.distance(prey2_pos);
        
        assert!(distance2 < distance1, "Prey2 should be closer than Prey1");
        assert_eq!(distance1, 32.0);
        assert_eq!(distance2, 16.0);
    }

    #[test]
    fn test_hunt_solo_behavior_state_check() {
        let config = create_test_hunter_config();
        
        // Wolf in looking_for_food state should have hunt_solo behavior
        let hunt_behavior = config.get_behaviour_config("wolf", "looking_for_food");
        assert!(hunt_behavior.is_some());
        assert!(matches!(hunt_behavior.unwrap(), BehaviourConfig::Simple(BehaviourType::HuntSolo)));
        
        // Wolf in idle state should not have hunt_solo behavior
        let idle_behavior = config.get_behaviour_config("wolf", "idle");
        assert!(idle_behavior.is_none());
        
        // Rabbit should not have hunt_solo behavior in any state
        let rabbit_hunt_behavior = config.get_behaviour_config("rabbit", "looking_for_food");
        assert!(rabbit_hunt_behavior.is_none());
    }
}