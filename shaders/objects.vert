#version 460
#extension GL_ARB_separate_shader_objects : enable

layout (location=0) in vec3 coord3d;
//layout (location=1) in vec3 colorIn;
layout (location=0) uniform mat4 m_transform;
layout (location=1) uniform mat4 m_transform1;

//flat out vec4 colorOut;
//out vec4 colorOut;

void main(void) {
    gl_Position = m_transform * m_transform1 * vec4(coord3d, 1.0);
    //colorOut = vec4(colorIn.x, colorIn.y, colorIn.z, 0.0);
}
