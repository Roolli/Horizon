
struct GBufferInputs{
[[location(0)]]  a_pos: vec3<f32>;
[[location(1)]] tex_coord: vec2<f32>;
[[location(2)]] a_normal: vec3<f32>;
[[location(3)]] tangent: vec3<f32>;
[[location(4)]] bitangent: vec3<f32>;
[[builtin(instance_index)]] instance_index:u32;  
};

struct GBufferOutputs {
    [[location(0)]] v_tex_coord: vec2<f32>;
    [[location(1)]] WorldFragNormal: vec3<f32>;
    [[location(2)]] WorldFragPos: vec3<f32>;
    [[builtin(position)]] pos:vec4<f32>;
};

[[block]]
struct Globals {
    u_view_positionu: vec4<f32>;
    u_view_proj: mat4x4<f32>;
    lights_num: vec4<u32>;
};

[[group(1),binding(0)]]
var<uniform> globals: Globals;

type Transforms = [[stride(64)]] array<mat4x4<f32>>;

[[group(1),binding(1)]]
var<storage,read> transform: Transforms;

[[group(1),binding(2)]]
var<storage,read> normals: array<mat4x4<f32>>;

[[stage(vertex)]]
fn main(in: GBufferInputs) -> GBufferOutputs
{
    var output: GBufferOutputs;
    output.v_tex_coord = in.tex_coord; 
    var  model_matrix: mat4x4<f32> = transform[in.instance_index];
    let normal: mat4x4<f32> = normals[in.instance_index];
    var normal_matrix: mat3x3<f32> = mat3x3<f32>(normal[0],normal[1],normal[2]);    
    // output.WorldFragNormal  = normalize(normal_matrix*a_normal);
    // var model_space: vec4<f32>  = model_matrix * vec4<f32>(a_pos,1.0);
    // output.WorldFragPos = model_space.xyz;    
    // output.pos= u_view_proj* model_space;
}