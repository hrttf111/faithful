#version 460
#extension GL_ARB_separate_shader_objects : enable

//flat layout (location=0) in vec4 colorIn;
//layout (location=0) in vec4 colorIn;
layout (location=3) uniform int fragNum;
layout (location=2) uniform int selectedFrag;

layout(location = 0) out vec4 outColor;

//layout(binding = 0) uniform sampler1D colors1;

void main(void) {
  if (selectedFrag > 0 && selectedFrag == gl_PrimitiveID) {
    outColor = vec4(1.0, 0.0, 0.0, 0.0);
  } else {
    //outColor = texture(colors1, float((gl_PrimitiveID % fragNum) / float(fragNum)));
    outColor = vec4(1.0, fragNum, 0.0, 0.0);
  }
}
