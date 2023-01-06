#version 460
#extension GL_ARB_separate_shader_objects : enable

layout (location=10, binding=6) uniform sampler2D tex_full;

layout (location=2) uniform vec4 levelShift;
layout (location=4) uniform vec4 selectedColor;
layout (location=6) uniform int selectedFrag;

layout (location=1) in vec3 coord3dIn;

layout(location = 0) out vec4 outColor;

float n = 4096.0;

float calc_tex_coord_f(float v, float shift)
{
    return (v + shift * (1.0 / 128.0));
}

void main(void) {
  if (selectedFrag > 0 && selectedFrag == gl_PrimitiveID) {
    outColor = selectedColor;
  } else {
    vec2 coordf = vec2(calc_tex_coord_f(coord3dIn.x / 8.0, levelShift.x), calc_tex_coord_f(coord3dIn.y / 8.0, levelShift.y));
    outColor = texture(tex_full, coordf);
  }
}
