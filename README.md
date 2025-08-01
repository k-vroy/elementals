# Elementals RPG

A 2D top-down RPG built with Rust and Bevy engine, featuring procedural world generation and multi-layer tilemaps.

## Quick Start

```bash
# Normal build and run
cargo run

# Fast development mode (recommended for testing)
./dev-run.sh
```

## Fast Development Workflow

For rapid iteration during development, use these optimized commands:

### Option 1: Auto-rebuild on Changes (Recommended)
```bash
./dev-run.sh
```
This watches for file changes and automatically rebuilds/runs the game.

### Option 2: Manual Fast Builds
```bash
# Fast compilation (minimal optimizations)
cargo run --profile dev-fast

# Regular dev build (balanced)
cargo run
```

## Development Features

- **Dynamic Linking**: Faster incremental builds
- **Optimized Profiles**: Multiple build profiles for different needs
- **Auto-rebuild**: Watches `src/`, `assets/`, and `settings.yaml` for changes
- **Fast Linker**: Uses lld for faster linking

## Game Features

- **Procedural World Generation**: 64x64 tile maps with multiple terrain types
- **Multi-layer Rendering**: Ground, objects, and decoration layers
- **Smooth Camera Controls**: 
  - WASD/Arrow keys for movement
  - Middle mouse drag for panning
  - Mouse wheel zoom (towards cursor)
- **Configurable Settings**: Adjust game parameters via `settings.yaml`
- **FPS Counter**: Toggle-able performance monitoring

## Configuration

Edit `settings.yaml` to customize:
- World size and tile dimensions
- Camera movement speed and zoom limits
- Mouse sensitivity
- FPS counter display

## Controls

- **WASD/Arrow Keys**: Move camera
- **Middle Mouse + Drag**: Pan camera
- **Mouse Wheel**: Zoom in/out (towards cursor)
- **+/-**: Keyboard zoom (centered)