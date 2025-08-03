use bevy::prelude::*;
use crate::systems::pawn::{Pawn, spawn_pawn};
use crate::systems::world_gen::TerrainMap;

pub fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    terrain_map: Res<TerrainMap>,
) {
    let player_pawn = Pawn::new_player();
    spawn_pawn(&mut commands, &asset_server, &terrain_map, player_pawn, None);
}

pub fn spawn_wolves(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    terrain_map: Res<TerrainMap>,
) {
    // Spawn a few wolves at random locations
    for _ in 0..3 {
        let wolf_pawn = Pawn::new_wolf();
        spawn_pawn(&mut commands, &asset_server, &terrain_map, wolf_pawn, None);
    }
}

pub fn spawn_rabbits(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    terrain_map: Res<TerrainMap>,
) {
    // Spawn several rabbits at random locations
    for _ in 0..5 {
        let rabbit_pawn = Pawn::new_rabbit();
        spawn_pawn(&mut commands, &asset_server, &terrain_map, rabbit_pawn, None);
    }
}