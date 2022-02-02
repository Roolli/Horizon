


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

struct TileInfo {
     tile_size: i32;
     tile_count_x: i32;
     tile_count_y: i32;
     num_tiles: u32;
     num_tile_light_slot: u32;
};

// Not possible as of 22/10/2021 use uniforms instead
// @override(0)
// let TILE_SIZE :i32;
struct TileLightData {
    light_count: atomic<u32>;
    light_ids: array<u32>;
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


struct CanvasSize {
     canvasConstants: vec2<f32>;
};

@group(0)
@binding(1)
var<storage,read>pointLights: PointLightContainer;

@group(0)
@binding(2)
var<storage,read> spotLights: SpotLightContainer;



@group(1)
@binding(0)
var<uniform> globals: Globals;


@group(2)
@binding(0)
var<uniform> tileInfo:TileInfo;
@group(2)
@binding(1)
var<uniform> canvasSize: CanvasSize;


 @stage(compute)
 @workgroup_size(64,1,1)
fn main(@builtin(global_invocation_id) Global_Invocation_Id: vec3<u32>) {
     let index = Global_Invocation_Id.x;
     if(index >= globals.lights_num.x)
     {
         return;
     }

     let view_near = globals.u_view_proj[3][2] / (-1.0 + globals.u_view_proj[2][2]);
     let view_far = globals.u_view_proj[3][2] / (1.0 + globals.u_view_proj[2][2]);

    var lightPos = pointLights.elements[index].position;
        lightPos = lightPos / lightPos.w;
        //TODO: calculate range based on attenuation values.
        var lightRadius: f32 = pointLights.elements[index].attenuation.x;
        //TODO: multiply by view matrix (and add view matrix to a uniform)
    var bounding_box_min = lightPos - vec4<f32>(vec3<f32>(lightRadius),0.0);
    var bounding_box_max = lightPos + vec4<f32>(vec3<f32>(lightRadius),0.0);

    var frustums: array<vec4<f32>,6>;
    frustums[4] = vec4<f32>(0.0,0.0,-1.0,view_near); // near plane
    frustums[5] = vec4<f32>(0.0,0.0,1.0,-view_far); // far plane
    for(var y: i32 = 0; y < tileInfo.tile_count_y; y= y +1)
    {
         for(var x: i32 = 0; x < tileInfo.tile_count_x; x= x +1)
        {
            var tilePixelIndex = vec2<i32>(x* tileInfo.tile_size,y*tileInfo.tile_size);
            // tile position in NDC space
            var floorCoord: vec2<f32> = 2.0 * vec2<f32>(tilePixelIndex) / canvasSize.canvasConstants.xy - vec2<f32>(1.0);  // -1, 1
            var ceilCoord: vec2<f32> = 2.0 * vec2<f32>(tilePixelIndex + vec2<i32>(tileInfo.tile_size)) / canvasSize.canvasConstants.xy - vec2<f32>(1.0);  // -1, 1
            var viewFloorCoord: vec2<f32> = vec2<f32>( (- view_near * floorCoord.x - globals.u_view_proj[2][0] * view_near) / globals.u_view_proj[0][0] , (- view_near * floorCoord.y - globals.u_view_proj[2][1] * view_near) / globals.u_view_proj[1][1] );
            var viewCeilCoord: vec2<f32> = vec2<f32>( (- view_near * ceilCoord.x - globals.u_view_proj[2][0] * view_near) / globals.u_view_proj[0][0] , (- view_near * ceilCoord.y - globals.u_view_proj[2][1] * view_near) / globals.u_view_proj[1][1] );
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
                // light is overlapping with the tile
                var tileId: u32 = u32(x + y * tileInfo.tile_count_x);
                if (tileId < 0u || tileId >= tileInfo.num_tiles) {
                    continue;
                }
                //var offset: u32 = atomicAdd(&(tileLightId.data[tileId].count), 1u);
                // if (offset >= config.numTileLightSlot) {
                //     continue;
                // }
                // tileLightId.data[tileId].lightId[offset] = GlobalInvocationID.x;
        }
    }




 }
 }