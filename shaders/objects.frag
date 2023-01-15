#version 460
#extension GL_ARB_separate_shader_objects : enable

layout (location=3) uniform int fragNum;
layout (location=2) uniform int selectedFrag;
layout (binding = 0, location=4) uniform usampler1D colors;

layout (location = 0) out vec4 outColor;

void main(void) {
  if (selectedFrag > 0 && selectedFrag == gl_PrimitiveID) {
    outColor = vec4(1.0, 0.0, 0.0, 0.0);
  } else {
    int size = textureSize(colors, 0);
    uvec4 color = texelFetch(colors, int(gl_PrimitiveID % size), 0);
    outColor = vec4(float(color.r) / 255.0, float(color.g) / 255.0, float(color.b) / 255.0, 0.0);
  }
}
