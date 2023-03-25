#version 460
#extension GL_ARB_separate_shader_objects : enable

layout (location=0) in ivec2 coordIn;

layout (location=0) uniform mat4 m_transform;
layout (location=1) uniform mat4 m_transform1;

layout (location=2) uniform ivec4 levelShift;
layout (location=3) uniform float heightScale;

layout (location=7) uniform float step;
layout (location=8) uniform int width;

layout (binding=9) readonly buffer heights_buffer
{
    uint heights[];
};
layout (location=21) uniform vec4 sunlight;

layout (location=1) out vec3 coord3dOut;
layout (location=2) out float brightness;

void main(void) {
    vec3 coord3d = vec3(float(coordIn.x) * step, float(coordIn.y) * step, 0.0);
    uint index = ((coordIn.y + levelShift.y) % width) * width
               + ((coordIn.x + levelShift.x) % width);
    vec3 coordf = vec3(coord3d.x, coord3d.y, float(heights[index]) * heightScale);
    vec4 coord = m_transform * m_transform1 * vec4(coordf, 1.0);
    gl_Position = coord;
    coord3dOut = vec3(coord3d.xy, coordf.z);

    uint index1 = (int(coordIn.y + levelShift.y + 1) % width) * width
                + (int(coordIn.x + levelShift.x) % width);
    uint index2 = (int(coordIn.y + levelShift.y) % width) * width
                + (int(coordIn.x + levelShift.x + 1) % width);

    int sunlight_var_1 = int(sunlight.x);
    int sunlight_var_2 = int(sunlight.y);
    int sunlight_var_3 = int(sunlight.z);

    int ch = int(heights[index]);
    int br_i = sunlight_var_3 +
               sunlight_var_2 * (int(heights[index1]) - ch) -
               sunlight_var_1 * (ch - int(heights[index2]));
    float br_f = float(br_i) / float(0x15e) + float(0x80);
    brightness = clamp(br_f, 0.0, 255.0);
}
