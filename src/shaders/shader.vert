#version 450
#extension GL_EXT_scalar_block_layout: require

const int MAX_LIGHTS = 10;

struct Light {
    mat4 u_projection;
    vec4 light_position;
    vec4 light_color;
};

layout(location=0) in vec3 a_pos;
layout(location=1) in vec2 tex_coord;
layout(location=2) in vec3 a_normal;
layout(location=3) in vec3 tangent;
layout(location=4) in vec3 bitangent;


layout (location=0) out vec2 v_tex_coord;
layout (location=1) out vec4 v_position;
layout (location=2) out vec3 v_light_position[MAX_LIGHTS];
layout (location=3+MAX_LIGHTS) out vec3 v_view_position[MAX_LIGHTS];

layout(set=1,binding=0)
uniform Globals {
    vec3 u_view_position;
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


layout(set=2,binding=0)
uniform Lights {
    Light u_lights[MAX_LIGHTS];
};

mat4 model_matrix;


void main()
{
    v_tex_coord = tex_coord; 
    model_matrix= transform[gl_InstanceIndex];
    mat3 normal_matrix = mat3(normal_matricies[gl_InstanceIndex]);
    vec3 normal = normalize(normal_matrix*a_normal);
    vec3 tangent = normalize(normal_matrix * tangent);
    vec3 bitangent = normalize(normal_matrix * bitangent);
    mat3 tangent_matrix = transpose(mat3(tangent,bitangent,normal));
    
    vec4 model_space = model_matrix * vec4(a_pos,1.0);
    vec3 temp_pos = tangent_matrix * model_space.xyz;
    v_position = vec4(temp_pos,1.0);

   for(int i =0; i < int(lights_num.x) &&i < MAX_LIGHTS;i++)
     {
         Light light = u_lights[i];
    v_light_position[i] = tangent_matrix * light.light_position.xyz;
    v_view_position[i] = tangent_matrix * u_view_position;
     }
    gl_Position = u_view_proj* model_space;
}

