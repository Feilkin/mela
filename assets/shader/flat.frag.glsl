#version 450
#extension GL_ARB_separate_shader_objects : enable

#define PI 3.1415926538
#define MAX_MATERIALS 256
#define MAX_LIGHTS 32

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

layout(set = 0, binding = 0) uniform Globals {
    mat4 view;
    mat4 proj;
    vec3 cameraPos;
} globals;

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

// from https://github.com/pboechat/cook_torrance/blob/master/application/shaders/cook_torrance_colored.fs.glsl
vec3 CookTorrance(
    vec3 materialColor,
    vec3 baseReflectivity,
    vec3 materialSpecularColor,
    vec3 normal,
    vec3 lightDir,
    vec3 viewDir,
    vec3 lightColor,
    float metallicFactor,
    float roughnessFactor)
{
    float NdotL = max(0.000001, dot(normal, lightDir));
    vec3 Rs = vec3(0.0);
    vec3 F = vec3(0.0);
    if (NdotL > 0)
    {
        vec3 H = normalize(lightDir + viewDir);
        float NdotH = max(0, dot(normal, H));
        float NdotV = max(0.0000001, dot(normal, viewDir));
        float VdotH = max(0, dot(lightDir, H));

        // Fresnel reflectance (Schlick approximation)
        F = baseReflectivity + (1.0 - baseReflectivity) * pow(1.0 - VdotH, 5.0);

        // Microfacet distribution by Beckmann
        float m_squared = roughnessFactor * roughnessFactor;
        float r1 = 1.0 / (4.0 * m_squared * pow(NdotH, 4.0));
        float r2 = (NdotH * NdotH - 1.0) / (m_squared * NdotH * NdotH);
        float D = r1 * exp(r2);

        // Geometric shadowing
        float two_NdotH = 2.0 * NdotH;
        float g1 = (two_NdotH * NdotV) / VdotH;
        float g2 = (two_NdotH * NdotL) / VdotH;
        float G = min(1.0, min(g1, g2));

        Rs = (F * D * G) / (PI * NdotL * NdotV);
    }

    vec3 diffuseFactor = vec3(1.0) - F;
    // pure metals have no diffuse light
    diffuseFactor *= 1.0 - metallicFactor;

    return max(vec3(0.), (diffuseFactor * materialColor + Rs * materialSpecularColor) * NdotL * lightColor);
}

void main() {
    vec3 ambient = vec3(0.05, 0.05, 0.05);
    Material material = materialsDefinitions.materials[model.materialIndex];

    vec3 base_reflectivity = mix(vec3(0.04), material.baseColor.rgb, material.metallicFactor);
    vec3 color = ambient * material.baseColor.rgb;
    vec3 light_combined = vec3(0.0);

    // TODO: what is this??
    vec3 view_direction = normalize(globals.cameraPos - vertexPosition.xyz);

    for (int i = 0; i < int(numLights) && i < MAX_LIGHTS; i++) {
        Light light = lights[i];
        vec4 light_space_coords = light.view_matrix * vertexPosition;
        float shadow = fetch_shadow(i, light_space_coords);

        color += shadow * CookTorrance(
            material.baseColor.rgb,
            base_reflectivity,
            vec3(1.0, 1.0, 1.0),
            normalize(fragNormal),
            normalize(-light.direction.xyz),
            view_direction,
            light.color.rgb,
            material.metallicFactor,
            material.roughnessFactor
        );
    }

    color += light_combined;
    vec3 hdr_color = fragColor.rgb * color;
    vec3 mapped = hdr_color / (hdr_color + vec3(1.0));

    outColor = vec4(mapped, 1.0);
}