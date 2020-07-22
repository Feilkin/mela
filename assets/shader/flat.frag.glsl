#version 450
#extension GL_ARB_separate_shader_objects : enable

#define MAX_MATERIALS 256
#define MAX_LIGHTS 256

struct Material {
    vec4 baseColor;
    float metallicFactor;
    float roughnessFactor;
    vec2 reserved;
};

struct Light {
    mat4 view_matrix;
    vec4 direction;
    vec4 color;
};

layout(set = 0, binding = 1, std140) uniform MaterialDef {
    Material materials[MAX_MATERIALS];
} materialsDefinitions;

layout(set = 0, binding = 2, std140) uniform Lights {
    uint numLights;
    Light lights[MAX_LIGHTS];
};

layout(set = 1, binding = 0) uniform Model {
    mat4 transform;
    uint materialIndex;
} model;

layout(location = 0) in vec3 fragNormal;
layout(location = 1) in vec4 fragColor;
layout(location = 2) in vec2 fragTexCoord;

layout(location = 0) out vec4 outColor;

void main() {
    vec3 ambient = vec3(0.05, 0.05, 0.05);

    vec3 color = ambient;
    for (int i = 0; i < int(numLights) && i < MAX_LIGHTS; i++) {
        Light light = lights[i];
        float diffuse = max(0.0, dot(fragNormal, -light.direction.xzy));
        color += diffuse * light.color.rgb;
    }

    vec4 materialColor = materialsDefinitions.materials[model.materialIndex].baseColor;

    outColor = materialColor * fragColor * vec4(color, 1.0);
}