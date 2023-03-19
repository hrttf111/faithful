#version 460
#extension GL_ARB_separate_shader_objects : enable

layout (location=3) uniform int fragNum;
layout (location=2) uniform int selectedFrag;
layout (binding = 0, location=4) uniform sampler2D texture_main;

layout (location=0) in vec2 uv;
layout (location=1) flat in int tex_id;

layout (location = 0) out vec4 outColor;

void main(void) {
  if ((tex_id < 0) || (tex_id > 255)) {
    outColor = vec4(1.0, 1.0, 0.0, 0.0);
  } else {
    int row = tex_id / 8;
    int column = tex_id % 8;
    float hor_k = 1.0 / 8.0;
    float ver_k = 1.0 / 32.0;
    float u = hor_k * column + hor_k * uv.x;
    float v = ver_k * row + ver_k * uv.y;
    outColor = texture(texture_main, vec2(u, v));
    if (outColor.w > 0.0) {
        discard;
    }
  }
}
