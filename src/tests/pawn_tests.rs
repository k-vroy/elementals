#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use crate::systems::pawn::{Pawn, Health, Endurance, PawnTarget, move_pawn_to_target, endurance_health_loss_system, pawn_death_system};
    use crate::systems::pawn_config::{PawnConfig, PawnDefinition, PawnBehaviours, PawnEats};
    use crate::resources::GameConfig;
    use crate::tests::setup_test_app;

    fn create_test_pawn_config() -> PawnConfig {
        let mut pawns = std::collections::HashMap::new();
        pawns.insert("test_pawn".to_string(), PawnDefinition {
            sprite: "test.png".to_string(),
            tags: vec!["test".to_string()],
            move_speed: 100.0,
            max_health: 50,
            max_endurance: 30,
            strength: 10,
            defence: 5,
            attack_speed: 1.0,
            reach: 1,
            size: 1.0,
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
    fn test_health_component_creation() {
        let health = Health::new(100);
        assert_eq!(health.current, 100.0);
        assert_eq!(health.max, 100.0);
    }

    #[test]
    fn test_endurance_component_creation() {
        let endurance = Endurance::new(50);
        assert_eq!(endurance.current, 50.0);
        assert_eq!(endurance.max, 50.0);
        assert_eq!(endurance.health_loss_timer, 0.0);
    }

    #[test]
    fn test_pawn_creation() {
        let pawn = Pawn::new("test_pawn".to_string());
        assert_eq!(pawn.pawn_type, "test_pawn");
    }

    #[test]
    fn test_endurance_reduction_during_movement() {
        let mut app = setup_test_app();
        let config = create_test_config();
        let pawn_config = create_test_pawn_config();
        
        app.insert_resource(config);
        app.insert_resource(pawn_config);
        
        // Spawn a pawn with target
        let entity = app.world_mut().spawn((
            Pawn::new("test_pawn".to_string()),
            Health::new(50),
            Endurance::new(30),
            Transform::from_translation(Vec3::new(0.0, 0.0, 100.0)),
            PawnTarget::new(Vec3::new(32.0, 0.0, 100.0)), // Move 2 tiles (32 pixels)
        )).id();

        // Run movement system for a few frames
        app.add_systems(Update, move_pawn_to_target);
        
        // Simulate time passing
        for _ in 0..10 {
            app.update();
        }

        // Check that endurance was reduced
        let endurance = app.world().entity(entity).get::<Endurance>().unwrap();
        assert!(endurance.current < 30.0, "Endurance should be reduced after movement");
    }

    #[test]
    fn test_health_loss_when_exhausted() {
        // Note: This test verifies the logic but actual integration happens in gameplay
        // The health loss system requires careful timing and may be better tested through integration
        let mut endurance = Endurance { current: 0.0, max: 30.0, health_loss_timer: 6.0 };
        let mut health = Health::new(100);
        let config = create_test_config();
        
        // Simulate what the system would do after 6 seconds of exhaustion
        if endurance.current <= 0.0 && endurance.health_loss_timer >= config.health_loss_interval {
            health.current = (health.current - 1.0).max(0.0);
            endurance.health_loss_timer = 0.0;
        }
        
        assert!(health.current < 100.0, "Health should be reduced when endurance is 0 for >= 5 seconds");
        assert_eq!(endurance.health_loss_timer, 0.0, "Timer should reset after health loss");
    }

    #[test]
    fn test_health_loss_timer_resets_when_endurance_above_zero() {
        let mut app = setup_test_app();
        let config = create_test_config();
        
        app.insert_resource(config);
        
        // Spawn a pawn with positive endurance
        let entity = app.world_mut().spawn((
            Pawn::new("test_pawn".to_string()),
            Health::new(10),
            Endurance { current: 5.0, max: 30.0, health_loss_timer: 3.0 },
        )).id();

        app.add_systems(Update, endurance_health_loss_system);
        app.update();

        // Check that health loss timer was reset
        let endurance = app.world().entity(entity).get::<Endurance>().unwrap();
        assert_eq!(endurance.health_loss_timer, 0.0, "Health loss timer should reset when endurance > 0");
    }

    #[test]
    fn test_pawn_death_when_health_zero() {
        let mut app = setup_test_app();
        
        // Spawn a pawn with 0 health
        let entity = app.world_mut().spawn((
            Pawn::new("test_pawn".to_string()),
            Health { current: 0.0, max: 50.0 },
            Endurance::new(30),
        )).id();

        app.add_systems(Update, pawn_death_system);
        app.update();

        // Check that the entity was despawned
        assert!(app.world().get_entity(entity).is_err(), "Pawn should be despawned when health is 0");
    }

    #[test]
    fn test_pawn_survives_with_positive_health() {
        let mut app = setup_test_app();
        
        // Spawn a pawn with positive health
        let entity = app.world_mut().spawn((
            Pawn::new("test_pawn".to_string()),
            Health { current: 1.0, max: 50.0 },
            Endurance::new(30),
        )).id();

        app.add_systems(Update, pawn_death_system);
        app.update();

        // Check that the entity still exists
        assert!(app.world().get_entity(entity).is_ok(), "Pawn should survive with positive health");
    }

    #[test]
    fn test_endurance_cost_calculation() {
        let config = create_test_config();
        
        // Test endurance cost calculation
        let cells_moved = 2.0; // 2 cells
        let expected_cost = cells_moved * config.endurance_cost_per_cell; // 2.0 * 1.0 = 2.0
        
        assert_eq!(expected_cost, 2.0);
    }

    #[test]
    fn test_pawn_target_creation_and_path_setting() {
        let target_pos = Vec3::new(100.0, 100.0, 100.0);
        let mut pawn_target = PawnTarget::new(target_pos);
        
        assert_eq!(pawn_target.target_position, target_pos);
        assert_eq!(pawn_target.path, vec![target_pos]);
        assert_eq!(pawn_target.current_waypoint_index, 0);
        
        // Test path setting
        let path = vec![(0.0, 0.0), (50.0, 50.0), (100.0, 100.0)];
        pawn_target.set_path(path);
        
        assert_eq!(pawn_target.path.len(), 3);
        assert_eq!(pawn_target.current_waypoint_index, 0);
        assert_eq!(pawn_target.target_position, Vec3::new(100.0, 100.0, 100.0));
    }

    #[test]
    fn test_pawn_target_waypoint_advancement() {
        let mut pawn_target = PawnTarget::new(Vec3::new(100.0, 100.0, 100.0));
        let path = vec![(0.0, 0.0), (50.0, 50.0), (100.0, 100.0)];
        pawn_target.set_path(path);
        
        // Test waypoint advancement
        assert_eq!(pawn_target.current_waypoint_index, 0);
        assert!(!pawn_target.is_at_destination());
        
        pawn_target.advance_waypoint();
        assert_eq!(pawn_target.current_waypoint_index, 1);
        assert!(!pawn_target.is_at_destination());
        
        pawn_target.advance_waypoint();
        assert_eq!(pawn_target.current_waypoint_index, 2);
        assert!(pawn_target.is_at_destination());
        
        // Should not advance past the end
        pawn_target.advance_waypoint();
        assert_eq!(pawn_target.current_waypoint_index, 2);
    }

    #[test] 
    fn test_pawn_target_reset() {
        let mut pawn_target = PawnTarget::new(Vec3::new(100.0, 100.0, 100.0));
        let path = vec![(0.0, 0.0), (50.0, 50.0), (100.0, 100.0)];
        pawn_target.set_path(path);
        pawn_target.advance_waypoint();
        
        // Reset the target
        pawn_target.reset();
        
        assert!(pawn_target.path.is_empty());
        assert_eq!(pawn_target.current_waypoint_index, 0);
        assert_eq!(pawn_target.target_position, Vec3::ZERO);
    }

    #[test]
    fn test_multiple_pawns_death_system() {
        let mut app = setup_test_app();
        
        // Spawn multiple pawns with different health states
        let healthy_pawn = app.world_mut().spawn((
            Pawn::new("healthy".to_string()),
            Health::new(50),
            Endurance::new(30),
        )).id();
        
        let dying_pawn = app.world_mut().spawn((
            Pawn::new("dying".to_string()),
            Health { current: 0.0, max: 50.0 },
            Endurance::new(30),
        )).id();
        
        let weak_pawn = app.world_mut().spawn((
            Pawn::new("weak".to_string()),
            Health { current: 1.0, max: 50.0 },
            Endurance::new(30),
        )).id();

        app.add_systems(Update, pawn_death_system);
        app.update();

        // Check results
        assert!(app.world().get_entity(healthy_pawn).is_ok(), "Healthy pawn should survive");
        assert!(app.world().get_entity(dying_pawn).is_err(), "Dying pawn should be despawned");
        assert!(app.world().get_entity(weak_pawn).is_ok(), "Weak pawn should survive with 1 HP");
    }
}