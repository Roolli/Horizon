struct TextureRendererVertexOutputs
{
    [[builtin(position)]] pos:vec4<f32>;
    [[location(0)]] out_coords: vec2<f32>;
};


[[group(0),binding(0)]]
var t_texture: texture_2d<f32>;
[[group(0),binding(1)]]
var t_sampler: sampler;

[[stage(vertex)]]
fn vs_main([[location(0)]] a_coord: vec2<f32>) -> TextureRendererVertexOutputs {
    var output: TextureRendererVertexOutputs;
    output.pos = vec4<f32>(a_coord,0.0,1.0);
    output.out_coords = vec2<f32>((a_coord.x+1.0) / 2.0, 1.0-(a_coord.y+1.0)/2.0);
    return output;
} 

[[stage(fragment)]]
fn fs_main(input: TextureRendererVertexOutputs) -> [[location(0)]] vec4<f32>
{
    return textureSample(t_texture,t_sampler,input.out_coords);
}