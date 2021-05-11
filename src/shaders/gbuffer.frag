#version 450

layout (location=0) in vec2 v_tex_coord;
layout (location=1) in vec3 WorldFragNormal;
layout (location=2) in vec3 WorldFragPos;

layout(set=0, binding=0) uniform texture2D t_texture;
layout(set=0, binding=1) uniform sampler textureSampler;
layout(set=0, binding=2) uniform texture2D t_normal;
layout(set=0, binding=3) uniform sampler normalSampler;

layout (location=0) out vec4 position; 
layout (location=1) out vec4 normal; 
layout (location=2) out vec4 albedo; 

void main()
{
    albedo = texture(sampler2D(t_texture,textureSampler),v_tex_coord);
    position = vec4(WorldFragPos,1.0);
    normal = vec4(WorldFragNormal * texture(sampler2D(t_normal,normalSampler),v_tex_coord).xyz,1.0);
}