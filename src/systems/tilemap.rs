use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use crate::components::TerrainLayer;
use crate::resources::GameConfig;

pub fn setup_tilemap(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture_handle: Handle<Image> = asset_server.load("tileset.png");
    let map_size = TilemapSize { x: 32, y: 32 };
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(map_size);

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: TileTextureIndex(0),
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size,
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
        ..Default::default()
    })
    .insert(TerrainLayer {
        layer_id: 0,
        z_index: 0.0,
    });
}

pub fn setup_multiple_terrain_layers(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: Res<GameConfig>,
) {
    let layers = vec![
        ("ground_tileset.png", 0, 0.0),
        ("objects_tileset.png", 1, 1.0),
        ("decoration_tileset.png", 2, 2.0),
    ];

    for (texture_path, layer_id, z_index) in layers {
        let texture_handle: Handle<Image> = asset_server.load(texture_path);
        let map_size = TilemapSize { 
            x: config.map_width, 
            y: config.map_height 
        };
        let tilemap_entity = commands.spawn_empty().id();
        let tile_storage = TileStorage::empty(map_size);

        let tile_size = TilemapTileSize { 
            x: config.tile_size, 
            y: config.tile_size 
        };
        let grid_size = tile_size.into();
        let map_type = TilemapType::default();

        commands.entity(tilemap_entity).insert(TilemapBundle {
            grid_size,
            map_type,
            size: map_size,
            storage: tile_storage,
            texture: TilemapTexture::Single(texture_handle),
            tile_size,
            transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, z_index),
            ..Default::default()
        })
        .insert(TerrainLayer {
            layer_id,
            z_index,
        });
    }
}