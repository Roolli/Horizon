#version 450

layout(location=0) in vec3 a_pos;
layout(location=1) in vec2 tex_coord;
layout (location=0) out vec2 v_tex_coord;

void main()
{
    v_tex_coord = tex_coord;
    gl_Position = vec4(a_pos,1.0);
}