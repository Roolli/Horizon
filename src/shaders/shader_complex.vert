#version 450

const vec2 positions[3] = vec2[3](
    vec2(0.0,0.5),
    vec2(-0.5,-0.5),
    vec2(0.5,-0.5)
);

layout (location=1)  out vec2 vertex_Color;

void main()
{
    gl_Position = vec4(positions[gl_VertexIndex],0.0,1.0);
    vertex_Color = positions[gl_VertexIndex];
}