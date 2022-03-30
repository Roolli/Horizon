struct Globals {
    u_view_position: vec4<f32>;
    u_view_proj: mat4x4<f32>;
    lights_num: vec4<u32>;
};
struct Transform {
 matrix: mat4x4<f32>;
}; 

[[group(0)
,binding(0)]]
var<uniform> globals: Globals;

[[group(1),binding(0)]]
var<uniform> transform:Transform;

[[stage(vertex)]]
fn vs_main([[location(0)]] pos:vec3<f32>,[[builtin(instance_index)]] instance_index:u32) -> [[builtin(position)]] vec4<f32>
{   
    return  globals.u_view_proj * transform.matrix * vec4<f32>(pos,1.0);
}
[[stage(fragment)]]
fn fs_main([[builtin(position)]] pos:vec4<f32>) -> [[location(0)]] vec4<f32>
{
    return vec4<f32>(1.0); // draw white stuff, (presumably)
} 