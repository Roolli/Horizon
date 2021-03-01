#version 450
#extension GL_EXT_scalar_block_layout: require

layout(location =0) in vec3 a_pos;

layout(set=0,binding=0) uniform ViewProj {
    mat4 u_view_proj;
};

layout(std430,set=0,binding=1)
buffer InstanceData
{
    mat4 transform[];
};


void main()
{
    gl_Position = u_view_proj * transform[gl_InstanceIndex] * vec4(a_pos,1.0);
}
