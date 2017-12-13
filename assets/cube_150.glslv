#version 150 core
in vec3 a_pos;
in vec4 a_color;
in float a_light;
in vec2 a_tex_coord;
out vec2 v_TexCoord;
out vec4 v_color;
out float v_light;
uniform mat4 u_model_view_proj;
void main() {
     v_TexCoord = a_tex_coord;
        v_color = a_color;
        v_light = a_light;
    gl_Position = u_model_view_proj * vec4(a_pos, 1.0);
}
