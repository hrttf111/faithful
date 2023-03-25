#version 460
#extension GL_ARB_separate_shader_objects : enable

layout (location=10, binding=1) uniform usampler1D palette;
layout (location=11, binding=2) uniform isamplerBuffer disp;
layout (location=12, binding=3) uniform usamplerBuffer bigf;
layout (location=13, binding=4) uniform usamplerBuffer sla;

layout (location=2) uniform ivec4 levelShift;
layout (location=3) uniform float heightScale;
layout (location=4) uniform vec4 selectedColor;
layout (location=6) uniform int selectedFrag;

layout (location=1) in vec3 coord3dIn;
layout (location=2) in float brightness;

layout(location = 0) out vec4 outColor;

float n = 4096.0;

vec3 mk_tex(uint val)
{
    uvec4 color = texelFetch(palette, int(val % 128), 0);
    return vec3(float(color.r) / 255.0, float(color.g) / 255.0, float(color.b) / 255.0);
}

uint mk_height(float z)
{
    uint height = uint(z / heightScale);
    if (height > 0) {
        return min(height + 0x96, 0x400);
    }
    if (z > 0.0) {
        height = uint(z*float(0x4b) / heightScale);
        return min(height+0x4b, 0x400);
    }
    return min(height + 0x4b, 0x400);
}

int get_disp(int x, int y)
{
    int sx = levelShift.x * 32;
    int sy = levelShift.y * 32;
    int dx = ((x + sx) % 256) * 256;
    int dy = (y + sy) % 256;
    return texelFetch(disp, dx + dy).r;
}

int get_disp_2(int x, int y)
{
    int sx = levelShift.x * 32;
    int sy = levelShift.y * 32;
    int ly = ((y + sy) % 32); // last line
    int dx = (ly == 31) ? 0 : 1;
    int x1 = ((x + dx + sx) % 256) * 256;
    int y1 = (y + 1 + sy) % 256;
    return texelFetch(disp, x1 + y1).r;
}

vec3 land_tex(vec3 coord)
{
    uint height = mk_height(coord.z);

    int disp_val = get_disp(int(coord.x), int(coord.y)+32);
    int disp_val_2 = get_disp_2(int(coord.x), int(coord.y)+32);
    int disp_param = int((float(disp_val_2) - float(disp_val)) / 4.0) + int(brightness);
    disp_param = clamp(disp_param, 0, 255);

    int static_component = int(texelFetch(sla, int(height)).r) * disp_val;
    uint static_component_u = uint(static_component);
    static_component_u = static_component_u & 0xfffffc03;
    static_component = int(static_component_u);
    static_component >>= 2;

    int height_component = int(height * 256) & 0x7fffff00;
    int index = static_component + height_component + disp_param;

    uint bigf_index = min(texelFetch(bigf, index).r, 128);
    return mk_tex(bigf_index);
}

void main(void) {
  if (selectedFrag > 0 && selectedFrag == gl_PrimitiveID) {
    outColor = selectedColor;
  } else {
    vec3 coordi = vec3(coord3dIn.x / 8.0 * n, coord3dIn.y / 8.0 * n, coord3dIn.z);
    vec3 c = land_tex(coordi);
    outColor = vec4(c, 0);
  }
}
