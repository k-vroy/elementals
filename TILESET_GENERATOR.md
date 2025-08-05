# Tileset Generator Tool

A standalone Rust binary that crawls through `assets/tilesets/` subdirectories and generates corresponding tilesets by merging all sprites.

## Features

- **Automatic Directory Crawling**: Scans all subdirectories in `assets/tilesets/`
- **Sprite Merging**: Combines all sprites in each directory into a single tileset image
- **YAML Index Generation**: Creates index files listing all sprites and their positions
- **Configurable Tile Size**: Default 16x16 pixels, 16 tiles per row
- **Multiple Format Support**: PNG, JPG, JPEG, BMP, TGA

## Usage

### Build and run the tool:

```bash
# Run with default assets path
cargo run --bin tileset_generator --features tileset-generator

# Run with custom assets path
cargo run --bin tileset_generator --features tileset-generator -- /path/to/assets
```

### Directory Structure

```
assets/
├── tilesets/
│   ├── terrain/
│   │   ├── grass.png
│   │   ├── stone.png
│   │   └── water.png
│   ├── characters/
│   │   ├── player.png
│   │   ├── enemy1.png
│   │   └── enemy2.png
│   └── items/
│       ├── sword.png
│       └── shield.png
└── (generated files)
    ├── terrain.png
    ├── terrain.yaml
    ├── characters.png
    ├── characters.yaml
    ├── items.png
    └── items.yaml
```

## Generated Files

### Tileset Images (`.png`)
- Combined sprite sheet with all sprites from a directory
- Sprites arranged in a grid (16 tiles per row by default)
- Each sprite resized to 16x16 pixels using nearest-neighbor filtering
- Transparent background

### Index Files (`.yaml`)
Example `terrain.yaml`:
```yaml
tileset_name: terrain
tile_size: 16
tiles_per_row: 16
total_tiles: 3
sprites:
- name: grass
  index: 0
  x: 0
  y: 0
  width: 16
  height: 16
- name: stone
  index: 1
  x: 16
  y: 0
  width: 16
  height: 16
- name: water
  index: 2
  x: 32
  y: 0
  width: 16
  height: 16
```

## Configuration

The tool uses these default settings:
- **Tile Size**: 16x16 pixels
- **Tiles Per Row**: 16
- **Output Format**: PNG with RGBA support
- **Resize Filter**: Nearest-neighbor (preserves pixel art)

## Notes

- Sprites are sorted alphabetically for consistent ordering
- All images are resized to the tile size (16x16 by default)
- Transparent backgrounds are preserved
- The tool is completely standalone and doesn't affect the main game code
- Only builds when the `tileset-generator` feature is enabled