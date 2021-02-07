#version 450

layout(location=0) in vec3 a_pos;
layout(location=1) in vec2 tex_coord;
layout(location=2) in vec3 a_normal;

layout(location=5) in vec4 model_matrix_c1;
layout(location=6) in vec4 model_matrix_c2;
layout(location=7) in vec4 model_matrix_c3;
layout(location=8) in vec4 model_matrix_c4;


layout (location=0) out vec2 v_tex_coord;

layout (location=1) out vec3 v_position;
layout (location=2) out vec3 v_normal;

layout(set=1,binding=0)
uniform Uniforms {
    mat4 u_view_proj;
};

mat4 model_matrix;


void main()
{
    v_tex_coord = tex_coord; 
    v_normal = a_normal;
    model_matrix= mat4(model_matrix_c1,model_matrix_c2,model_matrix_c3,model_matrix_c4);
    vec4 model_space = model_matrix * vec4(a_pos,1.0);
    v_position = model_space.xyz;
    gl_Position = u_view_proj* model_space;
}
