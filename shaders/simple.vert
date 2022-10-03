#version 430 core

layout(location=0) in vec3 position;
layout(location=1) in vec4 color;

layout(location=0) out vec4 out_color;

layout(location=0) uniform mat4 transform;

void main()
{
    vec4 inPosition = vec4(position, 1.0);
    vec4 newPosition = transform * inPosition;
    
    gl_Position = newPosition;
    out_color = color;
}