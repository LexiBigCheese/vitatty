#define termWidth 68
// #define bigAssUniformSize 1768

// assuming 8 pixel wide font

unsigned int in uvfg;
unsigned int in bg;
// float2 in corner;
// int uniform bigAssUniform[bigAssUniformSize] : BUFFER[0]; //pleeeease don't work
// int uniform otherBigAssUniform[bigAssUniformSize] : BUFFER[0];
float2 uniform charDim : BUFFER[0];
float3 uniform transform[2] : BUFFER[0];
unsigned int in gl_VertexIndex : INDEX;
unsigned int in gl_InstanceID : INSTANCE;
float4 out gl_Position : POSITION;
float4 out fg_color : COLOR0;
float4 out bg_color : COLOR1;
float4 out tex_coord : TEXCOORD0;

void main() {
    // unsigned int uvfg = bigAssUniform[gl_InstanceID];
    // unsigned int bg = otherBigAssUniform[gl_InstanceID];
    fg_color = float4((uvfg >> 16) & 0xFF, (uvfg >> 8) & 0xFF, uvfg & 0xFF, 255.0) / 255.0;
    bg_color = float4((bg >> 16) & 0xFF, (bg >> 8) & 0xFF, bg & 0xFF, 255.0) / 255.0;

    // unsigned short charRow = float(gl_InstanceID) / float(termWidth);
    // unsigned short charCol = gl_InstanceID - (charRow * termWidth);
    float2 corner = float2(gl_VertexIndex & 1, (gl_VertexIndex & 2) >> 1);
    gl_Position = float4(corner * 0.5, 0.0, 1.0);

    // tex_coord = float4((float2(uvfg >> 28, (uvfg >> 24) & 0xF) + corner) * charDim, 0.0, 0.0);
    // float3 homog = float3(float2(charCol, charRow) + corner, 1.0);
    // float3x3 transform_mat = float3x3(transform[0], transform[1], float3(0.0, 0.0, 1.0));
    // gl_Position = float4(homog * float3(0.1, 0.1, 1.0), 1.0);
    // //BODGE
}

//
