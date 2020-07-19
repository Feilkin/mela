#version 450
#extension GL_ARB_separate_shader_objects : enable

#define MAX_MATERIALS 256

struct Material {
    vec4 baseColor;
    float metallicFactor;
    float roughnessFactor;
    vec2 reserved;
};

layout(set = 0, binding = 1, std140) uniform MaterialDef {
    Material materials[MAX_MATERIALS];
} materialsDefinitions;

layout(set = 1, binding = 0) uniform Model {
    mat4 transform;
    uint materialIndex;
} model;

layout(location = 0) in vec3 fragNormal;
layout(location = 1) in vec4 fragColor;
layout(location = 2) in vec2 fragTexCoord;

layout(location = 0) out vec4 outColor;

void main() {
    outColor = materialsDefinitions.materials[model.materialIndex].baseColor * fragColor * min(dot(fragNormal, vec3(0, 0, 1)) + 0.3, 1);
}