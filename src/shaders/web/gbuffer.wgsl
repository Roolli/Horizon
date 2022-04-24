
struct GBufferInputs {
@location(0) a_pos: vec3<f32>,
@location(1) a_normal: vec3<f32>,
@location(2) tangent: vec4<f32>,
@location(3) tex_coord: vec2<f32>,
@location(4) vertex_color: u32,
@location(5) joint_weight: vec4<f32>,
@location(6) joint_id: u32,
@builtin(instance_index) instance_index: u32,
};

struct VertexOutputs {
    @builtin(position) pos: vec4<f32>,
    @location(0) v_tex_coord: vec2<f32>,
    @location(1) world_frag_pos: vec3<f32>,
    @location(2) tangent: vec4<f32>,
    @location(3) normal: vec3<f32>,
};


struct GBufferOutputs {
    @location(0) position: vec4<f32>,
    @location(1) normal: vec4<f32>,
    @location(2) specular:vec4<f32>,
    @location(3) albedo: vec4<f32>,
};
struct Globals {
    u_view_position: vec4<f32>,
    u_view_proj: mat4x4<f32>,
    lights_num: vec4<u32>,
};

struct MaterialUniforms {
    base_color_factor: vec4<f32>,
    roughness_metallic_double_sided: vec4<f32>,
    emissive_color: vec4<f32>,
};

@group(0)
@binding(0)
var<uniform> globals: Globals;

struct Transforms {
 elements: array<mat4x4<f32>>,
}; 
struct Normals {
    elements: array<mat4x4<f32> >,
};

@group(0)
@binding(1)
var<storage,read> transform: Transforms;

@group(0)
@binding(2)
var<storage,read> normals: Normals;

@stage(vertex)
fn vs_main(in: GBufferInputs) -> VertexOutputs {
     var output: VertexOutputs;
    output.v_tex_coord = in.tex_coord; 
    var model_matrix: mat4x4<f32> = transform.elements[in.instance_index];
    var normal: mat4x4<f32> = normals.elements[in.instance_index];
    let frag_tangent = normalize(normal * in.tangent);
    output.normal = normalize(vec3<f32>((normal * vec4<f32>(in.a_normal,0.0)).xyz));
    output.tangent = frag_tangent;   
    var model_space: vec4<f32>  = model_matrix * vec4<f32>(in.a_pos,1.0);
    output.world_frag_pos = model_space.xyz;    
    output.pos= globals.u_view_proj* model_space;
    return output;
}




@group(1)
@binding(0)
var t_texture: texture_2d<f32>;
@group(1)
@binding(1)
var t_sampler: sampler;
@group(1)
@binding(2)
var t_roughness: texture_2d<f32>;
@group(1)
@binding(3)
var t_normal: texture_2d<f32>;
@group(1)
@binding(4)
var t_occlusion: texture_2d<f32>;
@group(1)
@binding(5)
var t_emissive: texture_2d<f32>;
@group(1)
@binding(6)
var<uniform> material_uniforms: MaterialUniforms;

@stage(fragment)
fn fs_main(in: VertexOutputs, @builtin(front_facing) front_facing: bool) -> GBufferOutputs {
    var out: GBufferOutputs;
    var albedo = vec4<f32>(material_uniforms.base_color_factor.xyz,1.0);
    var texture_color = textureSample(t_texture,t_sampler,in.v_tex_coord);
    
    let occulison = textureSample(t_occlusion,t_sampler,in.v_tex_coord).r;
    albedo = vec4<f32>(material_uniforms.base_color_factor.xyz * texture_color.xyz,1.0);
    out.albedo = albedo;
    out.position = vec4<f32>(in.world_frag_pos,textureSample(t_roughness,t_sampler,in.v_tex_coord).b * material_uniforms.roughness_metallic_double_sided.y);
   
      var normal_map:vec3<f32> = textureSample(t_normal,t_sampler,in.v_tex_coord).xyz;
    normal_map = 2.0 * normal_map  -vec3<f32>(1.0,1.0,1.0);
     var frag_normal: vec3<f32>;
    if(!front_facing)
    {
        frag_normal= normalize(in.normal * material_uniforms.roughness_metallic_double_sided.z);
    }    
    else {
        frag_normal= in.normal;
    }
    var tangent:  vec3<f32> = in.tangent.xyz;   
    var bitangent: vec3<f32> = cross(tangent,frag_normal); // * in.tangent.w;
    //tangent = normalize(tangent - dot(tangent,frag_normal) * frag_normal);
    normal_map= normalize(mat3x3<f32>(tangent,bitangent,frag_normal) * normal_map);
    out.specular = vec4<f32>((textureSample(t_emissive,t_sampler,in.v_tex_coord).xyz*material_uniforms.emissive_color.xyz) ,1.0);
    out.normal = vec4<f32>(normal_map,textureSample(t_roughness,t_sampler,in.v_tex_coord).r * material_uniforms.roughness_metallic_double_sided.x);
    return out;
}