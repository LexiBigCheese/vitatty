void main(
    float3 aPosition,
    float2 aUv,
    float4 out gl_Position : POSITION,
float4 out vUv : TEXCOORD0
) {
gl_Position = float4(aPosition, 1.0);
vUv = float4(aUv, 0.0, 0.0);
}
