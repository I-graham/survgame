#version 450

layout(set=0, binding=0, std140)
uniform Uniforms{
	mat4 ortho;
};

struct Instance {
	vec4 tint;
	vec4 text_coords;
	vec2 scale;
	vec2 translate;
};

layout(set=1, binding=0, std140)
buffer InstanceData {
	Instance instances[];
};

vec2 positions[4] = vec2[](
    vec2(1.0, -1.0),
    vec2(-1.0, 1.0),
    vec2(1.0, 1.0),
    vec2(-1.0, -1.0)
);

layout(location=0) out vec2 text_coords;

void main() {
	vec2 coord = positions[gl_VertexIndex % 4];
    vec2 pos = coord * instances[gl_InstanceIndex].scale + instances[gl_InstanceIndex].translate;
    gl_Position = ortho * vec4(pos, 0.0, 1.0);
	text_coords = coord * vec2(1, -1) + vec2(0.5,0.5);
}