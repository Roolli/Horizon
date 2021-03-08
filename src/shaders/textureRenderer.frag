#version 450 

layout(location=0) in vec2 UV;

layout(location=0) out vec4 color;

layout(set=0,binding=0) uniform texture2D t_texture;
layout(set=0,binding=1) uniform sampler t_sampler;

void main()
{
    color = texture(sampler2D(t_texture,t_sampler),UV);
}