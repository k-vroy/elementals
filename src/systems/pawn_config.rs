use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

pub type PawnType = String;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BehaviourType {
    Null,
    Flee,
    HuntSolo,
    PlayerInput,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WanderingConfig {
    pub move_interval_min: f32,
    pub move_interval_max: f32,
    pub move_range: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum BehaviourConfig {
    Simple(BehaviourType),
    Wandering { wandering: WanderingConfig },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PawnBehaviours {
    pub idle: Option<BehaviourConfig>,
    pub hunted: Option<BehaviourConfig>,
    pub looking_for_food: Option<BehaviourConfig>,
    pub eat: Option<BehaviourConfig>,
    pub controlled: Option<BehaviourConfig>,
    pub flee: Option<BehaviourConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PawnEats {
    pub pawns: Vec<PawnType>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PawnDefinition {
    pub sprite: String,
    pub tags: Vec<String>,
    pub move_speed: f32,
    pub max_health: u32,
    pub max_endurance: u32,
    pub strength: u32,
    pub defence: u32,
    pub attack_speed: f32,
    pub reach: u32,
    pub spawn_count: u32,
    pub behaviours: PawnBehaviours,
    pub eats: PawnEats,
}

#[derive(Debug, Clone, Resource, Deserialize, Serialize)]
pub struct PawnConfig {
    #[serde(flatten)]
    pub pawns: HashMap<PawnType, PawnDefinition>,
}

impl PawnConfig {
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: PawnConfig = serde_yaml::from_str(&contents)?;
        Ok(config)
    }

    pub fn get_pawn_definition(&self, pawn_type: &str) -> Option<&PawnDefinition> {
        self.pawns.get(pawn_type)
    }

    pub fn get_pawn_types(&self) -> Vec<String> {
        self.pawns.keys().cloned().collect()
    }

    pub fn get_behaviour_config(&self, pawn_type: &str, state: &str) -> Option<&BehaviourConfig> {
        let def = self.get_pawn_definition(pawn_type)?;
        match state {
            "idle" => def.behaviours.idle.as_ref(),
            "hunted" => def.behaviours.hunted.as_ref(),
            "looking_for_food" => def.behaviours.looking_for_food.as_ref(),
            "eat" => def.behaviours.eat.as_ref(),
            "controlled" => def.behaviours.controlled.as_ref(),
            "flee" => def.behaviours.flee.as_ref(),
            _ => None,
        }
    }

    pub fn get_wandering_config(&self, pawn_type: &str, state: &str) -> Option<&WanderingConfig> {
        if let Some(BehaviourConfig::Wandering { wandering }) = self.get_behaviour_config(pawn_type, state) {
            Some(wandering)
        } else {
            None
        }
    }

    pub fn can_eat(&self, predator: &PawnType, prey: &PawnType) -> bool {
        if let Some(def) = self.get_pawn_definition(predator) {
            def.eats.pawns.contains(prey)
        } else {
            false
        }
    }

    pub fn can_eat_by_tags(&self, predator_type: &PawnType, prey_type: &PawnType) -> bool {
        let predator_def = match self.get_pawn_definition(predator_type) {
            Some(def) => def,
            None => return false,
        };
        
        let prey_def = match self.get_pawn_definition(prey_type) {
            Some(def) => def,
            None => return false,
        };

        // Check if prey has all the tags that predator eats
        // If predator eats nothing, it can't eat anything
        if predator_def.eats.pawns.is_empty() {
            return false;
        }
        
        predator_def.eats.pawns.iter().all(|required_tag| {
            prey_def.tags.contains(required_tag)
        })
    }
}