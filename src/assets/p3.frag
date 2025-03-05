#ifdef GL_ES
precision mediump float;
#endif
in vec2 vTextureCoord;
in vec4 vColor;

uniform sampler2D uTexture;

vec3 sRGBToLinear(vec3 sRGB) {
    return mix(
        sRGB / 12.92,
        pow((sRGB + 0.055) / 1.055, vec3(2.4)),
        step(0.04045, sRGB)
    );
}

vec3 linearToSRGB(vec3 linear) {
    return mix(
        linear * 12.92,
        1.055 * pow(linear, vec3(1.0 / 2.4)) - 0.055,
        step(0.0031308, linear)
    );
}

const mat3 sRGBToP3 = mat3(
    1.2248840, -0.2249440, 0.0000000,
    -0.0420569,  1.0419031, 0.0000000,
    -0.0196376, -0.0786361, 1.0982735
);

void main(void)
{
    vec4 textureColor = texture2D(uTexture, vTextureCoord);
    vec3 linearSRGB = sRGBToLinear(textureColor.rgb);
    vec3 linearP3 = sRGBToP3 * linearSRGB;
    vec3 p3Color = linearToSRGB(linearP3);
    gl_FragColor = vec4(p3Color, textureColor.a);
}