#version 460
#extension GL_ARB_separate_shader_objects : enable

layout (location=0) in vec3 coord3d;
layout (location=1) in vec2 uv;
layout (location=2) in int tex_id;

layout (location=0) uniform mat4 m_transform;
layout (location=1) uniform mat4 m_transform1;

layout (location=0) out vec2 uv_out;
layout (location=1) flat out int tex_id_out;

void main(void) {
    gl_Position = m_transform * m_transform1 * vec4(coord3d, 1.0);
    tex_id_out = tex_id;
    uv_out = uv;
}
