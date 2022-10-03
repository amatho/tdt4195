#version 430 core

in layout(location=0) vec4 in_color;
out vec4 color;

void main()
{
    color = in_color;
}