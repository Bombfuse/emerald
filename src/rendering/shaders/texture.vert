#version 100

attribute vec2 position;


varying lowp vec4 color;
varying lowp vec2 uv;

uniform vec4 Source;
uniform vec4 Color;

void main() {
    gl_Position = vec4(position, 0, 1);
    color = Color;
    color.z = Source.z;
    uv = position;
}