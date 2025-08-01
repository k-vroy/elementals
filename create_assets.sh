#!/bin/bash

# Script to create basic placeholder tilesets using ImageMagick
# Run with: chmod +x create_assets.sh && ./create_assets.sh

ASSETS_DIR="assets"
TILE_SIZE=16

# Check if ImageMagick is installed
if ! command -v convert &> /dev/null; then
    echo "ImageMagick is not installed. Please install it with:"
    echo "sudo apt install imagemagick"
    exit 1
fi

mkdir -p "$ASSETS_DIR"

echo "Creating basic tilesets..."

# Ground tileset (4x1 layout: grass, dirt, stone, water)
convert -size $((TILE_SIZE*4))x$TILE_SIZE xc:none \
    \( -size ${TILE_SIZE}x${TILE_SIZE} xc:"#4CAF50" \) -geometry +0+0 -composite \
    \( -size ${TILE_SIZE}x${TILE_SIZE} xc:"#8D6E63" \) -geometry +${TILE_SIZE}+0 -composite \
    \( -size ${TILE_SIZE}x${TILE_SIZE} xc:"#607D8B" \) -geometry +$((TILE_SIZE*2))+0 -composite \
    \( -size ${TILE_SIZE}x${TILE_SIZE} xc:"#2196F3" \) -geometry +$((TILE_SIZE*3))+0 -composite \
    "$ASSETS_DIR/ground_tileset.png"

# Objects tileset (4x1 layout: tree, rock, wall, chest)
convert -size $((TILE_SIZE*4))x$TILE_SIZE xc:none \
    \( -size ${TILE_SIZE}x${TILE_SIZE} xc:"#2E7D32" \) -geometry +0+0 -composite \
    \( -size ${TILE_SIZE}x${TILE_SIZE} xc:"#424242" \) -geometry +${TILE_SIZE}+0 -composite \
    \( -size ${TILE_SIZE}x${TILE_SIZE} xc:"#5D4037" \) -geometry +$((TILE_SIZE*2))+0 -composite \
    \( -size ${TILE_SIZE}x${TILE_SIZE} xc:"#FF9800" \) -geometry +$((TILE_SIZE*3))+0 -composite \
    "$ASSETS_DIR/objects_tileset.png"

# Decoration tileset (4x1 layout: flower, mushroom, small rock, bush)
convert -size $((TILE_SIZE*4))x$TILE_SIZE xc:none \
    \( -size ${TILE_SIZE}x${TILE_SIZE} xc:"#E91E63" \) -geometry +0+0 -composite \
    \( -size ${TILE_SIZE}x${TILE_SIZE} xc:"#9C27B0" \) -geometry +${TILE_SIZE}+0 -composite \
    \( -size ${TILE_SIZE}x${TILE_SIZE} xc:"#616161" \) -geometry +$((TILE_SIZE*2))+0 -composite \
    \( -size ${TILE_SIZE}x${TILE_SIZE} xc:"#689F38" \) -geometry +$((TILE_SIZE*3))+0 -composite \
    "$ASSETS_DIR/decoration_tileset.png"

# Basic tileset (same as ground for now)
cp "$ASSETS_DIR/ground_tileset.png" "$ASSETS_DIR/tileset.png"

# Player sprite (simple colored square)
convert -size ${TILE_SIZE}x${TILE_SIZE} xc:"#FF5722" "$ASSETS_DIR/player.png"

echo "Basic assets created in $ASSETS_DIR/"
echo "Ground tileset: grass(green), dirt(brown), stone(blue-gray), water(blue)"
echo "Objects tileset: tree(dark green), rock(gray), wall(brown), chest(orange)"
echo "Decoration tileset: flower(pink), mushroom(purple), small rock(gray), bush(light green)"
echo "Player sprite: orange square"