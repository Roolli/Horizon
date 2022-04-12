
struct ViewProj {
    u_view_proj: mat4x4<f32>;
};

struct Transforms {
 elements: array<mat4x4<f32>>;
}; 
@group(0)
@binding(1)
var<storage,read> transform: Transforms;

@group(0)
@binding(0)
var<uniform> viewProj: ViewProj;

@stage(vertex)
fn vs_main(@location(0) a_pos: vec3<f32>, @builtin(instance_index) index: u32) -> @builtin(position) vec4<f32> {
    return viewProj.u_view_proj * transform.elements[index] * vec4<f32>(a_pos,1.0);
}