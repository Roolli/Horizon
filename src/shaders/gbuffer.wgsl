
struct GBufferInputs{
[[builtin(instance_index)]] instance_index:u32;  
[[location(0)]] a_pos: vec3<f32>;
[[location(1)]] tex_coord: vec2<f32>;
[[location(2)]] a_normal: vec3<f32>;
[[location(3)]] tangent: vec3<f32>;
[[location(4)]] bitangent: vec3<f32>;
};

struct VertexOutputs {
    [[builtin(position)]] pos:vec4<f32>;
    [[location(0)]] v_tex_coord: vec2<f32>;
    [[location(1)]] worldFragNormal: vec3<f32>;
    [[location(2)]] worldFragPos: vec3<f32>;
};


[[block]]
struct Globals {
    u_view_position: vec4<f32>;
    u_view_proj: mat4x4<f32>;
    lights_num: vec4<u32>;
};

[[group(1),binding(0)]]
var<uniform> globals: Globals;

[[block]] struct Transforms {
 elements: array<mat4x4<f32>>;
}; 

[[block]] struct Normals {
    elements:  array<mat4x4<f32>>;
};

[[group(1),binding(1)]]
var<storage,read> transform: Transforms;

[[group(1),binding(2)]]
var<storage,read> normals: Normals;

[[stage(vertex)]]
fn vs_main(in: GBufferInputs) -> VertexOutputs
{
    var output: VertexOutputs;
    output.v_tex_coord = in.tex_coord; 
    var  model_matrix: mat4x4<f32> = transform.elements[in.instance_index];
    var normal: mat4x4<f32> = normals.elements[in.instance_index];
    var normal_matrix: mat3x3<f32> = mat3x3<f32>(normal[0].xyz,normal[1].xyz,normal[2].xyz);    
    output.worldFragNormal  = normalize(normal_matrix*in.a_normal);
    var model_space: vec4<f32>  = model_matrix * vec4<f32>(in.a_pos,1.0);
    output.worldFragPos = model_space.xyz;    
    output.pos= globals.u_view_proj* model_space;
    return output;
}


struct GBufferOutputs {
    [[location(0)]] position: vec4<f32>;
    [[location(1)]] normal: vec4<f32>;
    [[location(2)]] albedo: vec4<f32>;
};

[[group(0),binding(0)]]
var t_texture: texture_2d<f32>;

[[group(0),binding(1)]]
var t_sampler: sampler;

[[group(0),binding(2)]]
var t_normal: texture_2d<f32>;
[[group(0),binding(3)]]
var normal_sampler: sampler;

[[stage(fragment)]]
fn fs_main(in:VertexOutputs) -> GBufferOutputs
{
    var out: GBufferOutputs;
    out.albedo = textureSample(t_texture,t_sampler,in.v_tex_coord);
    out.position = vec4<f32>(in.worldFragPos,1.0);
    out.normal = vec4<f32>(in.worldFragNormal * textureSample(t_normal,normal_sampler,in.v_tex_coord).xyz,1.0);

    return out;
}