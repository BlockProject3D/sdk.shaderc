#stage vertex

#sal
const struct Root : ORDER_0
{
    mat4f ModelView;
    vec3f CamPos;
}

# Additional line
const struct Viewport : ORDER_1 # Allows binding the constant buffer to a fixed always same slot; currently not working
{
    mat4f Projection;
}

const mat4f ModelView;
const vec3f CamPos;

vformat struct Vertex
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
