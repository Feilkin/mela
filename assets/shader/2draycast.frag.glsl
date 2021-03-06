#version 450
#extension GL_ARB_separate_shader_objects : enable

#define M_PI 3.1415926535897932384626433832795

#define SCREEN_W 768
#define SCREEN_H 576

#define MAX_LIGHTS 30
struct Light {
    vec4 position;
    vec4 color;
    vec4 attributes; // r = angle, g = sector
};

layout(set = 0, binding = 0) uniform texture2D t_Color;
layout(set = 0, binding = 1) uniform texture2D t_Material;
layout(set = 0, binding = 2) uniform sampler s_Color;

layout(binding = 3, std140) uniform Lights {
    Light light[MAX_LIGHTS];
    uint numLights;

} lights;

layout(location = 0) in vec2 fragTexCoord;

layout(location = 0) out vec4 outColor;

void main() {
    vec4 material = texture(sampler2D(t_Material, s_Color), fragTexCoord);
    vec3 lightLevel = vec3(material.g);

    vec3 currentPos = vec3(fragTexCoord.x * SCREEN_W, fragTexCoord.y * SCREEN_H, material.x);

    for (int lightIndex = 0; lightIndex < lights.numLights; lightIndex++) {
        Light light = lights.light[lightIndex];
        bool obscured = false;
        float lightDistance = distance(currentPos, light.position.xyz);

        if (lightDistance > light.position.w) { continue; }
        // angle is in range -pi..pi
        float angle = atan(currentPos.y - light.position.y, currentPos.x - light.position.x);
        float angleDiff = max(light.attributes.x, angle) - min(light.attributes.x, angle);
        angleDiff = abs(mod(angleDiff + M_PI, M_PI * 2) - M_PI);

        if (angleDiff > light.attributes.y) { continue; }

        int stepInterval = int(floor(lightDistance / 50.)) + 1;

        for (int step = 0; step <= lightDistance; step+=stepInterval) {
            float stepFactor = step / lightDistance;
            vec3 stepPos = mix(currentPos.xyz, light.position.xyz, stepFactor);

            vec4 stepMaterial = texture(sampler2D(t_Material, s_Color), vec2(stepPos.x / SCREEN_W, stepPos.y / SCREEN_H));

            if (stepMaterial.r > stepPos.z) {
                obscured = true;
                break;
            }
        }

        if (obscured) { continue; }

        // light source not obscured, calculate light intensity and add to lightLevel
        float distanceNormalised = clamp(light.position.w - lightDistance, 0., light.position.w) / light.position.w;
        distanceNormalised = distanceNormalised * distanceNormalised;
        float intensity = clamp(distanceNormalised, 0., 1.) * light.color.a;

        lightLevel += light.color.rgb * intensity;
    }

    outColor = texture(sampler2D(t_Color, s_Color), fragTexCoord) * vec4(lightLevel, 1);
}