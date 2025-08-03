#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var<uniform> time: f32;

fn wave_noise(world_pos: vec2<f32>, time: f32) -> f32 {
    // Scale down the world position to create appropriate wave frequency
    let scaled_pos = world_pos * 0.01;
    
    let wave1 = sin(scaled_pos.x * 8.0 + time * 2.0) * 0.5 + 0.5;
    let wave2 = sin(scaled_pos.y * 6.0 + time * 1.5) * 0.5 + 0.5;
    let wave3 = sin((scaled_pos.x + scaled_pos.y) * 4.0 + time * 3.0) * 0.5 + 0.5;
    return (wave1 + wave2 + wave3) / 3.0;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let base_color = vec3<f32>(0.1, 0.3, 0.8); // Base water blue
    let highlight_color = vec3<f32>(0.3, 0.6, 1.0); // Lighter blue for waves
    
    // Use world position instead of UV coordinates for seamless tiling
    let world_pos = mesh.world_position.xy;
    let wave_intensity = wave_noise(world_pos, time);
    
    // Mix base color with wave highlights
    let final_color = mix(base_color, highlight_color, wave_intensity * 0.3);
    
    // Add some transparency for water effect
    let alpha = 0.8 + wave_intensity * 0.2;
    
    return vec4<f32>(final_color, alpha);
}