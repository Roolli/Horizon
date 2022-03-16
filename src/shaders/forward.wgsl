
[[stage(vertex)]]
fn vs_main([[location(0)]] pos: vec2<f32>) -> [[builtin(position)]] vec4<f32> {
    return vec4<f32>(pos,0.0,1.0);
}

struct SpotLight {
    position: vec4<f32>;
    direction: vec4<f32>;
    color: vec4<f32>;
    attenuation: vec4<f32>; // x constant, y linear, z quadratic
    cutoffs: vec4<f32>;  // X inner , Y outer
};

struct PointLight {
    position: vec4<f32>;
    color: vec4<f32>;
    attenuation: vec4<f32>; // x constant, y linear, z quadratic 
};

struct DirectionalLight {
    direction: vec4<f32>;
    color: vec4<f32>;
};

struct PointLightContainer {
    elements: array<PointLight>;
};

struct SpotLightContainer {
    elements: array<SpotLight>;
};
struct Globals {
    u_view_position: vec4<f32>;
    u_view_proj: mat4x4<f32>;
    lights_num: vec4<u32>;
};


[[group(0)
,binding(0)]]
var  texture_sampler: sampler;

[[group(0)
,binding(1)]]
var positions:texture_2d<f32>;
[[group(0)
,binding(2)]]
var normals: texture_2d<f32>;
[[group(0),binding(3)]]
var specular:texture_2d<f32>;
[[group(0)
,binding(4)]]
var albedo: texture_2d<f32>;

struct CanvasSize {
     canvasConstants: vec2<f32>;
};
[[group(0)
,binding(5)]]
var<uniform> canvasSize: CanvasSize;


[[group(1)
,binding(0)]]
var<uniform> globals: Globals;

[[group(1)
,binding(3)]]
var t_shadow: texture_depth_2d_array;
[[group(1)
,binding(4)]]
var s_shadow: sampler_comparison;

struct CascadeTransforms{
    elements: array<mat4x4<f32>>;
};

struct CascadeLengths {
    elements: array<f32>;
};

[[group(1),binding(5)]]
var<storage,read> cascade_transforms: CascadeTransforms;
[[group(1),binding(6)]]
var<storage,read> cascade_lengths: CascadeLengths;

[[group(2)
,binding(0)]]
var<uniform> dirLight: DirectionalLight;

[[group(2)
,binding(1)]]
var<storage,read> pointLights: PointLightContainer;

[[group(2),
binding(2)]]
var<storage,read> spotLights: SpotLightContainer;

struct FragmentInput {
[[builtin(position)]] fragPos: vec4<f32>;
};


fn get_shadow_value(coords:vec4<f32>) -> f32
{
    let depth:f32 = abs(coords.z); 
    let length:i32 = i32(arrayLength(&cascade_lengths.elements));
    var layer = -1;
    for(var i:i32 =0; i < length;i = i + 1)
    {
        if(depth < cascade_lengths.elements[i])
        {
            layer = i;
            break;
        }
    }
    if(layer == -1)
    {
        layer = length - 1;
    }
    let light_coords = cascade_transforms.elements[layer] * coords;

    if(light_coords.w <= 0.0)
    {
        return 1.0;
    }
    let flip = vec2<f32>(0.5,-0.5);
    let proj_correction = 1.0 / light_coords.w;
    let light_local = light_coords.xy * flip * proj_correction + vec2<f32>(0.5,0.5);

    return textureSampleCompareLevel(t_shadow,s_shadow,light_local,layer,light_coords.z*proj_correction);
}

let ambient_strength:f32 = 0.1;

fn calcPointLightContribution(light: PointLight, position: vec3<f32>, normal: vec3<f32>, view_dir: vec3<f32>, object_color: vec3<f32>) -> vec3<f32> {
    let ambient = ambient_strength * object_color;
    let norm = normalize(normal);
    let light_direction = normalize(light.position.xyz - position);

    let diffuse_strength = max(dot(norm,light_direction),0.0);
    let diffuse_color = light.color.xyz * diffuse_strength * object_color;

    let half_dir = normalize(view_dir - light_direction);
    let specular_strength = pow(max(dot(normal,half_dir),0.0),32.0);
    let specular_color = specular_strength * light.color.xyz * object_color;
    let dist = length(light.position.xyz - position);
    let attenuation = 5.0 / (light.attenuation.x + light.attenuation.y * dist + light.attenuation.z * (dist* dist));
    
    return (ambient * attenuation + specular_color * attenuation + diffuse_color * attenuation);

}

fn calcDirLightContribution(normal: vec3<f32>, view_direction: vec3<f32>, object_color: vec3<f32>,shadow:f32) -> vec3<f32> {

    let ambient = ambient_strength * object_color;
    let norm = normalize(normal);
    let light_direction = normalize(-dirLight.direction.xyz);

    let diffuse_strength = max(dot(norm,light_direction),0.0);
    let diffuse_color = dirLight.color.xyz * diffuse_strength * object_color;

    let half_dir = normalize(view_direction - light_direction);
    let specular_strength = pow(max(dot(normal,half_dir),0.0),32.0);
    let specular_color = specular_strength * dirLight.color.xyz * object_color;
  
    return (ambient + ((specular_color + diffuse_color) * shadow));
    
}

[[stage(fragment)]]
fn fs_main(in: FragmentInput) -> [[location(0)]] vec4<f32> {
    var result = vec3<f32>(0.0);
    let coordinates = in.fragPos.xy / canvasSize.canvasConstants;
    let position = textureSample(positions,texture_sampler,coordinates).xyz;
    // TODO: get occlusion factor & roughness from their respective channels
    let object_normal = textureSample(normals,texture_sampler,coordinates);
    let object_color = textureSample(albedo,texture_sampler,coordinates);
    let view_direction = normalize(-position);
    let shadow = get_shadow_value(vec4<f32>(position,1.0)); 
    result = result + calcDirLightContribution(object_normal.xyz,view_direction,object_color.xyz,shadow);
    for(var i:u32 =0u; i < arrayLength(&pointLights.elements) && i < u32(globals.lights_num.x) ;i = i+1u)
    {   
       result = result + calcPointLightContribution(pointLights.elements[i],position,object_normal.xyz,view_direction,object_color.xyz);
    }
    
    return vec4<f32>(result,1.0);
}

