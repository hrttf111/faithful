#version 460
#extension GL_ARB_separate_shader_objects : enable

layout (location=3) uniform float heightScale;
layout (location=4) uniform vec4 selectedColor;
layout (location=6) uniform int selectedFrag;

layout (location=1) in vec3 coord3dIn;

layout(location = 0) out vec4 outColor;

void main(void) {
  if (selectedFrag > 0 && selectedFrag == gl_PrimitiveID) {
    outColor = selectedColor;
  } else {
    uint height = uint(coord3dIn.z / heightScale);
    if (height <= 0) {
        outColor = vec4(0, 0.1, 1, 0);
    } else {
        float color_ratio = 0.2 * coord3dIn.z * 5.0;
        outColor = vec4(0.3 + color_ratio, 0.5 - color_ratio, 0.0, 0);
    }
  }
}
