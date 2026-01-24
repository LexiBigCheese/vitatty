void main(
    unsigned short uvfb,
    unsigned short uniform termWidthByFour : BUFFER[0],
float2 uniform charDim : BUFFER[0],
float4 uniform palette[16] : BUFFER[0],
unsigned int in gl_VertexIndex : INDEX,
float4 out gl_Position : POSITION,
float4 out fg_color : COLOR0,
float4 out bg_color : COLOR1,
float4 out tex_coord : TEXCOORD0
) {

fg_color = palette[(uvfb >> 4) & 0xF];
bg_color = palette[uvfb & 0xF];

unsigned short charRow = float(gl_VertexIndex) / float(termWidthByFour);
unsigned short charCol = (gl_VertexIndex - (charRow * termWidthByFour)) >> 2;
float2 corner = float2(gl_VertexIndex & 1, (gl_VertexIndex & 2) >> 1);

tex_coord = float4((float2(uvfb>>12, (uvfb>>8)&0xF)+corner)*charDim, 0.0, 0.0);

gl_Position = float4((float2(charCol, charRow)+corner)*charDim, 0.0, 1.0);

}

//
