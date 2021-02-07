 #version 450

 layout(location=0) in vec2 v_tex_coord;
 layout(location=1) in vec3 v_normal;
 layout(location=2) in vec3 v_position;
 layout (location=0) out vec4 f_color;

layout(set=0, binding=0) uniform texture2D t_diffuse;
layout(set=0, binding=1)  uniform sampler s_diffuse;

layout(set=2, binding=0) uniform Light{
    vec3 light_position;
    vec3 light_color;
};

 void main()
 {
     vec4 object_color = texture(sampler2D(t_diffuse,s_diffuse),v_tex_coord);
     float ambient_strength = 0.1;
     vec3 ambient_color = light_color *ambient_strength;

    vec3 normal = normalize(v_normal);

    vec3 directional_light = normalize(light_position-v_position);
    float diffuse_strength = max(dot(normal,directional_light),0.0);

    vec3 diffuse_color = light_color * diffuse_strength;

     vec3 result = (ambient_color + diffuse_color) * object_color.xyz;

     f_color = vec4(result,object_color.a);
 }

 