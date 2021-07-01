#stage vertex

#sal
vformat struct Vertex
{
    vec4f Color;
    vec3f Pos;
    vec3f Normal;
}

const struct Projection
{
    mat4f ProjectionMatrix;
}

const mat4f ModelView;
#sal

out vec3f normal;

void main()
{
    normal = Normal;
    gl_Position = ProjectionMatrix * ModelView * vec4f(Pos, 1.0f);
}
