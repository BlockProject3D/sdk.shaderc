#stage pixel

in vec2 tex_coords;

#sal
output vec4f FragColor;

const struct Material
{
    vec4f BaseColor;
    vec4f SpecularColor;
    float Specular : Pack;
    float UvMult : Pack;
}

const Sampler BaseSampler;
const Texture2D:vec4f BaseTexture : BaseSampler;
#sal

void main()
{
    vec4f color;
    if (Material_UvMult > 0.0f)
        color = BaseTexture_Sample(tex_coords * Material_UvMult);
    else
        color = BaseTexture_Sample(tex_coords);
    FragColor = Material_BaseColor * color;
}
