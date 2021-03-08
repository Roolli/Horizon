#version 450

layout (location=0) in vec2 a_coord;
layout (location=0) out vec2 out_coords;

void main()
{
    gl_Position = vec4(a_coord,0.0,1.0);
    out_coords = vec2((a_coord.x+1.0)/2.0,1-(a_coord.y+1.0)/2.0);
}