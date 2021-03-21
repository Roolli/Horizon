 #version 450
 #extension GL_EXT_scalar_block_layout: require


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
    vec4 attenuation;
    vec4 cutoffs; // X inner , Y outer
};


const int MAX_POINT_LIGHTS = 32;
const int MAX_SPOT_LIGHTS = 32;

struct Light {
    mat4 u_projection;
    vec4 light_position;
    vec4 light_color;
};

 layout(location=0) in vec2 v_tex_coord;
 layout(location=1) in vec4 TangentFragPos;// FragPos in Tangent space
 layout(location=2) in vec4 WorldFragPos; // Frapos in World Space
 layout(location=3) in vec3 v_view_position;
 layout(location=4) in vec3 v_light_position[MAX_POINT_LIGHTS+MAX_SPOT_LIGHTS];

layout (location=0) out vec4 f_color; // output to the frame view

layout(set=0, binding=0) uniform texture2D t_diffuse;
layout(set=0, binding=1) uniform sampler s_diffuse;
layout(set=0, binding=2) uniform texture2D t_normal;
layout(set=0, binding=3) uniform sampler s_normal;

layout(set=1,binding=0)
uniform Globals {
    vec3 u_view_position;
    mat4 u_view_proj;
    uvec4 lights_num; // X Point lights, Y spot lights
};
layout(std430,set=1,binding=1)
buffer InstanceData
{
    mat4 transform[];
};

layout(set=1,binding=3) uniform texture2DArray t_shadow; 
layout(set=1,binding=4) uniform samplerShadow s_shadow; 

layout(set=2,binding=0) uniform DirLight
{
     DirectionalLight dirLight;
};
layout(set=2,binding=1) uniform PointLights
{ 
   PointLight pointLights[MAX_POINT_LIGHTS];
};
layout(set=2,binding=2) uniform SpotLights
{ 
    SpotLight spotLights[MAX_SPOT_LIGHTS];
};


vec3 calculateDirectionalLight(DirectionalLight light,vec3 normal,vec3 viewDir,vec3 object_color,vec3 ambient);
vec3 calculatePointLights(PointLight light,vec3 tangent_light_position,vec3 normal,vec3 fragPos,vec3 viewDir,vec3 object_color,vec3 ambient);
vec3 calculateSpotLight(SpotLight light,vec3 normal,vec3 fragPos,vec3 viewDir,vec3 object_color,vec3 ambient);


float fetch_shadow(int light_id,vec4 homogeneous_coords)
{
         const vec2 flip_correction = vec2(0.5,-0.5);
        vec4 light_local = vec4(
       homogeneous_coords.xy * flip_correction/homogeneous_coords.w +0.5,
        light_id,
        homogeneous_coords.z/homogeneous_coords.w
        );
        
    return  texture(sampler2DArrayShadow(t_shadow,s_shadow),light_local);
}

 void main()
 {
     vec4 object_color = texture(sampler2D(t_diffuse,s_diffuse),v_tex_coord);
     vec4 object_normal = texture(sampler2D(t_normal,s_normal),v_tex_coord);
    //! TODO: change to push_constant
     float ambient_strength = 0.1;
     
    // TODO: change to single layer shadow map for directional light
   // float shadow = fetch_shadow(0, (light.u_projection* WorldFragPos));


        vec3 ambient = ambient_strength * object_color.xyz;
        vec3 normal = normalize(object_normal.rgb);
        // Tangent space view direction
        vec3 view_dir = normalize(v_view_position - TangentFragPos.xyz);
        // directional light
        vec3 result = calculateDirectionalLight(dirLight,normal,view_dir,object_color.xyz,ambient);
      //  result*= shadow;
      
    // for point lights 
     for(int i =0; i < int(lights_num.x) &&i < MAX_POINT_LIGHTS;i++)
     {
        PointLight light = pointLights[i];

        result+= calculatePointLights(light,v_light_position[i],normal,TangentFragPos.xyz,view_dir,object_color.xyz,ambient);
         
     }
    //  // Spotlights
    //  for(int i = 0;i < int(lights_num.y)&& i < MAX_SPOT_LIGHTS;i++)
    //  {
    //         SpotLight light = light_data.spotLights[i];
    //        // ambient lighting
    // vec3 ambient = ambient_strength * color;
    //  }

       
     f_color = vec4(result,1.0);
 }

 
vec3 calculateDirectionalLight(DirectionalLight light,vec3 normal,vec3 viewDir,vec3 color,vec3 ambient)
 {
        vec3 light_direction = normalize(-light.direction.xyz);

        // diffuse
        float diffuse_strength = max(dot(normal,light_direction),0.0);
        vec3 diffuse_color = light.color.xyz * diffuse_strength * color;

        vec3 half_dir = normalize(viewDir + light_direction);

        float specular_strength = pow(max(dot(normal,half_dir),0.0),32);
        vec3 specular_color = specular_strength * light.color.xyz * color;

        return (ambient  +  specular_color +    diffuse_color);
}
//TODO: ADD shadow mapping for point lights

vec3 calculatePointLights(PointLight light,vec3 tangent_light_position,vec3 normal,vec3 fragPos,vec3 viewDir,vec3 object_color,vec3 ambient) {

        //diffuse lighting 
        // TODO: change this for normal mapping
        vec3 light_direction = normalize(tangent_light_position-fragPos);
     
        float diffuse_strength = max(dot(normal,tangent_light_position),0.0);

        vec3 diffuse_color = light.color.xyz * diffuse_strength *object_color;

        // specular
        vec3 view_dir = normalize(viewDir - fragPos);
        vec3 half_dir = normalize(view_dir + tangent_light_position);

        float specular_strength = pow(max(dot(normal,half_dir),0.0),32);
        vec3 specular_color = specular_strength * light.color.xyz * object_color;
        float dist = length(tangent_light_position - WorldFragPos.xyz);
        float attenuation = 1.0 / (light.attenuation.x + light.attenuation.y * dist + light.attenuation.z * (dist * dist)); 

        return (ambient * attenuation +   specular_color * attenuation +     diffuse_color *attenuation);
}

vec3 calculateSpotLight(SpotLight light,vec3 normal,vec3 fragPos,vec3 viewDir,vec3 color,vec3 ambient) {

    vec3 light_direction = normalize(light.position.xyz-fragPos);
    float diffuse_strength = max(dot(normal,light_direction),0.0);

    vec3 diffuse_color = light.color.xyz * diffuse_strength * color;

    vec3 half_dir = normalize(viewDir + light_direction);

    float specular_strength = pow(max(dot(normal,half_dir),0.0),32);
    vec3 specular_color = specular_strength * light.color.xyz  * color;

    float theta = dot(light_direction,normalize(-light.direction.xyz));
    float epsilon = (light.cutoffs.x - light.cutoffs.y);
    float intensity = clamp((theta - light.cutoffs.y) / epsilon,0.0,1.0);

    diffuse_color *= intensity;
    specular_color *= intensity;

    float dist = length(light.position - WorldFragPos);
    float attenuation = 1.0 / (light.attenuation.x + light.attenuation.y * dist + light.attenuation.z * (dist * dist)); 

    return (ambient * attenuation +   specular_color * attenuation +     diffuse_color *attenuation);
}