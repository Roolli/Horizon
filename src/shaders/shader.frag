 #version 450

const int MAX_LIGHTS = 10;

struct Light {
    vec4 light_position;
    vec4 light_color;
};

 layout(location=0) in vec2 v_tex_coord;
 layout(location=1) in vec3 v_position;
 layout(location=2) in vec3 v_light_position[MAX_LIGHTS];
 layout(location=3+MAX_LIGHTS) in vec3 v_view_position[MAX_LIGHTS];

layout (location=0) out vec4 f_color;

layout(set=0, binding=0) uniform texture2D t_diffuse;
layout(set=0, binding=1) uniform sampler s_diffuse;
layout(set=0, binding=2) uniform texture2D t_normal;
layout(set=0, binding=3) uniform sampler s_normal;

layout(set=1,binding=0)
uniform Globals {
    vec3 u_view_position;
    mat4 u_view_proj;
    uvec4 lights_num;
};


layout(set=2, binding=0) uniform Lights{
    Light u_lights[MAX_LIGHTS];
};

 void main()
 {
     vec4 object_color = texture(sampler2D(t_diffuse,s_diffuse),v_tex_coord);
     vec4 object_normal = texture(sampler2D(t_normal,s_normal),v_tex_coord);
     vec3 ambient_color = vec3(0.1,0.1,0.1);
     // ambient lighting
     //float ambient_strength = 0.1;
     vec3 color = ambient_color * object_color.xyz;
     for(int i =0; i < int(lights_num.x) &&i < MAX_LIGHTS;i++)
     {
         Light light = u_lights[i];

        //diffuse lighting 
        vec3 normal = normalize(object_normal.rgb);
        vec3 directional_light = normalize(v_light_position[i].xyz-v_position);
        float diffuse_strength = max(dot(normal,directional_light),0.0);

        vec3 diffuse_color = light.light_color.xyz * diffuse_strength;

        // specular
        vec3 view_dir = normalize(v_view_position[i] - v_position);
        vec3 half_dir = normalize(view_dir + directional_light);

        float specular_strength = pow(max(dot(normal,half_dir),0.0),32);
        vec3 specular_color = specular_strength * light.light_color.xyz;

        color +=  ambient_color +  diffuse_color + specular_color;
     }

     f_color = vec4(color,1.0) * object_color;
 }

 