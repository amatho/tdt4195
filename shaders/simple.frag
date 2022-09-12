#version 430 core

out vec4 color;

void main()
{
    bool isEven = mod(floor(gl_FragCoord.x / 10) + floor(gl_FragCoord.y / 10), 2.0) == 0.0;
    float c = (isEven) ? 1.0 : 0.0;

    color = vec4(c, c, c, 1.0f);
}