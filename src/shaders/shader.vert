#version 450
#extension GL_EXT_scalar_block_layout: require

const int MAX_POINT_LIGHTS = 5;
const int MAX_SPOT_LIGHTS = 5;

struct DirectionalLight {
    mat4 dl_projection;
    vec4 direction;
    vec4 color;
};
struct PointLight {
    vec4 position;
    vec4 color;
    vec4 attenuation; // x constant, y linear, z quadratic
};
struct SpotLight {
    vec4 position;
    vec4 direction;
    vec4 color;
    vec4 cutoffs; // X inner , Y outer
};


layout(location=0) in vec3 a_pos;
layout(location=1) in vec2 tex_coord;
layout(location=2) in vec3 a_normal;
layout(location=3) in vec3 tangent;
layout(location=4) in vec3 bitangent;


layout (location=0) out vec2 v_tex_coord;
layout (location=1) out vec4 TangentFragPos;
layout (location=2) out vec4 WorldFragPos;
layout (location=3) out vec3 tangent_space_view_position;
layout (location=4) out vec3 v_light_position[MAX_SPOT_LIGHTS + MAX_POINT_LIGHTS];

layout(set=1,binding=0)
uniform Globals {
    vec4 u_view_position;
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

layout(set=2,binding=0) uniform DirLight
{
     DirectionalLight light;
}dir_light;
layout(std430, set=2,binding=1) buffer PointLights
{ 
   PointLight lights[MAX_POINT_LIGHTS];
}v_pointlights;
layout(std430, set=2,binding=2) buffer SpotLights
{ 
    SpotLight lights[MAX_SPOT_LIGHTS];
}v_spotlights;



void main()
{
    v_tex_coord = tex_coord; 
   mat4 model_matrix= transform[gl_InstanceIndex];
    mat3 normal_matrix = mat3(normal_matricies[gl_InstanceIndex]);
    vec3 normal = normalize(normal_matrix*a_normal);
    vec3 tangent = normalize(normal_matrix * tangent);
    vec3 bitangent = normalize(normal_matrix * bitangent);
    mat3 tangent_matrix = transpose(mat3(tangent,bitangent,normal));
    
    vec4 model_space = model_matrix * vec4(a_pos,1.0);
    WorldFragPos = model_space;
    vec3 temp_pos = tangent_matrix * model_space.xyz;
    TangentFragPos = vec4(temp_pos,1.0);
    tangent_space_view_position = tangent_matrix * u_view_position.xyz;
   for(int i =0; i < lights_num.x &&i < MAX_POINT_LIGHTS;i++)
     {
         PointLight light = v_pointlights.lights[i];
      
    v_light_position[i] = tangent_matrix * light.position.xyz;
    
     }
    //  for(int i =num_point_lights; i < num_point_lights + int(lights_num.y) &&i - num_point_lights < MAX_SPOT_LIGHTS;i++)
    //  {
    //      SpotLight light = v_spotlights.lights[i];
    // v_light_position[i] = tangent_matrix * light.position.xyz;
    
    //  }
    gl_Position = u_view_proj* model_space;
}

