#stage pixel

in vec2 tex_coords;

#sal
output vec4f FragColor : ORDER_0;

const struct Material
{
    vec4f BaseColor;
    vec4f SpecularColor;
    float Specular;
    float UvMult;
}

const Sampler BaseSampler;
const Texture2D:vec4f BaseTexture : BaseSampler;

const vec3f CamPos;
#sal

void main()
{
    vec4 color;
    if (Material_UvMult > 0.0f)
        color = texture(BaseTexture, tex_coords * Material_UvMult);
    else
        color = texture(BaseTexture, tex_coords);
    FragColor = Material_BaseColor * color;
}
