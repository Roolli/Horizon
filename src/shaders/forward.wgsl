
@stage(vertex)
fn vs_main(@location(0) pos: vec2<f32>) -> @builtin(position) vec4<f32> {
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
    dl_projection: mat4x4<f32>;
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

@group(0)
@binding(0)
var  texture_sampler: sampler;

@group(0)
@binding(1)
var positions:texture_2d<f32>;
@group(0)
@binding(2)
var normals: texture_2d<f32>;
@group(0)
@binding(3)
var albedo: texture_2d<f32>;

struct CanvasSize {
     canvasConstants: vec2<f32>;
};
@group(0)
@binding(4)
var<uniform> canvasSize: CanvasSize;


@group(1)
@binding(0)
var<uniform> globals: Globals;

@group(1)
@binding(1)
var t_shadow: texture_2d<f32>;
@group(1)
@binding(2)
var s_shadow: sampler;

@group(2)
@binding(0)
var<uniform> dirLight: DirectionalLight;

@group(2)
@binding(1)
var<storage,read>pointLights: PointLightContainer;

@group(2)
@binding(2)
var<storage,read> spotLights: SpotLightContainer;

struct FragmentInput {
@builtin(position) fragPos: vec4<f32>;
};


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

fn calcDirLightContribution(normal: vec3<f32>, view_direction: vec3<f32>, object_color: vec3<f32>) -> vec3<f32> {

    let ambient = ambient_strength * object_color;
    let norm = normalize(normal);
    let light_direction = normalize(-dirLight.direction.xyz);

    let diffuse_strength = max(dot(norm,light_direction),0.0);
    let diffuse_color = dirLight.color.xyz * diffuse_strength * object_color;

    let half_dir = normalize(view_direction - light_direction);
    let specular_strength = pow(max(dot(normal,half_dir),0.0),32.0);
    let specular_color = specular_strength * dirLight.color.xyz * object_color;
  
    return (ambient + specular_color + diffuse_color);
    
}

@stage(fragment)
fn fs_main(in: FragmentInput) -> @location(0) vec4<f32> {
    var result = vec3<f32>(0.0);
    let coordinates = in.fragPos.xy / canvasSize.canvasConstants;
    let position = textureSample(positions,texture_sampler,coordinates).xyz;
  
    
    let object_normal = textureSample(normals,texture_sampler,coordinates);
    let object_color = textureSample(albedo,texture_sampler,coordinates);
    let view_direction = normalize(-position);
    result = result + calcDirLightContribution(object_normal.xyz,view_direction,object_color.xyz);

    for(var i:u32 =0u; i < arrayLength(&pointLights.elements) && i < u32(globals.lights_num.x) ;i = i+1u)
    {   
       result = result + calcPointLightContribution(pointLights.elements[i],position,object_normal.xyz,view_direction,object_color.xyz);
    }
    return vec4<f32>(result,1.0);
}

