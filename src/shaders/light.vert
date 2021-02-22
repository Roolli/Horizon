#version 450

const int MAX_LIGHTS = 10;
float scale = 0.25;

layout (location=0) in vec3 a_position;
layout (location=0) out vec3 v_color;

struct Light {
    vec4 light_position;
    vec4 light_color;
};

layout(set=0,binding=0)
uniform Globals {
    vec3 u_view_position;
    mat4 u_view_proj;
    uvec4 lights_num;
};
layout(set=1,binding=0)
uniform Lights {
    Light u_lights[MAX_LIGHTS];
};


void main()
{   
   //transform * scale  + rotate
   
    vec3 v_position = a_position * scale + u_lights[gl_InstanceIndex].light_position.xyz;
    gl_Position = u_view_proj *  vec4(v_position,1);
    v_color = u_lights[gl_InstanceIndex].light_color.xyz;
}

