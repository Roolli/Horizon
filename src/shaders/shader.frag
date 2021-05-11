 #version 450
 #extension GL_EXT_scalar_block_layout: require

const float ambient_strength = 0.1;

struct SpotLight {
    vec4 position;
    vec4 direction;
    vec4 color;
    vec4 attenuation; // x constant, y linear, z quadratic
    vec4 cutoffs; // X inner , Y outer
};
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

layout (location=0) out vec4 f_color; // output to the frame view

layout(set=0, binding=0) uniform sampler textureSampler;
layout(set=0, binding=1) uniform texture2D positions;
layout(set=0, binding=2) uniform texture2D normal;
layout(set=0, binding=3) uniform texture2D albedo;
layout(set=0,binding=4) uniform CanvasSize {
    vec2 canvasConstants;
};

//TODO: split into own uniforms
layout(set=1,binding=0)
uniform Globals {
    vec4 u_view_position;
    mat4 u_view_proj;
    uvec4 lights_num; // X Point lights, Y spot lights
};


layout(set=1,binding=1) uniform texture2D t_shadow; 
layout(set=1,binding=2) uniform samplerShadow s_shadow; 


layout(set=2,binding=0) uniform DirLight
{
     DirectionalLight light;
}dir_light;
layout(std430, set=2,binding=1) buffer PointLights
{ 
   PointLight lights[];
}v_pointlights;
layout(std430, set=2,binding=2) buffer SpotLights
{ 
    SpotLight lights[];
}v_spotlights;


vec3 calcDirLightContribution(DirectionalLight light,vec3 normal,vec3 viewDir,vec3 color);
vec3 calcPointLightsContribution(PointLight light,vec3 position,vec3 normal,vec3 viewDir,vec3 object_color);
// vec3 calcSpotLightsContribution(SpotLight light,vec3 normal,vec3 fragPos,vec3 viewDir,vec3 object_color,vec3 ambient);


// float fetch_shadow(int light_id,vec4 homogeneous_coords)
// {
//          const vec2 flip_correction = vec2(0.5,-0.5);
//         vec3 light_local = vec3(
//        homogeneous_coords.xy * flip_correction/homogeneous_coords.w +0.5,
//         homogeneous_coords.z/homogeneous_coords.w
//         );
        
//     return  texture(sampler2DShadow(t_shadow,s_shadow),light_local);
// }

 void main()
 {



     vec3 result = vec3(0.0,0.0,0.0);
     vec2 coordinates = gl_FragCoord.xy / canvasConstants;

     vec3 position = texture(sampler2D(positions,textureSampler),coordinates).xyz;

    if(position.z > 10000.0)
    {
        discard;
    }
     vec4 object_normal = texture(sampler2D(normal,textureSampler),coordinates);

    // albedo
     vec4 object_color = texture(sampler2D(albedo,textureSampler),coordinates);
    
    //! TODO: change to push_constant

     
    // TODO: change to single layer shadow map for directional light
   // float shadow = fetch_shadow(0, (light.u_projection* WorldFragPos));

       
        vec3 viewDir = normalize(-position);
        // directional light
       //result+=  calcDirLightContribution(dir_light.light,object_normal.xyz,viewDir,object_color.xyz);
      //  result*= shadow;
      
    // for point lights 
    float radius;
     for(int i =0; i < int(lights_num.x);i++)
     {
        PointLight light = v_pointlights.lights[i];
        // radius = sqrt(1.0 / light.attenuation.z);
        // float dist = length(position-light.position.xyz);
        // if(dist >radius)
        // {
        //     continue;
        // }
        result+= calcPointLightsContribution(light,position,object_normal.xyz,viewDir,object_color.xyz);
         
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

 
vec3 calcDirLightContribution(DirectionalLight light,vec3 normal,vec3 viewDir,vec3 color)
 {
      vec3 ambient = ambient_strength * color;
      vec3 norm = normalize(normal);
        vec3 light_direction = normalize(-light.direction.xyz);

        // diffuse
        float diffuse_strength = max(dot(norm,light_direction),0.0);
        vec3 diffuse_color = light.color.xyz * diffuse_strength * color;

        vec3 half_dir = normalize(viewDir + light_direction);

        float specular_strength = pow(max(dot(normal,half_dir),0.0),32);
        vec3 specular_color = specular_strength * light.color.xyz * color;

        return (ambient  +  specular_color +    diffuse_color);
}

//TODO: ADD shadow mapping for point lights
vec3 calcPointLightsContribution(PointLight light,vec3 position,vec3 normal,vec3 viewDir,vec3 object_color) {

        vec3 ambient = ambient_strength * object_color;
        //diffuse lighting 
        // TODO: change this for normal mapping
        vec3 norm = normalize(normal);
        vec3 light_direction = normalize(light.position.xyz - position);
     
        float diffuse_strength = max(dot(norm,light_direction),0.0);

        vec3 diffuse_color = light.color.xyz * diffuse_strength *object_color;
        // specular
        vec3 half_dir = normalize(viewDir - light_direction);

        float specular_strength = pow(max(dot(normal,half_dir),0.0),32);
        vec3 specular_color = specular_strength * light.color.xyz * object_color;
        float dist = length(light.position.xyz - position);
        float attenuation = 5.0 / (light.attenuation.x + light.attenuation.y * dist + light.attenuation.z * (dist * dist)); 

        return (ambient * attenuation +   specular_color * attenuation +     diffuse_color *attenuation);
}

// vec3 calcSpotLightsContribution(SpotLight light,vec3 normal,vec3 fragPos,vec3 viewDir,vec3 color,vec3 ambient) {

//     vec3 light_direction = normalize(light.position.xyz-fragPos);
//     float diffuse_strength = max(dot(normal,light_direction),0.0);

//     vec3 diffuse_color = light.color.xyz * diffuse_strength * color;

//     vec3 half_dir = normalize(viewDir + light_direction);

//     float specular_strength = pow(max(dot(normal,half_dir),0.0),32);
//     vec3 specular_color = specular_strength * light.color.xyz  * color;

//     float theta = dot(light_direction,normalize(-light.direction.xyz));
//     float epsilon = (light.cutoffs.x - light.cutoffs.y);
//     float intensity = clamp((theta - light.cutoffs.y) / epsilon,0.0,1.0);

//     diffuse_color *= intensity;
//     specular_color *= intensity;

//     float dist = length(light.position - WorldFragPos);
//     float attenuation = 1.0 / (light.attenuation.x + light.attenuation.y * dist + light.attenuation.z * (dist * dist)); 

//     return (ambient * attenuation +   specular_color * attenuation +     diffuse_color *attenuation);
// }