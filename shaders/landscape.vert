#version 460
#extension GL_ARB_separate_shader_objects : enable

layout (location=0) in vec2 coordIn;

layout (location=0) uniform mat4 m_transform;
layout (location=1) uniform mat4 m_transform1;

layout (location=2) uniform vec4 levelShift;
layout (location=3) uniform float heightScale;

layout (location=7) uniform uint heights[128*128];
/*layout (binding=4) buffer heights
{
};*/

//flat out vec4 colorOut;
layout (location=1) out vec3 coord3dOut;

float step = 1.0 / 16.0;

void main(void) {
    vec3 coord3d = vec3(float(coordIn.x) * step, float(coordIn.y) * step, 0.0);
    uint index = (int(coordIn.y + levelShift.y) % 128) * 128
               + (int(coordIn.x + levelShift.x) % 128);
    vec3 coordf = vec3(coord3d.x, coord3d.y, float(heights[index]) * heightScale);
    vec4 coord = m_transform * m_transform1 * vec4(coordf, 1.0);
    gl_Position = coord;
    coord3dOut = vec3(coord3d.xy, coordf.z);
}
