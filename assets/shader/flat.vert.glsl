#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(set = 0, binding = 0) uniform Globals {
    mat4 view;
    mat4 proj;
} globals;

layout(set = 1, binding = 0) uniform Model {
    mat4 transform;
    uint materialIndex;
} model;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec2 inTexCoord;
//layout(location = 3) in vec4 inColor;

layout(location = 0) out vec3 fragNormal;
layout(location = 1) out vec4 fragColor;
layout(location = 2) out vec2 fragTexCoord;

void main() {
    gl_Position = globals.proj * globals.view * model.transform *  vec4(inPosition, 1.0);
    fragNormal = mat3(model.transform) * inNormal;
    fragColor = vec4(1.0);
    fragTexCoord = inTexCoord;
}