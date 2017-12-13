#version 150 core
in vec2 v_TexCoord;
in vec4 v_color;
in float v_light;
out vec4 o_Color;
uniform sampler2D t_color;
void main() {
    vec4 tex = texture(t_color, v_TexCoord);
    vec4 col = v_color * (v_color + tex * 2 * (1 - v_color));

    o_Color = col * vec4(v_light,v_light,v_light,1.0);
}
