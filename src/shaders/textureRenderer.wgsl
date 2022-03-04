struct TextureRendererVertexOutputs {
    @builtin(position) pos: vec4<f32>;
    @location(0) out_coords: vec2<f32>;
};


@group(0) @binding(0)
var t_texture: texture_2d<f32>;
@group(0) @binding(1)
var t_sampler: sampler;

@stage(vertex)
fn vs_main(@location(0) a_coord: vec2<f32>) -> TextureRendererVertexOutputs {
    var output: TextureRendererVertexOutputs;
    output.pos = vec4<f32>(a_coord,0.0,1.0);
    output.out_coords = vec2<f32>((a_coord.x+1.0) / 2.0, 1.0-(a_coord.y+1.0)/2.0);
    return output;
} 

@stage(fragment)
fn fs_main(input: TextureRendererVertexOutputs) -> @location(0) vec4<f32> {
    return textureSample(t_texture,t_sampler,input.out_coords);
}


@stage(vertex)
fn depth_vs_main(@builtin(vertex_index) in: u32) -> @builtin(position) vec4<f32> {
    var pos: array<vec2<f32>,6> = array<vec2<f32>,6>( vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(-1.0, 1.0),
    vec2<f32>(-1.0, 1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0));
    
    return vec4<f32>(pos[in],0.0,1.0);
}
@group(0)
@binding(0)
var depth_texture: texture_depth_2d;

@stage(fragment)
fn depth_fs_main(@builtin(position) in: vec4<f32>) -> @location(0) vec4<f32> {
    let depthValue = textureLoad(depth_texture,vec2<i32>(floor(in.xy)),0);
    return vec4<f32>(vec3<f32>(depthValue),1.0);
}