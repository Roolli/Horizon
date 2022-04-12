


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

struct TileInfo {
     tile_size: i32;
     tile_count_x: i32;
     tile_count_y: i32;
     num_tiles: u32;
     num_tile_light_slot: u32;
};

// determines the number of lights a tile can be influenced by.
let num_tile_light_slot:u32 = 128u; // needs to be a constant as you can't create a runtime array with an atomic next to it. pipeline overrides would solve this issue this being a constant but they are not implemeneted.

// Not possible as of 22/10/2021 use uniforms instead
// @override(0)
// let TILE_SIZE :i32;
struct TileLightData {
    light_count: atomic<u32>;
   light_ids: array<u32,num_tile_light_slot>;
};
struct Tiles {
    data: array<TileLightData>;
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
struct LightCullingUniforms {
    u_proj: mat4x4<f32>;
    u_view: mat4x4<f32>;
};

struct CanvasSize {
     canvasConstants: vec2<f32>;
};

[[group(0),
binding(1)]]
var<storage,read>pointLights: PointLightContainer;

[[group(0)
,binding(2)]]
var<storage,read> spotLights: SpotLightContainer;



[[group(1)
,binding(0)]]
var<uniform> globals: Globals;


[[group(2),
binding(0)]]
var<uniform> tileInfo:TileInfo;
[[group(2)
,binding(1)]]
var<uniform> canvasSize: CanvasSize;
[[group(2),binding(2)]]
var<uniform> light_culling_uniforms: LightCullingUniforms;
[[group(2),binding(3)]]
var<storage,read_write> tile_light_data: Tiles;

 [[stage(compute),workgroup_size(64,1,1)]]
fn main([[builtin(global_invocation_id)]] Global_Invocation_Id: vec3<u32>) {
     let index = Global_Invocation_Id.x;
     if(index >= globals.lights_num.x)
     {
         return;
     }
     let proj = light_culling_uniforms.u_proj;


     let view_near =  -proj[3][2] / (proj[2][2]);
     let view_far = -proj[3][2] / (1.0 + proj[2][2]);

    var lightPos = pointLights.elements[index].position;
        lightPos = light_culling_uniforms.u_view * lightPos;
        lightPos = lightPos / lightPos.w;
    var lightRadius: f32 = pointLights.elements[index].radius;
    var bounding_box_min = lightPos - vec4<f32>(vec3<f32>(lightRadius),0.0);
    var bounding_box_max = lightPos + vec4<f32>(vec3<f32>(lightRadius),0.0);
    var frustums: array<vec4<f32>,6>;
    frustums[4] = vec4<f32>(0.0,0.0,0.0,-view_near); // near
    frustums[5] = vec4<f32>(0.0,0.0,1.0,-view_far); // far 

    for(var y: i32 = 0; y < tileInfo.tile_count_y; y= y +1)
    {
         for(var x: i32 = 0; x < tileInfo.tile_count_x; x= x +1)
        {
            var tilePixelIndex = vec2<i32>(x* tileInfo.tile_size,y*tileInfo.tile_size);
            // tile position in NDC space
            var floorCoord: vec2<f32> = 2.0 * vec2<f32>(tilePixelIndex) / canvasSize.canvasConstants.xy - vec2<f32>(1.0);  // -1, 1
            var ceilCoord: vec2<f32> = 2.0 * vec2<f32>(tilePixelIndex + vec2<i32>(tileInfo.tile_size)) / canvasSize.canvasConstants.xy - vec2<f32>(1.0);  // -1, 1
            var viewFloorCoord: vec2<f32> = vec2<f32>( (- view_near * floorCoord.x - proj[2][0] * view_near) / proj[0][0], (- view_near * floorCoord.y - proj[2][1] * view_near) / proj[1][1] );
            var viewCeilCoord: vec2<f32> = vec2<f32>( (- view_near * ceilCoord.x - proj[2][0] * view_near) / proj[0][0], (- view_near * ceilCoord.y - proj[2][1] * view_near) / proj[1][1] );
            frustums[0] = vec4<f32>(1.0, 0.0, - viewFloorCoord.x / view_near, 0.0);       // left
            frustums[1] = vec4<f32>(-1.0, 0.0, viewCeilCoord.x / view_near, 0.0);   // right
            frustums[2] = vec4<f32>(0.0, 1.0, - viewFloorCoord.y / view_near, 0.0);       // bottom
            frustums[3] = vec4<f32>(0.0, -1.0, viewCeilCoord.y / view_near, 0.0);   // top
            var dp: f32 = 0.0;  // dot product
            for (var i: u32 = 0u; i < 6u; i = i + 1u)
            {
                var p: vec4<f32>;
                if (frustums[i].x > 0.0) {
                    p.x = bounding_box_max.x;
                } else {
                    p.x = bounding_box_min.x;
                }
                if (frustums[i].y > 0.0) {
                    p.y = bounding_box_max.y;
                } else {
                    p.y = bounding_box_min.y;
                }
                if (frustums[i].z > 0.0) {
                    p.z = bounding_box_max.z;
                } else {
                    p.z = bounding_box_min.z;
                }
                p.w = 1.0;
                dp = dp + min(0.0, dot(p, frustums[i]));
            }
            if (dp >= 0.0) {
                var tileId: u32 = u32(x + y * tileInfo.tile_count_x);
                if (tileId < 0u || tileId >= tileInfo.num_tiles) {
                    continue;
                }
                var offset: u32 = atomicAdd(&(tile_light_data.data[tileId].light_count), 1u);
                if (offset >= num_tile_light_slot) {
                    continue;
                }
             tile_light_data.data[tileId].light_ids[offset] = Global_Invocation_Id.x;
        }
    }




 }
 }