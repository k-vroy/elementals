use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::{Material2d, Material2dPlugin, MeshMaterial2d};
use crate::systems::world_gen::{TerrainMap, TerrainType};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct WaterMaterial {
    #[uniform(0)]
    pub time: f32,
}

impl Material2d for WaterMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/water.wgsl".into()
    }
}

pub struct WaterShaderPlugin;

impl Plugin for WaterShaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<WaterMaterial>::default())
            .add_systems(Update, update_water_time)
            .add_systems(Startup, spawn_water_overlays.after(crate::systems::world_gen::generate_world));
    }
}

fn update_water_time(
    time: Res<Time>,
    mut materials: ResMut<Assets<WaterMaterial>>,
) {
    for (_, material) in materials.iter_mut() {
        material.time = time.elapsed_secs();
    }
}

#[derive(Component)]
pub struct WaterTile;

pub fn spawn_water_overlays(
    mut commands: Commands,
    terrain_map: Res<TerrainMap>,
    mut materials: ResMut<Assets<WaterMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let water_material = materials.add(WaterMaterial { time: 0.0 });
    let quad_mesh = meshes.add(Rectangle::new(terrain_map.tile_size, terrain_map.tile_size));

    for x in 0..terrain_map.width {
        for y in 0..terrain_map.height {
            if matches!(terrain_map.tiles[x as usize][y as usize], TerrainType::Water) {
                let (world_x, world_y) = terrain_map.tile_to_world_coords(x as i32, y as i32);
                
                commands.spawn((
                    Mesh2d::from(quad_mesh.clone()),
                    MeshMaterial2d(water_material.clone()),
                    Transform::from_translation(Vec3::new(world_x, world_y, 0.5)),
                    WaterTile,
                ));
            }
        }
    }
}