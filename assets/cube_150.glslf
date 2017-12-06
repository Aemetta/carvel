#version 150 core
in vec2 v_TexCoord;
in vec4 v_color;
out vec4 o_Color;
uniform sampler2D t_color;
void main() {
    vec4 tex = texture(t_color, v_TexCoord);
    float blend = dot(v_TexCoord-vec2(0.5,0.5), v_TexCoord-vec2(0.5,0.5));
    o_Color = mix(tex, v_color, blend*1.0);
}
