#version 460
#extension GL_ARB_separate_shader_objects : enable

layout (location=1) in vec3 coord3dIn;
layout(location = 0) out vec4 outColor;

float n = 4096.0;

void main(void) {
    ivec3 coordi = ivec3(coord3dIn.x / 8.0 * n, coord3dIn.y / 8.0 * n, coord3dIn.z);
    if (((coordi.x % 32) == 0) || ((coordi.y % 32) == 0)) {
        outColor = vec4(1.0, 0.5, 0.0, 0.0);
    } else {
        discard;
    }
}
