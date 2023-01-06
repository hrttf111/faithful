SHADER_PATH=shaders

SHADERS_VERT = landscape.vert objects.vert
SHADERS_FRAG = landscape.frag landscape_cpu.frag landscape_full.frag landscape_grad.frag landscape_cell.frag objects.frag
SHADERS_SPV = $(SHADERS_VERT:.vert=.vert.spv) $(SHADERS_FRAG:.frag=.frag.spv)

GLSLC_ARGS = -fauto-bind-uniforms -fauto-map-locations

shaders: $(addprefix $(SHADER_PATH)/,$(SHADERS_SPV))

%.vert.spv: %.vert
	glslc --target-env=opengl ${GLSLC_ARGS} $< -o $@

%.frag.spv: %.frag
	glslc --target-env=opengl ${GLSLC_ARGS} $< -o $@

clean:
	rm -f $(SHADER_PATH)/*.spv

.PHONY: clean shaders
