#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(set = 0, binding = 0) uniform LightMatrix {
    mat4 lightMatrix;
};
layout(set = 1, binding = 0) uniform Model {
    mat4 transform;
    uint materialIndex;
} model;

layout(location = 0) in vec3 inPosition;

void main() {
    gl_Position = lightMatrix * model.transform * vec4(inPosition, 1);
}
