#version 450
#extension GL_EXT_scalar_block_layout: require

layout(location=0) in vec3 a_pos;
layout(location=1) in vec2 tex_coord;
layout(location=2) in vec3 a_normal;
layout(location=3) in vec3 tangent;
layout(location=4) in vec3 bitangent;

layout (location=0) out vec2 v_tex_coord;
layout (location=1) out vec3 WorldFragNormal;
layout (location=2) out vec3 WorldFragPos;

layout(set=1,binding=0)
uniform Globals {
    vec4 u_view_position;
    mat4 u_view_proj;
    uvec4 lights_num;
};
layout(std430,set=1,binding=1)
buffer InstanceData
{
    mat4 transform[];
};
layout(std430, set=1, binding=2) 
buffer NormalMatricies
{
    mat4 normal_matricies[];
};

void main()
{
    v_tex_coord = tex_coord; 
    mat4 model_matrix= transform[gl_InstanceIndex];
    mat3 normal_matrix = mat3(normal_matricies[gl_InstanceIndex]);    
    WorldFragNormal = normalize(normal_matrix*a_normal);
    vec4 model_space = model_matrix * vec4(a_pos,1.0);
    WorldFragPos = model_space.xyz;    
    gl_Position = u_view_proj* model_space;
}