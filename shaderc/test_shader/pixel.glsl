#stage pixel

in vec2 tex_coords;

#sal
output vec4f FragColor;

const struct Material
{
    vec4f BaseColor;
    vec4f SpecularColor;
    float Specular;
    float UvMult;
}

const Texture2D:4f BaseTexture;
const Sampler BaseSampler : BaseTexture;
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
