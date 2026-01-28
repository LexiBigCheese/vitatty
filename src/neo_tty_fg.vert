float3x3 uniform transform;
float2 uniform char_dim;
float uniform italic_shift;

// {lower_nibble u;upper_nibble v;byte x;byte y;byte style}
// STYLE_BOLD   = 0b00000001
// STYLE_DIM    = 0b00000010
// STYLE_ITALIC = 0b00000100
unsigned char4 in a_uvxyst;
float4 in a_color;
unsigned int in gl_VertexIndex : INDEX;

float4 out gl_Position : POSITION;
float4 out v_color : COLOR0;
float2 out v_uv : TEXCOORD0;

void main() {
    v_color = a_color.zyxw;
    float2 corner = float2(gl_VertexIndex & 1, (gl_VertexIndex & 2) >> 1);
    v_uv = (float2(a_uvxyst.x & 0xF, a_uvxyst.x >> 4) + corner) * char_dim;
    corner = corner + float2((gl_VertexIndex & 2 == 0 && a_uvxyst.w & 4 != 0) ? 0.0 : italic_shift, 0.0);
    float2 vtx_pos = float2(a_uvxyst.yz) + corner;
    gl_Position = float4(mul(transform, float3(vtx_pos, 1.0)), 1.0);
    //TODO: apply bold/dim style somehow?
}
