#version 430 core

layout(location=0) in vec3 position;
layout(location=1) in vec4 color;
layout(location=2) in vec3 normal;

layout(location=0) out vec4 out_color;
layout(location=1) out vec3 out_normal;

layout(location=0) uniform mat4 mvp_transform;
layout(location=1) uniform mat4 model_transform;

void main()
{
    gl_Position = mvp_transform * vec4(position, 1.0);
    out_color = color;
    out_normal = normalize(mat3(model_transform) * normal);
}