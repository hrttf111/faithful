#version 460
#extension GL_ARB_separate_shader_objects : enable

layout (location=8, binding=0) uniform usampler2D tex;
layout (location=9, binding=1) uniform usampler1D palette;

layout (location=2) uniform vec4 levelShift;
layout (location=4) uniform vec4 selectedColor;
layout (location=6) uniform int selectedFrag;

layout (location=1) in vec3 coord3dIn;

layout(location = 0) out vec4 outColor;

float n = 4096.0;

vec3 mk_tex(uint val)
{
    uvec4 color = texelFetch(palette, int(val % 128), 0);
    return vec3(float(color.r) / 255.0, float(color.g) / 255.0, float(color.b) / 255.0);
}

int calc_tex_coord(float v, float shift)
{
    return int(v / 8.0 * n + shift * 32.0) % 4096;
}

void main(void) {
  if (selectedFrag > 0 && selectedFrag == gl_PrimitiveID) {
    outColor = selectedColor;
  } else {
    ivec2 coordsInt = ivec2(calc_tex_coord(coord3dIn.x, levelShift.x), calc_tex_coord(coord3dIn.y, levelShift.y));
    uint color = texelFetch(tex, coordsInt, 0).r;
    vec3 c = mk_tex(color);
    outColor = vec4(c, 0);
  }
}
