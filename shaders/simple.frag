#version 430 core

in layout(location=0) vec4 in_color;
in layout(location=1) vec3 normal;
out vec4 color;

vec3 lightDirection = normalize(vec3(0.8, -0.5, 0.6));

void main()
{
    color = vec4(in_color.rgb * max(0, dot(normal, -lightDirection)), in_color.a);
}