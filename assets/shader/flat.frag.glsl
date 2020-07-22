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

layout(set = 0, binding = 3) uniform texture2DArray t_Shadow;
layout(set = 0, binding = 4) uniform samplerShadow s_Shadow;

layout(set = 1, binding = 0) uniform Model {
    mat4 transform;
    uint materialIndex;
} model;

layout(location = 0) in vec3 fragNormal;
layout(location = 1) in vec4 fragColor;
layout(location = 2) in vec2 fragTexCoord;
layout(location = 3) in vec4 vertexPosition;

layout(location = 0) out vec4 outColor;

// from wgpu-rs examples
float fetch_shadow(int light_id, vec4 light_space_coords) {
    if (light_space_coords.w <= 0.0) {
        return 1.0;
    }

    // compensate for the Y-flip difference between the NDC and texture coordinates
    const vec2 flip_correction = vec2(0.5, -0.5);
    // compute texture coordinates for shadow lookup
    vec4 light_local = vec4(
    light_space_coords.xy * flip_correction/light_space_coords.w + 0.5,
    light_id,
    light_space_coords.z / light_space_coords.w
    );
    // do the lookup, using HW PCF and comparison
    return texture(sampler2DArrayShadow(t_Shadow, s_Shadow), light_local);
}

void main() {
    vec3 ambient = vec3(0.05, 0.05, 0.05);

    vec3 color = ambient;
    for (int i = 0; i < int(numLights) && i < MAX_LIGHTS; i++) {
        Light light = lights[i];

        float shadow = fetch_shadow(i, light.view_matrix * vertexPosition);

        float diffuse = max(0.0, dot(fragNormal, -light.direction.xzy));
        color += shadow * diffuse * light.color.rgb * (light.color.a / 5.0);
    }

    vec3 material_color = materialsDefinitions.materials[model.materialIndex].baseColor.rgb;

    vec3 hdr_color = material_color * fragColor.rgb * color;
    vec3 mapped = hdr_color / (hdr_color + vec3(1.0));

    outColor = vec4(mapped, 1.0);
}