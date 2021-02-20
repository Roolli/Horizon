#version 450

const int MAX_LIGHTS = 10;

struct Light {
    vec3 light_position;
    vec3 light_color;
};

layout(location=0) in vec3 a_pos;
layout(location=1) in vec2 tex_coord;
layout(location=2) in vec3 a_normal;
layout(location=3) in vec3 tangent;
layout(location=4) in vec3 bitangent;
layout(location=5) in vec4 model_matrix_c1;
layout(location=6) in vec4 model_matrix_c2;
layout(location=7) in vec4 model_matrix_c3;
layout(location=8) in vec4 model_matrix_c4;


layout (location=0) out vec2 v_tex_coord;
layout (location=1) out vec3 v_position;
layout (location=2) out vec3 v_light_position[MAX_LIGHTS];
layout (location=3+MAX_LIGHTS) out vec3 v_view_position[MAX_LIGHTS];

layout(set=1,binding=0)
uniform Globals {
    vec3 u_view_position;
    mat4 u_view_proj;
    uvec4 lights_num;
};
layout(set=2,binding=0)
uniform Lights {
    Light u_lights[MAX_LIGHTS];
};

mat4 model_matrix;


void main()
{
    v_tex_coord = tex_coord; 
    model_matrix= mat4(model_matrix_c1,model_matrix_c2,model_matrix_c3,model_matrix_c4);
    // TODO: pass it as a uniform value to recude computation costs
    mat3 normal_matrix = mat3(transpose(inverse(model_matrix)));
    vec3 normal = normalize(normal_matrix*a_normal);
    vec3 tangent = normalize(normal_matrix * tangent);
    vec3 bitangent = normalize(normal_matrix * bitangent);
    mat3 tangent_matrix = transpose(mat3(tangent,bitangent,normal));
    
    vec4 model_space = model_matrix * vec4(a_pos,1.0);
    v_position = tangent_matrix * model_space.xyz;
   for(int i =0; i < int(lights_num.x) &&i < MAX_LIGHTS;i++)
     {
         Light light = u_lights[i];
    v_light_position[i] = tangent_matrix * light.light_position;
    v_view_position[i] = tangent_matrix * u_view_position;
     }
    gl_Position = u_view_proj* model_space;
}
