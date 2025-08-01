use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct TerrainLayer {
    pub layer_id: u32,
    pub z_index: f32,
}