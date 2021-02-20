#version 450
layout (location=0) in vec3 a_position;
layout (location=0) out vec3 v_color;

layout(set=0,binding=0)
uniform Globals {
    vec3 u_view_position;
    mat4 u_view_proj;
    uvec4 lights_num;
};
layout (set=1,binding=0)
uniform Light {
    vec3 u_position;
    vec3 u_color;
};
float scale = 0.25;


void main()
{   
    // transform * scale  + rotate
    vec3 v_position = a_position * scale + u_position;
    gl_Position = u_view_proj *  vec4(v_position,1);
    v_color = u_color;
}