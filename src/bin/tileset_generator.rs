#!/usr/bin/env cargo
//! Standalone Tileset Generator Tool
//! 
//! This tool crawls through assets/tilesets/ subdirectories and generates 
//! corresponding tilesets by merging all sprites. For each tileset, it creates
//! a YAML file listing all sprites and their indices.
//! 
//! Usage: cargo run --bin tileset_generator [assets_path]

use std::fs;
use std::path::Path;
use std::env;
use walkdir::WalkDir;
use image::{ImageBuffer, Rgba, RgbaImage};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SpriteInfo {
    name: String,
    index: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TilesetIndex {
    tileset_name: String,
    tile_size: u32,
    tiles_per_row: u32,
    total_tiles: u32,
    sprites: Vec<SpriteInfo>,
}

struct TilesetGenerator {
    tile_size: u32,
    tiles_per_row: u32,
}

impl TilesetGenerator {
    fn new(tile_size: u32, tiles_per_row: u32) -> Self {
        Self {
            tile_size,
            tiles_per_row,
        }
    }

    fn generate_tilesets(&self, assets_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let tilesets_path = Path::new(assets_path).join("tilesets");
        
        if !tilesets_path.exists() {
            fs::create_dir_all(&tilesets_path)?;
            println!("Created tilesets directory: {:?}", tilesets_path);
            return Ok(());
        }

        // Find all subdirectories in assets/tilesets/
        for entry in fs::read_dir(&tilesets_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    println!("Processing tileset directory: {}", dir_name);
                    self.process_tileset_directory(&path, dir_name, &tilesets_path)?;
                }
            }
        }

        Ok(())
    }

    fn process_tileset_directory(&self, dir_path: &Path, tileset_name: &str, tilesets_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        // Collect all image files in the directory and subdirectories
        let mut sprites = Vec::new();
        
        for entry in WalkDir::new(dir_path) {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
                    match extension.to_lowercase().as_str() {
                        "png" | "jpg" | "jpeg" | "bmp" | "tga" => {
                            sprites.push(path.to_path_buf());
                        }
                        _ => {}
                    }
                }
            }
        }

        if sprites.is_empty() {
            println!("No sprites found in directory: {}", tileset_name);
            return Ok(());
        }

        // Sort sprites by name for consistent ordering
        sprites.sort();

        // Load and process sprites
        let mut sprite_images = Vec::new();
        let mut sprite_infos = Vec::new();

        for (index, sprite_path) in sprites.iter().enumerate() {
            match self.load_and_resize_sprite(sprite_path) {
                Ok(sprite_img) => {
                    let sprite_name = sprite_path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let row = (index as u32) / self.tiles_per_row;
                    let col = (index as u32) % self.tiles_per_row;

                    let sprite_info = SpriteInfo {
                        name: sprite_name,
                        index: index as u32,
                        x: col * self.tile_size,
                        y: row * self.tile_size,
                        width: self.tile_size,
                        height: self.tile_size,
                    };

                    sprite_images.push(sprite_img);
                    sprite_infos.push(sprite_info);
                }
                Err(e) => {
                    println!("Failed to load sprite {:?}: {}", sprite_path, e);
                }
            }
        }

        if sprite_images.is_empty() {
            println!("No valid sprites loaded for tileset: {}", tileset_name);
            return Ok(());
        }

        // Create tileset image
        let tileset_image = self.create_tileset_image(&sprite_images)?;
        
        // Save tileset
        let tileset_filename = format!("{}.png", tileset_name);
        let tileset_path = tilesets_path.join(&tileset_filename);
        tileset_image.save(&tileset_path)?;
        println!("Generated tileset: {:?}", tileset_path);

        // Create and save index YAML
        let tileset_index = TilesetIndex {
            tileset_name: tileset_name.to_string(),
            tile_size: self.tile_size,
            tiles_per_row: self.tiles_per_row,
            total_tiles: sprite_infos.len() as u32,
            sprites: sprite_infos,
        };

        let yaml_filename = format!("{}.yaml", tileset_name);
        let yaml_path = tilesets_path.join(&yaml_filename);
        let yaml_content = serde_yaml::to_string(&tileset_index)?;
        fs::write(&yaml_path, yaml_content)?;
        println!("Generated index: {:?}", yaml_path);

        Ok(())
    }

    fn load_and_resize_sprite(&self, sprite_path: &Path) -> Result<RgbaImage, Box<dyn std::error::Error>> {
        let img = image::open(sprite_path)?;
        let resized = img.resize_exact(
            self.tile_size,
            self.tile_size,
            image::imageops::FilterType::Nearest
        );
        Ok(resized.to_rgba8())
    }

    fn create_tileset_image(&self, sprites: &[RgbaImage]) -> Result<RgbaImage, Box<dyn std::error::Error>> {
        let sprite_count = sprites.len() as u32;
        let rows = (sprite_count + self.tiles_per_row - 1) / self.tiles_per_row;
        let tileset_width = self.tiles_per_row * self.tile_size;
        let tileset_height = rows * self.tile_size;

        let mut tileset = ImageBuffer::new(tileset_width, tileset_height);

        // Fill with transparent pixels
        for pixel in tileset.pixels_mut() {
            *pixel = Rgba([0, 0, 0, 0]);
        }

        // Place sprites in the tileset
        for (index, sprite) in sprites.iter().enumerate() {
            let row = (index as u32) / self.tiles_per_row;
            let col = (index as u32) % self.tiles_per_row;
            let x_offset = col * self.tile_size;
            let y_offset = row * self.tile_size;

            for y in 0..self.tile_size {
                for x in 0..self.tile_size {
                    if x < sprite.width() && y < sprite.height() {
                        let sprite_pixel = sprite.get_pixel(x, y);
                        tileset.put_pixel(x_offset + x, y_offset + y, *sprite_pixel);
                    }
                }
            }
        }

        Ok(tileset)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Standalone Tileset Generator");
    println!("============================");
    
    // Get assets path from command line or use default
    let args: Vec<String> = env::args().collect();
    let assets_path = if args.len() > 1 {
        &args[1]
    } else {
        "assets"
    };

    // Check if assets directory exists
    if !Path::new(assets_path).exists() {
        eprintln!("Assets directory not found: {}", assets_path);
        std::process::exit(1);
    }

    // Generate tilesets
    let generator = TilesetGenerator::new(16, 16); // 16x16 tiles, 16 tiles per row
    generator.generate_tilesets(assets_path)?;
    
    println!("\nTileset generation completed!");
    Ok(())
}