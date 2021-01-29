#version 450

layout(location=0) in vec3 a_pos;
layout(location=1) in vec2 tex_coord;

layout(location=5) in vec4 model_matrix_c1;
layout(location=6) in vec4 model_matrix_c2;
layout(location=7) in vec4 model_matrix_c3;
layout(location=8) in vec4 model_matrix_c4;


layout (location=0) out vec2 v_tex_coord;

layout(set=1,binding=0)
uniform Uniforms {
    mat4 u_view_proj;
};

mat4 model_matrix;


void main()
{
    model_matrix= mat4(model_matrix_c1,model_matrix_c2,model_matrix_c3,model_matrix_c4);
    v_tex_coord = tex_coord;
    gl_Position = u_view_proj* model_matrix * vec4(a_pos,1.0);
}
