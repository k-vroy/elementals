use bevy::prelude::*;
use crate::systems::pawn::{Pawn, spawn_pawn};
use crate::systems::pawn_config::PawnConfig;
use crate::systems::world_gen::TerrainMap;

pub fn spawn_all_pawns(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    terrain_map: Res<TerrainMap>,
    pawn_config: Res<PawnConfig>,
) {
    // Loop through all pawn types defined in pawns.yaml
    for pawn_type in pawn_config.get_pawn_types() {
        if let Some(definition) = pawn_config.get_pawn_definition(&pawn_type) {
            // Spawn the specified number of each pawn type
            for _ in 0..definition.spawn_count {
                let pawn = Pawn::new(pawn_type.clone());
                spawn_pawn(&mut commands, &asset_server, &terrain_map, &pawn_config, pawn, None);
            }
        }
    }
}