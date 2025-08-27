#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var<uniform> time: f32;

// Convert RGB to HSV
fn rgb_to_hsv(color: vec3<f32>) -> vec3<f32> {
    let max_val = max(max(color.r, color.g), color.b);
    let min_val = min(min(color.r, color.g), color.b);
    let delta = max_val - min_val;
    
    var hue: f32 = 0.0;
    let saturation = select(0.0, delta / max_val, max_val > 0.0);
    let value = max_val;
    
    if (delta > 0.0) {
        if (max_val == color.r) {
            hue = (color.g - color.b) / delta;
        } else if (max_val == color.g) {
            hue = 2.0 + (color.b - color.r) / delta;
        } else {
            hue = 4.0 + (color.r - color.g) / delta;
        }
        hue = hue / 6.0;
        if (hue < 0.0) {
            hue = hue + 1.0;
        }
    }
    
    return vec3<f32>(hue, saturation, value);
}

// Convert HSV to RGB
fn hsv_to_rgb(hsv: vec3<f32>) -> vec3<f32> {
    let hue = hsv.x * 6.0;
    let saturation = hsv.y;
    let value = hsv.z;
    
    let c = value * saturation;
    let x = c * (1.0 - abs((hue % 2.0) - 1.0));
    let m = value - c;
    
    var rgb: vec3<f32>;
    
    if (hue < 1.0) {
        rgb = vec3<f32>(c, x, 0.0);
    } else if (hue < 2.0) {
        rgb = vec3<f32>(x, c, 0.0);
    } else if (hue < 3.0) {
        rgb = vec3<f32>(0.0, c, x);
    } else if (hue < 4.0) {
        rgb = vec3<f32>(0.0, x, c);
    } else if (hue < 5.0) {
        rgb = vec3<f32>(x, 0.0, c);
    } else {
        rgb = vec3<f32>(c, 0.0, x);
    }
    
    return rgb + m;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    // Create varied hue rotation using multiple frequencies
    let world_pos = mesh.world_position.xy;
    
    // Base hue rotation with time (very subtle)
    let base_rotation = time * 0.1;
    
    // Add spatial variation based on world position (subtle)
    let spatial_variation = sin(world_pos.x * 0.01 + time * 0.4) * 0.05 + 
                           cos(world_pos.y * 0.01 + time * 0.3) * 0.04;
    
    // Add oscillating variation (subtle)
    let wave_variation = sin(time * 0.6) * 0.03 + cos(time * 0.35) * 0.02;
    
    // Combine all variations (much smaller range)
    let total_hue_shift = base_rotation + spatial_variation + wave_variation;
    
    // Create a subtle tint based on hue shift
    let hue_color = hsv_to_rgb(vec3<f32>(total_hue_shift, 0.3, 0.8));
    
    // Return a very subtle additive tint
    return vec4<f32>(hue_color * 0.2, 0.6);
}