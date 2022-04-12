struct SkyVertexOutput {
    @builtin(position) position: vec4<f32>;
    @location(0) uv: vec3<f32>;
};
struct ReflectionData {
    projection_inverse: mat4x4<f32>;
    view: mat4x4<f32>;
};

@group(0)
@binding(0)
var<uniform> reflection_data: ReflectionData;

@stage(vertex)
fn sky_vs(@builtin(vertex_index) vertex_index: u32) -> SkyVertexOutput {
    // draw large triangle https://github.com/gfx-rs/wgpu/blob/master/wgpu/examples/skybox/shader.wgsl
    let tmp1 = i32(vertex_index) / 2;
    let tmp2 = i32(vertex_index) & 1;
    let pos = vec4<f32>(
        f32(tmp1) * 4.0 - 1.0,
        f32(tmp2) * 4.0 - 1.0,
        1.0,
        1.0
    );
    let inverted_model_view = transpose(mat3x3<f32>(reflection_data.view[0].xyz,reflection_data.view[1].xyz,reflection_data.view[2].xyz));
    let unprojected = reflection_data.projection_inverse * pos;

    var out: SkyVertexOutput;
    out.uv = inverted_model_view * unprojected.xyz;
    out.position = vec4<f32>(pos.x,pos.y,0.0,pos.w);
    return out;
}

@group(0)
@binding(1)
var r_texture: texture_cube<f32>;
@group(0)
@binding(2)
var r_sampler: sampler;

@stage(fragment)
fn sky_fs(in: SkyVertexOutput) -> @location(0) vec4<f32> {
    return textureSample(r_texture,r_sampler,in.uv);
}