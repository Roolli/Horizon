
struct VertexOutput {
@builtin(position) fragPos: vec4<f32>;
@location(0) fragUV:vec2<f32>;
};

@stage(vertex)
fn vs_main(@location(0) pos: vec4<f32>,@location(1) uv:vec2<f32>) -> VertexOutput {
    var vertex_output: VertexOutput;
        vertex_output.fragUV = uv;
        vertex_output.fragPos = pos;
    return vertex_output;
}

struct SpotLight {
    position: vec4<f32>;
    direction: vec4<f32>;
    color: vec3<f32>;
    radius:f32;
    cutoffs: vec4<f32>;  // X inner , Y outer
};

struct PointLight {
    position: vec4<f32>;
    color: vec3<f32>;
    radius: f32;
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

// determines the number of lights a tile can be influenced by.
let num_tile_light_slot:u32 = 128u; // needs to be a constant as you can't create a runtime array with an atomic next to it. pipeline overrides would solve this issue this being a constant but they are not implemeneted.

struct TileLightData {
    light_count: atomic<u32>;
   light_ids: array<u32,128>;
};
struct Tiles {
    data: array<TileLightData>;
};
struct TileInfo {
     tile_size: i32;
     tile_count_x: i32;
     tile_count_y: i32;
     num_tiles: u32;
     num_tile_light_slot: u32;
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
var specular:texture_2d<f32>;
@group(0)
@binding(4)
var albedo: texture_2d<f32>;

struct CanvasSize {
     canvasConstants: vec2<f32>;
};
@group(0)
@binding(5)
var<uniform> canvasSize: CanvasSize;
@group(0)
@binding(6)
var<storage,read_write> tile_light_data:Tiles;

@group(0)
@binding(7)
var<uniform> tile_info:TileInfo;

@group(1)
@binding(0)
var<uniform> globals: Globals;

@group(1)
@binding(3)
var t_shadow: texture_depth_2d_array;
@group(1)
@binding(3)
var t_shadow_single: texture_depth_2d;
@group(1)
@binding(4)
var s_shadow: sampler_comparison;

struct CascadeTransforms{
    elements: array<mat4x4<f32>>;
};

struct CascadeLengths {
    elements: array<f32>;
};

@group(1)
@binding(5)
var<storage,read> cascade_transforms: CascadeTransforms;
@group(1)
@binding(6)
var<storage,read> cascade_lengths: CascadeLengths;

@group(2)
@binding(0)
var<uniform> dirLight: DirectionalLight;

@group(2)
@binding(1)
var<storage,read> pointLights: PointLightContainer;

@group(2)
@binding(2)
var<storage,read> spotLights: SpotLightContainer;


fn get_shadow_value(coords:vec4<f32>) -> f32
{
    let depth:f32 = coords.z; 
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

fn get_shadow_value_web(light_coords:vec4<f32>) -> f32
{
    if(light_coords.w <= 0.0)
    {
        return 1.0;
    }  
    let flip_correction = vec2<f32>(0.5, -0.5);
    let proj_correction = 1.0 / light_coords.w;
    let light_local = light_coords.xy * flip_correction * proj_correction + vec2<f32>(0.5, 0.5);
    return textureSampleCompareLevel(t_shadow_single,s_shadow,light_local,light_coords.z*proj_correction);
}
fn get_tile_id(coordinates: vec2<f32>) -> u32
{
    // calculate tile position
    //let V = normalize(globals.u_view_position.xyz - position);
    var tile_scale = vec2<f32>(1.0 / f32(tile_info.tile_count_x),1.0 / f32(tile_info.tile_count_y));
    var uv_flip = vec2<f32>(coordinates.x,1.0-coordinates.y);
    var tile_coord = vec2<u32>(floor(uv_flip/tile_scale));
                                                    // tile_counts are i32
    return tile_coord.x + tile_coord.y * u32(tile_info.tile_count_x);
}

let ambient_strength:f32 = 0.1;

fn calcPointLightContribution(light: PointLight, position: vec3<f32>, normal: vec3<f32>, view_dir: vec3<f32>, object_color: vec3<f32>) -> vec3<f32> {
    
    let light_direction = normalize(light.position.xyz - position);

    let lambert = max(dot(light_direction,normal),0.0);    
    let half_dir = normalize(view_dir + light_direction);
    let specular = f32(lambert >0.0) * pow(max(dot(half_dir,normal),0.0),10.0);
    let dist = length(light.position.xyz - position);    

    return  clamp(light.color*pow(1.0-dist / light.radius,2.0) * (object_color * lambert + vec3<f32>(1.0) * specular),vec3<f32>(0.0),vec3<f32>(1.0));

}
fn addPointLightContributions(position:vec3<f32>,coordinates:vec2<f32>,object_normal:vec3<f32>,view_direction:vec3<f32>,object_color:vec3<f32>) -> vec3<f32>
{
    var result = vec3<f32>(0.0);
    let tile_id = get_tile_id(coordinates);
    let count = atomicLoad(&tile_light_data.data[tile_id].light_count);
    for(var i: u32 = 0u; i < num_tile_light_slot; i = i + 1u)
    {
        if(i >= count)
        {
            break;
        }
        var light =  pointLights.elements[tile_light_data.data[tile_id].light_ids[i] ];
        var dist = length(light.position.xyz - position);
        if(dist > light.radius)
        {
            continue;
        }

        result = result + calcPointLightContribution(light,position,object_normal,view_direction,object_color);
    }
    return result;
}

fn calcDirLightContribution(normal: vec3<f32>, view_direction: vec3<f32>, object_color: vec3<f32>,shadow:f32) -> vec3<f32> {

    let light_direction = normalize(dirLight.direction.xyz);

    let diffuse_strength = max(dot(normal,light_direction),ambient_strength);
    let diffuse_color = dirLight.color.xyz * diffuse_strength * object_color;

    let half_dir = normalize( light_direction+ view_direction);
    let specular_strength = pow(max(dot(half_dir,normal),0.0),10.0);
    let specular_color = specular_strength * dirLight.color.xyz * object_color;
  
    return ( diffuse_color * shadow + specular_color);
    
}


@stage(fragment)
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var result = vec3<f32>(0.0);
    let coordinates = vec2<i32>(floor(in.fragPos.xy)); // / canvasSize.canvasConstants;

    // TODO: get occlusion factor & roughness from their respective channels
    let position = textureLoad(positions,coordinates,0).xyz;
    if(position.z > 10000.0)
    {
        discard;
    }
    let object_normal = textureLoad(normals,coordinates,0).xyz;
    let object_color = textureLoad(albedo,coordinates,0).xyz;
    let view_direction = normalize(globals.u_view_position.xyz - position);
    
   
    let shadow = get_shadow_value(vec4<f32>(globals.u_view_position.xyz,1.0)); 

    result = result + calcDirLightContribution(object_normal,view_direction,object_color.xyz,shadow);

   result = result + addPointLightContributions(position,in.fragUV,object_normal,view_direction,object_color.xyz);
    
    return vec4<f32>(result,1.0);
}
@stage(fragment)
fn fs_main_web(in:VertexOutput) -> @location(0) vec4<f32>{
     var result = vec3<f32>(0.0);
    let coordinates = vec2<i32>(floor(in.fragPos.xy)); // / canvasSize.canvasConstants;
      let position = textureLoad(positions,coordinates,0).xyz;
    if(position.z > 10000.0)
    {
        discard;
    }
    let object_normal = textureLoad(normals,coordinates,0).xyz;
    let object_color = textureLoad(albedo,coordinates,0).xyz;
    let view_direction = normalize(-position);

    let shadow = get_shadow_value_web(cascade_transforms.elements[0]* vec4<f32>(position,1.0));
    result = result + calcDirLightContribution(object_normal.xyz,view_direction,object_color.xyz,shadow);

    result = result + addPointLightContributions(position,in.fragUV,object_normal.xyz,view_direction,object_color.xyz);
   
    return vec4<f32>(result,1.0);
}

