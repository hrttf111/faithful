#version 460
#extension GL_ARB_separate_shader_objects : enable

layout (location=0) in vec3 coord3d;
layout (location=0) uniform mat4 m_transform;
layout (location=1) uniform mat4 m_transform1;

void main(void) {
    gl_Position = m_transform * m_transform1 * vec4(coord3d, 1.0);
}
