#define termWidth 68.0

//Recompile if you need a different termWidth

float3x3 uniform transform;

float4 in a_color;
unsigned int in gl_VertexIndex : INDEX;
unsigned int in gl_InstanceID : INSTANCE;

float4 out gl_Position : POSITION;
float4 out v_color : COLOR0;

void main() {
    v_color = a_color.zyxw;
    float charRow = floor(float(gl_InstanceID) / termWidth);
    float charCol = gl_InstanceID - (charRow * termWidth);
    float2 corner = float2(gl_VertexIndex & 1, gl_VertexIndex >> 1); //if things look wild, use y = (gl_VertexIndex & 2) >> 1
    float2 vtx_pos = float2(charCol, charRow) + corner;
    gl_Position = float4(mul(transform, float3(vtx_pos, 1.0)), 1.0);
}
