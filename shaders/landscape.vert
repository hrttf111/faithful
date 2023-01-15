#version 460
#extension GL_ARB_separate_shader_objects : enable

layout (location=0) in vec2 coordIn;

layout (location=0) uniform mat4 m_transform;
layout (location=1) uniform mat4 m_transform1;

layout (location=2) uniform vec4 levelShift;
layout (location=3) uniform float heightScale;

layout (location=7) uniform float step;
layout (location=8) uniform int width;

layout (binding=9) readonly buffer heights_buffer
{
    uint heights[];
};

layout (location=1) out vec3 coord3dOut;

void main(void) {
    vec3 coord3d = vec3(float(coordIn.x) * step, float(coordIn.y) * step, 0.0);
    uint index = (int(coordIn.y + levelShift.y) % width) * width
               + (int(coordIn.x + levelShift.x) % width);
    vec3 coordf = vec3(coord3d.x, coord3d.y, float(heights[index]) * heightScale);
    vec4 coord = m_transform * m_transform1 * vec4(coordf, 1.0);
    gl_Position = coord;
    coord3dOut = vec3(coord3d.xy, coordf.z);
}
