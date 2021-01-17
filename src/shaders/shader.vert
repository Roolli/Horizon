#version 450

layout(location=0) in vec3 a_pos;
layout(location=1) in vec2 tex_coord;
layout (location=0) out vec2 v_tex_coord;

layout(set=1,binding=0)
uniform Uniforms {
    mat4 u_view_proj;
};

void main()
{
    v_tex_coord = tex_coord;
    gl_Position = u_view_proj* vec4(a_pos,1.0);
}