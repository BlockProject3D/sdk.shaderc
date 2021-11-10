#stage vertex

#sal
const struct Viewport
{
    mat4f Projection;
}

const mat4f ModelView;
const vec3f CamPos;

const vformat Vertex
{
    vec3f Position;
    vec3f Normal;
    vec2f Texture;
}
#sal

out vec2 tex_coords;

void main()
{
    gl_Position = vec4(Vertex_Position, 1.0f) * ModelView * Viewport_Projection;
    tex_coords = Vertex_Texture;
}